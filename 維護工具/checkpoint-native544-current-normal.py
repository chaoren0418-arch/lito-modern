"""Track only the original live normal pipeline; never promote old source claims."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile,time
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for attempt in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int);ap.add_argument('--dispatch-chunk-id',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native544-integration')
c=read(run/'cohort.json');own=read(run/'original-runner-process.json')
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(own['pid'])+"';if($p){[pscustomobject]@{pid=$p.ProcessId;created=$p.CreationDate.ToUniversalTime().ToString('o');command=$p.CommandLine}|ConvertTo-Json -Compress}"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps],encoding='utf-8-sig').strip();assert raw,'Poll the original session; do not relaunch'
live=json.loads(raw);assert 'run-native544-current-normal.py' in live['command'] and abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(own['creation_utc'])).total_seconds())<.001
binding=run/'original-session-binding.json'
if binding.exists():assert read(binding)['original_session_id']==a.session_id
else:save(binding,dict(original_session_id=a.session_id,dispatch_tool_chunk_id=a.dispatch_chunk_id,runner_identity=own,live_CIM=live,relaunched=False))
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1029 and all(sha(ROOT/n)==h for n,h in inputs.items())
record=dict(run=str(run),original_session_id=a.session_id,dispatch_tool_chunk_id=a.dispatch_chunk_id,runner_identity=own,status=c['status'],active_profile=c.get('active_profile'),
    owned_process=c['owned_process'],source_manifest_sha256=c['source_manifest_sha256'],qualified=False,GUI_verified=False,whole_goal_complete=False)
record['qualified_profiles']={}
for profile in ['Debug','Release']:
    qp=run/profile/'qualified/qualified.json'
    if qp.exists():
        q=read(qp);assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['recorded_profile_jobs_all_terminal']
        record['qualified_profiles'][profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),normal_sha256=q['normal_sha256'],unique_sprite_tests_executed=q['unique_sprite_tests_executed'])
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:
    j=read(ROOT/n);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    for k,v in j.get('active_workloads',{}).items():assert k=='native544-normal' and v['original_session_id']==a.session_id
out=run/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='來源'+c['source_manifest_sha256'][:8]+'精確544空白修正已套用一檔；正常建置／解碼器相關驗證沿原'+str(a.session_id)+'，狀態'+c['status']+'／'+str(c.get('active_profile'))+'。產品及對應QA凍結。舊89D4各批只保留原版本資格；新來源完整G06／同端點、其餘技能服務與GUI實體輸入音效／可攜G11G12仍待，完整G01–G12未完成。登入器唯讀且產物獨立，原時限／預算不變。'
for n in names[:5]:
    path=ROOT/n;j=read(path);identity=j['current_product_identity']
    for profile,info in c['profiles'].items():
        if info.get('normal'):
            assert sha(info['normal']['path'])==info['normal']['sha256']
            identity[profile+'_normal_sha256']=info['normal']['sha256'];identity[profile+'_normal_source_manifest_sha256']=c['source_manifest_sha256']
            identity[profile+'_normal_state']='built_pending_independent_qualification'
            if profile in record['qualified_profiles']:
                identity[profile+'_normal_state']='independently_qualified_sprite_related_only'
                identity[profile+'_normal_qualified_for_current_source']=True
        elif profile==c.get('active_profile'):
            identity[profile+'_normal_state']='building';identity[profile+'_normal_sha256']=None
    if all(c['profiles'].get(profile,{}).get('normal') for profile in ['Debug','Release']):
        identity.pop('existing_normal_source_manifest_sha256',None)
    j.update(current_native544_normal=record,active_workloads={'native544-normal':record},active_workload_sessions={'native544-normal':a.session_id},
        source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,goal_complete=False,whole_goal_complete=False,
        progress_updated_utc=datetime.now(timezone.utc).isoformat())
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='exact544-normal-active',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for n in names[5:]:
    path=ROOT/n;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(scope=record,before_files=before,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out),original_session_id=a.session_id,status=c['status'],active_profile=c.get('active_profile')),ensure_ascii=False))
