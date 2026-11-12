//! G07.1 (#336): run the real decoders over every member that the resource
//! matrix classifies as a decoder format, and report each disagreement with
//! the matrix verdict. Read-only; ignored by default because it needs the
//! original client resources and `scratch/resource-matrix/members.tsv`, which
//! `維護工具/resource-matrix/resource_matrix.py` regenerates.
//!
//! Output: `scratch/resource-matrix/rust-decode.tsv` (one row per disagreement)
//! and a summary on stdout. The test fails when any disagreement exists, so a
//! pass means "the matrix verdict equals the Rust decoder result per member".
use lito_modern::modern_lito::{
    audio::parse_pcm_wav,
    inventory_icons::decode_tbt,
    map::{parse_native_s32, parse_s32},
    native_mode_menu_skin::{decode_img, CopyMode},
    sprite::decode_classic_spr,
    tile::{decode_classic_til, decode_classic_til_selected_with_empty_tail, TileLimits},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
};

fn client_root() -> PathBuf {
    std::env::var_os("LITO_CLIENT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\lineage開發\0-客戶端"))
}

/// Plain old28 index: lowercase member name to (offset, size).
fn plain_index(path: &Path) -> BTreeMap<String, (u64, usize)> {
    let raw = fs::read(path).unwrap();
    let count = u32::from_le_bytes(raw[..4].try_into().unwrap()) as usize;
    assert_eq!(raw.len(), 4 + count * 28, "old28 length: {}", path.display());
    raw[4..]
        .chunks_exact(28)
        .map(|record| {
            let end = record[4..24].iter().position(|b| *b == 0).unwrap_or(20);
            let name: String = record[4..4 + end]
                .iter()
                .map(|b| (*b as char).to_ascii_lowercase())
                .collect();
            let offset = u64::from(u32::from_le_bytes(record[..4].try_into().unwrap()));
            let size = u32::from_le_bytes(record[24..].try_into().unwrap()) as usize;
            (name, (offset, size))
        })
        .collect()
}

/// `Ok(())` when the real decoder accepts the bytes, otherwise its message.
fn decode(format: &str, verdict: &str, native_map: bool, data: &[u8]) -> Result<(), String> {
    let text = |e: &dyn std::fmt::Display| e.to_string();
    match format {
        "spr" => decode_classic_spr(data).map(drop).map_err(|e| text(&e)),
        "til" if verdict == "til.container_ok_empty_tail" => {
            decode_classic_til_selected_with_empty_tail(
                data,
                TileLimits::default(),
                &BTreeSet::from([0usize]),
            )
            .map(drop)
            .map_err(|e| text(&e))
        }
        "til" => decode_classic_til(data).map(drop).map_err(|e| text(&e)),
        "tbt" | "lbt" => decode_tbt(data).map(drop).map_err(|e| text(&e)),
        "img" => decode_img(data, CopyMode::Opaque).map(drop).map_err(|e| text(&e)),
        "wav" => parse_pcm_wav(data).map(drop).map_err(|e| text(&e)),
        "s32" if native_map => parse_native_s32(data).map(drop).map_err(|e| text(&e)),
        "s32" => parse_s32(data).map(drop).map_err(|e| text(&e)),
        other => panic!("matrix lists an unexpected decoder format: {other}"),
    }
}

#[test]
#[ignore = "requires original client resources and scratch/resource-matrix/members.tsv"]
fn matrix_verdicts_match_real_decoders_for_every_member() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let work = repo.join("scratch").join("resource-matrix");
    let members = fs::read_to_string(work.join("members.tsv"))
        .expect("run 維護工具/resource-matrix/resource_matrix.py first");
    let root = client_root();
    let mut archives: BTreeMap<String, (BTreeMap<String, (u64, usize)>, File)> = BTreeMap::new();
    let mut report = String::from("namespace\tcontainer\tname\tformat\tmatrix_verdict\trust_result\n");
    let (mut checked, mut agreed, mut disagreed) = (0usize, 0usize, 0usize);
    let mut by_format: BTreeMap<String, [usize; 2]> = BTreeMap::new();
    for line in members.lines().skip(1) {
        let column: Vec<&str> = line.split('\t').collect();
        assert_eq!(column.len(), 9, "members.tsv column count");
        let (namespace, container, name, format, verdict, status) =
            (column[0], column[1], column[2], column[5], column[6], column[7]);
        let expected_ok = match status {
            "supported_container_ok" => true,
            "rejected_by_decoder_gate" => false,
            _ => continue,
        };
        if verdict.starts_with("index.") || verdict.starts_with("file.") {
            continue; // No bytes to hand to a decoder.
        }
        let archived = namespace == "SHARED" || namespace == "NATIVE_LITO";
        let data = if archived {
            let (index, pak) = archives.entry(container.to_owned()).or_insert_with(|| {
                let stem = root.join(container);
                (
                    plain_index(&stem.with_extension("idx")),
                    File::open(stem.with_extension("pak")).unwrap(),
                )
            });
            let &(offset, size) = index.get(name).expect("matrix member absent from index");
            let mut data = vec![0; size];
            pak.seek(SeekFrom::Start(offset)).unwrap();
            pak.read_exact(&mut data).unwrap();
            data
        } else {
            fs::read(root.join(container).join(name)).unwrap()
        };
        let native_map = namespace == "NATIVE_LITO/LTmap";
        let result = catch_unwind(AssertUnwindSafe(|| decode(format, verdict, native_map, &data)))
            .unwrap_or_else(|_| Err("PANIC".to_owned()));
        checked += 1;
        let counts = by_format.entry(format.to_owned()).or_default();
        if result.is_ok() == expected_ok {
            agreed += 1;
            counts[0] += 1;
        } else {
            disagreed += 1;
            counts[1] += 1;
            let shown = result.err().unwrap_or_else(|| "accepted".to_owned());
            report.push_str(&format!(
                "{namespace}\t{container}\t{name}\t{format}\t{verdict}\t{shown}\n"
            ));
        }
    }
    File::create(work.join("rust-decode.tsv"))
        .unwrap()
        .write_all(report.as_bytes())
        .unwrap();
    println!("checked={checked} agreed={agreed} disagreed={disagreed}");
    for (format, [same, different]) in &by_format {
        println!("{format}: agreed={same} disagreed={different}");
    }
    assert!(checked > 0, "members.tsv had no decoder-format members");
    assert_eq!(disagreed, 0, "see scratch/resource-matrix/rust-decode.tsv");
}
