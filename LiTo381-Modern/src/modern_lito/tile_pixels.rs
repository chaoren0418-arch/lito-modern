//! Lossless per-row span storage for a validated 24x24 RGBA tile.
//! No color conversion or alpha quantization; even RGB under alpha zero survives.
use anyhow::{ensure, Result};

pub(crate) const RGBA_BYTES: usize = 24 * 24 * 4;
pub(crate) struct CompactTilePixels {
    runs: Vec<u8>,
}
impl CompactTilePixels {
    /// None means retain the original dense allocation: compression cannot grow it.
    pub(crate) fn try_new(rgba: &[u8]) -> Result<Option<Self>> {
        ensure!(rgba.len() == RGBA_BYTES, "invalid compact tile dimensions");
        // Keep internal holes verbatim; only trim all-zero ends of each row.
        // Dense interiors need no per-pixel scan, and at most 24 copies paint
        // a tile. The 48-byte plan permits one exact-size heap allocation.
        let mut spans = [(0u8, 0u8); 24];
        let mut size = 0;
        for (row, span) in rgba.chunks_exact(96).zip(&mut spans) {
            let mut start = 0;
            while start < 24 && row[start * 4..start * 4 + 4] == [0; 4] {
                start += 1;
            }
            if start == 24 {
                continue;
            }
            let mut end = 24;
            while end > start && row[(end - 1) * 4..end * 4] == [0; 4] {
                end -= 1;
            }
            size += 3 + (end - start) * 4;
            if size >= RGBA_BYTES {
                return Ok(None);
            }
            *span = (start as u8, end as u8);
        }
        let mut runs = Vec::new();
        runs.try_reserve_exact(size)?;
        for (y, &(start, end)) in spans.iter().enumerate() {
            if start == end {
                continue;
            }
            runs.extend_from_slice(&[y as u8, start, end - start]);
            runs.extend_from_slice(&rgba[y * 96 + start as usize * 4..y * 96 + end as usize * 4]);
        }
        debug_assert_eq!(runs.len(), size);
        Ok(Some(Self { runs }))
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.runs.capacity()
    }
    pub(crate) fn write_rgba(&self, output: &mut [u8; RGBA_BYTES]) {
        output.fill(0);
        let mut cursor = 0;
        while cursor < self.runs.len() {
            let start = self.runs[cursor] as usize * 24 + self.runs[cursor + 1] as usize;
            let count = self.runs[cursor + 2] as usize;
            cursor += 3;
            output[start * 4..(start + count) * 4]
                .copy_from_slice(&self.runs[cursor..cursor + count * 4]);
            cursor += count * 4;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transparent_runs_preserve_all_rgba_bytes_including_covered_black_and_partial_alpha() {
        for pattern in 0..32 {
            let mut source = vec![0u8; RGBA_BYTES];
            for i in 0..576 {
                if (i + pattern) % 7 < 3 {
                    source[i * 4..i * 4 + 4].copy_from_slice(&[
                        i as u8,
                        (i >> 1) as u8,
                        pattern as u8,
                        ((i * 19) % 256) as u8,
                    ]);
                }
            }
            source[0..4].copy_from_slice(&[0, 0, 0, 255]);
            source[2300..2304].copy_from_slice(&[111, 222, 33, 0]);
            let packed = CompactTilePixels::try_new(&source).unwrap().unwrap();
            assert!(packed.retained_bytes() < RGBA_BYTES);
            let mut decoded = [255; RGBA_BYTES];
            packed.write_rgba(&mut decoded);
            assert_eq!(source.as_slice(), decoded);
        }
    }
    #[test]
    fn empty_dense_and_fragmented_inputs_are_bounded_and_never_expand_storage() {
        let empty = CompactTilePixels::try_new(&[0; RGBA_BYTES])
            .unwrap()
            .unwrap();
        assert_eq!(empty.retained_bytes(), 0);
        let mut out = [255; RGBA_BYTES];
        empty.write_rgba(&mut out);
        assert_eq!(out, [0; RGBA_BYTES]);
        assert!(CompactTilePixels::try_new(&[255; RGBA_BYTES])
            .unwrap()
            .is_none());
        let mut fragmented = [0; RGBA_BYTES];
        for i in (0..576).step_by(2) {
            fragmented[i * 4 + 3] = 255;
        }
        let packed = CompactTilePixels::try_new(&fragmented).unwrap().unwrap();
        assert_eq!(packed.retained_bytes(), 24 * (3 + 23 * 4));
        packed.write_rgba(&mut out);
        assert_eq!(out, fragmented);
        // Sparse edge pixels can span a whole row: use dense fallback if the
        // row headers would enlarge the original allocation.
        for y in 0..24 {
            fragmented[(y * 24 + 23) * 4 + 3] = 255;
        }
        assert!(CompactTilePixels::try_new(&fragmented).unwrap().is_none());
        assert!(CompactTilePixels::try_new(&[0; RGBA_BYTES - 1]).is_err());
        assert!(CompactTilePixels::try_new(&[0; RGBA_BYTES + 1]).is_err());
    }
}
