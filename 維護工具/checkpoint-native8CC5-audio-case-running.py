"""Record a birth-verified individual audio case without granting qualification."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--baseline-readback',type=Path,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native8CC5-teleport-audio') and a.session>0
r=read(run/'receipt.json');assert r['status']=='executing_current_normal_body_nonempty_audio_case'
root=read(run/'runner-identity.json');command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(root['pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
live=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',command]).decode('utf-8-sig'))
assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(root['creation_utc'])).total_seconds())<.001
build=Path(r['compile']);b=read(build/'receipt.json');assert b['status']=='compiled_current_normal_body_audio_diagnostics' and read(build/'compile-terminal-observation.json')['observed_terminal_exit_code']==0
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h
baseline=read(a.baseline_readback.resolve(strict=True));before=baseline.get('after_files',baseline.get('after'));assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and queue['source_and_corresponding_QA_frozen']
out=run/'active-checkpoint';out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
family=run.parent/'current-continuation.json';previous=read(family);(out/'family-current-before.json').write_bytes(family.read_bytes())
record=dict(status='running_current_normal_body_nonempty_audio_case',run=str(run),profile=r['profile'],scenario=r['scenario'],
    source_manifest_sha256=r['source_manifest_sha256'],diagnostic_input_manifest_sha256=r['diagnostic_input_manifest_sha256'],
    original_session_id=a.session,original_terminal_exit_code=None,runner_identity=root,observed_live=live,
    normal_source_bodies_unchanged=True,source_overlays_used=False,diagnostic_request_API_extension=True,
    normal_GUI_API_permission_inferred=False,normal_product_source_modified=False,qualification_granted=False,
    GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work=f"來源8CC5A560之{r['profile']}／{r['scenario']}正常模組音效診斷沿原{a.session}執行中；使用真實C1F／S20待播音效，資格尚未授予。正常模組本體未改，只在隔離副本附加請求API，正常GUI請求權限不推定。當版控制器兩版12案／96sessions與正常各130相關測試已限定核對；其他音效分支、正常GUI／實體音效、同端點及完整G06／G01–G12仍待。8／60秒、Release／G11預算不變，相鄰只讀、產物獨立。"
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==r['source_manifest_sha256']
        x['current_native8CC5_audio_active']=record
        x.update(active_workloads={'G06_audio':record},active_workload_sessions={'G06_audio':a.session},source_and_corresponding_QA_frozen=True,
            next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='current-normal-body-nonempty-audio-preflight',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
previous.update(record,active_checkpoint=str(out),must_poll_original_session_before_restarting=True,next_action='Poll the original case handle; preserve failures. Only independently qualify after the actual terminal exit.')
save(family,previous);print(json.dumps(dict(checkpoint=str(out),session=a.session,live=live),ensure_ascii=False))
