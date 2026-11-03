"""Combine only independently qualified current candidate scenarios; never normal-product acceptance."""
from pathlib import Path
from datetime import datetime,timezone
import json,hashlib,zipfile,subprocess
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
candidate=ROOT/'維護工具/candidates/native-selector-nine-integration-counter72-s39';cm=read(candidate/'candidate.json');digest=sha(candidate/'candidate.json');assert not cm['applied'] and len(cm['after'])==20
cases=[
 ('TW-positive','native-selector-nine-live/20261017-110133-916636',4,9,9,'tw13081901-v1',True),
 ('TW-low-MP','native-selector-nine-live/20261017-111127-043874',4,9,0,'tw13081901-v1',True),
 ('TW-missing-gems','native-selector-nine-live/20261017-111811-138519',4,9,7,'tw13081901-v1',True),
 ('TW-cooldown','native-selector-cooldown-live/20261017-113609-872103',4,2,2,'tw13081901-v1',True),
 ('TW-targeting-ally-heal','native-selector-semantic-live/20261017-120207-382591',6,5,4,'tw13081901-v1',True),
 ('legacy-positive','native-selector-nine-legacy-live/20261017-120617-421698',4,9,9,'lito-legacy',False),
 ('legacy-low-MP','native-selector-nine-legacy-live/20261017-120845-098329',4,9,0,'lito-legacy',False),
 ('legacy-missing-gems','native-selector-nine-legacy-live/20261017-121221-844149',4,9,7,'lito-legacy',False),
]
items={};identities={};qualification_files={};totals=dict(actual_sessions=0,actual_requests=0,accepted_casts=0,explicit_rejections=0,source_backed_no_reply=0,pending_duplicate_inputs_blocked=0,post_ack_cooldown_inputs_blocked=0,diagnostic_PNGs_viewed=0)
for name,relative,sessions,requests,casts,profile,extended in cases:
 run=ROOT/'驗證紀錄/current'/relative;qpath=run/'qualified/qualified.json';q=read(qpath);rec=read(run/'receipt.json');rb=read(run/'readback.json')
 assert q['observed_terminal_exit_code']==0 and q['candidate_metadata_sha256']==digest and q['source_manifest_sha256']==cm['base_source_manifest_sha256'] and q['candidate_files']==20 and not q['applied'] and not q['GUI_verified'] and not q['physical_audio_verified'] and not q['whole_goal_complete']
 assert (q['actual_new_sessions'],q['actual_new_requests'],q['actual_new_casts'])==(sessions,requests,casts)
 assert q['resource_profile']==rec['resource_profile']==profile and rec['extended_item_details']==extended and rec['candidate_after']==cm['after']
 assert q['reference_jar_sha256']==rec['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
 assert sha(run/'raw.zip')==rb['raw_archive_sha256']==q['runner_raw_archive_sha256']
 archive=run/'qualified/raw.zip';assert sha(archive)==q['raw_archive_sha256']
 with zipfile.ZipFile(archive) as z:
  assert set(z.namelist())==set(q['raw_files'])
  for n,h in q['raw_files'].items():
   with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
 assert sha(rec['product'])==q['actual_product_sha256']
 for n,h in rec['harness_class_pins'].items():assert sha(n)==h
 for row in rec['process_history']:
  assert row['exit_code']==0 and not row.get('cleanup_stop_identity');key=(row['pid'],row['creation_utc']);assert key not in identities;identities[key]=row
 qualification_files[name+'.json']=qpath
 items[name]=dict(qualification=str(qpath),qualification_sha256=sha(qpath),original_session_id=q['original_session_id'],observed_terminal_exit_code=0,terminal_tool_chunk_id=q['terminal_tool_chunk_id'],resource_profile=profile,extended_item_details=extended,product_sha256=q['actual_product_sha256'],sessions=sessions,requests=requests,accepted_casts=casts,diagnostic_PNGs_viewed=q['diagnostic_PNGs_viewed'],recorded_processes=len(rec['process_history']),compiler_product_reused=bool(rec.get('reused_compile_qualification')))
 totals['actual_sessions']+=sessions;totals['actual_requests']+=requests;totals['accepted_casts']+=casts
 totals['explicit_rejections']+=q.get('explicit_rejected_requests',0);totals['source_backed_no_reply']+=q.get('bounded_no_reply_requests',0)+q.get('source_backed_ambiguous_no_reply',0)
 for field in ['pending_duplicate_inputs_blocked','post_ack_cooldown_inputs_blocked','diagnostic_PNGs_viewed']:totals[field]+=q.get(field,0)
 if name=='TW-cooldown':assert q['post_ack_cooldown_recast_verified'] and q['original_reuse_millis']==12000 and q['second_request_after_first_ack_ms']>=12100
 if name=='TW-targeting-ally-heal':
  assert all(q[n] for n in ['server_chosen206_self_fallback_verified','server_chosen206_single_other_verified','server_chosen206_ambiguous_no_cost_no_reply_verified','server_chosen206_previous_combat_target_verified','eligible_sameClan_group_heal_verified']) and q['initial_clan_fixture_only'] and not q['clan_creation_join_UI_verified']
assert totals==dict(actual_sessions=34,actual_requests=61,accepted_casts=38,explicit_rejections=10,source_backed_no_reply=13,pending_duplicate_inputs_blocked=61,post_ack_cooldown_inputs_blocked=1,diagnostic_PNGs_viewed=38),totals
assert len(identities)==31
pins=read(ROOT/'驗證紀錄/current/native-selector-semantic-live/20261017-120207-382591/compiled-inputs.json');assert len(pins)==1025
for n,h in pins.items():assert sha(ROOT/n)==h,n
for n,h in cm['after'].items():assert sha(candidate/'sources'/Path(n).name)==h
normal={}
for profile,h in [('debug','83090FF2520321C15C5E06E770CDF3BB0DFB0D5E19B0407B5D96783459957893'),('release','037C10A6795277624FEF44EE7A202FED028C5F69864EFDEE07559E4E4BF38ECD')]:
 p=ROOT/f'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/{profile}/LiToModern.exe';assert sha(p)==h;normal[profile]=dict(path=str(p),sha256=h)
assert sha(ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
command='$ids=@('+','.join(str(pid) for pid,_ in identities)+');@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n="created";e={$_.CreationDate.ToUniversalTime().ToString("o")}})|ConvertTo-Json -Compress'
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
 for pid,created in identities:
  if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
out=ROOT/'驗證紀錄/current/native-selector-nine-live'/('qualified-eight-cohorts-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'CIM.json').write_bytes(p.stdout);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());files=dict(qualification_files,**{'CIM.json':out/'CIM.json','qualifier.py':out/'qualifier.py'});hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
q=dict(status='independently_combined_current_candidate_eight_C02_scenario_cohorts',candidate=str(candidate),candidate_metadata_sha256=digest,candidate_files=20,source_manifest_sha256=cm['base_source_manifest_sha256'],compiled_input_files=1025,cohorts=items,totals=totals,qualified_scope_only=True,unique_recorded_process_identities=31,all_recorded_processes_terminal=True,all_recorded_exit_codes_zero=True,normal_products=normal,normal_source_unchanged=True,normal_product_unchanged=True,prepared_standard_selector_tests=4,standard_selector_tests_compiled=False,applied=False,GUI_verified=False,physical_audio_verified=False,legacy_specific_multi_target_and_ally_heal_not_executed=True,full_gameplay_acceptance=False,whole_goal_complete=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(output=str(out),status=q['status'],totals=totals),ensure_ascii=False))
