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
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-installed-resource-profile-default117')
c=read(run/'cohort.json');assert c['status']in ['executing_current_default117_profile','independently_reading_current_default117_profile']
assert c['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
pid=c['pipeline_pid'];command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,CommandLine,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig'));assert live['ProcessId']==pid and 'run-text-index-order-default117-both.py'in live['CommandLine']
assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(c['pipeline_creation_utc'])).total_seconds())<.001
normal=Path(c['normal_qualification']).parent.parent
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
for n,h in c['validation_pins'].items():assert sha(ROOT/'維護工具'/n)==h,n
prior=Path(read(ROOT/'工作佇列.json')['last_goal_turn_evidence']).with_name('readback.json');before=read(prior)['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
assert not read(ROOT/'工作佇列.json')['active_workloads']
out=run/'active-checkpoint'
if out.exists():out=run/('active-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='running_current_catalog_broad_default117_cohort',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session,
    pipeline=dict(pid=pid,creation_utc=c['pipeline_creation_utc']),observed_live=live,qualification_granted=False,
    profiles=['Debug','Release'],integration_targets_per_profile=117,expected_executables_per_profile=118,
    full_library_in_this_scope=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源'+c['source_manifest_sha256'][:8]+'原'+str(a.session)+'：'+'當版正常兩版及控制器96已核對；廣域117整合目標及main兩版仍執行中，未給予整批資格。'+'修正版所有實測資格均須依當版來源與實際產物建立；2DAF同端點12、音效96及旅館32sessions只保留原版資格。正常GUI畫面擷取仍逾時，完整GUI、裝置、其他G01–G12未完成；先自行驗證才交人工，不請使用者登入，相鄰只讀，8秒／60秒與Release／G11預算不變。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
staged={}
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        x.update(current_native_catalog_broad_active=record,active_workloads={'broad_default':record},active_workload_sessions={'broad_default':a.session},source_and_corresponding_QA_frozen=True,
            next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'updated_date'in x:x['updated_date']=date
        x['current_focus']=dict(item='current-source-broad-default117',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        staged[n]=(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep
        staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(ROOT/n)==h for n,h in before.items())
for n,b in staged.items():(ROOT/n).write_bytes(b)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(status='original_current_broad_pipeline_live',run=str(run),original_session_id=a.session,pipeline=record['pipeline'],latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=True,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),original_session_id=a.session,pipeline_pid=pid,observed_live=True),ensure_ascii=False))
