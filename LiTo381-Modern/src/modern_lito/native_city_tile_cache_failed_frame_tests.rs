//! Deterministic OWN-cache fault fixtures. No BIN binding or native semantics.
use super::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::alloc::{GlobalAlloc, Layout, System};
use std::{cell::Cell, sync::{Mutex, MutexGuard}};

static PAYLOAD_LIVE: AtomicUsize = AtomicUsize::new(0);
static PAYLOAD_PEAK: AtomicUsize = AtomicUsize::new(0);
static PAYLOAD_TEST_LOCK: Mutex<()> = Mutex::new(());
thread_local! { static TRACK_PAYLOAD: Cell<bool> = const { Cell::new(false) }; }
struct PayloadScope { _lock: MutexGuard<'static, ()> }
fn payload_scope() -> PayloadScope {
    let lock = PAYLOAD_TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(PAYLOAD_LIVE.load(Ordering::Relaxed), 0);
    PAYLOAD_PEAK.store(0, Ordering::Relaxed);
    TRACK_PAYLOAD.with(|enabled| enabled.set(true));
    PayloadScope { _lock: lock }
}
impl Drop for PayloadScope {
    fn drop(&mut self) {
        TRACK_PAYLOAD.with(|enabled| enabled.set(false));
        if !std::thread::panicking() {
            assert_eq!(PAYLOAD_LIVE.load(Ordering::Relaxed), 0);
        }
    }
}
struct PayloadAllocator;
fn counted(l: Layout) -> bool {
    l.size() == TILE_BYTES && l.align() == 1
        && TRACK_PAYLOAD.try_with(Cell::get).unwrap_or(false)
}
fn add(l: Layout) {
    if counted(l) {
        let n = PAYLOAD_LIVE.fetch_add(TILE_BYTES, Ordering::Relaxed) + TILE_BYTES;
        PAYLOAD_PEAK.fetch_max(n, Ordering::Relaxed);
    }
}
unsafe impl GlobalAlloc for PayloadAllocator {
    unsafe fn alloc(&self,l:Layout)->*mut u8 { let p=unsafe{System.alloc(l)};if !p.is_null(){add(l)}p }
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8 { let p=unsafe{System.alloc_zeroed(l)};if !p.is_null(){add(l)}p }
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){if counted(l){PAYLOAD_LIVE.fetch_sub(TILE_BYTES,Ordering::Relaxed);}unsafe{System.dealloc(p,l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{
        let r=unsafe{System.realloc(p,l,n)};
        if !r.is_null(){if counted(l){PAYLOAD_LIVE.fetch_sub(TILE_BYTES,Ordering::Relaxed);}if n==TILE_BYTES&&l.align()==1{add(Layout::from_size_align(n,1).unwrap());}}
        r
    }
}
#[global_allocator] static PAYLOAD_ALLOCATOR:PayloadAllocator=PayloadAllocator;
fn hash(b:&[u8])->String{format!("{:X}",Sha256::digest(b))}
fn reset_peak(){PAYLOAD_PEAK.store(PAYLOAD_LIVE.load(Ordering::Relaxed),Ordering::Relaxed);}
fn emit(case:&str,details:Value){println!("{}",json!({"case":case,"details":details}));}
fn raw(color:u16)->Vec<u8>{
    let mut bytes=Vec::with_capacity(589);
    bytes.extend(1u32.to_le_bytes());bytes.extend(0u32.to_le_bytes());bytes.extend(577u32.to_le_bytes());
    bytes.push(0);
    for _ in 0..288{bytes.extend(color.to_le_bytes());}
    bytes
}
fn fixture()->TileStorage{
    assert_eq!(PAYLOAD_LIVE.load(Ordering::Relaxed),0);
    let mut members=BTreeMap::new();let mut total=0;
    for (id,color) in [(1,0x7c00),(2,0x03e0),(3,0x001f),(4,0x7fff),(5,0x7fe0)]{
        let bytes=raw(color);total+=bytes.len();
        let input=ValidatedTileInput::new_with_empty_tail(&bytes,TileLimits::default()).unwrap();
        let tile=input.decode_selected(&BTreeSet::from([0]),false).unwrap();
        assert_eq!(tile[&0].rgba.len(),TILE_BYTES);drop(tile);drop(input);
        members.insert(id,EncodedMember{raw:bytes,qualified:BTreeSet::from([0])});
    }
    TileStorage::paged(members,2*TILE_BYTES,5,total).unwrap()
}
fn borrow(storage:&TileStorage)->std::cell::Ref<'_,PagedTiles>{
    match storage{TileStorage::Paged(c)=>c.borrow(),_=>panic!("paged fixture")}
}
fn mutate(storage:&TileStorage, f:impl FnOnce(&mut PagedTiles)){
    match storage{TileStorage::Paged(c)=>f(&mut c.borrow_mut()),_=>panic!("paged fixture")}
}
fn snapshot(storage:&TileStorage)->Value{
    let c=borrow(storage);
    json!({"stamp":c.stamp,"resident":c.resident.iter().map(|(k,e)|json!({"key":k,"stamp":e.stamp,
        "rgba_bytes":e.tile.rgba.len(),"rgba_sha256":hash(&e.tile.rgba)})).collect::<Vec<_>>(),
        "order":c.order.iter().collect::<Vec<_>>(),"metrics":c.metrics,
        "decoded_progress":c.progress.decoded_slots.load(Ordering::Acquire),
        "evicted_progress":c.progress.evicted_slots.load(Ordering::Acquire)})
}
fn coherent(storage:&TileStorage){
    let c=borrow(storage);
    assert_eq!(c.resident.len(),c.order.len(),"resident/order cardinality");
    assert_eq!(c.metrics.resident_slots,c.resident.len());
    let bytes=c.resident.values().map(|e|{assert_eq!(e.tile.rgba.len(),TILE_BYTES);
        assert_eq!(e.tile.rgba.capacity(),TILE_BYTES);TILE_BYTES}).sum::<usize>();
    assert_eq!(bytes,c.metrics.resident_decoded_bytes);
    assert!(bytes<=c.metrics.capacity_bytes);
    assert_eq!(PAYLOAD_LIVE.load(Ordering::Relaxed),bytes,"actual live decoded payload");
    assert!(PAYLOAD_PEAK.load(Ordering::Relaxed)<=c.metrics.capacity_bytes,"residence plus decode temporaries");
    for(k,e)in &c.resident{assert!(e.stamp>0&&e.stamp<=c.stamp);assert!(c.order.contains(&(e.stamp,*k)));}
    for(s,k)in &c.order{assert_eq!(c.resident[k].stamp,*s);}
    assert_eq!(c.resident.values().map(|e|e.stamp).collect::<BTreeSet<_>>().len(),c.resident.len(),"resident stamps are unique");
}
fn fragments(ids:&[u16])->Vec<Fragment>{
    ids.iter().enumerate().map(|(i,id)|Fragment{
        key:NativeDrawCell{half_x:i as i32,y:0}.floor_key(),
        point:PixelPoint{x:(i*24) as i32,y:0},tile:(*id,0)}).collect()
}
fn draw(storage:&TileStorage,ids:&[u16])->Result<Vec<u8>>{
    let mut canvas=Canvas::new(72,24)?;canvas.clear([18,25,35]);
    storage.draw(&fragments(ids),&mut canvas,&AtomicBool::new(false))?;
    Ok(canvas.rgba().to_vec())
}
fn warm()->TileStorage{
    reset_peak();let storage=fixture();draw(&storage,&[1,2]).unwrap();coherent(&storage);storage
}
fn compare_fresh(storage:TileStorage,ids:&[u16],pixels:Vec<u8>){
    coherent(&storage);drop(storage);assert_eq!(PAYLOAD_LIVE.load(Ordering::Relaxed),0);
    reset_peak();let fresh=fixture();let reference=draw(&fresh,ids).unwrap();coherent(&fresh);
    assert_eq!(pixels,reference,"next valid frame must match fresh cache pixels");
    emit("recovery_pixels",json!({"keys":ids,"byte_equal_to_fresh_cache":true,"rgba_sha256":hash(&pixels),
        "payload_peak":PAYLOAD_PEAK.load(Ordering::Relaxed),"capacity":2*TILE_BYTES}));
    drop(fresh);assert_eq!(PAYLOAD_LIVE.load(Ordering::Relaxed),0);
}
#[test]
fn miss_counter_overflow_refuses_before_eviction(){
    let _payload_scope = payload_scope();
    let storage=warm();mutate(&storage,|c|c.metrics.misses=u64::MAX-1);
    let before=snapshot(&storage);reset_peak();
    let error=draw(&storage,&[3,4]).unwrap_err().to_string();let after=snapshot(&storage);
    emit("miss_counter_overflow",json!({"seed":(u64::MAX-1).to_string(),"error":error,
        "before":before,"after":after,"payload_peak":PAYLOAD_PEAK.load(Ordering::Relaxed)}));
    assert!(error.contains("city cache miss counter overflow"));
    coherent(&storage);assert_eq!(before,after,"arithmetic refusal must precede eviction/insertion/bookkeeping");
    mutate(&storage,|c|c.metrics.misses=2);let recovered=draw(&storage,&[3,4]).unwrap();
    compare_fresh(storage,&[3,4],recovered);
}
#[test]
fn completed_frame_overflow_refuses_before_preparation(){
    let _payload_scope = payload_scope();
    let storage=warm();mutate(&storage,|c|c.metrics.completed_frames=u64::MAX);
    let before=snapshot(&storage);reset_peak();
    let error=draw(&storage,&[3,5]).unwrap_err().to_string();let after=snapshot(&storage);
    emit("completed_frame_overflow",json!({"seed":u64::MAX.to_string(),"error":error,"before":before,"after":after}));
    assert!(error.contains("city cache frame counter overflow"));coherent(&storage);
    assert_eq!(before,after,"frame counter refusal must precede preparation/pixel work");
    mutate(&storage,|c|c.metrics.completed_frames=1);let recovered=draw(&storage,&[3,5]).unwrap();
    compare_fresh(storage,&[3,5],recovered);
}
#[test]
fn decode_fault_after_eviction_recovers_and_overbudget_stays_clean(){
    let _payload_scope = payload_scope();
    let storage=warm();
    mutate(&storage,|c|{
        let member=c.members.get_mut(&4).unwrap();
        member.raw.pop();member.raw[8..12].copy_from_slice(&576u32.to_le_bytes());
        emit("owned_decode_fault_input",json!({"key":[4,0],"fixture_only":true,"post_qualification_fault":true,
            "bytes":member.raw.len(),"sha256":hash(&member.raw),"hex":member.raw.iter().map(|b|format!("{b:02X}")).collect::<String>()}));
        assert!(ValidatedTileInput::new_with_empty_tail(&member.raw,TileLimits::default()).is_ok());
    });
    let before=snapshot(&storage);reset_peak();
    let error=draw(&storage,&[3,4]).unwrap_err().to_string();let after=snapshot(&storage);
    emit("partial_decode_failure",json!({"error":error,"before":before,"after":after,
        "payload_peak":PAYLOAD_PEAK.load(Ordering::Relaxed)}));
    assert!(error.contains("truncated TIL"),"fixture must fail during selected decoding");
    coherent(&storage);
    {let c=borrow(&storage);assert_eq!(c.resident.keys().copied().collect::<Vec<_>>(),vec![(3,0)]);
        assert_eq!(c.metrics.misses,3);assert_eq!(c.metrics.evictions,2);assert_eq!(c.metrics.hits,0);
        assert_eq!(c.metrics.completed_frames,1);assert_eq!(c.progress.decoded_slots.load(Ordering::Acquire),3);
        assert_eq!(c.progress.evicted_slots.load(Ordering::Acquire),2);}
    let recovered=draw(&storage,&[3,5]).unwrap();coherent(&storage);
    {let c=borrow(&storage);assert_eq!(c.metrics.hits,1);assert_eq!(c.metrics.misses,4);
        assert_eq!(c.metrics.evictions,2);assert_eq!(c.metrics.completed_frames,2);}
    let before=snapshot(&storage);reset_peak();
    let error=draw(&storage,&[1,2,5]).unwrap_err().to_string();let after=snapshot(&storage);
    assert!(error.contains("city cache visible pixel budget"));assert_eq!(before,after);coherent(&storage);
    let again=draw(&storage,&[3,5]).unwrap();assert_eq!(recovered,again);
    emit("post_failure_overbudget",json!({"error":error,"state_unchanged":before==after,
        "next_valid_pixels_unchanged":true,"before":before,"after":after}));
    compare_fresh(storage,&[3,5],recovered);
}
#[test]
fn existing_stamp_hit_and_eviction_preflights_preserve_state(){
    let _payload_scope = payload_scope();
    for case in ["stamp","hit","eviction"]{
        let storage=warm();
        mutate(&storage,|c|match case{"stamp"=>c.stamp=u64::MAX-1,"hit"=>c.metrics.hits=u64::MAX,
            "eviction"=>c.metrics.evictions=u64::MAX,_=>unreachable!()});
        let before=snapshot(&storage);reset_peak();
        let requested=if case=="hit"{[1,3]}else{[3,4]};
        let error=draw(&storage,&requested).unwrap_err().to_string();let after=snapshot(&storage);
        assert!(error.contains(&format!("city cache {case} counter overflow"))||case=="stamp"&&error.contains("city cache stamp overflow"));
        assert_eq!(before,after);coherent(&storage);
        emit("existing_arithmetic_preflight",json!({"counter":case,"error":error,"before":before,"after":after,
            "state_unchanged":true,"payload_peak":PAYLOAD_PEAK.load(Ordering::Relaxed)}));
        drop(storage);assert_eq!(PAYLOAD_LIVE.load(Ordering::Relaxed),0);
    }
}
#[test]
fn fixture_bytes_and_cancelled_frame_are_explicit_and_recover(){
    let _payload_scope = payload_scope();
    let storage=warm();
    for(id,member)in &borrow(&storage).members{
        emit("owned_fixture_input",json!({"tile_id":id,"fixture_only":true,"bytes":member.raw.len(),
            "sha256":hash(&member.raw),"hex":member.raw.iter().map(|b|format!("{b:02X}")).collect::<String>()}));
    }
    let before=snapshot(&storage);let mut canvas=Canvas::new(72,24).unwrap();canvas.clear([18,25,35]);
    let error=storage.draw(&fragments(&[3,4]),&mut canvas,&AtomicBool::new(true)).unwrap_err().to_string();
    let after=snapshot(&storage);assert_eq!(before,after);coherent(&storage);
    emit("cancelled_before_preparation",json!({"error":error,"before":before,"after":after,"state_unchanged":true}));
    let recovered=draw(&storage,&[1,5]).unwrap();compare_fresh(storage,&[1,5],recovered);
}
