//! Sparse I/O, not sparse validation or map cropping. The original complete
//! TIL offset table and original block offsets are retained. Only payloads for
//! the complete map's referenced sub-indices are read; aliases share one read.
use super::resources::SharedArchive;
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeSet,
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};

const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_TILES: usize = 16_384;
pub(crate) struct SelectedTileInput {
    pub data: Vec<u8>,
    pub read_bytes: usize,
}

pub(crate) fn load(
    archive: &mut SharedArchive,
    name: &str,
    selected: &BTreeSet<usize>,
    remaining: usize,
    cancel: &AtomicBool,
) -> Result<SelectedTileInput> {
    let size = archive.entry(name)?.size as usize;
    read_selected(size, selected, remaining, cancel, |offset, bytes| {
        Ok(archive.read_tile_range_into(name, offset, bytes)?)
    })
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "tile sparse read cancelled"
    );
    Ok(())
}
fn word(bytes: &[u8], offset: usize) -> usize {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}
fn plan(table: &[u8], size: usize, selected: &BTreeSet<usize>) -> Result<Vec<Range<usize>>> {
    ensure!(table.len() >= 4, "tile count absent");
    let count = word(table, 0);
    ensure!(
        (1..=MAX_TILES).contains(&count)
            && table.len() == 4 + (count + 1) * 4
            && table.len() < size,
        "invalid tile table bounds"
    );
    ensure!(
        !selected.is_empty() && *selected.last().unwrap() < count,
        "invalid tile sub-index selection"
    );
    let end = word(table, 4 + count * 4);
    ensure!(
        end == size - table.len(),
        "tile end offset or trailing bytes invalid"
    );
    let offsets: Vec<_> = (0..count).map(|i| word(table, 4 + i * 4)).collect();
    ensure!(
        offsets.iter().all(|&o| o < end),
        "unused tile offset outside body"
    );
    let mut boundaries: BTreeSet<_> = offsets.iter().copied().collect();
    ensure!(
        boundaries.first() == Some(&0),
        "tile body prefix unsupported"
    );
    boundaries.insert(end);
    let boundaries: Vec<_> = boundaries.into_iter().collect();
    let chosen: BTreeSet<_> = selected.iter().map(|&i| offsets[i]).collect();
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for offset in chosen {
        let next = boundaries[boundaries.binary_search(&offset).unwrap() + 1];
        let range = table.len() + offset..table.len() + next;
        if let Some(last) = ranges.last_mut() {
            if last.end == range.start {
                last.end = range.end;
                continue;
            }
        }
        ranges.push(range);
    }
    Ok(ranges)
}
fn read_selected(
    size: usize,
    selected: &BTreeSet<usize>,
    remaining: usize,
    cancel: &AtomicBool,
    mut read: impl FnMut(usize, &mut [u8]) -> Result<()>,
) -> Result<SelectedTileInput> {
    active(cancel)?;
    ensure!(
        (12..=MAX_ENTRY).contains(&size) && remaining >= 4,
        "tile input/read budget exceeded"
    );
    let mut count_bytes = [0; 4];
    read(0, &mut count_bytes)?;
    active(cancel)?;
    let count = word(&count_bytes, 0);
    ensure!(
        (1..=MAX_TILES).contains(&count),
        "invalid bounded tile count"
    );
    let header = 4 + (count + 1) * 4;
    ensure!(
        header < size && header <= remaining,
        "tile table/read budget exceeded"
    );
    let mut data = Vec::new();
    data.try_reserve_exact(size)
        .context("tile sparse buffer allocation")?;
    data.resize(size, 0);
    data[..4].copy_from_slice(&count_bytes);
    read(4, &mut data[4..header])?;
    active(cancel)?;
    let ranges = plan(&data[..header], size, selected)?;
    let read_bytes = ranges
        .iter()
        .try_fold(header, |n, r| n.checked_add(r.len()))
        .context("tile range budget overflow")?;
    ensure!(
        read_bytes <= remaining,
        "world encoded asset budget exceeded"
    );
    for range in ranges {
        active(cancel)?;
        read(range.start, &mut data[range])?;
    }
    active(cancel)?;
    Ok(SelectedTileInput { data, read_bytes })
}

