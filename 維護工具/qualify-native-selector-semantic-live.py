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
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-semantic-live')

r=read(run/'receipt.json');rb=read(run/'readback.json');assert r['status']==rb['status']=='executed_coherent_candidate_live_selector_semantics'
verify_zip(run/'raw.zip',rb['raw_archive_sha256'],rb['raw_files'])
assert r['owned_process'] is None and not r.get('owned_processes') and all(v['exit_code']==0 for v in r['process_history'])
assert not any(v.get('cleanup_stop_identity') for v in r['process_history'])
assert len(r['process_history'])==6 and not r.get('reused_compile_qualification')
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
assert r['resource_profile']=='tw13081901-v1' and r['extended_item_details']
assert sha(r['reference_jar'])==r['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')==r['reference_jar_sha256']
assert sha(r['common_catalog'])==r['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
server=next(v for v in r['process_history'] if v.get('role')=='server');assert '-Dlito.resourceProfile=tw13081901-v1' in server['arguments'] and '-Dlito.extendedItemDetails=true' in server['arguments']
profile=server['arguments'][-1];assert profile in ['selector-semantic']

assert profile=='selector-semantic' and sha(run/'probe.rs')==r['probe_sha256']
assert sha(ROOT/'維護工具/candidates/native-selector-semantic-live/probe.rs')==r['probe_sha256']
proof=ROOT/'維護工具/candidates/native-selector-semantic-live/source-readback/reviewed-semantic-methods.json';policy=read(proof);assert sha(policy['source'])==policy['source_sha256'] and policy['reference_jar_sha256']==r['reference_jar_sha256'] and len(policy['methods'])==9
result=read(runtime/'result.json');clients={role:read(runtime/(role+'-client.json')) for role in ['a','b','c']};bc=clients['b'];cc=clients['c']
assert result['status']=='sessions_closed' and result['actual_client_sessions']==6 and result['memberships_empty'] and result['authority_mutations']==0 and not result['sql_access'] and not result['state_persistent']
assert all(v['worker_quiescent'] and v['shutdown']=='closed' for v in clients.values())
assert [clients[v]['status'] for v in ['a','b','c']]==['selector_semantic_enemy_completed','selector_semantic_caster_completed','selector_semantic_ally_completed']
config=(runtime/'server.properties').read_text(encoding='utf-8-sig');assert 'database.mode=memory'+chr(10) in config
assert 'database.read.only=true'+chr(10) in config and 'state.persistence.enabled=false'+chr(10) in config and 'battle.ai.enabled=false'+chr(10) in config
initial_clan=read(runtime/'initial-clan-fixture.json');assert initial_clan['before_all_sessions'] and initial_clan['same_clan'] and initial_clan['runtime_mutations']==0 and initial_clan['APIs']==['createClan','addClanCandidate','admitClanCandidate']
raw=[]
for capture in sorted((runtime/'captures').rglob('frames.jsonl')):
 for line_no,line in enumerate(capture.read_text(encoding='utf-8-sig').splitlines(),1):
  packet=json.loads(line)
  if re.fullmatch('[a-fA-F0-9 ]*',packet.get('plain','')) is not None:raw.append((capture,line_no,packet,bytes.fromhex(packet['plain'])))
requests=[(i,row) for i,row in enumerate(raw) if row[2]['direction']=='client_to_server' and row[3][:1]==bytes([20])]
expected=[bytes.fromhex('141905')]*4+[bytes.fromhex('140600')];assert [row[3] for _,row in requests]==expected
assert [v['skill'] for v in bc['actions']]==[206,206,206,206,49] and [v['label'] for v in bc['completed']]==['self','single-other','ambiguous','previous-target','group-heal']
trace=[json.loads(s)['state'] for s in (runtime/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()];authority={i:[v for v in trace if v['object_id']==i and v['phase']=='BATTLE'] for i in [1,3,6]};assert all(authority.values())
for role,id,maxhp in [('a',1,1200),('b',3,1000),('c',6,900)]:
 initial=clients[role]['initial']['state'];assert initial['world']['status']['object_id']==id and initial['world']['hp']==[maxhp,maxhp]
assert bc['initial']['state']['world']['mp']==[1000,1000]
assert all(v['same_clan_as_caster'] for id in [3,6] for v in authority[id]) and not any(v['same_clan_as_caster'] for v in authority[1])
assert {v['source_identity'] for v in authority[3]}=={initial_clan['caster']} and {v['source_identity'] for v in authority[6]}=={initial_clan['ally']}
assert any(v['last_combat_target']==1 for v in authority[3]) and any(v['last_combat_target']==0 for v in authority[3])
assert not any(v['selector_statuses']['timed']['concentrationExpiresAtNanos']>0 for v in authority[6])
for id in [1,3]:assert any(v['selector_statuses']['timed']['concentrationExpiresAtNanos']>0 for v in authority[id])
def dist(state,a,b):
 aa=state['world']['actors'][str(a)];bb=state['world']['actors'][str(b)];return max(abs(aa['x']-bb['x']),abs(aa['y']-bb['y']))
per_request=[];cast_before=[0,1,2,2,3];targets=[3,1,None,1,3]
for index,(request_index,request_row) in enumerate(requests):
 response=bc['completed'][index];before=response['before'];immediate=response['immediately_after_queue'];after=response['after'];action=bc['actions'][index]
 assert action['no_target'] and action['duplicate_blocked'] and action['request_hex']==expected[index].hex() and response['client_duplicate_rejected']
 assert before['world']['hp']==immediate['world']['hp'] and before['world']['mp']==immediate['world']['mp'] and before['inventory']==immediate['inventory']
 assert before['world']['mp'][1]==after['world']['mp'][1]==1000 and before['inventory']==after['inventory']
 assert before['world']['own_cast_packets']==cast_before[index] and after['world']['own_cast_packets']==cast_before[index]+(index!=2)
 if index==0:assert dist(before,3,1)>3 and dist(before,3,6)>3
 if index==1:assert dist(before,3,1)<=3 and dist(before,3,6)>3
 if index in [2,3]:assert dist(before,3,1)<=3 and dist(before,3,6)<=3
 end=requests[index+1][0] if index+1<len(requests) else len(raw);window=raw[request_index+1:end];gfx=759 if index==4 else 6527
 effect_targets={int.from_bytes(b[1:5],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==7 and b[0]==55 and int.from_bytes(b[5:7],'little')==gfx}
 mp=[int.from_bytes(b[1:3],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==5 and b[0]==77 and int.from_bytes(b[3:5],'little')==1000]
 prior_mp=next((int.from_bytes(b[1:3],'little') for _,_,p,b in reversed(raw[:request_index]) if p['direction']=='server_to_client' and len(b)==5 and b[0]==77 and int.from_bytes(b[3:5],'little')==1000),before['world']['mp'][0]);mp_stream=[prior_mp]+mp
 if index==2:
  assert response['feedback']=='NoResponse' and response['elapsed_ms']>=2000 and not effect_targets and all(b>=a for a,b in zip(mp_stream,mp_stream[1:]))
  assert before['world']['last_skill_rejection']==after['world']['last_skill_rejection'] and before['world']['effects']==after['world']['effects']
 else:
  assert response['feedback']=='CastAction' and effect_targets=={targets[index]} and any(a-b==(50 if index==4 else 20) for a,b in zip(mp_stream,mp_stream[1:]))
  assert any(e['graphics']==gfx and e['target'][0]==targets[index] for e in after['world']['effects'])
  assert after['world']['mp'][0] in mp_stream and any(v['mp']==after['world']['mp'][0] for v in authority[3])
 frame=response['render']['frame'];assert frame['skill_effects']['load_error'] is None and frame['skill_effects']['omitted']==0
 per_request.append(dict(skill=action['skill'],label=action['label'],capture_line=request_row[1],feedback=response['feedback'],wire_MP=mp_stream,effect_targets=sorted(effect_targets),own_cast_packets=after['world']['own_cast_packets']))
# No melee input precedes the ambiguous request; a real attack selects target1 before the next 206.
assert not any(p['direction']=='client_to_server' and b[:1] in [bytes([23]),bytes([24])] for _,_,p,b in raw[:requests[2][0]])
between=raw[requests[2][0]:requests[3][0]];melee=[b for _,_,p,b in between if p['direction']=='client_to_server' and b[:1] in [bytes([23]),bytes([24])]];assert len(melee)==1 and int.from_bytes(melee[0][1:5],'little')==1
assert len(bc['attacks'])==1 and bc['attacks'][0]['target']==1 and clients['a']['attacks'] and all(v['target']==6 for v in clients['a']['attacks'])
window=raw[requests[4][0]+1:];prior=raw[:requests[4][0]]
for maxhp,cost in [(1000,30)]:
 hp=[next(int.from_bytes(b[1:3],'little') for _,_,p,b in reversed(prior) if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==maxhp)]
 hp += [int.from_bytes(b[1:3],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==maxhp]
 assert any(a-b==cost for a,b in zip(hp,hp[1:]));caster_hp_stream=hp
ally_hp=[next(int.from_bytes(b[1:3],'little') for _,_,p,b in reversed(prior) if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==900)]
ally_hp += [int.from_bytes(b[1:3],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==900]
assert any(b-a>1 for a,b in zip(ally_hp,ally_hp[1:])) and cc['wounded']['world']['hp'][0]<900 and cc['final']['world']['hp'][0]>cc['wounded']['world']['hp'][0]+1
assert any(v['hp']==cc['final']['world']['hp'][0] for v in authority[6])
enemy_hp=[next(int.from_bytes(b[1:3],'little') for _,_,p,b in reversed(prior) if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==1200)]
enemy_hp += [int.from_bytes(b[1:3],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==5 and b[0]==13 and int.from_bytes(b[3:5],'little')==1200]
assert all(b-a<=1 for a,b in zip(enemy_hp,enemy_hp[1:]))
assert sum(v['feedback']=='CastAction' for v in bc['completed'])==r['actual_new_casts']==4 and len(requests)==r['actual_new_requests']==5
assert any(v['moves'] for v in [clients['a'],clients['c']])
visual=read(run/'visual-observations.json');assert len(visual['images'])==6
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



files={'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json','receipt.json':run/'receipt.json','server-result.json':runtime/'result.json','visual-observations.json':run/'visual-observations.json','semantic-source-policy.json':proof,'initial-clan-fixture.json':runtime/'initial-clan-fixture.json'}
for role in clients:files[role+'-client.json']=runtime/(role+'-client.json')
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
verify_zip(out/'raw.zip',sha(out/'raw.zip'),hashes)
q=dict(status='independently_qualified_actual_C02_candidate206_targeting_and49_sameClan_heal_scope',original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,source_manifest_sha256=r['source_manifest_sha256'],candidate=str(candidate),candidate_metadata_sha256=sha(candidate/'candidate.json'),candidate_files=20,actual_product_sha256=r['product_sha256'],reference_jar_sha256=r['reference_jar_sha256'],resource_profile=r['resource_profile'],actual_new_sessions=6,actual_new_requests=5,actual_new_casts=4,pending_duplicate_inputs_blocked=5,source_backed_ambiguous_no_reply=1,extra_selector_requests_observed=0,per_request=per_request,group_heal_caster_HP_stream=caster_hp_stream,group_heal_ally_HP_stream=ally_hp,server_chosen206_self_fallback_verified=True,server_chosen206_single_other_verified=True,server_chosen206_ambiguous_no_cost_no_reply_verified=True,server_chosen206_previous_combat_target_verified=True,eligible_sameClan_group_heal_verified=True,initial_clan_fixture_only=True,clan_creation_join_UI_verified=False,wire_and_passive_authority_resources_compared=True,client_queue_does_not_spend_authority=True,actual_Model_Presentation_executed=True,recorded_direct_processes=len(r['process_history']),recorded_direct_processes_all_terminal=True,all_recorded_child_exit_codes_zero=True,diagnostic_PNGs_viewed=6,normal_product_rebuilt=False,product_source_changed=False,applied=False,GUI_verified=False,physical_audio_verified=False,full_gameplay_acceptance=False,runner_raw_archive_sha256=rb['raw_archive_sha256'],raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:q[k] for k in ['status','actual_new_sessions','actual_new_requests','actual_new_casts']}))
