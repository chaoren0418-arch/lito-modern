"""Apply one native UI focus repair after the actual owned GUI failure was sealed."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
C=R/'維護工具/candidates/native-hotel-return-focus-20261019-234458-107346'
OLD='38CF45B68805DCEF4EF2A2733A444C4B6D7FA74F6C16FE80565DCDFBEB82CA38'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==OLD and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
normal=w['current_native_installed_resource_profile_normal_qualification'];failure=w['current_normal_GUI_hotel_return_focus_failure'];assert normal['qualification_granted']and normal['source_manifest_sha256']==failure['source_manifest_sha256']==OLD
assert failure['observed_exit_code']==1 and not failure['return_C10_sent']and failure['recorded_original_owned_processes_terminal']and sha(failure['raw_archive'])==failure['raw_archive_sha256']
proofs={key:w[key]for key in ['current_native_library_resource_optin','current_native_library_shared_resource_optin','current_native_rental_return_reconnect_api_qualification','current_native_rental_return_reconnect_focused']}
for proof in [normal,*proofs.values(),*w['current_normal_GUI_rental_entry_keyboard'].values()]:
 assert proof['source_manifest_sha256']==OLD and sha(proof['qualification'])==proof['qualification_sha256']
library=w['current_native_catalog_library'];assert w['current_full_default_library_verified']and library['source_manifest_sha256']==OLD and sha(library['completion'])==library['completion_sha256']
base=Path(normal['run']);pins=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json');assert len(pins)==1048 and len(inputs)==1049 and sha(base/'source-inputs.json')==OLD
assert all(sha(R/name)==h for name,h in inputs.items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/name)==h for name,h in before.items())
c=read(C/'candidate.json');assert c['base_source_manifest_sha256']==OLD and c['production_source_modified']and not c['compiled']and not c['applied']and c['return_controller_bodies_unchanged']and not c['clocks_changed']
name='LiTo381-Modern/src/modern_lito/native_login_window.rs';assert c['changed_files']==[name]and sha(R/name)==c['before_files'][name]and sha(C/Path(name).name)==c['candidate_files'][name]
payload=(C/Path(name).name).read_bytes();newpins=dict(pins);newinputs=dict(inputs);newpins[name]=newinputs[name]=hashlib.sha256(payload).hexdigest().upper();source_bytes=data(newpins);source=hashlib.sha256(source_bytes).hexdigest().upper();assert source!=OLD
for keep in ['native_login.rs','native_rental_return_tests.rs','native_rental_window.rs']:
 key='LiTo381-Modern/src/modern_lito/'+keep;assert newinputs[key]==inputs[key]
out=V/'native-installed-resource-profile-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
record=dict(status='applied_hotel_RETURN_single_owner_enablement_repair_pending_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=source,changed_files=[name],created_files=[],after_files=c['candidate_files'],source_files=1048,compiled_input_files=1049,candidate=str(C/'candidate.json'),candidate_sha256=sha(C/'candidate.json'),normal_validation_complete=False,normal_product_source_modified=True,test_only_source_modified=False,source_overlays_used=False,diagnostic_API_extension=False,formal_JAR_modified=False,clocks_changed=False,normal_GUI_verified=False,whole_goal_complete=False,change_scope=c['change_scope'],before_normal_qualification=normal['qualification'],before_normal_qualification_sha256=normal['qualification_sha256'],focus_failure=failure)
work='來源'+source[:8]+'僅修native_login_window租裝RETURN重複停用造成焦點退役：generic更新跳過租裝RETURN，由既有refresh_rental統一決定。38CF45B6完整library各1974／488、81資源、API16及GUI入口兩版8sessions保留原版；旅館返回焦點失敗與2sessions原始證據已封存，尚未送C10。先定位十項，再正常兩版與完整library兩版、資源／API／GUI逐層驗證；原時限及JAR不變，不交人工。'
staged={name:payload};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
 p=R/n
 if p.suffix=='.json':
  j=read(p);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==OLD
  j['qualification_before_hotel_return_focus_repair']=dict(source_manifest_sha256=OLD,normal=normal,library=library,resource=proofs['current_native_library_resource_optin'],shared_resource=proofs['current_native_library_shared_resource_optin'],return_API=proofs['current_native_rental_return_reconnect_api_qualification'],focused=proofs['current_native_rental_return_reconnect_focused'],GUI_entry=j['current_normal_GUI_rental_entry_keyboard'],hotel_focus_failure=failure)
  identity.update(source_manifest_sha256=source,source_files=1048,compiled_input_files=1049,source_qualification_pending=True,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
  j.update(current_full_default_library_verified=False,current_native_installed_resource_profile_application=record,working_tree_matches_current_normal_qualification=False,working_tree_modified_since_current_normal_qualification=True,next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  for info in j['current_source_goal_validation'].values():info.update(complete=False,current_source_manifest_sha256=source,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
  for item in j.get('items',[]):item.update(complete=False,current_source_manifest_sha256=source,current_evidence_applies_to_current_source=False)
  j['current_focus']=dict(item='native-hotel-RETURN-focus-repair-validation',work=work,applies_to_current_source=True,goal_complete=False)
  if n.endswith('next-validation-action.json'):j['next_action']=work
  staged[n]=data(j)
 else:
  first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in [*before,name]:z.write(R/n,n)
expected={**before,**c['before_files']}
with zipfile.ZipFile(out/'before.zip')as z:assert set(z.namelist())==set(expected)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in expected.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED)as z:z.writestr(name,payload)
with zipfile.ZipFile(out/'applied-payload.zip')as z:assert z.read(name)==payload
for n,b in [('before-source-inputs.json',(base/'source-inputs.json').read_bytes()),('before-compiled-inputs.json',(base/'compiled-inputs.json').read_bytes()),('applied-source-inputs.json',source_bytes),('applied-compiled-inputs.json',data(newinputs))]:(out/n).write_bytes(b)
(out/'application.py').write_bytes(Path(__file__).read_bytes());assert len(staged)==9 and all(sha(R/n)==h for n,h in before.items())and all(sha(R/n)==h for n,h in inputs.items())
for n,b in staged.items():(R/n).write_bytes(b)
assert all(sha(R/n)==h for n,h in newinputs.items());record.update(before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files=expected)
(out/'application.json').write_bytes(data(record));(out/'evidence.json').write_bytes(data(record));(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},source_after_files=c['candidate_files'],before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(application=str(out),source_manifest_sha256=source,changed_files=[name],compiled=False),ensure_ascii=False))