#[cfg(test)]
mod tests {
    use super::super::tile::{decode_classic_til_selected, decode_roof_til_selected, TileLimits};
    use super::*;
    fn fixture() -> Vec<u8> {
        // Unsorted aliases, three original 577-byte diamonds, last unreferenced.
        let mut b = Vec::new();
        for n in [4u32, 577, 0, 577, 1154, 1731] {
            b.extend(n.to_le_bytes());
        }
        for color in [0x1234u16, 0, 0x7fff] {
            b.push(0);
            for _ in 0..288 {
                b.extend(color.to_le_bytes());
            }
        }
        b
    }
    #[test]
    fn sparse_io_keeps_original_offsets_pixels_roof_source_and_aliases() {
        let original = fixture();
        let selected = BTreeSet::from([0, 1, 2]);
        let mut reads = Vec::new();
        let sparse = read_selected(
            original.len(),
            &selected,
            1200,
            &AtomicBool::new(false),
            |offset, out| {
                reads.push(offset..offset + out.len());
                out.copy_from_slice(&original[offset..offset + out.len()]);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(reads, vec![0..4, 4..24, 24..1178]);
        assert_eq!(sparse.read_bytes, 1178);
        assert!(sparse.data[1178..].iter().all(|&b| b == 0));
        assert_eq!(
            decode_classic_til_selected(&original, TileLimits::default(), &selected).unwrap(),
            decode_classic_til_selected(&sparse.data, TileLimits::default(), &selected).unwrap()
        );
        assert_eq!(
            decode_roof_til_selected(&original, TileLimits::default(), &selected).unwrap(),
            decode_roof_til_selected(&sparse.data, TileLimits::default(), &selected).unwrap()
        );
    }
    #[test]
    fn all_offsets_and_whole_entry_limits_remain_checked_before_payload_io() {
        let good = fixture();
        let selected = BTreeSet::from([1]);
        for (offset, value) in [(16, 1731u32), (20, 1730), (0, 16_385), (4, 1731)] {
            let mut bytes = good.clone();
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            let mut payload = false;
            assert!(read_selected(
                bytes.len(),
                &selected,
                4096,
                &AtomicBool::new(false),
                |at, out| {
                    payload |= at >= 24;
                    out.copy_from_slice(&bytes[at..at + out.len()]);
                    Ok(())
                }
            )
            .is_err());
            assert!(!payload);
        }
        for selection in [BTreeSet::new(), BTreeSet::from([4])] {
            assert!(plan(&good[..24], good.len(), &selection).is_err());
        }
        assert!(read_selected(
            MAX_ENTRY + 1,
            &selected,
            usize::MAX,
            &AtomicBool::new(false),
            |_, _| panic!("invalid input must not read")
        )
        .is_err());
    }
    #[test]
    fn cancellation_or_io_failure_between_disjoint_ranges_never_publishes_partial_data() {
        let bytes = fixture();
        let selected = BTreeSet::from([1, 3]); // First and third physical blocks, not the middle.
        for fail_io in [false, true] {
            let cancelled = AtomicBool::new(false);
            let mut reads = Vec::new();
            let result = read_selected(bytes.len(), &selected, 4096, &cancelled, |at, out| {
                reads.push(at..at + out.len());
                if at == 24 && fail_io {
                    anyhow::bail!("fixture read failure");
                }
                out.copy_from_slice(&bytes[at..at + out.len()]);
                if at == 24 {
                    cancelled.store(true, Ordering::Release);
                }
                Ok(())
            });
            assert!(result.is_err());
            assert_eq!(reads, vec![0..4, 4..24, 24..601]);
        }
    }
    #[test]
    fn cancelled_and_exhausted_budgets_never_read_payload_or_return_partial_data() {
        let bytes = fixture();
        let selected = BTreeSet::from([0, 1]);
        assert!(read_selected(
            bytes.len(),
            &selected,
            4096,
            &AtomicBool::new(true),
            |_, _| panic!("cancelled read")
        )
        .is_err());
        for budget in [0, 3, 4, 23, 24, 1177] {
            let mut payload = false;
            assert!(read_selected(
                bytes.len(),
                &selected,
                budget,
                &AtomicBool::new(false),
                |at, out| {
                    payload |= at >= 24;
                    out.copy_from_slice(&bytes[at..at + out.len()]);
                    Ok(())
                }
            )
            .is_err());
            assert!(!payload);
        }
        let cancelled = AtomicBool::new(false);
        assert!(
            read_selected(bytes.len(), &selected, 4096, &cancelled, |at, out| {
                out.copy_from_slice(&bytes[at..at + out.len()]);
                if at == 4 {
                    cancelled.store(true, Ordering::Release);
                }
                Ok(())
            })
            .is_err()
        );
    }
}
