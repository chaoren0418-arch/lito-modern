"""Checkpoint an observed live original G06 job, allowing normal job-boundary races."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,hashlib,zipfile,subprocess,time
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--dispatch-chunk-id',required=True);ap.add_argument('--skill-audit',type=Path);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-G06-controller');live=None
for attempt in range(20):
 try:c=read(run/'cohort.json')
 except json.JSONDecodeError:time.sleep(.05);continue
 job=c.get('owned_process')
 if job is None:time.sleep(.1);continue
 cmd='[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter "ProcessId='+str(job['pid'])+'"|Select-Object ProcessId,ParentProcessId,CommandLine,@{n="created";e={$_.CreationDate.ToUniversalTime().ToString("o")}}|ConvertTo-Json -Compress'
 p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
 if not p.stdout.strip():time.sleep(.05);continue
 row=json.loads(p.stdout.decode('utf-8-sig'));identity=read(run/'runner-identity.json')
 if abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>=.001:time.sleep(.05);continue
 assert row['ProcessId']==job['pid'] and row['ParentProcessId']==identity['ProcessId']
 assert str(ROOT).casefold() in row['CommandLine'].casefold() or '--test native_teleport_' in row['CommandLine']
 live=row;break
assert live is not None,'No live child observed; inspect the same original runtime rather than relaunching'
pins=read(run/'source-inputs.json');assert len(pins)==1028 and sha(run/'source-inputs.json')==c['source_manifest_sha256']
for name,h in pins.items():assert sha(ROOT/name)==h,name
qualified_cases={}
for profile in c['profiles']:
 cases={}
 for qp in (run/profile).glob('qualified-*/qualified.json'):
  q=read(qp);assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['original_owned_processes_terminal'];cases[q['scenario']]=dict(qualification=str(qp),qualification_sha256=sha(qp),sessions=q['actual_formal_sessions'])
 qualified_cases[profile]=dict(cases=cases,sessions=sum(v['sessions'] for v in cases.values()))
audit=None
if a.skill_audit:
 apath=a.skill_audit.resolve(strict=True);assert apath.is_relative_to(ROOT/'驗證紀錄/current/native-current-skill-support');aq=read(apath);assert aq['source_manifest_sha256']==c['source_manifest_sha256'] and aq['new_casts_executed']==0 and aq['request_recognition_is_not_gameplay_acceptance'];audit=dict(path=str(apath),sha256=sha(apath),source_manifest_sha256=aq['source_manifest_sha256'],learned=aq['observed_unique_learned_ids'],recognized_requests=aq['request_recognized_learned_ids'],missing_requests=aq['missing_unique_learned_requests'],new_casts=0,request_recognition_is_not_gameplay_acceptance=True)
out=run/('active-continuation-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'owned-CIM.json').write_bytes(p.stdout);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md'];before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
 for n,h in before.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
now=datetime.now(timezone.utc).isoformat();active=dict(original_session_id=a.session_id,initial_tool_chunk_id=a.dispatch_chunk_id,run=str(run),status=c['status'],active_profile=c.get('active_profile'),active_scenario=c.get('active_scenario'),source_manifest_sha256=c['source_manifest_sha256'],observed_owned_process=job,CIM=live,observed_utc=now,whole_G06_complete=False)
active['qualified_case_scopes']=qualified_cases
scope='；'.join(profile+str(len(v['cases']))+'案／'+str(v['sessions'])+'sessions' for profile,v in qualified_cases.items())
work='新來源89D4B630正常兩版相關範圍已獨立核對（Debug155／Release151過、各0失敗3忽略）。G06原工作'+str(a.session_id)+'仍在'+str(c.get('active_profile'))+'執行；已獨立核對'+scope+'，部分案例不等於整批通過。原控制情境／時限不變，保留原handle；之後新來源廣域111／全library／地圖及其他玩法／GUI音效／可攜G11G12與完整目標仍待。登入器與重建版產物獨立，替換提案撤回。'
if audit:work+=' 來源綁定技能入口盤點'+str(audit['recognized_requests'])+'/'+str(audit['learned'])+'、缺'+str(audit['missing_requests'])+'；此為靜態入口盤點，非新增施法或玩法驗收。'
for n in names[:5]:
 path=ROOT/n;j=read(path);j['active_workloads']={'G06-selector-source':active};j['active_workload_sessions']={'G06-selector-source':active};j['current_selector_G06_controller']=active;j['source_freeze_for_current_G06_controller']=dict(frozen=True,run=str(run),source_manifest_sha256=c['source_manifest_sha256'],prerequisite=c['startup_qualification'],prerequisite_sha256=c['startup_qualification_sha256']);j['goal_complete']=False;j['progress_updated_utc']=now
 j['next_task']=work;j['independent_projects']=True;j['launcher_artifact_replacement_required']=False
 if audit:j['current_skill_gap_matrix']=audit
 if n!=names[4]:j['current_focus']=dict(item='new-source-G06-controller-active',work=work,applies_to_current_source=True,goal_complete=False);j['next_focus']=work
 else:j['next_action']=work
 if isinstance(j.get('items'),list):
  for item in j['items']:
   if item['id'].startswith('G06'):item['state']='source89D4_current_G06_controller_work_active';item['current_validation_run']=str(run)
   if item['id'].startswith('G01'):item['current_source_evidence_scope']='正常Debug與Release兩版29相關EXE、155／151過及唯一入口正常產品均已核對；僅此基準範圍通過。'
 path.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
paragraph='2026-10-17：'+work+' 正常Debug5D634328／Release1E452012；GUI／實體輸入／同端點與完整G06及G01–G12未完成。'
for n in names[5:]:
 path=ROOT/n;s=path.read_text(encoding='utf-8-sig');first,sep,rest=s.partition('\n\n');assert first.startswith('2026-10-17') and sep;path.write_text(paragraph+'\n\n'+rest,encoding='utf-8',newline='\n')
rb=dict(status='checkpointed_observed_live_current_source_G06_work',active=active,before_files=before,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n) for n in names},whole_goal_complete=False);(out/'readback.json').write_text(json.dumps(rb,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(status=rb['status'],session=a.session_id,observed_pid=job['pid'],phase=c['status']),ensure_ascii=False))
