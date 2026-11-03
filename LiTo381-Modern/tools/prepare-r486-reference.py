from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r486-paged-city'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
O.mkdir(exist_ok=False);(O/'native-map/563').mkdir(parents=True);(O/'reference').mkdir()
a=read(L/'r485-resource-input.json');b=next(b for b in a['batch']if b['map_id']==563)
assert sorted(p.name for p in Path(b['original_directory']).glob('*.s32'))==[Path(s['pin']['path']).name for s in b['original_sectors']]
sectors=[]
for s in b['original_sectors']:
 assert pin(Path(s['pin']['path']))==s['pin'];dest=O/'native-map/563'/Path(s['pin']['path']).name;shutil.copyfile(s['pin']['path'],dest);sectors.append(s|dict(pin=pin(dest)))
b=b|dict(map_root=str(O/'native-map'),sectors=sectors)
save(L/'r486-resource-input.json',dict(batch=[b],previous_R485_handoff=pin(L/'r485-native-format-two-map-handoff.json'),previous_563_all_required_slot_measurement=pin(P/'target/r485-format-study/budget-reference/reference.json'),fixed_R485_reference_library=read(L/'r485-player-candidate.json')['production_library'],GUI=0,LOADED=False))
t=(P/'tools/r485-complete-map-reference.rs').read_text(encoding='utf-8')
start=t.index(' let mut pixels=BTreeMap::new();');end=t.index(' let keys:BTreeSet<_>=',start)
t=t[:start]+''' let required_slots=required.values().map(|s|s.len()).sum::<usize>();ensure!(id==563&&required_slots==31444&&draws.len()==286665,"complete source qualification count");
 let non_floor_references=draws.iter().filter(|d|d.0.2>=0).count();let live_before_drop=LIVE.load(AO::Relaxed);drop(world);let world_retained_heap=live_before_drop-LIVE.load(AO::Relaxed);
 let budget=json!({"source_bytes":m["source_bytes"],"draw_references":draws.len(),"whole_required_slots":required_slots,"whole_required_decoded_bytes":required_slots*2304,"decoded_limit":67108864,"encoded_resident_limit":33554432,"independent_reference_full_vector":true});
'''+t[end:]
old='let cameras=[("initial",initial),("seam-before",before),("seam-after",after),("min-boundary",(bounds[0],bounds[1])),("max-boundary",(bounds[2]-800,bounds[3]-600))];'
new='let mut cameras=vec![("initial".to_owned(),initial),("seam-before".into(),before),("seam-after".into(),after),("min-boundary".into(),(bounds[0],bounds[1])),("max-boundary".into(),(bounds[2]-800,bounds[3]-600))];for gy in 0..4 {for gx in 0..5 {cameras.push((format!("grid-{gx}-{gy}"),(bounds[0]+(bounds[2]-bounds[0]-800)*gx/4,bounds[1]+(bounds[3]-bounds[1]-600)*gy/3)));}}'
assert old in t;t=t.replace(old,new)
old=' for(name,(cx,cy))in cameras{let mut canvas=Canvas::new(800,600)?;'
new=''' let mut cumulative_keys=BTreeSet::new();let mut max_visible_decoded_bytes=0usize;for(name,(cx,cy))in cameras{let mut needed:BTreeMap<u16,BTreeSet<usize>>=BTreeMap::new();for(_,x,y,id,slot,_,_)in &draws{if *x+24>cx&&*y+24>cy&&*x<cx+800&&*y<cy+600{needed.entry(*id).or_default().insert(*slot as usize);cumulative_keys.insert((*id,*slot));}}let mut pixels=BTreeMap::new();for(id,slots)in &needed{let &(off,len)=index.get(&format!("{id}.til")).context("member absent")?;for(slot,t)in decode_classic_til_selected_with_empty_tail(&pak[off..off+len],TileLimits::default(),slots)?{pixels.insert((*id,slot as u8),t);}}let working_set_bytes=pixels.len()*2304;max_visible_decoded_bytes=max_visible_decoded_bytes.max(working_set_bytes);ensure!(working_set_bytes<=16777216,"visible window exceeds proposed16MiB cache");let mut canvas=Canvas::new(800,600)?;'''
assert old in t;t=t.replace(old,new)
t=t.replace('"camera":[cx,cy,800,600],','"camera":[cx,cy,800,600],"visible_unique_tiles":pixels.len(),"visible_decoded_bytes":working_set_bytes,')
t=t.replace('"prospective_compact12byte_index_payload":non_floor_references*12,','"prospective_compact12byte_index_payload":non_floor_references*12,"cumulative_visible_unique_tiles":cumulative_keys.len(),"cumulative_visible_decoded_bytes":cumulative_keys.len()*2304,"maximum_visible_decoded_bytes":max_visible_decoded_bytes,')
t=t.replace('R485_ALL_COMPLETE_NATIVE_SOURCE_FULL_VECTOR_CSHARP_NEW_TILE_REFERENCE_PASS','R486_FIXED_R485_FULL_GEOMETRY_FRESH_VISIBLE_DECODE_REFERENCE_25VIEWS_PASS')
(P/'tools/r486-map563-reference.rs').write_text(t,encoding='utf-8')
build=(P/'tools/build-r485-map-reference.ps1').read_text(encoding='utf-8')
build=build.replace("$libRoot=Join-Path $project 'target\\r485-production-library';[IO.Directory]::CreateDirectory($libRoot)|Out-Null;$lib=Join-Path $libRoot 'liblito_modern.rlib';[IO.File]::Copy((Artifact 'lito_modern'),$lib,$false)","$fixed=Get-Content -LiteralPath (Join-Path $logs 'r485-player-candidate.json') -Raw|ConvertFrom-Json;$lib=$fixed.production_library.path;if((Get-FileHash -LiteralPath $lib).Hash -ne $fixed.production_library.sha256){throw 'Fixed R485 library drift'}")
build=build.replace("'target\\r485-format-study'","'target\\r486-paged-city'").replace('r485-complete-map-reference','r486-map563-reference').replace('r485_complete_map_reference','r486_map563_reference').replace('r485-reference-','r486-reference-').replace('r485-reference.','r486-reference.').replace('r485-resource-input','r486-resource-input')
(P/'tools/build-r486-reference.ps1').write_text(build,encoding='utf-8')
print('R48619 whole original sectors copied; independent fixed R48525view/working-set reference prepared')
