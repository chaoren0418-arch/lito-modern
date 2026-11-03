"""Reserve the actual birth-verified broad pipeline after WORLD and GUI retirement."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native44FE-default112')
c=read(run/'cohort.json');assert c['status']in ['executing_current_default112_profile','independently_reading_current_default112_profile']
assert c['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
pid=c['pipeline_pid'];command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,CommandLine,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig'));assert live['ProcessId']==pid and 'run-native44FE-default112-both.py'in live['CommandLine']
assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(c['pipeline_creation_utc'])).total_seconds())<.001
normal=Path(c['normal_qualification']).parent.parent
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
for n,h in c['validation_pins'].items():assert sha(ROOT/'維護工具'/n)==h,n
previous=read(V/'native44FE-world-tree/current-continuation.json');prior=Path(previous['latest_readback']);before=read(prior)['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
assert not read(ROOT/'工作佇列.json')['active_workloads']
out=run/'active-checkpoint';out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='running_current44FE_broad_default112_cohort',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session,
    pipeline=dict(pid=pid,creation_utc=c['pipeline_creation_utc']),observed_live=live,qualification_granted=False,
    profiles=['Debug','Release'],integration_targets_per_profile=112,expected_executables_per_profile=113,
    full_library_in_this_scope=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='當版44FEA738／75DA69B0兩版WORLD8sessions、旅館16sessions及控制器／正常模組音效各96sessions已各自獨立核對；同端點頻道契約失敗保留。當版廣域原'+str(a.session)+'正沿同一程序執行Debug／Release各112整合目標＋主程式，產品與對應QA固定；須取得原session實際終止並逐映像／來源／測試名稱／raw及原程序身分獨立讀回，廣域及完整library尚未授予資格。使用者要求先完成開發與自行端到端測試後才人工驗收，已回收登入視窗／服務，開發期間不再要求人工登入。既有時限／G11預算不變，相鄰只讀且產物獨立；其他服務、有效不同資料集ABA、正常GUI／實體輸入音效與完整G01–G12仍待。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        x.update(current_native44FE_broad_active=record,active_workloads={'broad_default':record},active_workload_sessions={'broad_default':a.session},source_and_corresponding_QA_frozen=True,
            next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'updated_date'in x:x['updated_date']=date
        x['current_focus']=dict(item='current-source-broad-default112',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep
        p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(status='original_current_broad_pipeline_live',run=str(run),original_session_id=a.session,pipeline=record['pipeline'],latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=True,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),original_session_id=a.session,pipeline_pid=pid,observed_live=True),ensure_ascii=False))
