"""Read actual candidate Presentation delivery results without claiming a GUI."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def member(z,n):
 with z.open(n) as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def verify_zip(path,digest,pins):
 assert sha(path)==digest
 with zipfile.ZipFile(path) as z:
  assert len(z.namelist())==len(pins) and set(z.namelist())==set(pins)
  for n,h in pins.items():assert member(z,n)==h,n
def function(text,name):
 start=text.index('fn '+name+'(');return text[start:text.index('\n}\n',start)+3]
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);ap.add_argument('--tool-chunk-id',required=True);a=ap.parse_args();assert a.session_id>0 and a.terminal_exit_code==0
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-nine-legacy-live')

r=read(run/'receipt.json');rb=read(run/'readback.json');assert r['status']==rb['status']=='executed_coherent_candidate_live_selector_requests'
verify_zip(run/'raw.zip',rb['raw_archive_sha256'],rb['raw_files'])
assert r['owned_process'] is None and not r.get('owned_processes') and all(v['exit_code']==0 for v in r['process_history'])
assert not any(v.get('cleanup_stop_identity') for v in r['process_history'])
assert len(r['process_history'])==(3 if r.get('reused_compile_qualification') else 5)
if r.get('reused_compile_qualification'):
 provenance=r['reused_compile_qualification'];assert sha(provenance['path'])==provenance['sha256']
 original_q=read(provenance['path']);assert original_q['profile']=='selector-nine' and original_q['observed_terminal_exit_code']==0
 assert original_q['actual_product_sha256']==r['product_sha256'] and original_q['candidate_metadata_sha256']==sha(Path(r['candidate'])/'candidate.json')
 assert len(provenance['original_compiler_processes'])==2 and all(v['exit_code']==0 for v in provenance['original_compiler_processes'])
 assert r['compiler_executed_this_run'] is False and r['compiler_product_reused'] and r['harness_classes_reused']

inputs=read(run/'compiled-inputs.json');assert len(inputs)==1025
for n,h in inputs.items():assert sha(ROOT/n)==h
candidate=Path(r['candidate']).resolve(strict=True);m=read(candidate/'candidate.json');assert m['after']==r['candidate_after'] and len(m['after'])==20 and not m['applied']
for n,h in m['after'].items():assert sha(candidate/'sources'/Path(n).name)==h
for v in r['libraries'].values():assert sha(v['path'])==v['sha256']
for v in r['baseline_gates'].values():assert sha(v['path'])==v['sha256'] and read(v['path'])['source_manifest_sha256']==r['source_manifest_sha256']
callbacks=Path(r['baseline_gates']['callbacks']['path']);cq=read(callbacks);assert cq['captured_S39_profile_and_lifecycle_verified'] and cq['candidate_metadata_sha256']==sha(candidate/'candidate.json')
product=Path(r['product']).resolve(strict=True);runtime=product.parent;assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current/native-selector-nine-live') and sha(product)==r['product_sha256']
with product.open('rb') as f:
 h=f.read(64);assert h[:2]==b'MZ';f.seek(int.from_bytes(h[60:64],'little'));h=f.read(26);assert h[:4]==b'PE\0\0' and h[4:6]==bytes([100,134]) and h[24:26]==bytes([11,2])
for n,h in rb['raw_files'].items():
 if n.startswith('live-evidence/'):assert sha(runtime/n.removeprefix('live-evidence/'))==h
assert len(r['harness_class_pins'])==2
for path,digest in r['harness_class_pins'].items():assert sha(path)==digest
assert r['resource_profile']=='lito-legacy' and not r['extended_item_details']
assert sha(r['reference_jar'])==r['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')==r['reference_jar_sha256']
assert sha(r['common_catalog'])==r['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
server=next(v for v in r['process_history'] if v.get('role')=='server');assert '-Dlito.resourceProfile=lito-legacy' in server['arguments'] and '-Dlito.extendedItemDetails=false' in server['arguments']
profile=server['arguments'][-1];assert profile in ['selector-nine','selector-nine-refused','selector-nine-missing']
result=read(runtime/'result.json');ac=read(runtime/'a-client.json');bc=read(runtime/'b-client.json');assert result['status']=='sessions_closed' and result['actual_client_sessions']==4 and result['memberships_empty'] and result['authority_mutations']==0 and not result['sql_access'] and not result['state_persistent']
assert all(v['worker_quiescent'] and v['shutdown']=='closed' for v in [ac,bc]);assert ac['status']=='selector_nine_observer_completed' and bc['status']=='selector_nine_requests_completed'
config=(runtime/'server.properties').read_text(encoding='utf-8-sig');assert 'database.mode=memory\n' in config and 'database.read.only=true\n' in config and 'state.persistence.enabled=false\n' in config and 'battle.ai.enabled=false\n' in config
skills=[14,31,32,49,52,54,72,206,78];gfx={14:2170,31:10701,32:2173,49:759,52:3936,54:3104,72:3934,206:6527,78:2234};cost={14:10,31:20,32:5,49:50,52:15,54:60,72:30,206:20,78:30};hp_cost={14:50,32:30,49:30}
assert [v['skill'] for v in bc['actions']]==[v['skill'] for v in bc['completed']]==skills
raw=[]
for capture in sorted((runtime/'captures').rglob('frames.jsonl')):
 for line_no,line in enumerate(capture.read_text(encoding='utf-8-sig').splitlines(),1):
  packet=json.loads(line)
  if re.fullmatch('[a-fA-F0-9 ]*',packet.get('plain','')) is not None:raw.append((capture,line_no,packet,bytes.fromhex(packet['plain'])))
requests=[(i,row) for i,row in enumerate(raw) if row[2]['direction']=='client_to_server' and row[3][:1]==bytes([20])]
expected=[bytes([20,(skill-1)//8,(skill-1)%8]) for skill in skills];assert [row[3] for _,row in requests]==expected
trace=[json.loads(s) for s in (runtime/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()];authority=[v['state'] for v in trace if v['state']['object_id']==3 and v['state']['phase']=='BATTLE'];assert authority
initial=bc['initial']['state'];assert initial['world']['status']['object_id']==3 and initial['world']['hp']==[1000,1000]
assert set(skills)<=set(initial['world']['learned_skills'])
max_mp=initial['world']['mp'][1];assert max_mp==(2 if profile=='selector-nine-refused' else 1000)
gem_ids={item['id'] for v in authority for item in v['native_inventory'] if item['catalog_id']==40318};assert len(gem_ids)==(0 if profile=='selector-nine-missing' else 1)
expected_accepted=set() if profile=='selector-nine-refused' else set(skills)-({31,78} if profile=='selector-nine-missing' else set())
per_skill=[]
for index,(request_index,request_row) in enumerate(requests):
 skill=skills[index];response=bc['completed'][index];before=response['before'];immediate=response['immediately_after_queue'];after=response['after'];action=bc['actions'][index]
 assert action['no_target'] and action['duplicate_blocked'] and action['request_hex']==expected[index].hex() and response['client_duplicate_rejected']
 assert before['world']['hp']==immediate['world']['hp'] and before['world']['mp']==immediate['world']['mp'] and before['inventory']==immediate['inventory']
 assert before['inventory']['owner']==after['inventory']['owner'] and before['inventory']['owner']['character'].startswith('PeerFixture2(')
 end=requests[index+1][0] if index+1<len(requests) else len(raw);window=raw[request_index+1:end]
 caster_mp=[int.from_bytes(b[1:3],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==5 and b[0]==77 and int.from_bytes(b[3:5],'little')==max_mp]
 caster_hp=[int.from_bytes(b[1:3],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==1000]
 prior_mp=next((int.from_bytes(b[1:3],'little') for _,_,p,b in reversed(raw[:request_index]) if p['direction']=='server_to_client' and len(b)==5 and b[0]==77 and int.from_bytes(b[3:5],'little')==max_mp),before['world']['mp'][0])
 prior_hp=next((int.from_bytes(b[1:3],'little') for _,_,p,b in reversed(raw[:request_index]) if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==1000),before['world']['hp'][0])
 mp_stream=[prior_mp]+caster_mp;hp_stream=[prior_hp]+caster_hp
 is_accepted=skill in expected_accepted
 if is_accepted:
  assert response['feedback']=='CastAction' and after['world']['own_cast_packets']==before['world']['own_cast_packets']+1
  assert any(a-b==cost[skill] for a,b in zip(mp_stream,mp_stream[1:])),(skill,mp_stream)
  if skill in hp_cost:assert any(a-b==hp_cost[skill] for a,b in zip(hp_stream,hp_stream[1:])),(skill,hp_stream)
  if skill==72:
   packets=[b for _,_,p,b in window if p['direction']=='server_to_client' and b[:1]==bytes([57])]
   assert packets and all(b[1]==19 and int.from_bytes(b[2:6],'little')==3 and int.from_bytes(b[15:17],'little')==3934 and b[17:20]==bytes([0,0,0]) for b in packets)
   assert after['world']['many_attacks'][-1]['attack']['graphics']==3934 and after['world']['many_attacks'][-1]['attack']['caster']==3
   assert not any(e['graphics']==3934 for e in after['world']['effects'])
   own=before['world']['actors']['3'];assert all([int.from_bytes(b[6:8],'little'),int.from_bytes(b[8:10],'little')]==[own['x'],own['y']] for b in packets)
  else:
   assert any(p['direction']=='server_to_client' and len(b)==7 and b[0]==55 and int.from_bytes(b[1:5],'little')==3 and int.from_bytes(b[5:7],'little')==gfx[skill] for _,_,p,b in window)
   assert any(e['graphics']==gfx[skill] and e['target'][0]==3 for e in after['world']['effects'])
 else:
  code=278 if profile=='selector-nine-refused' else 299
  silent=(profile=='selector-nine-refused' and skill in [49,52,54,72,78]) or (profile=='selector-nine-missing' and skill==31)
  if silent:
   assert response['feedback']=='NoResponse' and response['elapsed_ms']>=2000
   assert after['world']['last_skill_rejection']==before['world']['last_skill_rejection'] and after['world']['message_count']==before['world']['message_count']
   assert not any(p['direction']=='server_to_client' and b[:1]==bytes([16]) for _,_,p,b in window)
   policy_path=(ROOT/'驗證紀錄/current/native-selector-nine-live/20261017-111127-043874/low-MP-source-policy-readback/reviewed-guard-paths.json') if profile=='selector-nine-refused' else run/'reviewed-missing-gem-policy.json'
   policy=read(policy_path)
   assert sha(policy['source'])==policy['source_sha256'] and policy['reference_jar_sha256']==r['reference_jar_sha256']
   assert policy['methods'][str(skill)]['no_protocol_send_on_branch']
  else:
   assert response['feedback']=={'Rejected':code} and after['world']['last_skill_rejection'][1]==code
   assert any(p['direction']=='server_to_client' and len(b)>=4 and b[0]==16 and int.from_bytes(b[1:3],'little')==code for _,_,p,b in window)
  assert after['world']['own_cast_packets']==before['world']['own_cast_packets']
  assert not any(a>b for a,b in zip(mp_stream,mp_stream[1:])) and before['inventory']==after['inventory']
 for gem_id in gem_ids:
  old=before['inventory']['items'][str(gem_id)]['count'];new=after['inventory']['items'][str(gem_id)]['count'];used=(1 if skill==31 else 2 if skill==78 else 0) if is_accepted else 0
  assert old-new==used
  if used:assert any(p['direction']=='server_to_client' and len(b)==9 and b[0]==26 and int.from_bytes(b[1:5],'little')==gem_id and int.from_bytes(b[5:9],'little')==new for _,_,p,b in window)
 assert after['world']['mp'][0] in mp_stream and after['world']['hp'][0] in hp_stream
 assert any(v['mp']==after['world']['mp'][0] and v['hp']==after['world']['hp'][0] for v in authority)
 frame=response['render']['frame'];assert frame['skill_effects']['load_error'] is None and frame['skill_effects']['omitted']==0 and frame['many_effects']['load_error'] is None and frame['many_effects']['omitted']==0
 per_skill.append(dict(skill=skill,request_capture_line=request_row[1],request_hex=expected[index].hex(),accepted=is_accepted,feedback=response['feedback'],wire_MP_stream=mp_stream,wire_HP_stream=hp_stream,client_HP=after['world']['hp'],client_MP=after['world']['mp']))
if profile!='selector-nine-refused':
 assert any(v['selector_statuses']['weight']['decreaseWeightExpiresAtNanos']>0 for v in authority)
 assert any(v['selector_statuses']['timed']['meditationExpiresAtNanos']>0 for v in authority)
 assert any(v['selector_statuses']['timed']['braveMode']==4 and v['selector_statuses']['movement_skill']==52 for v in authority)
 assert any(v['selector_statuses']['timed']['hasteSkillId']==54 for v in authority)
 assert any(v['selector_statuses']['timed']['concentrationExpiresAtNanos']>0 for v in authority)
 assert any(row['graphics']==3934 and row['drawn'] for z in bc['completed'] for row in z['render']['frame']['many_effects']['effects'])
 assert ac['final']['world']['hp'][0]<ac['initial']['state']['world']['hp'][0]
else:assert all(v['selector_statuses']['weight']['decreaseWeightExpiresAtNanos']==0 and v['selector_statuses']['timed']['meditationExpiresAtNanos']==0 and v['selector_statuses']['timed']['hasteSkillId']==0 and v['selector_statuses']['timed']['concentrationExpiresAtNanos']==0 for v in authority)
for field in ['counterMagicExpiresAtNanos','absoluteBarrierExpiresAtNanos']:
 assert any(v['selector_statuses']['timed'][field]>0 for v in authority)==(profile=='selector-nine')
assert sum(v['accepted'] for v in per_skill)==r['actual_new_casts'] and len(per_skill)==r['actual_new_requests']==9
visual=read(run/'visual-observations.json');assert len(visual['images'])==5
for row in visual['images']:
 path=Path(row['path']).resolve(strict=True);assert path.is_relative_to(runtime) and row['viewed'] and sha(path)==row['sha256']
 with Image.open(path) as im:assert im.size==(800,600) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest().upper()==row['rgba_sha256']
ids=[(p['pid'],p['creation_utc']) for p in r['process_history']]
command="$ids=@("+','.join(str(pid) for pid,_ in ids)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
 for pid,created in ids:
  if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
out=run/'qualified';assert not out.exists();out.mkdir();(out/'CIM.json').write_bytes(p.stdout);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())


files={'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json','receipt.json':run/'receipt.json','server-result.json':runtime/'result.json','a-client.json':runtime/'a-client.json','b-client.json':runtime/'b-client.json','visual-observations.json':run/'visual-observations.json'}
if profile=='selector-nine-refused':files['source-negative-policy.json']=ROOT/'驗證紀錄/current/native-selector-nine-live/20261017-111127-043874/low-MP-source-policy-readback/reviewed-guard-paths.json'
if profile=='selector-nine-missing':files['source-negative-policy.json']=run/'reviewed-missing-gem-policy.json'
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
verify_zip(out/'raw.zip',sha(out/'raw.zip'),hashes)
qualified=dict(status='independently_qualified_actual_C02_candidate_nine_selector_requests_scope',original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,source_manifest_sha256=r['source_manifest_sha256'],candidate=str(candidate),candidate_metadata_sha256=sha(candidate/'candidate.json'),candidate_files=20,actual_product_sha256=r['product_sha256'],reference_jar_sha256=r['reference_jar_sha256'],resource_profile=r['resource_profile'],extended_item_details=False,profile=profile,actual_new_sessions=4,actual_new_requests=9,actual_new_casts=sum(v['accepted'] for v in per_skill),expected_rejected_requests=9-sum(v['accepted'] for v in per_skill),pending_duplicate_inputs_blocked=9,explicit_rejected_requests=sum(isinstance(v['feedback'],dict) for v in per_skill),bounded_no_reply_requests=sum(v['feedback']=='NoResponse' for v in per_skill),extra_selector_requests_observed=0,per_skill=per_skill,wire_and_passive_authority_resources_compared=True,status_leases_observed=True,client_queue_does_not_spend_authority=True,counter72_S39_acknowledgement_verified=72 in expected_accepted,counter72_actual_many_effect_visible=72 in expected_accepted,server_chosen206_self_fallback_scope=206 in expected_accepted,eligible_ally_group_heal_verified=False,other_or_ambiguous206_selection_verified=False,post_ack_cooldown_recast_verified=False,actual_Model_Presentation_executed=True,recorded_direct_processes=len(r['process_history']),recorded_direct_processes_all_terminal=True,all_recorded_child_exit_codes_zero=True,diagnostic_PNGs_viewed=5,normal_product_rebuilt=False,product_source_changed=False,applied=False,GUI_verified=False,physical_audio_verified=False,full_gameplay_acceptance=False,runner_raw_archive_sha256=rb['raw_archive_sha256'],raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(qualified,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:qualified[k] for k in ['status','profile','actual_new_sessions','actual_new_requests','actual_new_casts','expected_rejected_requests']},ensure_ascii=False))
