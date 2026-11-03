"""Audit the complete current14 GUI cohort and retain all raw owned evidence."""
from pathlib import Path
import argparse,hashlib,json,shutil,zipfile
from native_lighting_GUI_evidence import load_scope
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('cohort',type=Path);ap.add_argument('--normal-qualification',type=Path,required=True);ap.add_argument('--debug-rust-evidence',type=Path,required=True);ap.add_argument('--release-rust-evidence',type=Path,required=True);ap.add_argument('--resource-profile',choices=['lito-legacy','tw13081901-v1'],default='lito-legacy');ap.add_argument('--extended-item-details',action='store_true');a=ap.parse_args();cohort=a.cohort.resolve(strict=True)
assert cohort.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-current');assert not (cohort/'qualified.json').exists()
normal=a.normal_qualification.resolve(strict=True);assert normal.is_relative_to(ROOT/'驗證紀錄/current');n=read(normal);pins=read(normal.parent/'source-inputs.json')
assert len(pins)==n['source_files'] and len(pins) in [994,999,1002,1005] and sha(normal.parent/'source-inputs.json')==n['source_manifest_sha256']
for name,h in pins.items():assert sha(ROOT/name)==h,name
expected=['cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry'];jar_sha='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
reference=read(ROOT/'驗證紀錄/current/reference-snapshot.json');assert reference['sha256']==sha(reference['snapshot'])==sha(reference['source'])==jar_sha
files={};profiles={};totals={k:0 for k in ['actual_formal_sessions','exact_C_MOVE','extra_C_MOVE','original_world_PNGs','original_glyph_matches','stale_GUI_notifications']};login_sessions=game_sessions=0
for profile in ['Debug','Release']:
 batch_file=cohort/(profile+'-current-cohort.json');batch=read(batch_file);main=n['profiles'][profile]
 assert batch['status']=='verified_current_lighting_main_GUI_selected_teleport_scenarios' and [x['scenario'] for x in batch['cases']]==expected
 assert batch['resource_profile']==a.resource_profile and batch['extended_item_details'] is a.extended_item_details
 if a.resource_profile=='tw13081901-v1':assert a.extended_item_details and batch['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
 assert batch['main_sha256']==main['sha256']==sha(main['path']) and main['features']==[] and main['unique_entry'] and main['machine']=='AMD64' and main['optional_header']=='PE32+'
 assert batch['normal_qualification_sha256']==sha(normal) and batch['driver_sha256']==sha(ROOT/'維護工具/run-native-lighting-current-teleport-GUI.ps1')
 rust_path=(a.debug_rust_evidence if profile=='Debug' else a.release_rust_evidence).resolve(strict=True)
 _,_,rust,fixtures,scope_path=load_scope(normal,rust_path,profile)
 assert batch['Rust_evidence_sha256']==sha(rust_path) and rust['normal_mains'][profile]==main['sha256']
 files['profile-evidence/'+profile+'-lighting-scope.json']=scope_path
 counts={k:0 for k in totals};cases=[]
 for q in batch['cases']:
  case=Path(q['case']).resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-gui');r=read(case/'receipt.json');scenario=q['scenario']
  assert read(case/'current-lighting-source-qualification.json')==q and sha(case/'receipt.json')==q['receipt_sha256']
  launch=read(case/'server-launch-identity.json');assert sha(case/'server-launch-identity.json')==q['server_launch_identity_sha256']
  assert q['resource_profile']==r['resource_profile']==launch['resource_profile']==a.resource_profile
  assert q['extended_item_details'] is r['extended_item_details'] is launch['extended_item_details'] is a.extended_item_details
  assert q['common_catalog_sha256']==r['common_catalog_sha256']==launch['common_catalog_sha256']==batch['common_catalog_sha256']
  assert '-Dlito.resourceProfile='+a.resource_profile in launch['command_line'] and '-Dlito.extendedItemDetails='+str(a.extended_item_details).lower() in launch['command_line']
  assert q['main_sha256']==r['exe_sha256']==main['sha256'] and q['reference_sha256']==r['jar_sha256']==jar_sha
  assert q['source_inputs_unchanged'] and q['source_files']==len(pins) and not q['physical_input_verified'] and r['private_desktop']
  assert q['scoped_Rust_evidence_sha256']==sha(rust_path) and q['normal_build_qualification_sha256']==sha(normal)
  assert q['additional_fixture_inputs']==fixtures and q['compiled_input_files']==len(pins)+len(fixtures) and not q['full_default_regression_verified']
  assert q['lighting_scope_evidence_sha256']==sha(scope_path) and q['GUI_evidence_helper_sha256']==sha(ROOT/'維護工具/native_lighting_GUI_evidence.py')
  assert read(case/'compilation-inputs.json')['files']=={**pins,**fixtures}
  assert sha(case/'compilation-inputs.json')==r['compilation_inputs_sha256']
  assert q['status']==('qualified_lighting_scope_real_cross_endpoint_GUI' if scenario=='cross-endpoint' else 'qualified_lighting_scope_real_recovery_GUI')
  if a.resource_profile=='tw13081901-v1' and scenario=='cross-endpoint':
   inventory=read(case/'TW-inventory-continuity-comparison.json');assert q['TW_full_cross_endpoint_inventory_continuity'] and q['TW_inventory_continuity_comparison_sha256']==sha(case/'TW-inventory-continuity-comparison.json')
   assert inventory['receipt_sha256']==q['receipt_sha256'] and inventory['capture_sha256']==q['capture_sha256'] and inventory['objects']==25
   assert all(x['source_destination_and_battle_items_identical'] and x['wire_detail_bytes_preserved'] for x in inventory['checks'])
  if a.resource_profile=='tw13081901-v1' and scenario!='cross-endpoint':
   inventory=read(case/'recovery-inventory-continuity-comparison.json');assert q['TW_recovery_complete_inventory_continuity'] and q['TW_recovery_inventory_comparison_sha256']==sha(case/'recovery-inventory-continuity-comparison.json')
   assert inventory['receipt_sha256']==q['receipt_sha256'] and inventory['capture_sha256']==q['capture_sha256'] and inventory['objects']==25 and inventory['resource_profile']=='tw13081901-v1' and inventory['scenario']==scenario
   assert all(x['fresh_rental_reentry_new_IDs_and_all_nonID_fields_match'] and x['reentry_native_inventory_still_matches_continued_snapshot'] and x['exact_wire_fields_and_extended_details_preserved'] for x in inventory['checks'])
   assert all(x['expired_old_IDs_never_regranted'] if scenario=='cancel-expiry' else x['continued_stage_all_six_fields_preserved'] for x in inventory['checks'])
  for source,h in r['source_pins'].items():assert sha(ROOT/source)==h and sha(case/'source-snapshot'/source)==h,source
  for step in r['steps']:
   if 'png' in step:assert sha(step['png'])==step['png_sha256']
  server=r['server'];assert server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
  if scenario=='cross-endpoint':assert server['staged_loadouts_empty'] and server['authenticated_accounts_empty']
  else:assert server['staged_loadouts_after_reentry']==0 and server['full_game_reentry_verified']
  captures=list((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));assert len(captures)==1;cap=captures[0];assert sha(cap)==q['capture_sha256']
  frames=[json.loads(line) for line in cap.read_text(encoding='utf-8-sig').splitlines()]
  sessions=sum(x['direction']=='server_to_client' and x.get('note')=='S_KEY' for x in frames)
  logins=sum(x['direction']=='server_to_client' and x.get('note')=='S_LITO_LOGIN_OK' for x in frames)
  assert sessions==q['actual_formal_sessions']==server['actual_client_sessions'];assert logins==(2 if scenario=='cross-endpoint' else 6 if scenario=='wrong-service' else 4)
  login_sessions+=logins;game_sessions+=sessions-logins
  for key in totals:counts[key]+=q[key];totals[key]+=q[key]
  prefix=profile+'/'+scenario+'/'
  for f in case.rglob('*'):
   if f.is_file():assert f.resolve().is_relative_to(case) and not f.is_symlink();files[prefix+str(f.relative_to(case)).replace('\\','/')]=f
  files[prefix+'raw-capture/frames.jsonl']=cap
  cases.append({'scenario':scenario,'case':str(case),'receipt_sha256':q['receipt_sha256'],'qualification_sha256':sha(case/'current-lighting-source-qualification.json'),'actual_formal_sessions':sessions,'LOGIN_sessions':logins,'GAME_sessions':sessions-logins,'main_sha256':main['sha256']})
 assert counts=={'actual_formal_sessions':56,'exact_C_MOVE':110,'extra_C_MOVE':0,'original_world_PNGs':40,'original_glyph_matches':28,'stale_GUI_notifications':64}
 profiles[profile]={'main_sha256':main['sha256'],'cases':cases,**counts,'scoped_Rust_evidence_sha256':sha(rust_path),'scoped_Rust_counts':rust['counts'][profile],'test_products_per_profile':rust.get('test_products_per_profile',rust['counts'][profile].get('actual_test_executables')),'full_default_regression_verified':False}
 files['profile-evidence/'+profile+'-batch.json']=batch_file;files['profile-evidence/'+profile+'-Rust.json']=rust_path
assert totals=={'actual_formal_sessions':112,'exact_C_MOVE':220,'extra_C_MOVE':0,'original_world_PNGs':80,'original_glyph_matches':56,'stale_GUI_notifications':128}
assert login_sessions==game_sessions==56
files['normal-source-inputs.json']=normal.parent/'source-inputs.json';files['normal-qualification.json']=normal;files['qualifier.py']=Path(__file__)
for name in ['run-native-lighting-current-teleport-GUI.ps1','qualify-native-lighting-teleport-current-case.py','native_lighting_GUI_evidence.py']:
 files['validation-sources/'+name]=ROOT/'維護工具'/name
archive=cohort/'GUI-raw-evidence.zip'
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for name,file in files.items():z.write(file,name)
file_pins={name:sha(file) for name,file in files.items()}
with zipfile.ZipFile(archive) as z:
 assert set(z.namelist())==set(file_pins)
 for name,h in file_pins.items():
  with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
for name,h in {**pins,**fixtures}.items():assert sha(ROOT/name)==h,name
for item in n['profiles'].values():assert sha(item['path'])==item['sha256']
proof={'resource_profile':a.resource_profile,'extended_item_details':a.extended_item_details,'full_shared_profile_compatibility_verified':False,'TW_cross_endpoint_six_item_fields_and_wire_details_verified':a.resource_profile=='tw13081901-v1','TW_recovery_phase_specific_complete_item_and_wire_details_verified':a.resource_profile=='tw13081901-v1','status':'qualified_current_lighting_normal_mains14_GUI_teleport_and_recovery_cases','profiles':profiles,'source_files':len(pins),'additional_compile_fixture_inputs':fixtures,'compiled_input_files':len(pins)+len(fixtures),'source_manifest_sha256':sha(normal.parent/'source-inputs.json'),'reference_sha256':jar_sha,'scenarios':14,**totals,'LOGIN_sessions':login_sessions,'GAME_sessions':game_sessions,'raw_archive_sha256':sha(archive),'raw_archive_readback_verified':True,'raw_archive_files':file_pins,'full_default_regression_verified':False,'scoped_Rust_evidence_per_profile':{p:profiles[p]['scoped_Rust_counts'] for p in profiles},'world_scene_check_scope':'Original-source provenance, actual PNG SHA and terrain color diversity; no independent full-world framebuffer oracle.','physical_input_verified':False,'multi_DPI_verified':False,'same_endpoint_verified':False,'full_current_TCP_regression_verified':False,'full_library_regression_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(__file__)}
(cohort/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','scenarios','actual_formal_sessions','exact_C_MOVE','extra_C_MOVE','original_world_PNGs','original_glyph_matches','stale_GUI_notifications']},ensure_ascii=False))
