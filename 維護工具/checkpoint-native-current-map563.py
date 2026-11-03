"""Bind the original map563 handle to verified live/terminal state and current source."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True)
a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-startup-siege563')
c=read(run/'cohort.json');assert c['input_kind']=='current-default110'
base=Path(c['input_cohort']);b=read(base/'completion-observation.json')
assert b['observed_terminal_exit_code']==0 and b['recorded_original_owned_processes_all_terminal']
assert b['source_manifest_sha256']==c['source_manifest_sha256']
normal=Path(c['normal_qualification']).parent;pins=read(normal/'compiled-inputs.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
active=c['status'] not in ['failed_preserved','executed_current_selected_map563_thirty_player_TW_extended']
identity=c['runner_identity'];live=None
if active:
    script=f"$p=Get-CimInstance Win32_Process -Filter 'ProcessId={identity['pid']}';if($p){{[pscustomobject]@{{pid=$p.ProcessId;created=$p.CreationDate.ToUniversalTime().ToString('o');command=$p.CommandLine}}|ConvertTo-Json -Compress}}"
    raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',script],encoding='utf-8-sig').strip()
    assert raw,'Re-read the original handle/cohort; never restart from an expired observation'
    live=json.loads(raw);assert 'run-native-startup-siege563.py' in live['command']
    assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(identity['creation_utc'])).total_seconds())<.001
binding=run/'original-session-binding.json'
if binding.exists():assert read(binding)['original_session_id']==a.session_id
else:save(binding,dict(original_session_id=a.session_id,dispatch_tool_chunk_id='19759e',runner_identity=identity,live_observation=live,relaunched=False))
qualification=run/'qualified/qualified.json';qualified=qualification.exists()
if qualified:
    q=read(qualification);assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['observed_terminal_exit_code']==0
record=dict(run=str(run),original_session_id=a.session_id,runner_identity=identity,status=c['status'],
    source_manifest_sha256=c['source_manifest_sha256'],active=active,profiles=c['profiles'],qualified=qualified,
    actual_sessions_qualified=120 if qualified else 0,product_source_frozen=not qualified,
    normal_EXE_runtime_verified=False,GUI_verified=False,whole_goal_complete=False)
app=Path(read(normal/'cohort.json')['application']);nextp=app/'next-validation-action.json'
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',nextp.relative_to(ROOT).as_posix()]
out=run/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
for name in names[:4]:
    x=read(ROOT/name);assert x['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    x.update(current_source91B_map563=record,active_workload_sessions={'current_map563_both':a.session_id} if active else {},
        source_freeze_for_current_map563=not qualified,source_freeze_while_Cargo=False,
        next_task='來源91B廣域兩版111映像已500/496過、0敗及各21既有忽略；原'+str(a.session_id)+' map563兩版三十人'+c['status']+'。保持本版產品／對應QA凍結，後續完整library、必要原資源、正常可見性與GUI/physical仍待。',
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    if name=='工作佇列.json':
        for item in x.get('items',[]):
            if item['id'].startswith('G03'):
                item.update(state='source91B_default111_both_qualified_library_map_visibility_pending',complete=False)
    save(ROOT/name,x)
x=read(nextp);x.update(current_map563=record,source_and_corresponding_QA_frozen=not qualified)
save(nextp,x)
save(out/'record.json',dict(scope=record,before=before,before_archive_sha256=sha(out/'before.zip'),after={n:sha(ROOT/n) for n in names}))
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tracker=str(out),status=c['status'],active=active,qualified=qualified,original_session_id=a.session_id),ensure_ascii=False))
