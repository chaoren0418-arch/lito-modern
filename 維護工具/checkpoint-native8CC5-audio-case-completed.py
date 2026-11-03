"""Record one independently qualified current-source branch without promoting the rest."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native8CC5-teleport-audio')
r=read(run/'receipt.json');assert r['status']=='executed_current_normal_body_nonempty_audio_case' and r['owned_process'] is None
qp=next(run.glob('qualified-*/qualified.json'));q=read(qp)
assert q['status']=='independently_qualified_current_normal_bodies_nonempty_audio_branch' and q['original_session_id']==a.session and q['original_exit_code']==0 and q['terminal_tool_chunk']==a.terminal_chunk
assert q['recorded_owned_all_terminal'] and q['retained_source_cues']==2 and q['unplayed_source_audio_rejection_verified'] and q['new_scene_own_sound_events_delivered_once_each']
assert q['source_manifest_sha256']==r['source_manifest_sha256'] and q['normal_source_bodies_unchanged'] and not q['source_overlays_used'] and q['diagnostic_request_API_extension']
assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'raw.zip') as z:
    assert set(z.namelist())==set(q['raw_files'])
    for n,h in q['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
for n,h in read(Path(r['compile'])/'compiled-inputs.json').items():assert sha(ROOT/n)==h
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('qualified-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(run=str(run),qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=q['source_manifest_sha256'],
    diagnostic_input_manifest_sha256=q['diagnostic_input_manifest_sha256'],profile=q['profile'],scenario=q['scenario'],
    original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_sessions=q['actual_sessions'],
    actual_product_sha256=q['actual_product_sha256'],recorded_owned_identities=q['recorded_owned_identities'],recorded_owned_all_terminal=True,
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,
    normal_GUI_API_permission_inferred=False,normal_product_source_modified=False,actual_Presentation_executed=True,
    unplayed_source_audio_rejection_verified=True,retained_source_cues=2,new_scene_own_sound_events_delivered_once_each=True,
    normal_EXE_runtime_inferred=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record);key=q['profile']+'-'+q['scenario']
work=f"來源8CC5A560之{q['profile']}／{q['scenario']}正常模組音效診斷原{a.session}已實際退出0並獨立核對：{q['actual_sessions']}條sessions、每端1個真實S20待播舊音效Stale／重複AlreadyConsumed與零交付，新的場景音效各交付一次；來源、原始封包、完整物品、4張PNG與記錄程序終止已核對。只授予此版別分支、隔離API與假Player範圍，正常GUI／實體音效不推定；其他音效分支與完整G06／G01–G12仍待。正常各130相關檢查及控制器兩版96sessions保留其限定資格，8／60秒、Release／G11預算不變，相鄰只讀且產物獨立。"
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
        x.setdefault('current_native8CC5_audio_cases',{})[key]=record
        x['current_native8CC5_audio_active'].update(status='completed_independently_qualified_branch',qualification=str(qp),qualification_sha256=sha(qp),original_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk)
        x.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='remaining-current-normal-body-nonempty-audio-branches',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
family=run.parent/'current-continuation.json';current=read(family);(out/'family-current-before.json').write_bytes(family.read_bytes())
current.setdefault('qualified_cases',{})[key]=record
current.update(status='current_normal_body_nonempty_audio_case_qualified_remaining_cases_pending',original_terminal_exit_code=0,
    must_poll_original_session_before_restarting=False,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),
    next_action='Continue only the remaining unqualified profile/scenario pairs with the same four compiled probes; preserve this result instead of rerunning it.',
    retained_limitation='Same-World new-Model/resource-root ABA event admission, normal GUI and physical audio remain unqualified.')
save(family,current);print(json.dumps(dict(checkpoint=str(out),profile=q['profile'],scenario=q['scenario'],actual_sessions=q['actual_sessions'],whole_goal_complete=False),ensure_ascii=False))
