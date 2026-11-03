"""Bind source89 map563 progress to the original owned session and evidence."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile
ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(p, v):
    p.write_text(json.dumps(v, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')
ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', required=True, type=int)
ap.add_argument('--dispatch-chunk-id', required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-startup-siege563')
c = read(run/'cohort.json')
assert c['input_kind'] == 'current-default111' and c['selected_profiles'] == ['Debug','Release']
assert c['source_manifest_sha256'] == '89D4B630285B2ED137EFA81CBDB429CF31FE2928C68FD2D4C9EE3180D3D83CC1'
normal_root = Path(c['normal_qualification']).parent
pins = read(normal_root/'compiled-inputs.json')
assert len(pins) == 1029 and all(sha(ROOT/n) == h for n,h in pins.items())
active = c['status'] not in ['failed_preserved', 'executed_current_selected_map563_thirty_player_TW_extended']
identity = c['runner_identity']
live = None
if active:
    script = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(identity['pid'])+"';if($p){[pscustomobject]@{pid=$p.ProcessId;created=$p.CreationDate.ToUniversalTime().ToString('o');command=$p.CommandLine}|ConvertTo-Json -Compress}"
    raw = subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',script], encoding='utf-8-sig').strip()
    assert raw, 'Re-read original session; do not restart an expired observation.'
    live = json.loads(raw)
    assert 'run-native-selector-siege563.py' in live['command']
    assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(identity['creation_utc'])).total_seconds()) < .001
binding = run/'original-session-binding.json'
if binding.exists():
    saved = read(binding)
    assert saved['original_session_id'] == a.session_id and saved['dispatch_tool_chunk_id'] == a.dispatch_chunk_id
    assert saved['runner_identity'] == identity
else:
    save(binding, dict(original_session_id=a.session_id, dispatch_tool_chunk_id=a.dispatch_chunk_id,
        runner_identity=identity, live_observation=live, relaunched=False))
qualified_path = run/'qualified/qualified.json'
qualified = qualified_path.exists()
qualification = None
if qualified:
    q = read(qualified_path)
    assert q['source_manifest_sha256'] == c['source_manifest_sha256'] and q['observed_terminal_exit_code'] == 0
    assert q['actual_formal_sessions'] == 120 and q['original_session_id'] == a.session_id
    assert not active and c['owned_process'] is None
    qualification = dict(path=str(qualified_path), sha256=sha(qualified_path))
record = dict(run=str(run), original_session_id=a.session_id, dispatch_tool_chunk_id=a.dispatch_chunk_id,
    runner_identity=identity, status=c['status'], active_profile=c.get('active_profile'),
    source_manifest_sha256=c['source_manifest_sha256'], active=active, qualified=qualified,
    qualification=qualification, actual_sessions_qualified=120 if qualified else 0,
    normal_EXE_runtime_verified=False, GUI_verified=False, whole_goal_complete=False)
names = ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json',
    'README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for name in names[:5]:
    j = read(ROOT/name)
    assert j['current_product_identity']['source_manifest_sha256'] == c['source_manifest_sha256']
    for key,value in j.get('active_workloads',{}).items():
        assert key == 'selector-map563' and value['original_session_id'] == a.session_id
out = run/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names: z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n,h in before.items())
scope = '已獨立核對兩版120 LOGIN/GAME sessions' if qualified else ('沿原'+str(a.session_id)+'執行，當前'+c['status'])
work = ('來源89D4B630廣域兩版504／500過、完整library兩版各1937過0敗473原忽略均已獨立核對；map563兩版三十人'+scope+'。'
    '必要原資源opt-in、正常可見性與後續技能／服務／GUI／實體音效／同端點／可攜G11G12仍待。'
    '產品及對應QA於本批完成前凍結，原時限／預算不變；完整G01–G12未完成。登入器唯讀且產物獨立，替換提案已撤回。')
for name in names[:5]:
    p = ROOT/name
    j = read(p)
    j.update(current_selector_map563=record,
        active_workloads={'selector-map563':record} if active else {},
        active_workload_sessions={'selector-map563':a.session_id} if active else {},
        source_freeze_for_current_map563=not qualified, source_and_corresponding_QA_frozen=not qualified,
        source_freeze_while_Cargo=False, next_task=work, next_focus=work,
        last_goal_turn_classification='progress' if qualified else 'verified_wait',
        last_goal_turn_evidence=record, goal_complete=False, whole_goal_complete=False,
        progress_updated_utc=datetime.now(timezone.utc).isoformat())
    if name == names[4]: j['next_action'] = work
    else: j['current_focus'] = dict(item='source89D4-map563', work=work, applies_to_current_source=True, goal_complete=False)
    save(p,j)
for name in names[5:]:
    p = ROOT/name
    first, separator, rest = p.read_text(encoding='utf-8-sig').partition('\n\n')
    assert first.startswith('2026-10-17') and separator
    p.write_text('2026-10-17：'+work+'\n\n'+rest, encoding='utf-8', newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(record=record,before_files=before,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out), active=active, qualified=qualified, original_session_id=a.session_id), ensure_ascii=False))
