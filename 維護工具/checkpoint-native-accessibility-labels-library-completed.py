"""Publish only terminal independently read current-source full library scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-accessibility-labels-library-default') and a.terminal_chunk not in ['','pending','unknown']
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='executed_both_current_full_library_default_selections' and c['owned_process'] is None
assert o['status']=='independently_qualified_current1047_library_both_profiles_and_terminal_processes' and o['cohort_sha256']==sha(run/'cohort.json')
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and o['original_session_id']==a.session and o['terminal_tool_chunk']==a.terminal_chunk
assert c['source_manifest_sha256']==o['source_manifest_sha256']=='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
profiles={}
for profile in ['Debug','Release']:
    info=o['profiles'][profile];qp=Path(info['qualification']);q=read(qp)
    assert sha(qp)==info['qualification_sha256'] and q['source_manifest_sha256']==c['source_manifest_sha256']
    assert q['status']=='independently_qualified_current_full_library_default_selection' and q['failed']==q['filtered_out']==q['Rust_exit_code']==0
    assert q['passed']==c['profiles'][profile]['counts']['passed'] and q['ignored']==c['profiles'][profile]['counts']['ignored']
    assert not q['ignored_bodies_executed'] and not q['GUI_verified'] and not q['whole_goal_complete']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        assert set(z.namelist())==set(q['raw_files'])
        for n,h in q['raw_files'].items():
            with z.open(n) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),passed=q['passed'],failed=0,ignored=q['ignored'],inventory=q['inventory'],actual_test_sha256=q['actual_test_sha256'])
    visual_path=qp.parent/'visual-readback.json'
    if visual_path.exists():
        visual=read(visual_path);assert visual['source_manifest_sha256']==c['source_manifest_sha256']and visual['actual_test_sha256']==q['actual_test_sha256']and visual['qualification_sha256']==sha(qp)
        assert not visual['normal_GUI_verified']and not visual['all_required_maps_verified']
        profiles[profile]['diagnostic_visual_readback']=dict(path=str(visual_path),sha256=sha(visual_path),required_followup=visual['required_followup'],normal_GUI_verified=False)
        followup_path=qp.parent/'pixel-source-followup.json'
        if followup_path.exists():
            followup=read(followup_path);sample=read(followup['qualification'])
            assert sha(followup['qualification'])==followup['qualification_sha256']and sample['reference_qualification_sha256']==sha(qp)
            assert sample['source_manifest_sha256']==c['source_manifest_sha256']and sample['gray_samples']==4 and not sample['normal_GUI_verified']
            profiles[profile]['diagnostic_visual_readback'].update(sampled_pixel_followup=dict(path=str(followup_path),sha256=sha(followup_path),qualification=followup['qualification'],qualification_sha256=followup['qualification_sha256']),required_followup=sample['remaining_followup'])
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
before=read(Path(read(ROOT/'工作佇列.json')['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(broad_default_integration_regression_claimed=False,status='current_source_both_full_library_default_selections_qualified_limited',source_manifest_sha256=c['source_manifest_sha256'],source_files=1047,compiled_input_files=1048,profiles=profiles,run=str(run),original_session_id=a.session,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,completion=str(op),completion_sha256=sha(op),recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,ignored_bodies_executed=False,GUI_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
broad=read(Path(c['normal_qualification']))
assert broad['source_manifest_sha256']==c['source_manifest_sha256'] and broad['recorded_owned_processes_all_terminal']
work='來源4F2F911B原'+str(a.session)+'：'+'當版完整library兩版已獨立核對，忽略本文未執行。'+'AB3C廣域、控制器、資源及限定GUI只保留原版本資格，不藉此宣告重驗；修正版正常技能名稱與面板／巡覽、其餘GUI與G01–G12仍待。全部必要自測後才交人工，不請使用者登入，相鄰只讀，8秒／60秒、30秒數量有效期與Release／G11預算不變。'

staged={}
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        j['current_native_catalog_library']=record
        j['current_full_default_library_verified']=True
        j['current_native_catalog_library_active'].update(qualification_granted=True,status='completed_qualified_default_library_scope',completion=str(op),completion_sha256=sha(op),terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0)
        j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='current-catalog-remaining-WORLD-services-visibility-GUI',work=work,applies_to_current_source=True,goal_complete=False)
        j['current_source_goal_validation']['G03-current-regression']=dict(state='current_full_library_default_both_qualified_broad_not_repeated_GUI_pending',complete=False,evidence=record)
        for item in j.get('items',[]):
            if item['id']=='G03-current-regression':item.update(state='current_full_library_default_both_qualified_broad_not_repeated_GUI_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
            if item['id'].startswith('G07-')and profiles['Debug'].get('diagnostic_visual_readback'):item.update(current_library_diagnostic_visual=profiles['Debug']['diagnostic_visual_readback'],complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');staged[n]=((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(ROOT/n)==h for n,h in before.items())
for n,payload in staged.items():(ROOT/n).write_bytes(payload)
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},evidence_sha256=sha(out/'evidence.json'),before_archive_sha256=sha(out/'before.zip')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(**record,checkpoint=str(out),latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_session_before_restarting=False,next_action='Current-source ability/finite resources and actual sessions; no manual login during development or GUI/physical promotion.'))
print(json.dumps(dict(checkpoint=str(out),profiles=profiles,whole_goal_complete=False),ensure_ascii=False))
