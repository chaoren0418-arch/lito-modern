//! Focused CPU regression for requested city residence below the decode ceiling.
//! No native input, player window, provider, wire, READY, claim or scene authority.
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    native_city_flow::OfflineMapFlow,
    native_multisector_resources::{self as catalog_resources, Inputs},
    native_resources::{BytePin, Limits, city::{CityAssets, LoadTrace, DEFAULT_CACHE_BYTES}},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{alloc::{GlobalAlloc, Layout, System}, fs::{self, OpenOptions}, io::Write,
    path::{Path, PathBuf}, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

const TILE_BYTES: usize = 24 * 24 * 4;
const WHOLE_BYTES: usize = 32_601_600;
const QUALIFIED_SLOTS: usize = 14_150;
const MAP_SHA: &str = "7D454156D49C03B364E1BB71A7C38BE3FE17D8C0935C0BECF5DD0701B439FACE";
const MANIFEST_SHA: &str = "14805BC5D26A88EEAAE3B182A4A477D30AE7B0F26212B760F1CFA29B452DCA6E";
const PNG_SHA: &str = "D7B9C260F35297EE7E9505D2170667C6CE4EE8F8FA77E9C96EFC3AB81E6B7CB5";
const RGBA_SHA: &str = "C2BE6D99900BA66BACF40025E299B75C3315733D578CE16E26C9475249A7C0FF";

// Every successful byte-aligned allocation with the exact qualified RGBA size
// is counted, including decode temporaries before cache insertion. This is a
// conservative allocation-size measurement, not whole-process/OS memory usage.
static SIZE_LIVE: AtomicUsize = AtomicUsize::new(0);
static SIZE_PEAK: AtomicUsize = AtomicUsize::new(0);
struct MeasuredAllocator;
fn counted(layout: Layout) -> bool { layout.size() == TILE_BYTES && layout.align() == 1 }
fn add(layout: Layout) {
    if counted(layout) {
        let live = SIZE_LIVE.fetch_add(TILE_BYTES, Ordering::Relaxed) + TILE_BYTES;
        SIZE_PEAK.fetch_max(live, Ordering::Relaxed);
    }
}
fn remove(layout: Layout) {
    if counted(layout) { SIZE_LIVE.fetch_sub(TILE_BYTES, Ordering::Relaxed); }
}
unsafe impl GlobalAlloc for MeasuredAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() { add(layout); }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() { add(layout); }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        remove(layout);
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, old: Layout, new_size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(pointer, old, new_size) };
        if !result.is_null() {
            remove(old);
            if new_size == TILE_BYTES && old.align() == 1 {
                add(Layout::from_size_align(TILE_BYTES, 1).unwrap());
            }
        }
        result
    }
}
#[global_allocator] static ALLOCATOR: MeasuredAllocator = MeasuredAllocator;
fn reset_peak() { SIZE_PEAK.store(SIZE_LIVE.load(Ordering::Relaxed), Ordering::Relaxed); }
fn allocation_metrics() -> Value {
    json!({"definition":"all live byte-aligned 2304-byte allocations; includes decoded temporaries",
        "live_bytes":SIZE_LIVE.load(Ordering::Relaxed),"peak_bytes":SIZE_PEAK.load(Ordering::Relaxed)})
}
fn hash(bytes: &[u8]) -> String { format!("{:X}", Sha256::digest(bytes)) }
fn new_json(path: &Path, value: &Value) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.write_all(b"\n")?;
    Ok(())
}
fn emit(case: &str, details: Value) { println!("{}", json!({"case":case,"details":details})); }
fn input(project: &Path) -> Result<Inputs> {
    let manifest = project.join("logs/continuation-20261003/r486-map-binding.json");
    let bytes = fs::read(&manifest)?;
    ensure!(hash(&bytes) == MANIFEST_SHA, "qualified manifest drift");
    Ok(Inputs { project: project.into(), manifest,
        manifest_pin: BytePin { bytes: bytes.len() as u64, sha256: Sha256::digest(&bytes).into() } })
}
fn reference(project: &Path) -> Result<Vec<u8>> {
    let path = project.join("target/r483-city-visible-rows/reference/reference-534-initial.png");
    ensure!(hash(&fs::read(&path)?) == PNG_SHA, "retained reference drift");
    let mut decoder = png::Decoder::new(fs::File::open(path)?).read_info()?;
    let mut rgba = vec![0; decoder.output_buffer_size()];
    let frame = decoder.next_frame(&mut rgba)?;
    ensure!((frame.width, frame.height, frame.color_type) == (800, 600, png::ColorType::Rgba), "reference format");
    rgba.truncate(frame.buffer_size());
    ensure!(hash(&rgba) == RGBA_SHA, "reference RGBA identity");
    Ok(rgba)
}
fn bounded(inputs: &Inputs, cancel: &AtomicBool) -> Result<CityAssets> {
    reset_peak();
    let trace = LoadTrace::default();
    let assets = CityAssets::load_catalog_bounded(inputs, 1, 534, Limits::default(), DEFAULT_CACHE_BYTES, cancel, &trace)?;
    let metrics = assets.metrics();
    emit("requested16MiB_below64MiB_decode_ceiling", json!({"metrics":metrics,
        "validated_slots":trace.validated_slots.load(Ordering::Acquire),"allocation":allocation_metrics()}));
    ensure!(metrics.whole_required_decoded_bytes == WHOLE_BYTES && WHOLE_BYTES > DEFAULT_CACHE_BYTES
        && WHOLE_BYTES <= Limits::default().decoded_bytes, "regression interval must be requested < whole <= decode limit");
    // First pre-fix execution deliberately fails HERE; its eager metrics remain raw.
    ensure!(metrics.paged, "DEFECT: requested16MiB ignored; eager {} bytes beneath64MiB decode ceiling", metrics.decoded_bytes);
    ensure!(metrics.qualified_slots == QUALIFIED_SLOTS && trace.validated_slots.load(Ordering::Acquire) == QUALIFIED_SLOTS,
        "all complete-map slots must qualify before exposure");
    ensure!(metrics.decoded_bytes == 0 && metrics.decoded_slots == 0, "paged initial residence must be empty");
    ensure!(assets.geometry().canonical_digest().iter().map(|b| format!("{b:02X}")).collect::<String>() == MAP_SHA,
        "canonical geometry identity");
    let cache = assets.cache_metrics().context("bounded cache absent")?;
    ensure!(cache.capacity_bytes == DEFAULT_CACHE_BYTES && cache.validation_peak_decoded_bytes == TILE_BYTES,
        "requested capacity and one-tile qualification shape");
    ensure!(SIZE_PEAK.load(Ordering::Relaxed) <= DEFAULT_CACHE_BYTES, "qualification transient allocations exceed16MiB");
    Ok(assets)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    ensure!(args.len() == 4, "usage: harness reproduce|verify <canonical-project> <fresh-output>");
    let mode = args[1].to_str().context("ASCII mode")?;
    ensure!(mode == "reproduce" || mode == "verify", "explicit mode");
    let project = PathBuf::from(&args[2]);
    let output = PathBuf::from(&args[3]);
    ensure!(project.is_absolute() && output.starts_with(project.join("target/modern-city-requested-cache-budget-20261008")), "owned output root");
    fs::create_dir_all(&output)?;
    let inputs = input(&project)?;
    let cancel = AtomicBool::new(false);
    let assets = bounded(&inputs, &cancel)?;
    ensure!(mode == "verify", "pre-fix defect was not reproduced; stop for diagnosis");
    let reference = reference(&project)?;
    let camera = assets.initial_camera();
    ensure!((camera.x,camera.y,camera.width,camera.height) == (5744,468,800,600), "initial camera identity");
    let (canvas, frame) = assets.render_measured(camera, &cancel)?;
    ensure!(canvas.rgba() == reference, "paged initial pixels differ from retained reference");
    let cache = assets.cache_metrics().unwrap();
    ensure!(cache.resident_decoded_bytes <= DEFAULT_CACHE_BYTES && cache.peak_resident_decoded_bytes <= DEFAULT_CACHE_BYTES
        && cache.maximum_frame_decoded_bytes <= DEFAULT_CACHE_BYTES && SIZE_PEAK.load(Ordering::Relaxed) <= DEFAULT_CACHE_BYTES,
        "retained/peak/temporary decoded residence exceeds requested16MiB");
    let png_path = output.join("paged-534-initial.png");
    let mut encoder = png::Encoder::new(OpenOptions::new().write(true).create_new(true).open(&png_path)?,800,600);
    encoder.set_color(png::ColorType::Rgba); encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(canvas.rgba())?;
    emit("paged_reference_pixels_and_transient_residence", json!({"cache":cache,"frame":frame,
        "rgba_sha256":hash(canvas.rgba()),"allocation":allocation_metrics(),"image":png_path,"image_sha256":hash(&fs::read(&png_path)?)}));
    drop(canvas); drop(assets);

    reset_peak();
    let trace = LoadTrace::default();
    let tight = CityAssets::load_catalog_bounded(&inputs,1,534,Limits::default(),TILE_BYTES,&cancel,&trace)?;
    ensure!(trace.validated_slots.load(Ordering::Acquire) == QUALIFIED_SLOTS, "tight profile incomplete qualification");
    let before = serde_json::to_value(tight.cache_metrics().unwrap())?;
    let error = match tight.render(camera,&cancel) { Ok(_) => anyhow::bail!("oversized visible frame incorrectly admitted"), Err(e) => format!("{e:#}") };
    ensure!(error.contains("city cache visible pixel budget"), "unexpected tight-budget refusal: {error}");
    let after = serde_json::to_value(tight.cache_metrics().unwrap())?;
    ensure!(before == after, "rejected frame mutated cache/residence/counters");
    emit("tight_frame_budget_clean_refusal", json!({"budget":TILE_BYTES,"error":error,"before":before,"after":after,"validated_slots":QUALIFIED_SLOTS}));
    drop(tight);

    let (catalog,maps,_) = catalog_resources::load(&inputs)?;
    let binding = maps.iter().find(|b|b.mode == 1 && b.plan.map_id == 534).context("qualified534 binding absent")?;
    let eager = CityAssets::load(&binding.sources,&binding.plan,Limits::default(),&cancel,&LoadTrace::default())?;
    let eager_metrics = eager.metrics();
    ensure!(!eager_metrics.paged && eager.cache_metrics().is_none() && eager_metrics.decoded_bytes == WHOLE_BYTES,
        "explicit unbounded API policy changed");
    ensure!(eager.render(camera,&cancel)?.rgba() == reference, "eager/paged/reference pixel mismatch");
    emit("explicit_unbounded_API_consistency", json!({"metrics":eager_metrics,"rgba_sha256":RGBA_SHA}));
    drop(eager);

    // New boundary cases use an existing fully qualified map, never an invented
    // positive binding. This checks the changed predicate's equality semantics.
    for requested in [10_920_960, 10_920_960 + TILE_BYTES] {
        let trace = LoadTrace::default();
        let fitting = CityAssets::load_catalog_bounded(&inputs,2,527,Limits::default(),requested,&cancel,&trace)?;
        let metrics = fitting.metrics();
        ensure!(metrics.qualified_slots == 4_740 && metrics.whole_required_decoded_bytes == 10_920_960
            && trace.validated_slots.load(Ordering::Acquire) == 4_740, "boundary complete-map qualification");
        ensure!(!metrics.paged && fitting.cache_metrics().is_none() && metrics.decoded_bytes == 10_920_960
            && metrics.decoded_bytes <= requested, "complete pixels fitting requested budget must stay eager");
        emit("equality_or_above_whole_remains_eager", json!({"mode":2,"map":527,"requested":requested,"metrics":metrics}));
    }

    reset_peak();
    let mut flow = OfflineMapFlow::new(true,catalog,maps)?;
    flow.select(1,534)?;
    let ready = flow.begin_entry()?.load(&cancel)?;
    let preview = flow.complete_entry(ready)?;
    ensure!(preview.rgba()? == reference, "actual >=16sector flow pixels differ");
    ensure!(SIZE_PEAK.load(Ordering::Relaxed) <= DEFAULT_CACHE_BYTES, "actual bounded flow temporary/residence exceeds16MiB");
    emit("production16sector_bounded_flow", json!({"mode":1,"map":534,"allocation":allocation_metrics(),"rgba_sha256":RGBA_SHA}));
    flow.back()?;
    ensure!(preview.rgba().is_err(), "returned-to-menu frame remains live");
    let summary = json!({"task_id":"modern-city-requested-cache-budget-20261008","state":"review","self_accepted":false,
        "cases":7,"qualified_slots":QUALIFIED_SLOTS,"requested_cache_bytes":DEFAULT_CACHE_BYTES,
        "whole_required_decoded_bytes":WHOLE_BYTES,"decode_limit":Limits::default().decoded_bytes,
        "reference_RGBA":RGBA_SHA,"historical_PASS_replays":0,"GUI":0,"network":0,"provider":0,"READY":0,"claims":0});
    new_json(&output.join("summary.json"),&summary)?;
    emit("summary",summary);
    Ok(())
}
