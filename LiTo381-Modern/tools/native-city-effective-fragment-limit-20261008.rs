//! One new qualified OWN CPU camera, with caller-derived fragment limits.
//! Root links the pinned production library, executes each phase, and seals it.
//! Historical frame models/pixels/source references are not current authority.
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    native_multisector_resources::Inputs,
    native_resources::{city::{CityAssets, FrameMetrics, LoadTrace, DEFAULT_CACHE_BYTES}, BytePin, Limits},
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

const TASK_ID: &str = "modern-city-effective-fragment-limit-20261008";
const PROJECT: &str = r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern";
const CONTRACT: &str = r"D:\lineage開發\3-lito開發\lito脫殼資源\visible-working-set-20261008-contract.json";
const CONTRACT_SHA: &str = "A64644A403F1C9444D745AF35D47515BC1DE2C6D1AA99A8D1BCDFDC60103F069";
const QUALIFIED_INPUT: &str = r"D:\lineage開發\3-lito開發\lito脫殼資源\visible-working-set-20261008\inputs.json";
const QUALIFIED_INPUT_SHA: &str = "6E0FC9717DCEC59985172BD9D3C116B32F709BBE1F3A2F38DF8A9399FA585D79";
const MANIFEST_SHA: &str = "14805BC5D26A88EEAAE3B182A4A477D30AE7B0F26212B760F1CFA29B452DCA6E";
const MAP_SHA: &str = "7D454156D49C03B364E1BB71A7C38BE3FE17D8C0935C0BECF5DD0701B439FACE";
const CALLER_LIMIT: usize = 1024;
const PRE_RETURNED_LIMIT: usize = 262_144;

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
fn camera() -> CameraRect { CameraRect { x:5753, y:489, width:239, height:181 } }

