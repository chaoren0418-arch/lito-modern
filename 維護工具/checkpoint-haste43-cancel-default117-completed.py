"""Register only the current, independently read default integration/main scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-installed-resource-profile-default117')and a.session>0 and a.terminal_chunk not in ['','unknown','pending']
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='independently_qualified_current_default117_both_profiles' and c['owned_process']is None
assert o['status']=='independently_qualified_current1048_default117_both_profiles_and_terminal_processes'
assert o['original_session_id']==a.session and o['terminal_tool_chunk']==a.terminal_chunk and o['observed_terminal_exit_code']==0
assert o['cohort_sha256']==sha(run/'cohort.json')and o['source_manifest_sha256']==c['source_manifest_sha256']
assert o['recorded_original_owned_processes_all_terminal']and o['actual_test_executables']==236
assert not o['full_library_qualified']and not o['GUI_verified']and not o['whole_goal_complete']
normal=Path(c['normal_qualification']);startup=normal.parent.parent;inputs=read(startup/'compiled-inputs.json');assert len(inputs)==1049
for n,h in inputs.items():assert sha(R/n)==h,n
for profile,pin in c['profiles'].items():
    qp=Path(pin['qualification']);q=read(qp);assert sha(qp)==pin['qualification_sha256']and q['counts']==pin['counts']and q['counts']['failed']==0
    assert q['source_manifest_sha256']==c['source_manifest_sha256']and q['actual_test_executables']==118 and q['integration_targets']==117
    assert not q['full_library_scope_inferred']and not q['GUI_verified']
    assert o['profiles'][profile]['qualification_sha256']==sha(qp)
for profile,h in o['profiles']['Debug']['normal_mains'].items():assert sha(read(normal)['profiles'][profile]['normal']['path'])==h
w=read(R/'工作佇列.json');assert w['active_workload_sessions']=={'broad_default':a.session}
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='current_catalog_source_default117_both_profiles_qualified_limited',run=str(run),completion_observation=str(op),completion_observation_sha256=sha(op),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session,observed_actual_exit_code=0,terminal_tool_chunk=a.terminal_chunk,profiles=o['profiles'],actual_test_executables=236,recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,ignored_bodies_executed=False,full_library_qualified=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
counts='／'.join(profile+str(info['counts']['passed'])+'過'+str(info['counts']['failed'])+'敗'+str(info['counts']['ignored'])+'忽略'for profile,info in o['profiles'].items())
work='來源'+c['source_manifest_sha256'][:8]+'原'+str(a.session)+'：'+'當版廣域兩版實際完成並獨立核對：'+counts+'；完整library為獨立先行範圍，本廣域不執行其忽略本文。'+'修正版所有實測資格均須依當版來源與實際產物建立；2DAF同端點12、音效96及旅館32sessions只保留原版資格。正常GUI畫面擷取仍逾時，完整GUI、裝置、其他G01–G12未完成；先自行驗證才交人工，不請使用者登入，相鄰只讀，8秒／60秒與Release／G11預算不變。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);assert v['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        v['current_native_catalog_broad_qualification']=record;v['current_source_goal_validation'].setdefault('G03-current-regression',{}).update(state='current_source_default_broad_qualified_other_required_scopes_pending',complete=False,evidence=record)
        for item in v.get('items',[]):
            if item.get('id')=='G03-current-regression':item.update(state='current_source_default_broad_qualified_other_required_scopes_pending',complete=False,current_evidence=record,current_evidence_applies_to_current_source=True)
        v.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,candidate_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='current-catalog-default117-qualified-limited',work=work,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in v:v['updated_date']=date
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=False,next_action='Current-source full library and the remaining actual service/visibility/GUI/device scopes; no historical qualification transfer.'))
print(json.dumps(dict(checkpoint=str(out),counts={p:x['counts']for p,x in o['profiles'].items()},whole_goal_complete=False),ensure_ascii=False))
