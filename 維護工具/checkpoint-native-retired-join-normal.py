"""Publish only the independently observed current-source normal/related scope."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
run=V/'native-retired-join-normal/validation-20261018-063154-317211';c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert o['status']=='independently_qualified_retired_join_normal_both_profiles_and_terminal_processes'
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal']and sha(run/'cohort.json')==o['cohort_sha256']
assert c['status']=='executed_retired_join_normals_related'and c['owned_process']is None
assert len(c['process_history'])==86 and all(j['exit_code']==0 for j in c['process_history'])
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
for profile,item in o['profiles'].items():
    qp=Path(item['qualification']);q=read(qp);assert sha(qp)==item['qualification_sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256']and q['passed']==90 and q['integration_passed']==70 and q['selected_library_passed']==20
    assert len(q['targets'])==11 and q['recorded_owned_all_terminal']and q['failed']==q['ignored']==0
    assert sha(q['normal']['path'])==q['normal']['sha256']==item['normal_sha256']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip')as z:
        for n,h in q['raw_files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(formal)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_normal_both_profiles_eleven_integration_targets_and_two_selected_library_groups_qualified_limited',
    source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,
    normal_completion=str(op),normal_completion_sha256=sha(op),normal_profiles=o['profiles'],
    normal_recorded_terminal_identities=o['recorded_original_owned_identities'],integration_passed_each=70,selected_library_passed_each=20,
    normal_related_validation_complete=True,normal_product_registered=True,current_source_C02_runtime_verified=False,
    full_library_verified=False,current_broad_default_verified=False,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,
    retired_join_order_repair_qualification=dict(path=str(V/'native-retired-join-order/qualified-20261018-062835-125224/qualified.json'),sha256=sha(V/'native-retired-join-order/qualified-20261018-062835-125224/qualified.json'),repair_applied=True,current_source_C02_revalidation_pending=True))
save(out/'evidence.json',record)
work='來源4E1F7A4E正常兩版唯一入口建置及相關驗證已沿原99608實際終止0，逐來源／映像／raw與記錄程序身分終止獨立核對：每版11整合目標70過與2個選定library群組20過，各0敗0忽略。包含既有十四入房案例與新增四個舊房間peer通知順序案例（2個Rust測試），及18空間測試。只代表此相關範圍，完整library本文／廣域及當版實際C02／G06尚待；874B與候選結果不移記。下一步優先當版G06六分支角色接續／場景退役／失敗恢復，再冰錐實際施放與必要回歸。來源與QA保持凍結。登入器／正式JAR只讀且產物獨立；GUI／實體輸入音效、完整G06與G01–G12未完成，8／60秒、Release與G11不變。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);identity=value['current_product_identity'];assert identity['source_manifest_sha256']==c['source_manifest_sha256']
        identity.update(source_qualification_pending=False,Debug_normal_sha256=o['profiles']['Debug']['normal_sha256'],Release_normal_sha256=o['profiles']['Release']['normal_sha256'],existing_normal_source_manifest_sha256=c['source_manifest_sha256'],Debug_normal_qualified_for_current_source=True,Release_normal_qualified_for_current_source=True,qualification_scope='normal builds, eleven integration targets and two selected library groups; not full-library/current TCP/GUI/full-G06')
        for key,item in value.items():
            if key.startswith('current_native')and isinstance(item,dict)and item.get('source_manifest_sha256')and item['source_manifest_sha256']!=c['source_manifest_sha256']:item.update(applies_to_current_source=False,retained_source_version_only=True)
        value['current_native_retired_join_application'].update(normal_validation_pending=False,qualification=str(out/'evidence.json'),qualification_sha256=sha(out/'evidence.json'))
        value['current_native_retired_join_normal']=record
        value['current_native_retired_join_normal_active'].update(status='completed_qualified_related_scope',completion=str(op),completion_sha256=sha(op))
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current1040-G06-controller-and-source-runtime',work=work,applies_to_current_source=True,goal_complete=False)
        for key in value.get('current_source_goal_validation',{}):value['current_source_goal_validation'][key]=dict(state='4E1F_related_qualified_broader_acceptance_pending',complete=False)
        for item in value.get('items',[]):
            if item['id'].startswith('G01-'):item.update(state='4E1F_current_normal_source_baseline_verified_limited',complete=True)
            if item['id'].startswith('G03-'):item.update(state='4E1F_related_qualified_broad_library_pending',complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),passed_each=90,whole_goal_complete=False),ensure_ascii=False))
