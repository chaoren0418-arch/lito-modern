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
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native33A3-default116')
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='independently_qualified_current_default116_both_profiles'and c['owned_process']is None
assert o['status']=='independently_qualified_current1047_default116_both_profiles_and_terminal_processes'
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal']and o['cohort_sha256']==sha(run/'cohort.json')
assert a.session==o['original_session_id'] and a.terminal_chunk==o['terminal_tool_chunk']and a.terminal_chunk not in ['pending','unknown','']
assert c['source_manifest_sha256']==o['source_manifest_sha256']=='33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21'
profiles={}
for profile in ['Debug','Release']:
    info=c['profiles'][profile];qp=Path(info['qualification']);q=read(qp)
    assert info['qualified']and info['actual_runner_exit_code']==0 and sha(qp)==info['qualification_sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256']and q['counts']['failed']==0
    assert q['actual_test_executables']==117 and q['integration_targets']==116 and not q['full_library_scope_inferred']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip')as z:
        for n,h in q['raw_files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),counts=q['counts'],integration_targets=116,actual_test_executables=117,normal_mains=q['normal_mains'])
normal=Path(c['normal_qualification']).parent.parent;inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(ROOT/n)==h,n
continuation=read(run.parent/'current-continuation.json');before=read(continuation['latest_readback'])['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    if n.endswith('.json'):
        value=read(ROOT/n);assert value['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        assert value['active_workload_sessions']=={'broad_default':a.session}and value['source_and_corresponding_QA_frozen']
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current33A3_default116_both_profiles_qualified_limited',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1047,compiled_input_files=1048,
    profiles=profiles,original_session_id=a.session,terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,completion=str(op),completion_sha256=sha(op),
    recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,full_library_qualified=False,ignored_bodies_executed=False,
    GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)

save(out/'evidence.json',record)
work='當版33A3CCF7廣域原'+str(a.session)+'已actual exit0並獨立核對：Debug'+str(profiles['Debug']['counts']['passed'])+'／Release'+str(profiles['Release']['counts']['passed'])+'過，各0敗、既有及本版opt-in忽略本文未執行；每版116整合＋main共117實際測試映像，原程序身分均終止。只授予廣域default範圍，不推廣完整library或TCP／GUI。正常兩版各31相關案與能力雙玩家8sessions、G06控制器96sessions各保留當版範圍；下一步完整library及Presentation傳送非空音效／其他服務／必要正常可見性。開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立，完整G01–G12未完成。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        value=read(p);value['current_native33A3_default116']=record
        value['current_native33A3_broad_active'].update(status='completed_qualified_default_scope',completion=str(op),completion_sha256=sha(op),terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,qualification_granted=True)
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current33A3-full-library-default',work=work,applies_to_current_source=True,goal_complete=False)
        if 'current_full_default_library_verified'in value:value['current_full_default_library_verified']=False
        if 'current_source_goal_validation'in value:value['current_source_goal_validation']['G03-current-regression']=dict(state='current33A3_default116_qualified_full_library_and_other_acceptance_pending',complete=False,evidence=record)
        for item in value.get('items',[]):
            if item['id']=='G03-current-regression':item.update(state='current33A3_default116_qualified_full_library_and_other_acceptance_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        if 'updated_date'in value:value['updated_date']=date
        save(p,value)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d')
        p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
continuation.update(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=False,next_action='Run current33A3 full-library default selections, then current Presentation nonempty-audio handoff; no manual login during development.')
save(run.parent/'current-continuation.json',continuation)
print(json.dumps(dict(checkpoint=str(out),counts={p:r['counts']for p,r in profiles.items()},whole_goal_complete=False),ensure_ascii=False))
