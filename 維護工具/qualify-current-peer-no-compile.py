"""Qualify current native peer battle/movement cases after the owned build ended."""
from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
normal=ROOT/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json';q=read(normal);pins=read(normal.parent/'source-inputs.json')
for name,h in pins.items():assert sha(ROOT/name)==h,name
assert len(pins)==968
names=[('Debug','melee','20261015-124720-928'),('Debug','movement','20261015-125002-422'),('Release','melee','20261015-125113-796'),('Release','movement','20261015-125324-543')]
out=ROOT/'驗證紀錄/current/native-peer-current-no-compile';out.mkdir(exist_ok=True);assert not (out/'qualified.json').exists()
results=[];files={};sessions=0;melee_images=0;movement_stages=0;minimap_snapshots=0
for profile,action,name in names:
 case=ROOT/'驗證紀錄/current/native-real-peers'/name;r=read(case/'receipt.json');runtime=Path(r['runtime']);main=q['profiles'][profile]
 assert r['profile']==profile and r.get('action_profile','melee')==action and r['validation_completed']
 assert r['exe_sha256']==main['sha256']==sha(main['path'])==sha(runtime/'qualified-client.exe') and r['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
 assert r['source_matches_snapshot'] and r['source_sha256_at_end']==r['jar_sha256']
 s=r['server'];assert s['status']=='sessions_closed' and s['actual_client_sessions']==4 and s['game_sessions']==2 and s['memberships_empty']
 assert s['classic_world_members_after_disconnect']==0 and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
 source_pins=read(case/'source-pins.json')
 for rel,h in source_pins.items():assert sha(PROJECT/rel)==h;files['source/'+rel]=PROJECT/rel
 comparison=read(case/'authority-comparison.json');assert comparison['receipt_sha256'].upper()==sha(case/'receipt.json') and comparison['exe_sha256']==main['sha256']
 captures=list((runtime/'captures').glob('*/frames.jsonl'));assert len(captures)==1;cap=captures[0];rows=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()]
 assert sum(x['direction']=='server_to_client' and x.get('note')=='S_KEY' for x in rows)==4
 for role,c in r['clients'].items():
  assert c['worker_quiescent'] and c['shutdown']=='closed' and c['transport']['epoch']=='2'
  raw=read(case/(role+'-client.json'));observed={k:v for k,v in c.items() if k not in ['process_peak_working_set_bytes','memory_measurement']};assert observed==raw
  assert c['memory_measurement'].startswith('Windows process lifetime peak')
  if action=='melee':
   assert c['status']=='two_real_peer_rounds_completed' and len(c['rounds'])==2
   assert [v['start']['world']['generation'] for v in c['rounds']]==[1,2]
   assert all(v['start']['world']['battle_result'] is None and v['start']['world']['own_attack_packets']==0 for v in c['rounds'])
  else:assert c['status']=='real_peer_world_completed'
 if action=='melee':
  assert comparison['status']=='verified_cross_client_and_server_authority' and len(comparison['checks'])==4
  assert {(c['role'],c['round'],c['winner']) for c in comparison['checks']}=={('a',1,True),('a',2,False),('b',1,False),('b',2,True)}
  count=sum(len(x['images']) for x in comparison['checks']);assert count==8;melee_images+=count
 else:
  assert comparison['status']=='verified_real_peer_world_authority' and len(comparison['checks'])==35;movement_stages+=35
  mini=read(case/'minimap-comparison.json');assert mini['receipt_sha256'].upper()==sha(case/'receipt.json') and mini['unique_wire_markers']==29 and len(mini['checks'])==70;minimap_snapshots+=70
 prefix=profile+'/'+action+'/'
 for file in case.rglob('*'):
  if file.is_file():assert file.resolve().is_relative_to(case) and not file.is_symlink();files[prefix+'output/'+str(file.relative_to(case)).replace('\\','/')]=file
 for file in runtime.rglob('*'):
  if file.is_file() and file.suffix in ['.png','.json','.jsonl','.properties'] and 'oracle' not in file.relative_to(runtime).parts:
   assert file.resolve().is_relative_to(runtime) and not file.is_symlink();files[prefix+'runtime/'+str(file.relative_to(runtime)).replace('\\','/')]=file
 if action=='melee':files['products/'+profile+'/LiToModern.exe']=runtime/'qualified-client.exe'
 sessions+=4;results.append({'profile':profile,'action':action,'case':str(case),'receipt_sha256':sha(case/'receipt.json'),'authority_comparison_sha256':sha(case/'authority-comparison.json'),'capture_sha256':sha(cap),'main_sha256':main['sha256'],'actual_ClientSessions':4,'LOGIN_sessions':2,'GAME_sessions':2,'memberships_empty':True})
files['source-inputs.json']=normal.parent/'source-inputs.json';files['normal-qualification.json']=normal;files['qualifier.py']=Path(__file__)
for name in ['verify-native-real-peers.ps1','verify-real-peer-authority.py','verify-real-peer-world.py','verify-real-peer-minimap.py']:
 p=ROOT/'維護工具'/name
 if p.exists():files['validation-sources/'+name]=p
archive=out/'raw-peer-evidence.zip'
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for name,file in files.items():z.write(file,name)
file_pins={name:sha(file) for name,file in files.items()}
with zipfile.ZipFile(archive) as z:
 for name,h in file_pins.items():
  with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
assert sessions==16 and melee_images==16 and movement_stages==70 and minimap_snapshots==140
for name,h in pins.items():assert sha(ROOT/name)==h,name
proof={'status':'qualified_current_both_normal_mains_two_round_battle_and_movement','cases':results,'source_files':968,'source_manifest_sha256':sha(normal.parent/'source-inputs.json'),'actual_ClientSessions':16,'LOGIN_sessions':8,'GAME_sessions':8,'battle_cases':2,'battle_rounds_per_case':2,'original_battle_PNGs':16,'movement_cases':2,'movement_stages_per_case':35,'minimap_snapshots_per_case':70,'unique_minimap_wire_markers_per_case':29,'probe_deadline_seconds_unchanged':210,'own_compile_absent_during_cases':True,'performance_cause_of_previous_failure_proven':False,'G11_performance_acceptance':False,'GUI_verified':False,'full_current_TCP_regression_verified':False,'full_library_regression_verified':False,'whole_goal_complete':False,'raw_archive_sha256':sha(archive),'raw_archive_readback_verified':True,'raw_file_pins':file_pins,'previous_failed_case_retained':'驗證紀錄/current/native-real-peers/20261015-115846-221/failure-review.json','qualifier_sha256':sha(__file__)}
(out/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Qualified current both mains:two-round battle+35stage movement each,16 actual ClientSessions; previous failure retained,not G11.')
