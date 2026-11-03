//! Newly qualified cross-frame CPU consumer checks of unchanged CityAssets/PagedTiles.
//! No GUI/native/runtime acceptance. Only the existing valid ASCII manifest is loaded.
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    native_multisector_resources::Inputs,
    native_resources::{BytePin, Limits, city::{CityAssets, LoadTrace}},
    scene::CameraRect,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

const TILE_BYTES: usize = 2304;
const CONTRACT_SHA: &str = "A64644A403F1C9444D745AF35D47515BC1DE2C6D1AA99A8D1BCDFDC60103F069";
const ASSESSMENT_SHA: &str = "3CF77B7FA39175407E63CEDDA8B4521E5366BB4CB6F5123487AE7C99F7770E16";
const MANIFEST_SHA: &str = "14805BC5D26A88EEAAE3B182A4A477D30AE7B0F26212B760F1CFA29B452DCA6E";
const MAP_SHA: &str = "7D454156D49C03B364E1BB71A7C38BE3FE17D8C0935C0BECF5DD0701B439FACE";
const ALLOCATION_SCOPE: &str = "All live successful size=2304 align=1 allocations, including decoded tile temporaries; conservative size-class payload measurement. Excludes other sizes/alignments, encoded bytes, map/index/container storage, canvas, allocator metadata and OS/process memory. No private resident-key inspection.";
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static FREES: AtomicUsize = AtomicUsize::new(0);
struct PayloadAllocator;
fn counted(l: Layout) -> bool { l.size() == TILE_BYTES && l.align() == 1 }
fn add(l: Layout) {
    if counted(l) {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        let n = LIVE.fetch_add(TILE_BYTES, Ordering::Relaxed) + TILE_BYTES;
        PEAK.fetch_max(n, Ordering::Relaxed);
    }
}
fn remove(l: Layout) {
    if counted(l) {
        FREES.fetch_add(1, Ordering::Relaxed);
        LIVE.fetch_sub(TILE_BYTES, Ordering::Relaxed);
    }
}
unsafe impl GlobalAlloc for PayloadAllocator {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() { add(l); }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() { add(l); }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        remove(l);
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, old: Layout, n: usize) -> *mut u8 {
        let result = unsafe { System.realloc(p, old, n) };
        if !result.is_null() {
            remove(old);
            if n == TILE_BYTES && old.align() == 1 {
                add(Layout::from_size_align(TILE_BYTES, 1).unwrap());
            }
        }
        result
    }
}
#[global_allocator] static ALLOCATOR: PayloadAllocator = PayloadAllocator;
fn reset_peak() { PEAK.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed); }
fn allocation() -> Value {
    json!({"scope":ALLOCATION_SCOPE,"live_bytes":LIVE.load(Ordering::Relaxed),
        "peak_bytes":PEAK.load(Ordering::Relaxed),"allocations":ALLOCS.load(Ordering::Relaxed),
        "deallocations":FREES.load(Ordering::Relaxed)})
}
fn hash(b: &[u8]) -> String { format!("{:X}", Sha256::digest(b)) }
fn pinned(p: &Path, sha: &str, size: Option<u64>) -> Result<Vec<u8>> {
    let b = fs::read(p).with_context(|| format!("read pin {}",p.display()))?;
    ensure!(hash(&b) == sha && size.is_none_or(|n| n == b.len() as u64), "pin differs {}",p.display());
    Ok(b)
}
fn pinned_value(pin: &Value) -> Result<Value> {
    Ok(serde_json::from_slice(&pinned(Path::new(pin["path"].as_str().context("pin path")?),
        pin["sha256"].as_str().context("pin SHA")?,pin["bytes"].as_u64())?)?)
}
fn number(v: &Value, k: &str) -> Result<u64> { v[k].as_u64().with_context(||format!("integer {k}")) }
fn write_new(p: &Path, b: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(p)?;
    f.write_all(b)?;
    Ok(())
}
fn json_new(p: &Path, v: &Value) -> Result<()> { write_new(p,&serde_json::to_vec_pretty(v)?) }
fn record(out: &Path, v: &Value) -> Result<()> {
    let mut f = OpenOptions::new().create(true).append(true).open(out.join("observations.jsonl"))?;
    serde_json::to_writer(&mut f,v)?;
    f.write_all(b"\n")?;
    f.flush()?;
    println!("{v}");
    Ok(())
}
fn keys(frame: &Value) -> Result<BTreeSet<(u16,u8)>> {
    let p = &frame["keys_file"];
    let bytes = pinned(Path::new(p["path"].as_str().context("CSV path")?),
        p["sha256"].as_str().context("CSV SHA")?,p["bytes"].as_u64())?;
    let text = std::str::from_utf8(&bytes)?;
    let mut lines = text.lines();
    ensure!(lines.next() == Some("tile_id,slot,visible_reference_count"),"CSV schema");
    let mut set = BTreeSet::new();
    let mut previous = None;
    let mut total = 0u64;
    let mut packed = Vec::new();
    for line in lines {
        let columns: Vec<_> = line.split(',').collect();
        ensure!(columns.len()==3,"CSV column count");
        let key = (columns[0].parse::<u16>()?,columns[1].parse::<u8>()?);
        let multiplicity = columns[2].parse::<u64>()?;
        ensure!(multiplicity>0 && previous.is_none_or(|last|last<key) && set.insert(key),"CSV sorted unique keys");
        previous=Some(key);
        total=total.checked_add(multiplicity).context("CSV count overflow")?;
        packed.extend_from_slice(&key.0.to_le_bytes()); packed.push(key.1);
    }
    ensure!(set.len() as u64 == number(frame,"unique_keys")?
        && total == number(frame,"visible_fragments")?,"qualified CSV counts");
    ensure!(hash(&packed)==frame["keyset_sha256_le_u16_u8"].as_str().context("key hash")?,"qualified keyset hash");
    ensure!(set.len()*TILE_BYTES==number(frame,"frame_decoded_bytes")? as usize,"qualified key payload");
    Ok(set)
}
fn camera(frame: &Value) -> Result<CameraRect> {
    let a=frame["camera"].as_array().context("camera array")?;
    ensure!(a.len()==4,"camera length");
    Ok(CameraRect{x:i32::try_from(a[0].as_i64().context("camera x")?)?,
        y:i32::try_from(a[1].as_i64().context("camera y")?)?,
        width:u32::try_from(a[2].as_u64().context("camera width")?)?,
        height:u32::try_from(a[3].as_u64().context("camera height")?)?})
}
fn main() -> Result<()> {
    let args:Vec<_>=std::env::args_os().collect();
    ensure!(args.len()==3,"usage: harness <canonical-project> <fresh-output>");
    let project=PathBuf::from(&args[1]); let out=PathBuf::from(&args[2]);
    ensure!(project==PathBuf::from(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern")
        && out==project.join("target/modern-visible-working-set-lru-20261008/run"),"canonical/owned roots");
    fs::create_dir(&out)?;
    let qualified=Path::new(r"D:\lineage開發\3-lito開發\lito脫殼資源\visible-working-set-20261008-contract.json");
    let contract:Value=serde_json::from_slice(&pinned(qualified,CONTRACT_SHA,None)?)?;
    let assessment:Value=serde_json::from_slice(&pinned(Path::new(r"D:\lineage開發\3-lito開發\監製工作流\bin_resources-modern-visible-working-set-input-20261008-assessment.json"),ASSESSMENT_SHA,None)?)?;
    ensure!(contract["task_id"]=="bin_resources-modern-visible-working-set-input-20261008"
        && assessment["decision"]=="accepted","qualified task/decision identity");
    let mut demands=None; let mut model=None;
    for p in contract["evidence"].as_array().context("evidence pins")? {
        let path=Path::new(p["path"].as_str().context("evidence path")?);
        let bytes=pinned(path,p["sha256"].as_str().context("evidence SHA")?,p["bytes"].as_u64())?;
        if path.file_name().is_some_and(|n|n=="frame-demands.json") { demands=Some(serde_json::from_slice::<Value>(&bytes)?); }
        if path.file_name().is_some_and(|n|n=="cache-sequence-model.json") { model=Some(serde_json::from_slice::<Value>(&bytes)?); }
    }
    let _qualified_inputs=pinned_value(&contract["inputs"])?;
    let demands=demands.context("frame demands")?; let model=model.context("cache sequence model")?;
    ensure!(demands==contract["frames"] && model==contract["sequences"],"contract/model consistency");
    let mut keysets=BTreeMap::new();
    for name in ["initial","seam_before","seam_after","minimum","maximum"] {
        keysets.insert(name.to_owned(),keys(&demands[name])?);
    }
    let manifest=project.join("logs/continuation-20261003/r486-map-binding.json");
    let b=pinned(&manifest,MANIFEST_SHA,None)?;
    let inputs=Inputs{project:project.clone(),manifest,
        manifest_pin:BytePin{bytes:b.len() as u64,sha256:Sha256::digest(&b).into()}};
    drop(b);
    let cancel=AtomicBool::new(false); let mut results=Vec::new();
    ensure!(LIVE.load(Ordering::Relaxed)==0,"unexpected counted allocations before resource load");
    for (sequence_index,expected) in model.as_array().context("models")?.iter().enumerate() {
        let capacity=number(expected,"capacity_bytes")? as usize;
        ensure!([16<<20,5_750_784].contains(&capacity) && capacity/TILE_BYTES==number(expected,"max_slots")? as usize,"capacity identity");
        reset_peak(); let allocs=ALLOCS.load(Ordering::Relaxed); let frees=FREES.load(Ordering::Relaxed);
        let trace=LoadTrace::default();
        let assets=CityAssets::load_catalog_bounded(&inputs,1,534,Limits::default(),capacity,&cancel,&trace)?;
        let load=allocation();
        record(&out,&json!({"stage":"load","capacity_bytes":capacity,"metrics":assets.metrics(),
            "cache":assets.cache_metrics(),"validated_slots":trace.validated_slots.load(Ordering::Acquire),"allocation":load}))?;
        ensure!(assets.metrics().paged && assets.metrics().qualified_slots==14150
            && assets.metrics().whole_required_decoded_bytes==32_601_600
            && trace.validated_slots.load(Ordering::Acquire)==14150,"complete city qualification");
        ensure!(assets.geometry().canonical_digest().iter().map(|b|format!("{b:02X}")).collect::<String>()==MAP_SHA,"canonical geometry identity");
        ensure!(assets.verified_plan().map_id==534,"qualified map");
        ensure!(LIVE.load(Ordering::Relaxed)==0 && PEAK.load(Ordering::Relaxed)==TILE_BYTES
            && ALLOCS.load(Ordering::Relaxed)-allocs==14150 && FREES.load(Ordering::Relaxed)-frees==14150,"qualification temporary measurement");
        let frames=expected["frames"].as_array().context("sequence frames")?;
        ensure!(frames.len()==5,"five frames");
        for (index,e) in frames.iter().enumerate() {
            let name=e["frame"].as_str().context("frame name")?;
            ensure!(name==["initial","initial","seam_before","seam_after","initial"][index],"sequence identity");
            let demand=&demands[name]; let needed=keysets[name].len();
            let before=assets.cache_metrics().context("paged cache")?;
            ensure!(LIVE.load(Ordering::Relaxed)==before.resident_decoded_bytes,"pre-frame measured/live residence");
            reset_peak(); let allocs=ALLOCS.load(Ordering::Relaxed); let frees=FREES.load(Ordering::Relaxed);
            let (canvas,frame)=assets.render_measured(camera(demand)?,&cancel)?;
            let after=assets.cache_metrics().context("cache after")?;
            let measured=allocation();
            let rgba_sha=hash(canvas.rgba());
            let observed=json!({"stage":"sequence_frame","capacity_bytes":capacity,"sequence_index":sequence_index,
                "frame_index":index,"camera_name":name,"camera":demand["camera"],"frame":frame,
                "expected_static":e,"qualified_keyset_sha256":demand["keyset_sha256_le_u16_u8"],
                "before":before,"after":after,"hits_delta":after.hits-before.hits,
                "misses_delta":after.misses-before.misses,"evictions_delta":after.evictions-before.evictions,
                "allocation":measured,"decoded_allocations_delta":ALLOCS.load(Ordering::Relaxed)-allocs,
                "decoded_deallocations_delta":FREES.load(Ordering::Relaxed)-frees,"rgba_sha256":rgba_sha,
                "actual_resident_key_identity_inspected":false});
            record(&out,&observed)?;
            ensure!(frame.candidate_fragments==number(demand,"candidate_fragments")? as usize
                && frame.visible_fragments==number(demand,"visible_fragments")? as usize
                && frame.visited_index_rows==number(demand,"visited_index_rows")? as usize,"frame projection counts");
            for (k,got) in [("hits_delta",after.hits-before.hits),("misses_delta",after.misses-before.misses),
                ("evictions_delta",after.evictions-before.evictions),("resident_slots",after.resident_slots as u64),
                ("resident_decoded_bytes",after.resident_decoded_bytes as u64),("cumulative_hits",after.hits),
                ("cumulative_misses",after.misses),("cumulative_evictions",after.evictions)] {
                ensure!(got==number(e,k)?,"observed {k} differs from static model in {name}");
            }
            ensure!(after.hits-before.hits+after.misses-before.misses==needed as u64
                && after.completed_frames==index as u64+1
                && after.completed_frames-before.completed_frames==1,"observed working-set/frame counters");
            ensure!(after.resident_decoded_bytes==after.resident_slots*TILE_BYTES
                && LIVE.load(Ordering::Relaxed)==after.resident_decoded_bytes
                && PEAK.load(Ordering::Relaxed)<=capacity
                && after.peak_resident_decoded_bytes<=capacity
                && after.maximum_frame_decoded_bytes<=capacity,"resident plus decoded temporary capacity");
            ensure!(ALLOCS.load(Ordering::Relaxed)-allocs==(after.misses-before.misses) as usize
                && FREES.load(Ordering::Relaxed)-frees==(after.evictions-before.evictions) as usize,"allocation/deallocation counts");
            if name.starts_with("seam_") {
                let baseline=out.join(format!("{name}-default.rgba"));
                if sequence_index==0 { write_new(&baseline,canvas.rgba())?; }
                else {
                    ensure!(fs::read(&baseline)?==canvas.rgba(),"new seam RGBA byte equality across budgets");
                    record(&out,&json!({"stage":"new_seam_pixel_comparison","camera_name":name,
                        "byte_equal":true,"rgba_bytes":canvas.rgba().len(),"rgba_sha256":rgba_sha,
                        "baseline_path":baseline,"evidence_level":"CPU_RGBA_only","GUI":0}))?;
                }
            }
            results.push(observed);
        }
        ensure!(assets.cache_metrics().unwrap().peak_resident_decoded_bytes
            ==number(expected,"peak_resident_decoded_bytes")? as usize,"observed sequence peak");
        for name in ["minimum","maximum"] {
            let before=assets.cache_metrics().unwrap(); reset_peak();
            let allocs=ALLOCS.load(Ordering::Relaxed); let frees=FREES.load(Ordering::Relaxed);
            let (canvas,frame)=assets.render_measured(camera(&demands[name])?,&cancel)?;
            let after=assets.cache_metrics().unwrap();
            let observed=json!({"stage":"new_empty_boundary_after_sequence","capacity_bytes":capacity,
                "camera_name":name,"frame":frame,"before":before,"after":after,"allocation":allocation(),
                "rgba_sha256":hash(canvas.rgba())});
            record(&out,&observed)?;
            ensure!(frame.visible_fragments==0 && frame.candidate_fragments==5724 && frame.visited_index_rows==53
                && keysets[name].is_empty(),"empty boundary source projection");
            ensure!(after.hits==before.hits && after.misses==before.misses && after.evictions==before.evictions
                && after.resident_slots==before.resident_slots && after.resident_decoded_bytes==before.resident_decoded_bytes
                && after.completed_frames==before.completed_frames+1,"empty frame residence/counters");
            ensure!(ALLOCS.load(Ordering::Relaxed)==allocs && FREES.load(Ordering::Relaxed)==frees
                && LIVE.load(Ordering::Relaxed)==after.resident_decoded_bytes && PEAK.load(Ordering::Relaxed)<=capacity,"empty boundary payload accounting");
            ensure!(canvas.rgba().chunks_exact(4).all(|p|p==[18,25,35,255]),"empty boundary clear pixels");
            results.push(observed);
        }
        drop(assets);
        ensure!(LIVE.load(Ordering::Relaxed)==0,"cache drop releases measured decoded payload");
        record(&out,&json!({"stage":"drop","capacity_bytes":capacity,"live_bytes":0}))?;
    }
    let summary=json!({"task_id":"modern-qualified-visible-working-set-lru-20261008",
        "state":"review","passed":true,"self_accepted":false,"consumer_reviewed":false,"product_complete":false,
        "observed_sequence_frames":10,"observed_new_empty_boundaries":4,"new_seam_byte_equality_checks":2,
        "constrained_eviction_deltas":[0,0,1605,406,1562],"allocation_scope":ALLOCATION_SCOPE,
        "production_source_edits":0,"production_rebuilds":0,"old_PASS_replays":0,"GUI":0,
        "native_processes":0,"provider":0,"network":0,"READY":0,"claims":0,"observations":results});
    json_new(&out.join("results.json"),&summary)?;
    record(&out,&json!({"stage":"summary","passed":true,"observed_sequence_frames":10,
        "new_empty_boundaries":4,"new_seam_byte_equality_checks":2,"evidence_level":"actual_CPU_resource_cache_render"}))?;
    Ok(())
}
