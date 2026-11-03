"""Apply the reviewed three-file repair only after the original baseline counterexamples."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';C=R/'維護工具/candidates/native-quantity-enter-failure-retirement-20261020-034156-331023'
OLD='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041';NEW='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(j):return(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==OLD and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
c=read(C/'candidate.json');review=read(C/'review.json');assert c['before_source_manifest_sha256']==OLD and c['candidate_source_manifest_sha256']==NEW
assert sha(C/'candidate.json')==review['candidate_sha256'] and review['candidate_source_manifest_sha256']==NEW and review['normal_source_still_unchanged']
baseline=w['current_failure_retirement_negative_baseline'];assert baseline['source_manifest_sha256']==OLD and baseline['baseline_counterexamples_verified'] and baseline['original_source_and_library_PE_restored']
assert sha(baseline['qualification'])==baseline['qualification_sha256']
controller=w['current_native_catalog_controller_qualification'];assert controller['source_manifest_sha256']==OLD and controller['qualified']and controller['actual_sessions']==96 and sha(controller['qualification'])==controller['qualification_sha256']
normal=w['current_native_installed_resource_profile_normal_qualification'];assert normal['source_manifest_sha256']==OLD and normal['qualification_granted']and sha(normal['qualification'])==normal['qualification_sha256']
library=w['current_native_catalog_library'];assert w['current_full_default_library_verified']and library['source_manifest_sha256']==OLD and sha(library['completion'])==library['completion_sha256']
base=Path(normal['run']);pins=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json');assert len(pins)==1048 and len(inputs)==1049 and sha(base/'source-inputs.json')==OLD
assert all(sha(R/n)==h for n,h in inputs.items())
payload={n:(C/Path(n).name).read_bytes()for n in c['changed_files']};assert len(payload)==3
for n,b in payload.items():assert sha(R/n)==c['before_files'][n]and hashlib.sha256(b).hexdigest().upper()==c['after_files'][n]
newpins=dict(pins);newinputs=dict(inputs);newpins.update(c['after_files']);newinputs.update(c['after_files']);assert hashlib.sha256(data(newpins)).hexdigest().upper()==NEW
assert set(newpins)==set(pins)and set(newinputs)==set(inputs)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
history=dict(source_manifest_sha256=OLD,normal=normal,library=library,resource=w['current_native_library_resource_optin'],shared_resource=w['current_native_library_shared_resource_optin'],return_API=w['current_native_rental_return_reconnect_api_qualification'],focused=w['current_native_rental_return_reconnect_focused'],controller=controller,negative_baseline=baseline,
    GUI_room={p:q for p,q in w.get('current_normal_GUI_room_service_keyboard',{}).items()if q['source_manifest_sha256']==OLD},GUI_entry={p:q for p,q in w.get('current_normal_GUI_rental_entry_keyboard',{}).items()if q['source_manifest_sha256']==OLD},GUI_hotel_return={p:q for p,q in w.get('current_normal_GUI_hotel_return_keyboard',{}).items()if q['source_manifest_sha256']==OLD},GUI_operator_budget_failure=w['current_normal_GUI_hotel_return_failure'],GUI_quantity_failure=w['current_normal_GUI_hotel_return_quantity_failure'])
for proof in [history['resource'],history['shared_resource'],history['return_API'],history['focused'],*history['GUI_room'].values(),*history['GUI_entry'].values(),*history['GUI_hotel_return'].values()]:assert proof['source_manifest_sha256']==OLD and sha(proof['qualification'])==proof['qualification_sha256']
assert set(history['GUI_room'])==set(history['GUI_entry'])=={'Debug','Release'}and set(history['GUI_hotel_return'])=={'Debug'}
out=V/'native-installed-resource-profile-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
record=dict(status='applied_quantity_Enter_and_failure_retirement_pending_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=NEW,source_files=1048,compiled_input_files=1049,changed_files=list(payload),created_files=[],after_files=c['after_files'],candidate=str(C/'candidate.json'),candidate_sha256=sha(C/'candidate.json'),candidate_review=str(C/'review.json'),candidate_review_sha256=sha(C/'review.json'),normal_validation_complete=False,normal_product_source_modified=True,test_only_source_modified=False,test_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,formal_JAR_modified=False,clocks_changed=False,normal_GUI_verified=False,whole_goal_complete=False,before_normal_qualification=normal['qualification'],before_normal_qualification_sha256=normal['qualification_sha256'],negative_baseline=baseline,qualification_before_repair=history)
work='來源'+NEW[:8]+'：數量欄Enter僅轉原ORDER、抑制重複；失敗保存必要傳送關聯後退役世界／房間／服務／角色，兩個原負向測試加清理斷言。D308全部已核對成果與兩批Release GUI失敗保留原來源。先正常19兩版與完整library Debug/Release、81資源及API／控制器，再GUI含Enter送出與EOF清理；不改JAR／時限，不交人工。'
staged=dict(payload);date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        j['qualification_before_quantity_enter_failure_retirement']=history
        identity.update(source_manifest_sha256=NEW,source_files=1048,compiled_input_files=1049,source_qualification_pending=True,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        j.update(current_full_default_library_verified=False,current_native_installed_resource_profile_application=record,working_tree_matches_current_normal_qualification=False,working_tree_modified_since_current_normal_qualification=True,temporary_test_overlay_active=False,next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        for item in j['current_source_goal_validation'].values():item.update(complete=False,current_source_manifest_sha256=NEW,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in j.get('items',[]):item.update(complete=False,current_source_manifest_sha256=NEW,current_evidence_applies_to_current_source=False)
        j['current_focus']=dict(item='quantity-keyboard-and-failure-authority-retirement',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
expected={**before,**c['before_files']}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in expected:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert set(z.namelist())==set(expected)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in expected.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,b in payload.items():z.writestr(n,b)
with zipfile.ZipFile(out/'applied-payload.zip')as z:assert set(z.namelist())==set(payload)and all(z.read(n)==b for n,b in payload.items())
for n,b in [('before-source-inputs.json',(base/'source-inputs.json').read_bytes()),('before-compiled-inputs.json',(base/'compiled-inputs.json').read_bytes()),('applied-source-inputs.json',data(newpins)),('applied-compiled-inputs.json',data(newinputs))]:(out/n).write_bytes(b)
(out/'application.py').write_bytes(Path(__file__).read_bytes())
assert len(staged)==11 and all(sha(R/n)==h for n,h in before.items())and all(sha(R/n)==h for n,h in inputs.items())
for n,b in staged.items():(R/n).write_bytes(b)
assert all(sha(R/n)==h for n,h in newinputs.items())
record.update(before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files=expected)
(out/'application.json').write_bytes(data(record));(out/'evidence.json').write_bytes(data(record));(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},source_after_files=c['after_files'],before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(application=str(out),source_manifest_sha256=NEW,compiled=False,GUI_verified=False),ensure_ascii=False))
