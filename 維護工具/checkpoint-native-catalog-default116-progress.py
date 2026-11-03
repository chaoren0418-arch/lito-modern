"""Read qualified profile progress without restarting or widening the live broad scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-catalog-default116')and a.session>0
c=read(run/'cohort.json');assert c['status']in ['executing_current_default116_profile','independently_reading_current_default116_profile']
w=read(R/'工作佇列.json');assert w['active_workload_sessions']=={'broad_default':a.session}and w['source_and_corresponding_QA_frozen']and w['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
normal=Path(c['normal_qualification']);inputs=read(normal.parent.parent/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(R/n)==h,n
for n,h in c['validation_pins'].items():assert sha(R/'維護工具'/n)==h,n
profiles={}
for profile,info in c['profiles'].items():
    if not info.get('qualified'):continue
    p=Path(info['qualification']);q=read(p);assert sha(p)==info['qualification_sha256']
    assert q['status']=='independently_qualified_current_default116_integrations_and_main'and q['source_manifest_sha256']==c['source_manifest_sha256']and q['profile']==profile
    assert q['actual_test_executables']==117 and q['integration_targets']==116 and q['counts']==info['counts']and q['counts']['failed']==0
    assert not q['full_library_scope_inferred']and not q['GUI_verified']and sha(p.parent/'raw.zip')==q['raw_archive_sha256']
    profiles[profile]=dict(qualification=str(p),sha256=sha(p),counts=q['counts'],actual_test_executables=117,raw_archive_sha256=q['raw_archive_sha256'],normal_mains=q['normal_mains'])
cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(c['pipeline_pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
z=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert z.returncode==0 and not z.stderr;live=json.loads(z.stdout.decode('utf-8-sig'));assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(c['pipeline_creation_utc'])).total_seconds())<.001
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('progress-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'original-pipeline-CIM.json').write_bytes(z.stdout)
classification='progress'if profiles!=w.get('current_native_catalog_broad_progress',{}).get('qualified_profiles',{})else'verified_wait'
record=dict(status='original_current_catalog_broad_pipeline_live_profile_qualifications_scoped',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session,pipeline=dict(pid=c['pipeline_pid'],creation_utc=c['pipeline_creation_utc']),observed_live=live,qualified_profiles=profiles,classification=classification,full_library_verified=False,GUI_verified=False,ignored_bodies_executed=False,whole_goal_complete=False)
save(out/'evidence.json',record)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
text='；'.join(name+str(q['counts']['passed'])+'過／0敗／'+str(q['counts']['ignored'])+'忽略、117實際映像已獨立核對'for name,q in profiles.items())or'尚無已完成profile資格'
work='來源'+c['source_manifest_sha256'][:8]+'廣域原'+str(a.session)+'仍沿同一程序執行，'+text+'。當前'+str(c.get('active_profile'))+'，來源與對應QA固定；未由部分profile或既有忽略本文推廣整批／完整library／GUI。正常16各版、控制器96、同端點租裝12、音效96及旅館32sessions維持各自限定範圍；map501六個灰色點已核對原S32／0.til寫入，完整相機／GUI未驗。接續讀原session終止，再整批程序身分讀回與完整library，開發期間不請使用者登入；8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立，G01–G12未完成。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d');staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);assert v['active_workload_sessions']=={'broad_default':a.session}and v['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        v['current_native_catalog_broad_progress']=record
        v['current_source_goal_validation'].setdefault('G03-current-regression',{}).update(state='current_source_broad_profile_progress_remaining_profiles_library_and_other_acceptance_pending',complete=False,qualified_broad_profiles=profiles)
        v.update(next_task=work,next_focus=work,last_goal_turn_classification=classification,last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='current-catalog-broad-profile-progress',work=work,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in v:v['updated_date']=date
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
current=read(run.parent/'current-continuation.json');current.update(latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),qualified_profiles=profiles,must_poll_original_before_restart=True);save(run.parent/'current-continuation.json',current)
print(json.dumps(dict(checkpoint=str(out),classification=classification,qualified_profiles={n:p['counts']for n,p in profiles.items()},original_session=a.session,original_pipeline_live=True),ensure_ascii=False))
