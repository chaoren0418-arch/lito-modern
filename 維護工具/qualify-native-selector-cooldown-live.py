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
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-cooldown-live')

r=read(run/'receipt.json');rb=read(run/'readback.json');assert r['status']==rb['status']=='executed_coherent_candidate_post_ack_cooldown_requests'
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
assert r['resource_profile']=='tw13081901-v1' and r['extended_item_details']
assert sha(r['reference_jar'])==r['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')==r['reference_jar_sha256']
assert sha(r['common_catalog'])==r['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
server=next(v for v in r['process_history'] if v.get('role')=='server');assert '-Dlito.resourceProfile=tw13081901-v1' in server['arguments'] and '-Dlito.extendedItemDetails=true' in server['arguments']
profile=server['arguments'][-1];assert profile in ['selector-nine','selector-nine-refused','selector-nine-missing']
assert profile=='selector-nine'
assert sha(run/'probe.rs')==r['probe_sha256']
assert sha(ROOT/'維護工具/candidates/native-selector-cooldown-live/probe.rs')==r['probe_sha256']
assert '78=>12000' in (candidate/'sources/native_skills.rs').read_text(encoding='utf-8')
result=read(runtime/'result.json');ac=read(runtime/'a-client.json');bc=read(runtime/'b-client.json')
assert result['status']=='sessions_closed' and result['actual_client_sessions']==4 and result['memberships_empty'] and result['authority_mutations']==0 and not result['sql_access'] and not result['state_persistent']
assert all(v['worker_quiescent'] and v['shutdown']=='closed' for v in [ac,bc])
assert ac['status']=='selector_cooldown_observer_completed' and bc['status']=='selector_cooldown_requests_completed'
assert bc['original_reuse_millis']==12000 and [v['skill'] for v in bc['actions']]==[78,78] and [v['skill'] for v in bc['completed']]==[78,78]
config=(runtime/'server.properties').read_text(encoding='utf-8-sig');assert 'database.mode=memory\n' in config and 'database.read.only=true\n' in config and 'state.persistence.enabled=false\n' in config and 'battle.ai.enabled=false\n' in config
raw=[]
for capture in sorted((runtime/'captures').rglob('frames.jsonl')):
 for line_no,line in enumerate(capture.read_text(encoding='utf-8-sig').splitlines(),1):
  packet=json.loads(line)
  if re.fullmatch('[a-fA-F0-9 ]*',packet.get('plain','')) is not None:raw.append((capture,line_no,packet,bytes.fromhex(packet['plain'])))
requests=[(i,row) for i,row in enumerate(raw) if row[2]['direction']=='client_to_server' and row[3][:1]==bytes([20])]
assert [row[3] for _,row in requests]==[bytes.fromhex('140905')]*2
block=bc['post_ack_blocked'];assert block['body_ready'] and block['error']=='等待施法動作完成' and block['no_local_authority_change']
assert block['before']==block['after'] and 2000<=block['since_first_queue_ms']<12000
assert block['before']['world']['own_cast_packets']==1 and block['before']['world']['mp']==[970,1000]
assert bc['actions'][1]['since_first_ack_ms']>=12100
trace=[json.loads(s)['state'] for s in (runtime/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
authority=[v for v in trace if v['object_id']==3 and v['phase']=='BATTLE'];assert authority
assert bc['initial']['state']['world']['hp']==[1000,1000] and bc['initial']['state']['world']['mp']==[1000,1000]
gem_ids={item['id'] for v in authority for item in v['native_inventory'] if item['catalog_id']==40318};assert len(gem_ids)==1;gem_id=next(iter(gem_ids))
per_request=[]
for index,(request_index,request_row) in enumerate(requests):
 response=bc['completed'][index];before=response['before'];immediate=response['immediately_after_queue'];after=response['after'];action=bc['actions'][index]
 assert action['no_target'] and action['duplicate_blocked'] and action['request_hex']=='140905' and response['client_duplicate_rejected']
 assert before['world']['hp']==immediate['world']['hp'] and before['world']['mp']==immediate['world']['mp'] and before['inventory']==immediate['inventory']
 assert response['feedback']=='CastAction' and before['world']['own_cast_packets']==index and after['world']['own_cast_packets']==index+1
 assert before['world']['mp']==[1000-30*index,1000] and after['world']['mp']==[970-30*index,1000]
 assert before['world']['hp']==after['world']['hp']==[1000,1000]
 assert before['inventory']['items'][str(gem_id)]['count']==8-2*index and after['inventory']['items'][str(gem_id)]['count']==6-2*index
 end=requests[index+1][0] if index+1<len(requests) else len(raw);window=raw[request_index+1:end]
 mp=[int.from_bytes(b[1:3],'little') for _,_,p,b in window if p['direction']=='server_to_client' and len(b)==5 and b[0]==77 and int.from_bytes(b[3:5],'little')==1000]
 assert 970-30*index in mp
 assert any(p['direction']=='server_to_client' and len(b)==9 and b[0]==26 and int.from_bytes(b[1:5],'little')==gem_id and int.from_bytes(b[5:9],'little')==6-2*index for _,_,p,b in window)
 assert any(p['direction']=='server_to_client' and len(b)==7 and b[0]==55 and int.from_bytes(b[1:5],'little')==3 and int.from_bytes(b[5:7],'little')==2234 for _,_,p,b in window)
 assert any(e['graphics']==2234 and e['target'][0]==3 for e in after['world']['effects'])
 assert any(v['mp']==970-30*index and v['hp']==1000 and any(i['catalog_id']==40318 and i['count']==6-2*index for i in v['native_inventory']) for v in authority)
 frame=response['render']['frame'];assert frame['skill_effects']['load_error'] is None and frame['skill_effects']['omitted']==0
 assert any(e['graphics']==2234 and e['drawn'] for e in frame['skill_effects']['effects'])
 per_request.append(dict(skill=78,request_hex='140905',capture_line=request_row[1],feedback=response['feedback'],wire_MP=mp,MP=after['world']['mp'][0],gems=after['inventory']['items'][str(gem_id)]['count']))
leases=sorted({v['selector_statuses']['timed']['absoluteBarrierExpiresAtNanos'] for v in authority if v['selector_statuses']['timed']['absoluteBarrierExpiresAtNanos']>0});assert len(leases)==2 and leases[1]>leases[0]
assert r['actual_new_requests']==r['actual_new_casts']==2 and r['actual_new_sessions']==4
visual=read(run/'visual-observations.json');assert len(visual['images'])==2
for row in visual['images']:
 path=Path(row['path']).resolve(strict=True);assert path.is_relative_to(runtime) and row['viewed'] and sha(path)==row['sha256']
 with Image.open(path) as im:assert im.size==(800,600) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest().upper()==row['rgba_sha256']
terminal=read(run/'terminal-observation.json');assert terminal['original_session_id']==a.session_id and terminal['observed_terminal_exit_code']==a.terminal_exit_code and terminal['terminal_tool_chunk_id']==a.tool_chunk_id
ids=[(p['pid'],p['creation_utc']) for p in r['process_history']]
command="$ids=@("+','.join(str(pid) for pid,_ in ids)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
 for pid,created in ids:
  if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
out=run/'qualified';assert not out.exists();out.mkdir();(out/'CIM.json').write_bytes(p.stdout);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())


files={'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json','receipt.json':run/'receipt.json','server-result.json':runtime/'result.json','a-client.json':runtime/'a-client.json','b-client.json':runtime/'b-client.json','visual-observations.json':run/'visual-observations.json','terminal-observation.json':run/'terminal-observation.json'}
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
verify_zip(out/'raw.zip',sha(out/'raw.zip'),hashes)
q=dict(status='independently_qualified_actual_C02_candidate78_post_ack_cooldown_scope',original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,source_manifest_sha256=r['source_manifest_sha256'],candidate=str(candidate),candidate_metadata_sha256=sha(candidate/'candidate.json'),candidate_files=20,actual_product_sha256=r['product_sha256'],reference_jar_sha256=r['reference_jar_sha256'],resource_profile=r['resource_profile'],actual_new_sessions=4,actual_new_requests=2,actual_new_casts=2,pending_duplicate_inputs_blocked=2,post_ack_cooldown_inputs_blocked=1,extra_selector_requests_observed=0,blocked_after_queue_ms=block['since_first_queue_ms'],blocked_with_body_ready=True,second_request_after_first_ack_ms=bc['actions'][1]['since_first_ack_ms'],original_reuse_millis=12000,per_request=per_request,barrier_lease_expiry_values=leases,wire_and_passive_authority_resources_compared=True,client_queue_does_not_spend_authority=True,post_ack_cooldown_recast_verified=True,actual_Model_Presentation_executed=True,recorded_direct_processes=5,recorded_direct_processes_all_terminal=True,all_recorded_child_exit_codes_zero=True,diagnostic_PNGs_viewed=2,normal_product_rebuilt=False,product_source_changed=False,applied=False,GUI_verified=False,physical_audio_verified=False,full_gameplay_acceptance=False,runner_raw_archive_sha256=rb['raw_archive_sha256'],raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:q[k] for k in ['status','actual_new_sessions','actual_new_requests','actual_new_casts','blocked_after_queue_ms','second_request_after_first_ack_ms']}))
