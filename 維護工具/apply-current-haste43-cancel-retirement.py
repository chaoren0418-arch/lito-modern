"""Apply six reviewed source files; prior qualifications remain bound to A695."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
candidate_path=R/'維護工具/candidates/native-haste43-cancel-apply-20261020-114502-695797/candidate.json';c=read(candidate_path)
OLD='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6';NEW='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==OLD and not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert c['before_source_manifest_sha256']==OLD and c['candidate_source_manifest_sha256']==NEW and not c['source_applied'] and not c['game_or_protocol_clocks_changed']
assert sha(candidate_path.with_name('review.diff'))==c['review_sha256']
for key,proof in c['proofs'].items():
    assert sha(proof['path'])==proof['sha256'];q=read(proof['path']);assert q['source_manifest_sha256']==OLD and q['qualified'] and not q.get('candidate_applied',False)
    assert w[key]['qualification']==proof['path'] and w[key]['qualification_sha256']==proof['sha256']
assert w['current_full_default_library_verified'] and w['current_native_catalog_library']['source_manifest_sha256']==OLD
for key in ['current_normal_GUI_room_service_keyboard','current_normal_GUI_rental_entry_keyboard','current_normal_GUI_hotel_return_keyboard','current_normal_GUI_EOF_cleanup']:
    for profile in ['Debug','Release']:
        q=w[key][profile];assert q['source_manifest_sha256']==OLD and sha(q['qualification'])==q['qualification_sha256']
normal=w['current_native_installed_resource_profile_normal_qualification'];base=Path(normal['run']);pins=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json');assert sha(base/'source-inputs.json')==OLD and all(sha(R/n)==h for n,h in inputs.items())
payload={n:Path(p).read_bytes() for n,p in c['candidate_files'].items()};assert len(payload)==6 and all(sha(R/n)==c['before_files'][n] and hashlib.sha256(b).hexdigest().upper()==c['after_files'][n] for n,b in payload.items())
newpins=dict(pins);newpins.update(c['after_files']);newinputs=dict(inputs);newinputs.update(c['after_files']);assert hashlib.sha256(data(newpins)).hexdigest().upper()==NEW
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=V/'native-installed-resource-profile-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
history_path=out/'before-work-queue.json';history_path.write_bytes((R/'工作佇列.json').read_bytes());(out/'master-report-before.md').write_bytes((R.parent/'LiTo381-總開發報告.md').read_bytes())
history=dict(source_manifest_sha256=OLD,state=str(history_path),state_sha256=sha(history_path),prior_normal=normal,prior_library=w['current_native_catalog_library'],prior_report=str(out/'master-report-before.md'),prior_report_sha256=sha(out/'master-report-before.md'),prior_source_qualifications_retained=True,new_source_qualification_granted=False)
expected={**before,**c['before_files']}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in expected:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in expected.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,b in payload.items():z.writestr(n,b)
with zipfile.ZipFile(out/'applied-payload.zip') as z:assert all(z.read(n)==b for n,b in payload.items())
for name,obj in [('before-source-inputs',pins),('before-compiled-inputs',inputs),('applied-source-inputs',newpins),('applied-compiled-inputs',newinputs)]:(out/(name+'.json')).write_bytes(data(obj))
review=dict(status='reviewed_six_file_haste43_and_cancel_retirement',candidate_sha256=sha(candidate_path),candidate_source_manifest_sha256=NEW,exact_six_files=True,original_assertions_preserved=True,original_reuse10ms_and_duration1200s_retained=True,no_prediction=True,normal_skill_range_and_stale_target_guards_retained=True,cancel_recovery_saved_before_scene_retirement=True,formal_positive_negative_and_cancel_components_independently_qualified=True,proofs=c['proofs'])
(out/'review.json').write_bytes(data(review));(out/'application.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='applied_haste43_cancel_retirement_pending_normal_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=NEW,source_files=1048,compiled_input_files=1049,changed_files=list(payload),created_files=[],after_files=c['after_files'],candidate=str(candidate_path),candidate_sha256=sha(candidate_path),candidate_review=str(out/'review.json'),candidate_review_sha256=sha(out/'review.json'),normal_validation_complete=False,normal_product_source_modified=True,test_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,formal_JAR_modified=False,clocks_changed=False,normal_GUI_verified=False,whole_goal_complete=False,before_normal_qualification=normal['qualification'],before_normal_qualification_sha256=normal['qualification_sha256'],qualification_before_repair=history,before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files=expected,expected_default_library_passed=1975,expected_default_library_ignored=488)
work='新來源E0E6409E：加入#43原生加速術自用／對人選擇與正式範圍，取消先保存必要回復關聯再清空舊世界／房間／技能，原測試保留並新增取消回歸1項。六檔／1048來源／1049輸入；A695所有限定資格與反例保留原來源，不移記新來源。先正常19兩版，再完整library Debug／Release預期1975／488忽略、額外資源／廣域／API／控制器及GUI重驗，自行確認後才交人工。'
staged=dict(payload)
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        j['qualification_before_haste43_cancel_retirement']=history
        identity.update(source_manifest_sha256=NEW,source_files=1048,compiled_input_files=1049,source_qualification_pending=True,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        j.update(current_full_default_library_verified=False,current_native_installed_resource_profile_application=record,working_tree_matches_current_normal_qualification=False,working_tree_modified_since_current_normal_qualification=True,temporary_test_overlay_active=False,next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        for item in j['current_source_goal_validation'].values():item.update(complete=False,current_source_manifest_sha256=NEW,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in j.get('items',[]):item.update(complete=False,current_source_manifest_sha256=NEW,current_evidence_applies_to_current_source=False)
        j['current_focus']=dict(item='haste43_cancel_retirement_current_source_validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==14 and all(sha(R/n)==h for n,h in inputs.items()) and all(sha(R/n)==h for n,h in before.items())
try:
    for n,b in staged.items():(R/n).write_bytes(b)
    assert all(sha(R/n)==h for n,h in newinputs.items())
except BaseException:
    with zipfile.ZipFile(out/'before.zip') as z:
        for n in expected:(R/n).write_bytes(z.read(n))
    assert all(sha(R/n)==h for n,h in expected.items());raise
(out/'application.json').write_bytes(data(record));(out/'evidence.json').write_bytes(data(record));(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},source_after_files=c['after_files'],before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(application=str(out),source_manifest_sha256=NEW,normal_built=False,GUI_verified=False),ensure_ascii=False))