struct Attempt {
    name: &'static str,
    requested_limit: usize,
    frame: Option<FrameMetrics>,
    rgba: Option<Vec<u8>>,
    pin: Option<Value>,
    shape_ok: Option<bool>,
    error: Option<String>,
    cache_before: Value,
    cache_after: Value,
}
impl Attempt {
    fn observation(&self) -> Value {
        json!({"stage":"new_fragment_limit_attempt","name":self.name,
            "camera":[5753,489,239,181],"requested_fragment_limit":self.requested_limit,
            "succeeded":self.frame.is_some(),"frame":self.frame,"rgba":self.pin,
            "surface_shape_ok":self.shape_ok,"error":self.error,
            "cache_before":self.cache_before,"cache_after":self.cache_after,
            "metrics_expected_model":null,"evidence_level":"actual_OWN_CPU_RGBA_and_public_metrics"})
    }
}
fn render_attempt(assets: &CityAssets, limit: usize, name: &'static str,
    out: &Path, observations: &mut File, cancel: &AtomicBool) -> Result<Attempt> {
    let cache_before = serde_json::to_value(assets.cache_metrics())?;
    // Exactly one production render call per named attempt. Even an unexpected
    // success at N-1 retains its pixels, rather than discarding the witness.
    let rendered = assets.render_with_fragment_limit(camera(),limit,cancel);
    let cache_after = serde_json::to_value(assets.cache_metrics())?;
    let attempt = match rendered {
        Ok((canvas,frame)) => {
            let rgba = canvas.rgba().to_vec();
            let pin = write_new(&out.join(format!("{name}.rgba")), &rgba)?;
            let shape_ok = canvas.width() == 239 && canvas.height() == 181
                && rgba.len() == 239 * 181 * 4;
            Attempt { name, requested_limit:limit, frame:Some(frame), rgba:Some(rgba),
                pin:Some(pin), shape_ok:Some(shape_ok), error:None, cache_before, cache_after }
        }
        Err(error) => Attempt { name, requested_limit:limit, frame:None, rgba:None,
            pin:None, shape_ok:None, error:Some(format!("{error:#}")), cache_before, cache_after },
    };
    // The complete returned metrics/cache snapshots and raw bytes are flushed
    // before any success, limit, equality, or refusal acceptance check.
    record(observations, &attempt.observation())?;
    Ok(attempt)
}
fn rgba_equal(a: &Attempt, b: &Attempt) -> bool {
    match (&a.rgba,&b.rgba) { (Some(a),Some(b)) => a == b, _ => false }
}
fn first_difference(a: &Attempt, b: &Attempt) -> Value {
    match (&a.rgba,&b.rgba) {
        (Some(a),Some(b)) => {
            let index = a.iter().zip(b).position(|(a,b)| a != b)
                .or_else(|| (a.len() != b.len()).then_some(a.len().min(b.len())));
            match index {
                Some(index) => json!({"byte_index":index,
                    "camera_local_pixel":[(index/4)%239,(index/4)/239],
                    "channel":(["R","G","B","A"][index%4]),
                    "first_value":a.get(index).copied(),"other_value":b.get(index).copied()}),
                None => Value::Null,
            }
        }
        _ => json!({"comparison_unavailable":"one render did not return RGBA"}),
    }
}
fn reported_limit(attempt: &Attempt, expected: usize) -> bool {
    attempt.frame.is_some_and(|frame| frame.fragment_limit == expected)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    ensure!(args.len() == 4, "usage: harness <canonical-project> <fresh-owned-pre-run-or-post-run> <pre|post>");
    let project = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[2]);
    let phase = args[3].to_str().context("phase must be pre or post")?;
    ensure!(phase == "pre" || phase == "post", "phase must be pre or post");
    ensure!(project == PathBuf::from(PROJECT)
        && out == project.join("target").join(TASK_ID).join(format!("{phase}-run"))
        && std::env::current_dir()?.canonicalize()? == project.canonicalize()?,
        "canonical project/workdir and exact owned phase directory required");
    // Root prepares the parent task directory. Existing phase runs are refused.
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
    // Nested input source refs are historical. Root reconciles the exact linked
    // current library externally; do not read old frame models/keysets/PNGs.
    let manifest = project.join("logs/continuation-20261003/r486-map-binding.json");
    let manifest_raw = read_pin(&manifest,208039,MANIFEST_SHA)?;
    let inputs = Inputs { project:project.clone(), manifest:manifest.clone(),
        manifest_pin:BytePin { bytes:manifest_raw.len() as u64,
            sha256:Sha256::digest(&manifest_raw).into() } };
    let input_record = json!({"task_id":TASK_ID,"phase":phase,"static_contract":{
            "path":CONTRACT,"bytes":contract_raw.len(),"sha256":CONTRACT_SHA},
        "qualified_input_file":{"path":QUALIFIED_INPUT,"bytes":qualified_raw.len(),"sha256":QUALIFIED_INPUT_SHA},
        "valid_ASCII_manifest":{"path":manifest,"bytes":manifest_raw.len(),"sha256":MANIFEST_SHA},
        "mode":1,"map_id":534,"cache_capacity_bytes":DEFAULT_CACHE_BYTES,
        "canonical_geometry_sha256":MAP_SHA,"new_camera":[5753,489,239,181],
        "first_and_retry_caller_limit":CALLER_LIMIT,
        "new_call_sequence":["1024","actual first visible count N","N-1","1024 retry"],
        "expected_first_visible_count_range":"1 < N < 1024",
        "pre_expected_returned_fragment_limit":PRE_RETURNED_LIMIT,
        "post_expected_returned_fragment_limits":"1024, N, refused, 1024",
        "historical_source_refs":"history only; Root reconciles current linked production library",
        "old_models_keysets_and_PNGs_executed":false,
        "cross_phase_comparison":"Root verifies new current-task pre/post raw buffer and metric pins externally"});
    json_new(&out.join("new-inputs.json"), &input_record)?;
    let cancel = AtomicBool::new(false);
    let trace = LoadTrace::default();
    let assets = CityAssets::load_catalog_bounded(&inputs,1,534,Limits::default(),
        DEFAULT_CACHE_BYTES,&cancel,&trace)?;
    let metrics = assets.metrics();
    let validated = trace.validated_slots.load(Ordering::Acquire);
    let canonical_sha: String = assets.geometry().canonical_digest().iter()
        .map(|byte| format!("{byte:02X}")).collect();
    record(&mut observations, &json!({"stage":"complete_static_load","metrics":metrics,
        "cache":assets.cache_metrics(),"validated_slots":validated,
        "canonical_geometry_sha256":canonical_sha,"verified_map_id":assets.verified_plan().map_id,
        "new_input_definition":input_record}))?;
    ensure!(metrics.paged && metrics.qualified_slots == 14150 && validated == 14150
        && metrics.whole_required_decoded_bytes == 32_601_600
        && canonical_sha == MAP_SHA && assets.verified_plan().map_id == 534,
        "complete qualified Map534 gate");

    let first = render_attempt(&assets,CALLER_LIMIT,"first-1024",&out,&mut observations,&cancel)?;
    let n = first.frame.map(|frame| frame.visible_fragments).unwrap_or(0);
    let valid_n = n > 1 && n < CALLER_LIMIT;
    // Without a valid successful first count, N-1 is not the approved input.
    // Retain the first evidence and explicit result before this viability gate.
    if !valid_n {
        let summary = json!({"task_id":TASK_ID,"phase":phase,"passed":false,
            "derived_visible_fragments":n,"valid_derived_count":false,
            "attempts":[first.observation()],"render_call_count":1,
            "refusal":"first call must succeed with 1 < N < 1024 before deriving limits",
            "product_accepted":false,"system_Goal_status":"blocked"});
        json_new(&out.join("results.json"), &summary)?;
        record(&mut observations, &json!({"stage":"summary","phase":phase,"passed":false,"derived_visible_fragments":n}))?;
        ensure!(valid_n,"first frame did not provide approved 1 < N < 1024; actual evidence preserved");
    }
    let exact = render_attempt(&assets,n,"exact-N",&out,&mut observations,&cancel)?;
    let under = render_attempt(&assets,n-1,"under-N-minus-1",&out,&mut observations,&cancel)?;
    let retry = render_attempt(&assets,CALLER_LIMIT,"retry-1024",&out,&mut observations,&cancel)?;

    let first_expected = if phase == "pre" { PRE_RETURNED_LIMIT } else { CALLER_LIMIT };
    let exact_expected = if phase == "pre" { PRE_RETURNED_LIMIT } else { n };
    let retry_expected = if phase == "pre" { PRE_RETURNED_LIMIT } else { CALLER_LIMIT };
    let limits_match = reported_limit(&first,first_expected)
        && reported_limit(&exact,exact_expected) && reported_limit(&retry,retry_expected);
    let exact_and_retry_succeeded = exact.frame.is_some() && retry.frame.is_some();
    let visible_counts_match = exact.frame.is_some_and(|frame| frame.visible_fragments == n)
        && retry.frame.is_some_and(|frame| frame.visible_fragments == n);
    let exact_rgba_equal = rgba_equal(&first,&exact);
    let retry_rgba_equal = rgba_equal(&first,&retry);
    let surfaces_match_camera = first.shape_ok == Some(true)
        && exact.shape_ok == Some(true) && retry.shape_ok == Some(true);
    let expected_refusal = under.error.as_ref().is_some_and(|error| error.contains("city visible fragment budget"))
        && under.frame.is_none();
    let before_completed = under.cache_before["completed_frames"].as_u64();
    let after_completed = under.cache_after["completed_frames"].as_u64();
    let completed_frames_unchanged = before_completed.is_some() && before_completed == after_completed;
    let full_cache_snapshot_unchanged = under.cache_before == under.cache_after;
    let passed = limits_match && exact_and_retry_succeeded && visible_counts_match
        && exact_rgba_equal && retry_rgba_equal && surfaces_match_camera && expected_refusal
        && completed_frames_unchanged && full_cache_snapshot_unchanged;
    let summary = json!({"task_id":TASK_ID,"phase":phase,"state":"review","passed":passed,
        "new_camera":[5753,489,239,181],"derived_visible_fragments":n,"valid_derived_count":valid_n,
        "render_call_count":4,"expected_successful_raw_frame_count":3,
        "actual_successful_raw_frame_count":([&first,&exact,&under,&retry].iter().filter(|a|a.frame.is_some()).count()),
        "attempts":[first.observation(),exact.observation(),under.observation(),retry.observation()],
        "expected_returned_fragment_limits":[first_expected,exact_expected,null,retry_expected],
        "returned_limits_match_phase":limits_match,"exact_and_retry_succeeded":exact_and_retry_succeeded,
        "visible_counts_match":visible_counts_match,"surfaces_match_camera":surfaces_match_camera,
        "exact_rgba_equals_first":exact_rgba_equal,"retry_rgba_equals_first":retry_rgba_equal,
        "exact_first_difference":first_difference(&first,&exact),
        "retry_first_difference":first_difference(&first,&retry),
        "N_minus_1_refused_visible_fragment_budget":expected_refusal,
        "refusal_completed_frames_before":before_completed,"refusal_completed_frames_after":after_completed,
        "refusal_completed_frames_unchanged":completed_frames_unchanged,
        "refusal_full_cache_snapshot_unchanged":full_cache_snapshot_unchanged,
        "final_cache":assets.cache_metrics(),"harness_process_edits_source":false,"harness_process_rebuilds_library":false,
        "old_PASS_replays":0,"self_accepted":false,"consumer_reviewed":false,"product_accepted":false,
        "system_Goal_status":"blocked","evidence_level":"new qualified OWN CPU caller-limit result and refusal evidence",
        "cross_phase_comparison":"external Root size/SHA verified new pre/post evidence only",
        "GUI":0,"native_processes":0,"provider":0,"network":0,"READY":0,"claims":0});
    // Normal success pixels, any unexpected success pixels, all actual metrics,
    // and first differences are already saved. Flush the full result before
    // checking phase acceptance so a failed pre/post expectation stays visible.
    json_new(&out.join("results.json"), &summary)?;
    record(&mut observations, &json!({"stage":"summary","phase":phase,"passed":passed,
        "derived_visible_fragments":n,"render_call_count":4,"product_accepted":false}))?;
    ensure!(exact_and_retry_succeeded && visible_counts_match && surfaces_match_camera,
        "exact/retry render result differed; actual frames and observations preserved");
    ensure!(exact_rgba_equal && retry_rgba_equal,"exact/retry RGBA differed from first; evidence preserved");
    ensure!(expected_refusal && completed_frames_unchanged && full_cache_snapshot_unchanged,
        "N-1 did not refuse before cache mutation; evidence preserved");
    ensure!(limits_match,"returned fragment_limit differs from {phase} phase expectation; actual metrics preserved");
    Ok(())
}
