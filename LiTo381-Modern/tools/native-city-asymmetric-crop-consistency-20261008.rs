//! Five new OWN CPU views of pinned Map534. Crops test translation/clipping;
//! historical keysets/models/PNGs supply no expected pixels or frame metrics.
//! Root links the accepted current production library and controls execution.
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    native_multisector_resources::Inputs,
    native_resources::{city::{CityAssets, LoadTrace, DEFAULT_CACHE_BYTES}, BytePin, Limits},
    scene::CameraRect,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const TASK_ID: &str = "modern-city-asymmetric-crop-consistency-20261008";
const PROJECT: &str = r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern";
const CONTRACT: &str = r"D:\lineage開發\3-lito開發\lito脫殼資源\visible-working-set-20261008-contract.json";
const CONTRACT_SHA: &str = "A64644A403F1C9444D745AF35D47515BC1DE2C6D1AA99A8D1BCDFDC60103F069";
const QUALIFIED_INPUT: &str = r"D:\lineage開發\3-lito開發\lito脫殼資源\visible-working-set-20261008\inputs.json";
const QUALIFIED_INPUT_SHA: &str = "6E0FC9717DCEC59985172BD9D3C116B32F709BBE1F3A2F38DF8A9399FA585D79";
const MANIFEST_SHA: &str = "14805BC5D26A88EEAAE3B182A4A477D30AE7B0F26212B760F1CFA29B452DCA6E";
const MAP_SHA: &str = "7D454156D49C03B364E1BB71A7C38BE3FE17D8C0935C0BECF5DD0701B439FACE";

