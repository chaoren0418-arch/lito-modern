"""Qualify completed Release children without claiming their failed parent passed."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,sys,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('parent',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--parent-terminal-chunk',required=True);ap.add_argument('--parent-terminal-exit-code',type=int,required=True);a=ap.parse_args()
parent=a.parent.resolve(strict=True);assert parent.is_relative_to(V/'native8CC5-audio-cohort') and a.parent_terminal_exit_code==1
c=read(parent/'cohort.json');assert c['status']=='failed_preserved' and c['active_profile']=='Debug' and c['owned_process'] is None
fp=next(parent.glob('failure-checkpoint-*/failure.json'));failure=read(fp)
assert failure['original_session_id']==a.session and failure['original_terminal_exit_code']==a.parent_terminal_exit_code and failure['terminal_tool_chunk']==a.parent_terminal_chunk
assert failure['recorded_owned_all_terminal'] and failure['recorded_owned_identities']==43 and failure['actual_C1F_requests']==0
assert len(c['cases'])==6 and c['actual_sessions']==48 and all(e['profile']=='Release' for e in c['cases'].values())
inputs=read(parent/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,h in c['tool_pins'].items():assert sha(n)==h
out=parent/'qualified-Release-after-parent-failure';out.mkdir(exist_ok=True);assert not(out/'qualified.json').exists()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());index=out/'case-qualifications.json';proofs=read(index) if index.exists() else {}
reader=ROOT/'維護工具/qualify-native8CC5-audio-completed-Release-child.py'
for key,entry in c['cases'].items():
    run=Path(entry['run']);assert sha(run/'receipt.json')==entry['receipt_sha256']
    if key not in proofs:
        if entry.get('retained_without_rerun'):
            qp=Path(entry['retained_qualification']);assert sha(qp)==entry['retained_qualification_sha256']
        else:
            existing=list(run.glob('qualified-*/qualified.json'))
            if existing:assert len(existing)==1;qp=existing[0]
            else:
                command=[sys.executable,'-B','-X','utf8',str(reader),str(run),'--session-id',str(a.session),'--terminal-chunk',a.parent_terminal_chunk,
                    '--terminal-exit-code','0','--parent-cohort',str(parent),'--parent-terminal-exit-code','1']
                child=subprocess.run(command,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
                (out/(key+'.stdout.txt')).write_bytes(child.stdout);(out/(key+'.stderr.txt')).write_bytes(child.stderr)
                assert child.returncode==0,(key,'Preserve reader failure; do not rerun successful native cases')
                paths=[line[len('QUALIFICATION='):] for line in child.stdout.decode('utf-8-sig').splitlines() if line.startswith('QUALIFICATION=')];assert len(paths)==1;qp=Path(paths[0])
        proofs[key]=dict(path=str(qp),sha256=sha(qp));save(index,proofs)
    qp=Path(proofs[key]['path']);q=read(qp);assert sha(qp)==proofs[key]['sha256']
    assert q['profile']=='Release' and q['scenario']==entry['scenario'] and q['actual_sessions']==entry['actual_sessions']
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['diagnostic_input_manifest_sha256']==c['diagnostic_input_manifest_sha256']
    assert q['normal_source_bodies_unchanged'] and not q['source_overlays_used'] and q['diagnostic_request_API_extension']
    assert q['retained_source_cues']==2 and q['unplayed_source_audio_rejection_verified'] and q['new_scene_own_sound_events_delivered_once_each'] and q['recorded_owned_all_terminal']
    if entry.get('retained_without_rerun'):
        assert q['status']=='independently_qualified_current_normal_bodies_nonempty_audio_branch' and q['original_exit_code']==0 and q['original_session_id']==entry['retained_original_session_id']
    else:
        assert q['status']=='independently_qualified_completed_Release_audio_child_of_failed_Debug_parent'
        assert q['observed_parent_session_id']==a.session and q['observed_parent_terminal_exit_code']==1 and q['parent_terminal_tool_chunk']==a.parent_terminal_chunk and q['observed_case_process_exit_code']==0
    assert not q['GUI_verified'] and not q['normal_EXE_runtime_inferred'] and not q['physical_audio_verified']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        assert set(z.namelist())==set(q['raw_files'])
        for n,h in q['raw_files'].items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    print(key+' independently qualified as a completed Release child; parent remains exit1',flush=True)
assert set(proofs)==set(c['cases'])
old=0;wave=0;recovery=0;heal=0;pngs=[];products=set()
for key,pin in proofs.items():
    q=read(pin['path']);old+=q['retained_source_cues'];products.add(q['actual_product_sha256']);pngs+=q['diagnostic_PNGs']
    for proof in q['wire_audio_proofs']:
        assert proof['old_first']=='Stale' and proof['old_repeat']=='AlreadyConsumed' and proof['old_deliveries']==0;wave+=1
        if key=='Release-cross-endpoint':assert proof['new_audio']['delivered_once'];heal+=1
        else:assert proof['new_audio']['S20_lines'];recovery+=1;wave+=1
assert(old,wave,recovery,heal,len(pngs),len(products))==(12,22,10,2,24,2)
for n,h in inputs.items():assert sha(ROOT/n)==h
files={'parent-cohort.json':parent/'cohort.json','parent-failure.json':fp,'case-qualifications.json':index,'reader.py':Path(__file__),'child-reader.py':reader}
pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
record=dict(status='independently_qualified_Release_six_completed_audio_branches_only_after_Debug_parent_failure',
    source_manifest_sha256=c['source_manifest_sha256'],diagnostic_input_manifest_sha256=c['diagnostic_input_manifest_sha256'],
    observed_parent_session_id=a.session,observed_parent_terminal_exit_code=1,parent_terminal_tool_chunk=a.parent_terminal_chunk,
    parent_failure=dict(path=str(fp),sha256=sha(fp)),whole_parent_cohort_passed=False,profile='Release',actual_cases=6,actual_sessions=48,
    actual_diagnostic_executables=2,cases=proofs,retained_source_cues=12,old_cue_deliveries=0,actual_C1F_wave_requests=22,
    recovered_ordinary_wave_deliveries=10,destination_native_heal_deliveries=2,new_scene_sound_events_delivered_once=12,
    actual_Presentation_executed=True,unplayed_source_audio_rejection_verified=True,normal_source_bodies_unchanged=True,source_overlays_used=False,
    diagnostic_request_API_extension=True,normal_GUI_API_permission_inferred=False,product_runtime_reexecuted=False,
    normal_product_source_modified=False,normal_EXE_runtime_inferred=False,diagnostic_PNGs=pngs,raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,
    Debug_scope_qualified=False,same_World_new_Model_ABA_event_admission_verified=False,GUI_verified=False,physical_audio_verified=False,
    same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'qualified.json',record);print('RELEASE_QUALIFICATION='+str(out/'qualified.json'));print(json.dumps(dict(cases=6,sessions=48,parent_exit_code=1,old_cues_rejected=12,new_events_delivered_once=12,runtimes_reexecuted=0),ensure_ascii=False))
