"""Publish only terminal independently read current-source full library scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ability_prep=read(ROOT/'維護工具/candidates/native44FE-ability-buff-requests/preparation.json')
assert ability_prep['world_integration_typecheck_verified']and ability_prep['world_integration_metadata_only']and not ability_prep['world_integration_executed']
metadata_path=Path(ability_prep['world_integration_typecheck']);metadata=read(metadata_path)
assert sha(metadata_path)==ability_prep['world_integration_typecheck_sha256']and metadata['status']=='isolated_whole_crate_metadata_typecheck_qualified_only'
assert not metadata['machine_code_generated']and not metadata['tests_executed']and not metadata['normal_product_modified']
metadata_record=dict(qualification=str(metadata_path),qualification_sha256=sha(metadata_path),candidate_manifest_sha256=metadata['candidate_manifest_sha256'],whole_crate_metadata_typecheck_verified=True,machine_code_generated=False,tests_executed=False,actual_sessions=0,normal_product_modified=False)
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native44FE-library-default') and a.terminal_chunk not in ['','pending','unknown']
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='executed_both_current_full_library_default_selections' and c['owned_process'] is None
assert o['status']=='independently_qualified_current1040_library_both_profiles_and_terminal_processes' and o['cohort_sha256']==sha(run/'cohort.json')
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and o['original_session_id']==a.session and o['terminal_tool_chunk']==a.terminal_chunk
assert c['source_manifest_sha256']==o['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
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
before=read(read(run.parent/'current-continuation.json')['latest_readback'])['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_source_both_full_library_default_selections_qualified_limited',source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,profiles=profiles,run=str(run),original_session_id=a.session,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,completion=str(op),completion_sha256=sha(op),recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,ignored_bodies_executed=False,GUI_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
broad=read(Path(c['input_cohort'])/'completion-observation.json')
assert broad['source_manifest_sha256']==c['source_manifest_sha256'] and broad['recorded_original_owned_processes_all_terminal']
work='當版44FEA738完整library default原'+str(a.session)+'兩版已actual exit0並獨立核對：Debug'+str(profiles['Debug']['passed'])+'／Release'+str(profiles['Release']['passed'])+'過、各0敗，忽略'+str(profiles['Debug']['ignored'])+'／'+str(profiles['Release']['ignored'])+'本文未執行，逐名稱／來源／實際映像／raw與'+str(o['recorded_original_owned_identities'])+'組原程序身分終止均符。廣域Debug'+str(broad['profiles']['Debug']['counts']['passed'])+'／Release'+str(broad['profiles']['Release']['counts']['passed'])+'過維持同來源限定資格。下一步兩種能力強化／S6B-S6C與11特效之隔離完整資源／生命週期及雙玩家實際施法，再必要玩法／正常GUI；元件5案與整庫metadata型別檢查不能替代實際驗證，正常產品尚未套候選。WORLD8sessions、旅館16sessions及控制器／正常模組音效各96sessions保留各自範圍，同端點契約失敗與35辨識／39缺口保留。先自行開發及自動端到端後才人工驗收，開發期間不請使用者登入；8秒／60秒／Release與G11預算不變，相鄰只讀且產物獨立，完整G01–G12未完成。'

for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        j['current_native44FE_library']=record; j['current_native44FE_ability_metadata']=metadata_record
        j['current_full_default_library_verified']=True
        j['current_native44FE_library_active'].update(qualification_granted=True,status='completed_qualified_default_library_scope',completion=str(op),completion_sha256=sha(op),terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0)
        j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='current44FE-audio-handoff-and-necessary-resource-qualification',work=work,applies_to_current_source=True,goal_complete=False)
        j['current_source_goal_validation']['G03-current-regression']=dict(state='current44FE_broad_and_full_library_default_both_qualified_other_acceptance_pending',complete=False,evidence=record)
        for item in j.get('items',[]):
            if item['id']=='G03-current-regression':item.update(state='current44FE_broad_and_full_library_default_both_qualified_other_acceptance_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
            if item['id'].startswith('G07-')and profiles['Debug'].get('diagnostic_visual_readback'):item.update(current_library_diagnostic_visual=profiles['Debug']['diagnostic_visual_readback'],complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(path,j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},evidence_sha256=sha(out/'evidence.json'),before_archive_sha256=sha(out/'before.zip')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(**record,checkpoint=str(out),latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_session_before_restarting=False,next_action='Current-source ability/finite resources and actual sessions; no manual login during development or GUI/physical promotion.'))
print(json.dumps(dict(checkpoint=str(out),profiles=profiles,whole_goal_complete=False),ensure_ascii=False))
