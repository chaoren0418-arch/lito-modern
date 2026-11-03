"""Apply one reviewed cfg(test) reference; keep all earlier source qualifications."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
cp=R/'維護工具/candidates/text-index-order-20261020-123010-825391/candidate.json';c=read(cp)
qp=V/'native-text-index-order-components/20261020-123103-917380/qualified/qualified.json';q=read(qp)
OLD=c['before_source_manifest_sha256'];NEW=c['candidate_source_manifest_sha256'];w=read(R/'工作佇列.json')
assert w['current_product_identity']['source_manifest_sha256']==OLD and not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert q['qualified'] and q['candidate_sha256']==sha(cp) and q['source_manifest_sha256']==OLD and q['recorded_original_owned_processes_terminal'] and q['original_assertions_preserved'] and not q['Runtime_changed']
assert c['only_cfg_test_reference_changed'] and not c['Runtime_changed'] and not c['source_applied'] and sha(c['comparison'])==c['comparison_sha256']
normal=w['current_native_installed_resource_profile_normal_qualification'];base=Path(normal['run']);sources=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json');assert sha(base/'source-inputs.json')==OLD and len(sources)==1048 and len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
payload={n:Path(p).read_bytes() for n,p in c['candidate_files'].items()};assert len(payload)==1
name=next(iter(payload));assert name=='LiTo381-Modern/src/modern_lito/resources.rs' and sha(R/name)==c['before_files'][name] and hashlib.sha256(payload[name]).hexdigest().upper()==c['after_files'][name]
old=(R/name).read_text('utf-8-sig');new=payload[name].decode('utf-8');boundary=old.index('#[cfg(test)]');assert old[:boundary]==new[:boundary] and all(line in new for line in old.splitlines() if 'assert' in line)
newsources=dict(sources);newsources.update(c['after_files']);newinputs=dict(inputs);newinputs.update(c['after_files']);assert hashlib.sha256(data(newsources)).hexdigest().upper()==NEW
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=V/'native-installed-resource-profile-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'before-work-queue.json').write_bytes((R/'工作佇列.json').read_bytes());(out/'master-report-before.md').write_bytes((R.parent/'LiTo381-總開發報告.md').read_bytes())
history=dict(source_manifest_sha256=OLD,state=str(out/'before-work-queue.json'),state_sha256=sha(out/'before-work-queue.json'),prior_report=str(out/'master-report-before.md'),prior_report_sha256=sha(out/'master-report-before.md'),prior_source_qualifications_retained=True,new_source_qualification_granted=False)
expected={**before,**c['before_files']}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in expected:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in expected.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,b in payload.items():z.writestr(n,b)
for label,values in [('before-source-inputs',sources),('before-compiled-inputs',inputs),('applied-source-inputs',newsources),('applied-compiled-inputs',newinputs)]:(out/(label+'.json')).write_bytes(data(values))
review=dict(candidate_sha256=sha(cp),exact_one_cfg_test_file=True,Runtime_unchanged=True,original_assertions_preserved=True,independent_CSharp_comparison=c['comparison'],independent_CSharp_comparison_sha256=c['comparison_sha256'],component_qualification=str(qp),component_qualification_sha256=sha(qp),baseline_expected_failures_each=1,candidate_Text_passed_each=1,candidate_default_passed_each=33)
(out/'review.json').write_bytes(data(review));(out/'application.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='applied_exact_text_index_order_reference_pending_normal_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=NEW,source_files=1048,compiled_input_files=1049,changed_files=list(payload),created_files=[],after_files=c['after_files'],candidate=str(cp),candidate_sha256=sha(cp),candidate_review=str(out/'review.json'),candidate_review_sha256=sha(out/'review.json'),normal_validation_complete=False,normal_product_source_modified=False,test_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,formal_JAR_modified=False,clocks_changed=False,normal_GUI_verified=False,whole_goal_complete=False,before_normal_qualification=normal['qualification'],before_normal_qualification_sha256=normal['qualification_sha256'],qualification_before_repair=history,before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files=expected,expected_default_library_passed=1975,expected_default_library_ignored=488)
work='來源'+NEW[:8]+'僅補Text.idx已獨立C#核對之精確新雜湊；37497筆完整多重集合相同、192列限同名順序，PAK／所有原斷言及Runtime不變。兩版隔離原反例各1、候選Text各1及預設33過0敗4忽略；E0正常19／完整library1975過0敗488忽略及原71項70過1敗保留原來源。先當版正常兩版、完整library，再額外資源與GUI；不要求人工操作。'
staged=dict(payload)
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        j['qualification_before_text_index_order_reference']=history
        identity.update(source_manifest_sha256=NEW,source_qualification_pending=True,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        j.update(current_full_default_library_verified=False,current_native_installed_resource_profile_application=record,working_tree_matches_current_normal_qualification=False,working_tree_modified_since_current_normal_qualification=True,temporary_test_overlay_active=False,next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        for item in j['current_source_goal_validation'].values():item.update(complete=False,current_source_manifest_sha256=NEW,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in j.get('items',[]):item.update(complete=False,current_source_manifest_sha256=NEW,current_evidence_applies_to_current_source=False)
        j['current_focus']=dict(item='text_index_order_reference_validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==9 and all(sha(R/n)==h for n,h in inputs.items()) and all(sha(R/n)==h for n,h in before.items())
try:
    for n,b in staged.items():(R/n).write_bytes(b)
    assert all(sha(R/n)==h for n,h in newinputs.items())
except BaseException:
    with zipfile.ZipFile(out/'before.zip') as z:
        for n in expected:(R/n).write_bytes(z.read(n))
    raise
(out/'application.json').write_bytes(data(record));(out/'evidence.json').write_bytes(data(record));(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},source_after_files=c['after_files'],before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(application=str(out),source_manifest_sha256=NEW,Runtime_changed=False,normal_built=False),ensure_ascii=False))
