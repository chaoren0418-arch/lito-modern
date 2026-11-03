"""Track the original full-library workload without transferring candidate scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True)
a=ap.parse_args();r=a.run.resolve(strict=True)
assert r.is_relative_to(ROOT/'驗證紀錄/current/native-startup-library-default')
x=read(r/'cohort.json');binding=read(r/'original-session-binding.json');identity=read(r/'original-runner-process.json')
assert binding['original_session_id']==a.session_id and binding['runner_identity']==identity
pins=read(r/'compiled-inputs.json');assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
active=x['status'] not in ['failed_preserved','executed_both_current_full_library_default_selections']
if active:
    script=f"$p=Get-CimInstance Win32_Process -Filter 'ProcessId={identity['pid']}';if($p){{[pscustomobject]@{{created=$p.CreationDate.ToUniversalTime().ToString('o');command=$p.CommandLine}}|ConvertTo-Json -Compress}}"
    raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',script],encoding='utf-8-sig').strip();assert raw
    live=json.loads(raw);assert 'run-native-startup-library-default.py' in live['command']
    assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(identity['creation_utc'].replace('Z','+00:00'))).total_seconds())<.001
completion=r/'completion-observation.json';qualified=completion.exists()
if qualified:assert read(completion)['observed_terminal_exit_code']==0
record=dict(run=str(r),original_session_id=a.session_id,original_runner=identity,status=x['status'],
    source_manifest_sha256=x['source_manifest_sha256'],profiles=x['profiles'],active_profile=x.get('active_profile'),
    active=active,qualified=qualified,source_and_corresponding_QA_frozen=not qualified,
    ignored_bodies_executed=False,normal_GUI_verified=False,whole_goal_complete=False)
normal=Path(x['normal_qualification']).parent;app=Path(read(normal/'cohort.json')['application']);nextp=app/'next-validation-action.json'
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json',nextp.relative_to(ROOT).as_posix()]
out=r/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
focus_text=('來源91B廣域Debug500／Release496各0敗21既有忽略，以及map563兩版120 sessions已限定核對。'
    +'完整library原'+str(a.session_id)+'目前'+x['status']+'。'
    +('兩版函式庫預設選擇已獨立核對；接續必要原資源與當版正常可見性驗證。' if qualified else '保持產品及對應QA凍結，沿同一原工作讀回。')
    +'九技能／音效候選仍未套用；正常GUI、實體輸入／音效及完整G01–G12未完成。')
for name in names[:4]:
    value=read(ROOT/name);assert value['current_product_identity']['source_manifest_sha256']==x['source_manifest_sha256']
    value.update(current_source91B_full_library=record,active_workload_sessions={'current_full_library_both':a.session_id} if active else {},
        source_freeze_for_current_full_library=not qualified,source_freeze_while_Cargo=bool(active and 'compil' in x['status']),
        next_task='來源91B廣域500/496與map563兩版120sessions已限定核對；原'+str(a.session_id)+'完整library目前'+x['status']+'，保持產品／對應QA凍結。其後必要原資源、正常可見性、GUI/physical與完整G01–G12仍待。',
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    value['current_focus']=dict(item='source91B-current-full-library-and-required-resource-validation',
        work=focus_text,source_manifest_sha256=x['source_manifest_sha256'],
        original_session_id=a.session_id,goal_complete=False,applies_to_current_source=True)
    value['next_focus']=focus_text
    value['next_task']=focus_text
    if name=='工作佇列.json':
        for item in value.get('items',[]):
            if item['id'].startswith('G03'):item.update(state=('source91B_default111_map563_full_library_qualified_required_optins_pending' if qualified else 'source91B_default111_and_map563_qualified_full_library_in_progress'),complete=False)
    save(ROOT/name,value)
value=read(nextp);value.update(current_full_library=record,source_and_corresponding_QA_frozen=not qualified)
save(nextp,value)
save(out/'record.json',dict(scope=record,before=before,before_archive_sha256=sha(out/'before.zip'),after={n:sha(ROOT/n) for n in names}))
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tracker=str(out),status=x['status'],active=active,qualified=qualified,original_session_id=a.session_id),ensure_ascii=False))