fn hash(bytes: &[u8]) -> String { format!("{:X}", Sha256::digest(bytes)) }
fn read_pin(path: &Path, bytes: u64, sha: &str) -> Result<Vec<u8>> {
    let raw = fs::read(path).with_context(|| format!("read pinned {}", path.display()))?;
    ensure!(raw.len() as u64 == bytes && hash(&raw) == sha, "input pin differs {}", path.display());
    Ok(raw)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<Value> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(json!({"path":path,"bytes":bytes.len(),"sha256":hash(bytes)}))
}
fn json_new(path: &Path, value: &Value) -> Result<Value> {
    write_new(path, &serde_json::to_vec_pretty(value)?)
}
fn record(file: &mut File, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    println!("{value}");
    Ok(())
}
fn camera(x: i32, y: i32, width: u32, height: u32) -> CameraRect {
    CameraRect { x, y, width, height }
}
fn camera_array(c: CameraRect) -> [i64; 4] {
    [i64::from(c.x), i64::from(c.y), i64::from(c.width), i64::from(c.height)]
}
struct Frame {
    name: &'static str,
    camera: CameraRect,
    rgba: Vec<u8>,
    pin: Value,
    visible_fragments: usize,
}
fn draw_once(assets: &CityAssets, c: CameraRect, name: &'static str,
    out: &Path, observations: &mut File, cancel: &AtomicBool) -> Result<Frame> {
    let before = assets.cache_metrics();
    let (canvas, frame) = match assets.render_measured(c, cancel) {
        Ok(rendered) => rendered,
        Err(error) => {
            record(observations, &json!({"stage":"render_failed","name":name,
                "camera":camera_array(c),"before":before,"after":assets.cache_metrics(),
                "error":format!("{error:#}")}))?;
            return Err(error.context(format!("render new frame {name}")));
        }
    };
    let rgba = canvas.rgba().to_vec();
    ensure!(canvas.width() == c.width && canvas.height() == c.height
        && rgba.len() == c.width as usize * c.height as usize * 4, "rendered surface shape");
    let non_background_pixels = rgba.chunks_exact(4)
        .filter(|pixel| *pixel != [18,25,35,255]).count();
    let pin = write_new(&out.join(format!("{name}.rgba")), &rgba)?;
    record(observations, &json!({"stage":"new_frame","name":name,
        "camera":camera_array(c),"frame":frame,"cache_before":before,
        "cache_after":assets.cache_metrics(),"rgba":pin,
        "non_background_pixels":non_background_pixels,
        "total_pixels":c.width as usize * c.height as usize,
        "metrics_expected_model":null,"evidence_level":"actual_OWN_CPU_RGBA"}))?;
    Ok(Frame { name, camera:c, rgba, pin, visible_fragments:frame.visible_fragments })
}
/// A row slice of the fresh large RGBA surface. No renderer/culling logic is
/// copied into this oracle, and no prior pixel artifact is opened.
fn crop(large: &Frame, small: &Frame) -> Result<Vec<u8>> {
    let x = i64::from(small.camera.x) - i64::from(large.camera.x);
    let y = i64::from(small.camera.y) - i64::from(large.camera.y);
    ensure!(x >= 0 && y >= 0
        && x + i64::from(small.camera.width) <= i64::from(large.camera.width)
        && y + i64::from(small.camera.height) <= i64::from(large.camera.height),
        "new small camera must lie inside new large camera");
    let stride = large.camera.width as usize * 4;
    let row_bytes = small.camera.width as usize * 4;
    let mut result = Vec::with_capacity(row_bytes * small.camera.height as usize);
    for row in 0..small.camera.height as usize {
        let start = (y as usize + row) * stride + x as usize * 4;
        result.extend_from_slice(&large.rgba[start..start + row_bytes]);
    }
    Ok(result)
}
fn compare_crop(large: &Frame, small: &Frame, out: &Path,
    observations: &mut File) -> Result<Value> {
    let cropped = crop(large, small)?;
    let crop_pin = write_new(&out.join(format!("{}-crop-of-{}.rgba",small.name,large.name)), &cropped)?;
    let first_index = small.rgba.iter().zip(&cropped).position(|(a,b)| a != b)
        .or_else(|| (small.rgba.len() != cropped.len()).then_some(small.rgba.len().min(cropped.len())));
    let first_difference = first_index.map(|index| {
        let pixel = index / 4;
        let x = pixel % small.camera.width as usize;
        let y = pixel / small.camera.width as usize;
        json!({"byte_index":index,"small_pixel":[x,y],
            "camera_pixel":[i64::from(small.camera.x)+x as i64,i64::from(small.camera.y)+y as i64],
            "large_pixel":[i64::from(small.camera.x)-i64::from(large.camera.x)+x as i64,
                i64::from(small.camera.y)-i64::from(large.camera.y)+y as i64],
            "channel":(["R","G","B","A"][index%4]),
            "rendered_value":small.rgba.get(index).copied(),
            "expected_large_crop_value":cropped.get(index).copied()})
    });
    let mismatched_bytes = small.rgba.iter().zip(&cropped).filter(|(a,b)| a != b).count()
        + small.rgba.len().abs_diff(cropped.len());
    let result = json!({"stage":"new_crop_comparison","large":large.name,"small":small.name,
        "large_camera":camera_array(large.camera),"small_camera":camera_array(small.camera),
        "crop_offset":[i64::from(small.camera.x)-i64::from(large.camera.x),
            i64::from(small.camera.y)-i64::from(large.camera.y)],
        "large_rgba":large.pin,"small_rgba":small.pin,"fresh_large_crop_rgba":crop_pin,
        "byte_equal":small.rgba == cropped,"mismatched_bytes":mismatched_bytes,
        "first_difference":first_difference,
        "scope":"OWN translation, visibility/culling and clipping consistency; no native semantic inference"});
    // Persist both buffers and the first mismatch before any equality refusal.
    record(observations, &result)?;
    Ok(result)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    ensure!(args.len() == 3, "usage: harness <canonical-project> <fresh-owned-run-directory>");
    let project = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[2]);
    ensure!(project == PathBuf::from(PROJECT)
        && out == project.join("target/modern-city-asymmetric-crop-consistency-20261008/run")
        && std::env::current_dir()?.canonicalize()? == project.canonicalize()?,
        "canonical project/workdir and exact owned run directory required");
    // Root prepares the parent task directory; an existing run is refused.
    fs::create_dir(&out)?;
    let mut observations = OpenOptions::new().write(true).create_new(true)
        .open(out.join("observations.jsonl"))?;
    let contract_raw = read_pin(Path::new(CONTRACT),15381,CONTRACT_SHA)?;
    let contract: Value = serde_json::from_slice(&contract_raw)?;
    ensure!(contract["task_id"] == "bin_resources-modern-visible-working-set-input-20261008",
        "qualified static contract task identity");
    let declared_input = &contract["inputs"];
    ensure!(declared_input["path"].as_str() == Some(QUALIFIED_INPUT)
        && declared_input["bytes"].as_u64() == Some(1995)
        && declared_input["sha256"].as_str() == Some(QUALIFIED_INPUT_SHA),
        "qualified input file declaration");
    let qualified_raw = read_pin(Path::new(QUALIFIED_INPUT),1995,QUALIFIED_INPUT_SHA)?;
    let _qualified_schema: Value = serde_json::from_slice(&qualified_raw)?;
    // Its nested source refs describe historical projection/cache code. Root
    // reconciles those refs with the accepted current library; do not follow
    // them, historical models, CSV keysets, or PNGs as current expected results.
    let manifest = project.join("logs/continuation-20261003/r486-map-binding.json");
    let manifest_raw = read_pin(&manifest,208039,MANIFEST_SHA)?;
    let inputs = Inputs { project:project.clone(), manifest:manifest.clone(),
        manifest_pin:BytePin { bytes:manifest_raw.len() as u64,
            sha256:Sha256::digest(&manifest_raw).into() } };
    let input_record = json!({"task_id":TASK_ID,"static_contract":{
            "path":CONTRACT,"bytes":contract_raw.len(),"sha256":CONTRACT_SHA},
        "qualified_input_file":{"path":QUALIFIED_INPUT,"bytes":qualified_raw.len(),"sha256":QUALIFIED_INPUT_SHA},
        "valid_ASCII_manifest":{"path":manifest,"bytes":manifest_raw.len(),"sha256":MANIFEST_SHA},
        "mode":1,"map_id":534,"cache_capacity_bytes":DEFAULT_CACHE_BYTES,
        "canonical_geometry_sha256":MAP_SHA,
        "new_cameras":[[5751,481,800,600],[5752,486,257,193],[5752,486,1,193],
            [3759,1105,800,600],[3782,1116,73,61]],
        "expected_invariant":"Each new smaller RGBA surface equals the exact crop of its fresh large RGBA surface",
        "historical_source_refs":"history only, Root reconciles accepted current library",
        "old_models_keysets_and_PNGs_executed":false});
    json_new(&out.join("new-inputs.json"), &input_record)?;
    let cancel = AtomicBool::new(false);
    let trace = LoadTrace::default();
    let assets = CityAssets::load_catalog_bounded(&inputs,1,534,Limits::default(),
        DEFAULT_CACHE_BYTES,&cancel,&trace)?;
    let metrics = assets.metrics();
    let validated = trace.validated_slots.load(Ordering::Acquire);
    // canonical_digest is already SHA256; preserve its bytes as hex.
    let canonical_sha: String = assets.geometry().canonical_digest().iter()
        .map(|byte| format!("{byte:02X}")).collect();
    record(&mut observations, &json!({"stage":"complete_static_load","metrics":metrics,
        "cache":assets.cache_metrics(),"validated_slots":validated,
        "canonical_geometry_sha256":canonical_sha,
        "verified_map_id":assets.verified_plan().map_id,"new_input_definition":input_record}))?;
    ensure!(metrics.paged && metrics.qualified_slots == 14150 && validated == 14150
        && metrics.whole_required_decoded_bytes == 32_601_600
        && canonical_sha == MAP_SHA && assets.verified_plan().map_id == 534,
        "complete qualified Map534 gate");
    let initial_large = draw_once(&assets,camera(5751,481,800,600),"initial-large",&out,&mut observations,&cancel)?;
    let initial_small = draw_once(&assets,camera(5752,486,257,193),"initial-small",&out,&mut observations,&cancel)?;
    let initial_strip = draw_once(&assets,camera(5752,486,1,193),"initial-strip",&out,&mut observations,&cancel)?;
    let seam_large = draw_once(&assets,camera(3759,1105,800,600),"seam-large",&out,&mut observations,&cancel)?;
    let seam_small = draw_once(&assets,camera(3782,1116,73,61),"seam-small",&out,&mut observations,&cancel)?;
    // All five frames and all three comparisons are recorded even if any
    // earlier comparison differs; a failure retains the complete new witness.
    let comparisons = vec![
        compare_crop(&initial_large,&initial_small,&out,&mut observations)?,
        compare_crop(&initial_large,&initial_strip,&out,&mut observations)?,
        compare_crop(&seam_large,&seam_small,&out,&mut observations)?,
    ];
    let crop_equal = comparisons.iter().all(|v| v["byte_equal"] == true);
    let new_frames = [&initial_large,&initial_small,&initial_strip,&seam_large,&seam_small];
    let all_have_visible_fragments = new_frames.iter().all(|frame| frame.visible_fragments > 0);
    let passed = crop_equal && all_have_visible_fragments;
    let summary = json!({"task_id":TASK_ID,"state":"review","passed":passed,
        "new_frame_count":5,"new_crop_comparison_count":3,"comparisons":comparisons,
        "all_have_visible_fragments":all_have_visible_fragments,
        "visible_fragments_by_frame":new_frames.iter().map(|frame|
            json!({"name":frame.name,"visible_fragments":frame.visible_fragments})).collect::<Vec<_>>(),
        "final_cache":assets.cache_metrics(),"production_source_edits":0,
        "production_rebuilds":0,"old_PASS_replays":0,"self_accepted":false,
        "consumer_reviewed":false,"product_accepted":false,"system_Goal_status":"blocked",
        "evidence_level":"new OWN CPU RGBA crop invariants from qualified static resources",
        "GUI":0,"native_processes":0,"provider":0,"network":0,"READY":0,"claims":0});
    json_new(&out.join("results.json"), &summary)?;
    record(&mut observations, &json!({"stage":"summary","passed":passed,
        "new_frame_count":5,"new_crop_comparison_count":3,"product_accepted":false}))?;
    ensure!(all_have_visible_fragments,"new camera had zero visible fragments; all buffers/observations preserved");
    ensure!(crop_equal,"new OWN asymmetric subview differs from fresh large crop; buffers and first differences preserved");
    Ok(())
}
