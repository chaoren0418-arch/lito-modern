"""Release the original current source reservation only after independent terminal readback."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native44FE-default112')
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='independently_qualified_current_default112_both_profiles'and c['owned_process']is None
assert o['status']=='independently_qualified_current1040_default112_both_profiles_and_terminal_processes'
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal']and o['cohort_sha256']==sha(run/'cohort.json')
assert a.session==o['original_session_id']==67934 and a.terminal_chunk==o['terminal_tool_chunk']and a.terminal_chunk not in ['pending','unknown','']
assert c['source_manifest_sha256']==o['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
profiles={}
for profile in ['Debug','Release']:
    info=c['profiles'][profile];qp=Path(info['qualification']);q=read(qp)
    assert info['qualified']and info['actual_runner_exit_code']==0 and sha(qp)==info['qualification_sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256']and q['counts']['failed']==0
    assert q['actual_test_executables']==113 and q['integration_targets']==112 and not q['full_library_scope_inferred']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip')as z:
        for n,h in q['raw_files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),counts=q['counts'],integration_targets=112,actual_test_executables=113,normal_mains=q['normal_mains'])
normal=Path(c['normal_qualification']).parent.parent;inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
continuation=read(run.parent/'current-continuation.json');before=read(continuation['latest_readback'])['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
for n in before:
    if n.endswith('.json'):
        value=read(ROOT/n);assert value['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        assert value['active_workload_sessions']=={'broad_default':a.session}and value['source_and_corresponding_QA_frozen']
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current44FE_default112_both_profiles_qualified_limited',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,
    profiles=profiles,original_session_id=a.session,terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,completion=str(op),completion_sha256=sha(op),
    recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,full_library_qualified=False,ignored_bodies_executed=False,
    GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
ability_prep=read(ROOT/'維護工具/candidates/native44FE-ability-buff-requests/preparation.json')
assert ability_prep['world_integration_typecheck_verified']and ability_prep['world_integration_metadata_only']and not ability_prep['world_integration_executed']
metadata_path=Path(ability_prep['world_integration_typecheck']);metadata=read(metadata_path)
assert sha(metadata_path)==ability_prep['world_integration_typecheck_sha256']and metadata['status']=='isolated_whole_crate_metadata_typecheck_qualified_only'
assert not metadata['machine_code_generated']and not metadata['tests_executed']and not metadata['normal_product_modified']
metadata_record=dict(qualification=str(metadata_path),qualification_sha256=sha(metadata_path),source_manifest_sha256=metadata['source_manifest_sha256'],candidate_manifest_sha256=metadata['candidate_manifest_sha256'],whole_crate_metadata_typecheck_verified=True,machine_code_generated=False,tests_executed=False,actual_sessions=0,normal_product_modified=False)
record['isolated_ability_metadata_typecheck']=metadata_record
save(out/'evidence.json',record)
work='當版44FEA738廣域原67934已actual exit0並逐映像／來源／測試名稱與原程序終止獨立讀回：Debug'+str(profiles['Debug']['counts']['passed'])+'／Release'+str(profiles['Release']['counts']['passed'])+'過，各0敗、既有忽略各'+str(profiles['Debug']['counts']['ignored'])+'/'+str(profiles['Release']['counts']['ignored'])+'，各112整合目標＋main／113實際映像。僅此廣域default範圍通過，完整library與忽略本文未驗；下一步同來源完整library兩版，再隔離11特效資源／生命週期整合候選。當版WORLD8sessions、旅館16sessions及控制器／正常模組音效各96sessions維持各自範圍；同端點契約失敗保留。技能書74ID辨識35／缺39，候選不是施法資格。先自行完成開發及自動化端到端後才人工驗收，開發期間不請使用者登入；8秒／60秒／Release與G11預算不變，相鄰只讀且產物獨立，完整G01–G12未完成。'
work+='兩種能力強化／S6B-S6C與11特效之隔離整庫metadata型別檢查已通過，沒有產生執行碼或資源／施法資格，尚未套正常產品。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        value=read(p);value['current_native44FE_default112']=record;value['current_native44FE_ability_metadata']=metadata_record
        value['current_native44FE_broad_active'].update(status='completed_qualified_default_scope',completion=str(op),completion_sha256=sha(op),terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,qualification_granted=True)
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current44FE-full-library-default',work=work,applies_to_current_source=True,goal_complete=False)
        if 'current_full_default_library_verified'in value:value['current_full_default_library_verified']=False
        if 'current_source_goal_validation'in value:value['current_source_goal_validation']['G03-current-regression']=dict(state='current44FE_default112_qualified_full_library_and_other_acceptance_pending',complete=False,evidence=record)
        for item in value.get('items',[]):
            if item['id']=='G03-current-regression':item.update(state='current44FE_default112_qualified_full_library_and_other_acceptance_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        if 'updated_date'in value:value['updated_date']=date
        save(p,value)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d')
        p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
continuation.update(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=False,next_action='Run current44FE full-library default selections, then isolated finite effect integration; no manual login during development.')
save(run.parent/'current-continuation.json',continuation)
print(json.dumps(dict(checkpoint=str(out),counts={p:r['counts']for p,r in profiles.items()},whole_goal_complete=False),ensure_ascii=False))
