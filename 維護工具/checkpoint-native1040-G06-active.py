"""Record a birth-verified current-source G06 continuation before awaiting it."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native1040-G06-controller')and a.session_id>0
c=read(run/'cohort.json');job=read(run/'runner-identity.json');pid=job['ProcessId']
cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
live=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',cmd]).decode('utf-8-sig'))
assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())<.001
normal=Path(c['startup_prerequisite']);completion=read(c['startup_qualification'])
assert completion['observed_terminal_exit_code']==0 and completion['recorded_original_owned_processes_all_terminal']
assert completion['source_manifest_sha256']==c['source_manifest_sha256']and c['source_files']==1040
assert sha(c['startup_qualification'])==c['startup_qualification_sha256']
for n,h in read(run/'source-inputs.json').items():assert sha(ROOT/n)==h
previous=next(normal.glob('checkpoint-completed-*/readback.json'));before=read(previous)['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads')
out=run/'active-checkpoint';out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='running_current1040_G06_controller_scope',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session_id,pipeline=job,observed_live=live,source_files=1040,qualification_granted=False,GUI_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源4E1F7A4E兩版正常建置及各90相關檢查已獨立核對。當版G06六分支正在原程序執行，須沿同一session讀取實際終止與逐案封包／角色接續／舊場景退役／失敗恢復證據；不得把舊86E6或候選資格移記。來源與對應QA凍結；八秒／六十秒、Release與G11不變。登入器／正式JAR只讀且產物獨立，GUI／實體輸入音效、完整G06及G01–G12未完成。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        value.update(active_workloads={'G06_controller':record},active_workload_sessions={'G06_controller':a.session_id},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_native1040_G06_active']=record;value['current_focus']=dict(item='current1040-G06-controller-continuation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        for item in value.get('items',[]):
            if item['id'].startswith('G06-'):item.update(state='running_current1040_G06_controller_six_cases',complete=False)
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(**record,active_checkpoint=str(out/'readback.json'),must_poll_original_session_before_restarting=True,normal_validation=str(normal),next_action='Read original G06 terminal, then independently qualify cohort and owned identities before completed checkpoint.'))
print(json.dumps(dict(checkpoint=str(out),original_session_id=a.session_id,live_pipeline=live),ensure_ascii=False))
