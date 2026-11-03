"""Verify effect source pins, scalar references, and live common-queue receipts."""
import datetime,hashlib,importlib.util,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'驗證紀錄/current/native-world-effects'
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def pin(p):return {'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)}
def main():
 exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';eh=sha(exe)
 ref=read('驗證紀錄/current/reference-snapshot.json');jh=ref['sha256'];assert jh==sha(Path(ref['source']))==sha(Path(ref['snapshot']))
 spec=importlib.util.spec_from_file_location('actor',ROOT/'維護工具/verify-native-actor-layers.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
 r=read('LiTo381-Modern/target/current/verification/native-world-effects-reference.json');assert r['pixel_comparisons']==r['owner_surface_comparisons']==len(r['cases'])==10
 pins={s['member']:s for s in r['sources']};pngs=[]
 for c in r['cases']:
  assert c['fragment_count']>0;p=Path(c['png']).resolve();p.relative_to(ROOT)
  with Image.open(p) as im:assert im.mode=='RGBA' and im.size==(320,240) and hashlib.sha256(im.tobytes()).hexdigest()==c['rgba_sha256']
  pngs.append(pin(p))
 assert sum(c['terrain_occluded_effect_pixels']>0 for c in r['cases'])>=5
 assert any(c['visible_effect_pixels']>0 for c in r['cases'])
 cases=[];counts={};total=0;frames=0
 for path in ['native-combat/receipt.json','native-item-actions/receipt.json','native-reagent-skills/receipt.json','native-ground-pickup/receipt.json','native-combat/skills-receipt.json','native-combat/attack-skills-receipt.json','native-combat/ground-skills-receipt.json']:
  x=read('驗證紀錄/current/'+path);assert x['exe_sha256']==eh and x['jar_sha256']==jh
  for case in x['cases']:
   combat=case['client'].get('combat')
   if combat is None:assert case['mode'] in ['inventory-actions','inventory-actions-refused'];continue
   submitted=0
   for f in combat['frames']:
    comp=f['fragment_composition'];assert comp['effect_interleaving'] and comp['effect_ownership']=='click_through' and comp['effect_insertion_policy']=='profile_then_active_snapshot_order'
    drawn=0
    for key in ['projectiles','spell_projectiles','skill_effects','item_effects','reagent_effects','ground_effects']:
     fx=f[key];assert fx['load_error'] is None
     for s in fx.get('sources') or []:
      name=s['name'];p={'member':name,'archive':mod.selected(name),'bytes':s['bytes'],'sha256':s['sha256'].upper()}
      if name in pins:assert pins[name]==p
      pins[name]=p
     for e in fx.get('flights',fx.get('effects',[])):
      assert 'error' not in e
      if e['drawn']:
       assert not e.get('finished',False);drawn+=1;counts[str(e['graphics'])]=counts.get(str(e['graphics']),0)+1
    assert comp['effect_submissions']==drawn and 0<=drawn<=512
    submitted+=drawn;frames+=1
   total+=submitted;cases.append({'mode':case['mode'],'frames':len(combat['frames']),'submissions':submitted})
 assert len(cases)==20 and total>0 and all(counts.get(str(g),0)>0 for g in [66,167,219,221,744,2231,168])
 for s in pins.values():mod.verify_source(s)
 evidence=read('驗證紀錄/current/native-world-effects/native-effect-insertion-raw.json');assert sha(Path(evidence['native_image']['path']))==evidence['native_image']['sha256']
 old=read('驗證紀錄/20261012/native-world-roofs-build-receipt.json')
 for key,p in [('native_world_layers','native-layer-reference.json'),('native_actor_layers','native-actor-layer-reference.json')]:
  assert [c['rgba_sha256'] for c in read('LiTo381-Modern/target/current/verification/'+p)['cases']]==[c['rgba_sha256'] for c in old[key]['reference']['cases']]
 result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':eh,'jar_sha256':jh,'independent_pixel_comparisons':10,'per_pixel_owner_comparisons':10,'reference':r,'pngs':pngs,'source_pins':list(pins.values()),'runtime_cases':cases,'runtime_frames':frames,'runtime_submissions':total,'runtime_graphics':counts,'native_evidence':pin(OUT/'native-effect-insertion-raw.json'),'gui_tested':False,'audio_device_tested':False,'full_native_render_equivalence':False,'limits':['Original timer insertion history and dynamic flags remain unqualified.','Effect click-through ownership and profile submission order are explicit modern policies.','Shared decoder; independent scalar composition. RGB555 is not RGB565 parity.','Runtime receipts prove queue submission; positive occlusion uses actual-resource scalar tests.']}
 (OUT/'effects-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'frames':frames,'submissions':total,'graphics':counts,'pixel_comparisons':10}))
if __name__=='__main__':main()
