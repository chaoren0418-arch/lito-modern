"""Register one live cohort; its children share this explicit runtime reservation."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--baseline-readback',type=Path,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native33A3-audio-cohort') and a.session>0
c=read(run/'cohort.json');assert c['status']=='executing_current_normal_body_audio_branch'
identity=read(run/'runner-identity.json');command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(identity['pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
live=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',command]).decode('utf-8-sig'))
assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(identity['creation_utc'])).total_seconds())<.001
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
baseline=read(a.baseline_readback.resolve(strict=True));before=baseline.get('after_files',baseline.get('after'));assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
assert not read(ROOT/'工作佇列.json').get('active_workloads')
out=run/'active-checkpoint';out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='running_current_normal_body_audio_cohort',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],
    diagnostic_input_manifest_sha256=c['diagnostic_input_manifest_sha256'],original_session_id=a.session,original_terminal_exit_code=None,
    runner_identity=identity,observed_live=live,retained_qualified_cases=1,new_cases_pending=11,source_overlays_used=False,
    normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,normal_product_source_modified=False,
    normal_GUI_API_permission_inferred=False,qualification_granted=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work=f'來源33A3CCF7正常模組音效兩版六分支沿原{a.session}執行；保留已獨立核對的Debug拒絕8sessions，不重跑，剩11案須等原程序實際退出及逐案／整批讀回。四個探針先編譯完成，期間不另編譯；正常本體未改，只用隔離C1F API與診斷Player，正常GUI權限／實體音效不推定。當版正常兩版各31相關案與11原資源特效及控制器96sessions維持限定資格。44FE舊資格與當時失敗保留原版；本版同端點租裝接續另案核對。8／60秒、Release及G11界限不變，相鄰只讀且產物獨立，其他服務與完整G06／G01–G12仍待。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        x['current_native33A3_audio_cohort_active']=record
        x.update(active_workloads={'G06_audio':record},active_workload_sessions={'G06_audio':a.session},source_and_corresponding_QA_frozen=True,
            next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='current-normal-body-audio-cohort',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d') and sep
        p.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
cohort_current=run.parent/'current-continuation.json';assert not cohort_current.exists()
save(cohort_current,dict(**record,active_checkpoint=str(out),must_poll_original_session_before_restarting=True,next_action='Poll the same original cohort; only qualify after observed terminal.'))
family=V/'native33A3-teleport-audio/current-continuation.json';previous=read(family);(out/'audio-family-current-before.json').write_bytes(family.read_bytes())
previous.update(status='current_normal_body_audio_cohort_running',cohort=str(run),original_session_id=a.session,original_terminal_exit_code=None,
    must_poll_original_session_before_restarting=True,next_action='Poll the original cohort handle; retain the already qualified Debug refused case.')
save(family,previous);print(json.dumps(dict(checkpoint=str(out),session=a.session,live=live),ensure_ascii=False))
