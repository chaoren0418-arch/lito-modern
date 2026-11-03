"""Record prepared integration without mistaking preparation for executed resource evidence."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import ast,hashlib,json,subprocess,zipfile,time
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
broad=V/'native44FE-default112/20261018-174151-282798';b=read(broad/'cohort.json')
assert b['status']=='executing_current_default112_profile'and b['active_profile']=='Release'
continuation=read(broad.parent/'current-continuation.json');before=read(continuation['latest_readback'])['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
for n,h in read(Path(b['normal_qualification']).parent.parent/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(b['pipeline_pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
r=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert r.returncode==0 and not r.stderr
live=json.loads(r.stdout.decode('utf-8-sig'));assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(b['pipeline_creation_utc'])).total_seconds())<.001
candidate=ROOT/'維護工具/candidates/native44FE-extra-skill-effects-integration';prep=read(candidate/'preparation.json')
assert not prep['applied_to_normal_product']and not prep['typecheck_verified']and not prep['resource_probe_executed']
assert sha(candidate/'resource_probe.rs')==prep['resource_probe_sha256']
for n,h in prep['files'].items():assert sha(candidate/n)==h,n
scripts=[ROOT/('維護工具/'+n+'-native44FE-extra-finite-effects.py')for n in ['compile','run','qualify']]+[ROOT/'維護工具/checkpoint-native44FE-default112-completed.py']
for p in scripts:ast.parse(p.read_text('utf-8-sig'))
files={p.relative_to(ROOT).as_posix():p for p in scripts}
files.update({p.relative_to(ROOT).as_posix():p for p in candidate.iterdir()if p.is_file()})
out=V/'native44FE-extra-finite-effects'/('preparation-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'prepared-inputs.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'prepared-inputs.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
record=dict(status='isolated_lazy_finite_effect_integration_and_resource_lifecycle_probes_prepared_only',source_manifest_sha256=b['source_manifest_sha256'],
    candidate=str(candidate),graphics=[746,750,755,763,2171,2176,2177,2232,2236,2251,6342],prepared_inputs=pins,prepared_archive_sha256=sha(out/'prepared-inputs.zip'),
    next_checks=['compile complete isolated library','23 existing resource lifecycle regressions','11 original SPR and PCM decode/visible frames/source-event traversal','pre-cancel keeps old Model','same-map new generation retires bank','reused effect sequence plays fresh cue once'],
    prepared_scripts_AST_parsed=True,candidate_integrated_library_compiled=False,SPR_decode_verified=False,PCM_decode_verified=False,resource_probe_executed=False,
    actual_sessions=0,actual_casts=0,normal_source_modified=False,candidate_applied=False,normal_GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',dict(preparation=record,original_broad_live=live,original_broad_session=67934))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
base=read(ROOT/'工作佇列.json')['next_task'];sentence='11種特效的延後載入整合候選、23案既有資源回歸及新增繪製／事件／取消與世界世代探針已備妥，尚未編譯／執行整合候選，不能當作資源或施法通過。'
work=base+sentence;date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['active_workload_sessions']=={'broad_default':67934}and x['source_and_corresponding_QA_frozen']
        x.update(current_native44FE_finite_effect_preparation=record,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'updated_date'in x:x['updated_date']=date
        if 'current_focus'in x:x['current_focus']['work']=work
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d')
        p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
continuation.update(latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),finite_effect_preparation=record);save(broad.parent/'current-continuation.json',continuation)
print(json.dumps(dict(checkpoint=str(out),original_broad_session=67934,original_broad_live=True,candidate_prepared_only=True,normal_modified=False),ensure_ascii=False))
