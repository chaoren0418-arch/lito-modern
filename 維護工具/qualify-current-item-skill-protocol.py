"""Independently bind current item/self-skill protocol cases and retained frame pixels."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,zipfile
from PIL import Image
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('cohort',type=Path);ap.add_argument('--normal-qualification',type=Path);a=ap.parse_args();out=a.cohort.resolve(strict=True);assert out.is_relative_to(R/'驗證紀錄/current')
assert not (out/'qualified.json').exists(),'Preserve existing qualification'
index=read(out/'index.json');assert index['status']=='runtime_completed_pending_independent_qualification';assert len(index['steps'])==10
pins=read(index['source_manifest']);assert len(pins)==968
for n,h in pins.items():assert sha(R/n)==h,n
normal_path=(a.normal_qualification or Path(index.get('normal_qualification',R/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json'))).resolve(strict=True)
assert normal_path.is_relative_to(R/'驗證紀錄/current')
normal=read(normal_path);reference=read(R/'驗證紀錄/current/reference-snapshot.json')
assert normal['features']==[] and normal['source_files']==968 and sha(Path(index['source_manifest']))==normal['source_manifest_sha256']==sha(normal_path.parent/'source-inputs.json')
if 'normal_qualification_sha256' in index:assert index['normal_qualification_sha256']==sha(normal_path)
for profile in ['Debug','Release']:
 main=normal['profiles'][profile];assert main['features']==[] and main['unique_entry'] and main['machine']=='AMD64' and main['optional_header']=='PE32+' and sha(main['path'])==main['sha256']
assert reference['sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539' and sha(reference['source'])==sha(reference['snapshot'])==reference['sha256']
expected={'inventory-actions':['inventory-actions','inventory-actions-refused'],'combat-self':['room-world-shield','room-world-refused'],'item-drop':['room-world-item-drop','room-world-item-drop-refused'],'reagent-skills':['room-world-stream','room-world-stream-no-reagent','room-world-stream-no-mana'],'ground-pickup':['room-world-pickup','room-world-pickup-stale']}
spec=importlib.util.spec_from_file_location('owned_inventory_wire_decoder',R/'維護工具/verify-native-ground-pickup.py');decoder=importlib.util.module_from_spec(spec);spec.loader.exec_module(decoder)
files={'index.json':out/'index.json','qualifier.py':Path(__file__),'inventory-wire-decoder.py':R/'維護工具/verify-native-ground-pickup.py','source-inputs.json':Path(index['source_manifest']),'normal-qualification.json':normal_path};results=[];seen=set();total=0;png_count=0
for step in index['steps']:
 profile,group=step['profile'],step['group'];assert profile in ['Debug','Release'] and group in expected and (profile,group) not in seen;seen.add((profile,group));receipt=Path(step['receipt']).resolve(strict=True);assert receipt.is_relative_to(R/'驗證紀錄/current');d=read(receipt);base=receipt.parent;key=profile+'/'+group
 assert d['schema']==2 and d['profile']==profile and d['compilation_inputs_checked']==968 and d['compilation_source_manifest_sha256']==sha(index['source_manifest'])
 assert d['exe_sha256']==sha(normal['profiles'][profile]['path'])==normal['profiles'][profile]['sha256'] and d['jar_sha256']==reference['sha256']
 assert Path(d['immutable_receipt']).resolve()==receipt and not d['sql_access'] and not d['playable_game']
 assert [c['mode'] for c in d['cases']]==expected[group]
 for name,h in d['source_pins'].items():assert sha(R/name)==h and sha(base/'source'/name)==h,name
 for parity in d['fixture_parity']:
  assert parity['all_non_jar_fields_equal'] and not parity['historical_fixture_relabelled'] and parity['current_jar_sha256']==reference['sha256']
 details=[]
 for case in d['cases']:
  mode=case['mode'];c=case['client'];s=case['server'];assert c==read(base/(mode+'-client.json'))
  assert case['verified'] and c['pointer_bits']==64 and c['worker_quiescent'] and c['shutdown']=='closed' and c['transport']['epoch']=='2' and c['game']['selected_source']=='測試乙'
  runtime=Path(d['runtime'])/mode;assert runtime.resolve().is_relative_to(P/'target/current') and s==read(runtime/'result.json')
  assert s['status']=='verified' and s['actual_client_sessions'] and not s['sql_access'] and not s['state_persistent'] and s['jar_sha256'].upper()==reference['sha256'] and s['mode']==mode
  capture=Path(s['capture_file']).resolve(strict=True);assert capture.is_relative_to(P/'target/current')
  rows=[json.loads(line) for line in capture.read_text(encoding='utf-8-sig').splitlines()];count=sum(x['direction']=='server_to_client' and x.get('note')=='S_KEY' for x in rows);assert count==2
  assert not (runtime/'state.properties').exists()
  replay,_=decoder.replay_inventory(capture)
  view=c['lobby_history'][0]['inventory']['items'] if group=='inventory-actions' else c['combat']['result']['inventory']['items']
  assert replay==view,(profile,group,mode,'wire inventory authority')
  retained=case['presentation_file_pins'];used=set();images=0
  def check_pngs(v):
   global images,png_count
   if isinstance(v,list):
    for x in v:check_pngs(x)
   elif isinstance(v,dict):
    if isinstance(v.get('png'),str) and v['png']:
     name=str(Path(v['png']));assert name in retained,name;meta=retained[name];saved=(base/meta['saved_relative']).resolve(strict=True);assert saved.is_relative_to(base) and sha(saved)==meta['sha256']
     with Image.open(saved) as im:
      assert im.mode=='RGBA' and im.size==(800,600)
      if 'rgba_sha256' in v:assert hashlib.sha256(im.tobytes()).hexdigest().upper()==v['rgba_sha256'].upper()
     used.add(name);images+=1;png_count+=1
    for x in v.values():check_pngs(x)
  check_pngs(c);assert used==set(retained)
  for f in runtime.rglob('*'):
   if f.is_file() and 'oracle' not in f.relative_to(runtime).parts:files[key+'/runtime/'+mode+'/'+f.relative_to(runtime).as_posix()]=f
  details.append({'mode':mode,'actual_ClientSessions':count,'LOGIN_sessions':1,'GAME_sessions':1,'inventory_independent_wire_replay_equal':True,'retained_PNGs':images,'capture_sha256':sha(capture)});total+=count
 for f in base.rglob('*'):
  if f.is_file():files[key+'/output/'+f.relative_to(base).as_posix()]=f
 files[key+'/wrapper.log']=out/(profile+'-'+group+'.log')
 results.append({'profile':profile,'group':group,'receipt':str(receipt),'receipt_sha256':sha(receipt),'cases':details})
assert len(seen)==10 and total==44
file_pins={name:sha(p) for name,p in files.items()};archive=out/'raw-protocol-evidence.zip';assert not archive.exists()
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for name,p in files.items():z.write(p,name)
with zipfile.ZipFile(archive) as z:
 assert set(z.namelist())==set(file_pins)
 for name,h in file_pins.items():
  with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,name
for n,h in pins.items():assert sha(R/n)==h
for main in normal['profiles'].values():assert sha(main['path'])==main['sha256']
proof={'status':'qualified_both_current_normal_mains_five_item_self_skill_groups','source_files':968,'source_manifest_sha256':sha(index['source_manifest']),'normal_mains':{k:v['sha256'] for k,v in normal['profiles'].items()},'normal_qualification_sha256':sha(normal_path),'qualifier_sha256':sha(__file__),'groups_per_profile':5,'cases':22,'actual_ClientSessions':44,'LOGIN_sessions':22,'GAME_sessions':22,'retained_PNGs':png_count,'independent_inventory_wire_replay_cases':22,'groups':results,'raw_file_pins':file_pins,'raw_archive_sha256':sha(archive),'archive_readback_verified':True,'GUI_verified':False,'physical_audio_verified':False,'full_current_TCP_verified':False,'all_skills_or_services_verified':False,'whole_goal_complete':False}
(out/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:proof[k] for k in ['status','cases','actual_ClientSessions','retained_PNGs']},ensure_ascii=False))
