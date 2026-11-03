"""Audit the independently composed native terrain/body samples and current runtime."""
import datetime,hashlib,importlib.util,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'驗證紀錄/current/native-world-layers'
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def pin(p):return {'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)}
def main():
 exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';reference=read('驗證紀錄/current/reference-snapshot.json')
 jar=reference['sha256'];assert jar==sha(Path(reference['source']))==sha(Path(reference['snapshot']))
 r=read('LiTo381-Modern/target/current/verification/native-layer-reference.json');assert r['pixel_comparisons']==8 and len(r['cases'])==8 and r['independent_composition'] and not r['decoder_independent']
 assert r['cases'][1]['terrain_occluded_actor_pixels']>0 and r['cases'][1]['visible_actor_pixels']>0
 pngs=[]
 for c in r['cases']:
  p=Path(c['png']).resolve();p.relative_to(ROOT)
  with Image.open(p) as im:assert im.size==(320,240) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest()==c['rgba_sha256']
  pngs.append(pin(p))
 spec=importlib.util.spec_from_file_location('pickup_audit',ROOT/'維護工具/verify-native-ground-pickup.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
 sources=mod.sprites({'3225-0.spr','773-0.spr'});assert {s['sha256'].lower() for s in sources}==set(r['source_raw_sha256'])
 raw=read('驗證紀錄/current/native-world-layers/native-depth-raw.json');assert sha(Path(raw['native_image']['path']))==raw['native_image']['sha256']
 perf=read('驗證紀錄/current/native-performance/receipt.json');assert perf['exe_sha256']==sha(exe) and perf['jar_sha256']==jar and perf['status']=='verified_layered_composition_timings'
 assert len(perf['historical_pixel_comparisons'])==19 and perf['historical_pixel_changes']>0 and not perf['historical_pixel_identity_required']
 cases=[]
 for receipt in ['native-combat/receipt.json','native-item-actions/receipt.json','native-reagent-skills/receipt.json','native-ground-pickup/receipt.json','native-combat/skills-receipt.json','native-combat/attack-skills-receipt.json','native-combat/ground-skills-receipt.json']:
  x=read(f'驗證紀錄/current/{receipt}');assert x['exe_sha256']==sha(exe) and x['jar_sha256']==jar
  for c in x['cases']:
   combat=c['client'].get('combat')
   if combat is None:
    assert c['mode'] in ['inventory-actions','inventory-actions-refused'];continue
   frames=combat['frames'];assert frames
   for f in frames:
    comp=f['fragment_composition'];assert comp['terrain_and_bodies'] and comp['pixel_ownership'] and comp['roof_runtime'] and comp['effect_interleaving']
    p=f['render_profile_us'];assert 0<p['prepared_terrain_bytes']<=64<<20 and p['hit_surface_bytes']==960000 and p['terrain_fragments']>0
   cases.append({'mode':c['mode'],'frames':len(frames)})
 assert len(cases)==20
 report={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':sha(exe),'jar_sha256':jar,'independent_pixel_comparisons':8,'per_pixel_owner_comparisons':8,'reference':r,'pngs':pngs,'sources':sources,'runtime_cases':cases,'historical_pixel_changes':perf['historical_pixel_changes'],'native_depth_evidence':pin(OUT/'native-depth-raw.json'),'decoder_independent':False,'gui_tested':False,'audio_device_tested':False,'full_native_render_equivalence':False,'limitations':['Native runtime insertion history not reconstructed.','Motion uses nearest whole-cell anchor with residual pixel translation.','Native roof timer queue/pointer alias equivalence, complete dynamic actor flags remain incomplete; modern roofs, neutral actor layers and qualified effects have separate audits.']}
 (OUT/'layers-receipt.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:report[k] for k in ['independent_pixel_comparisons','per_pixel_owner_comparisons','historical_pixel_changes']},ensure_ascii=False))
if __name__=='__main__':main()
