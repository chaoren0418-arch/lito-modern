"""Seal the observed failure; never grant gameplay success or modify a launcher artifact."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, time, zipfile

ROOT = Path(__file__).resolve().parents[1]
V = ROOT / '驗證紀錄/current'

def read(p):
    for i in range(10):
        try:
            return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i == 9:
                raise
            time.sleep(.05)

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    Path(p).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session', required=True, type=int)
ap.add_argument('--terminal-chunk', required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(V / 'native44FE-same-endpoint')
r = read(run / 'receipt.json')
case = Path(r['case']).resolve(strict=True)
c = read(case / 'receipt.json')
runtime = Path(c['runtime']).resolve(strict=True)
assert case.is_relative_to(V / 'native-teleport-handoff')
assert runtime.is_relative_to(ROOT / 'LiTo381-Modern/target/current/native-teleport-handoff')
assert r['status'] == 'current_same_endpoint_run_terminal_pending_independent_readback'
assert c['status'] == 'failed' and c['scenario'] == 'same-endpoint'
assert (r['actual_PowerShell_exit_code'], c['test_exit_code'], c['server_exit_code']) == (1, 101, 0)
assert sha(case / 'receipt.json') == r['case_receipt_sha256']
assert sha(runtime / 'immutable-teleport-test.exe') == c['test_binary_sha256']
assert r['owned_process'] is None and r['process_history'][0]['exit_code'] == 1
for n, h in read(run / 'source-inputs.json').items():
    assert sha(ROOT / n) == h, n
for n, h in r['pins'].items():
    assert sha(n) == h, n

result = read(runtime / 'result.json')
assert result == read(case / 'result.json')
assert result['status'] == 'sessions_closed' and result['actual_client_sessions'] == 6
assert result['jar_sha256'].upper() == r['reference_jar_sha256']
assert result['authority_mutations'] == 0 and not result['sql_access'] and not result['state_persistent']
assert all(result[k] for k in ['memberships_empty', 'staged_loadouts_empty', 'authenticated_accounts_empty'])
assert all((runtime / (role + '-transferred')).read_bytes() == b'done' for role in ['a', 'b'])
assert not any((runtime / (role + '-handoff.json')).exists() for role in ['a', 'b'])
assert (case / 'test.stderr.txt').read_text('utf-8-sig').count('native login response deadline exceeded') == 2

captures = list((runtime / 'captures').glob('*/frames.jsonl'))
assert len(captures) == 1
frames = [json.loads(x) for x in captures[0].read_text('utf-8-sig').splitlines()]
requests = []
for i, frame in enumerate(frames):
    if frame['direction'] == 'client_to_server' and frame['opcode'] == '0xCA' and bytes.fromhex(frame['plain']) == b'\xca\x00':
        requests.append(i)
assert len(requests) == 2
wire = []
for i in requests:
    request, reply = frames[i:i+2]
    assert reply['direction'] == 'server_to_client'
    assert bytes.fromhex(reply['plain']) == b'\xc9' + bytes(12)
    assert 'rental-item-set accepted' in reply['note']
    wire.append(dict(request_row=i+1, reply_row=i+2, request=request, reply=reply))
assert not any(f['direction'] == 'server_to_client' and bytes.fromhex(f['plain'])[:1] == b'\xca' for f in frames[requests[0]+1:])
trace = [json.loads(x) for x in (runtime / 'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
assert sha(runtime / 'server-trace.jsonl') == sha(case / 'server-trace.jsonl')
sessions = {}
for row in trace:
    sessions.setdefault(row['session_identity'], []).append(row)
assert len(sessions) == 4  # LOGIN sockets do not appear in this GAME-session trace.
summary = []
for sid, rows in sessions.items():
    last = rows[-1]
    summary.append(dict(session_identity=sid, account=last['test_account'], character=last['character'],
                        phases=list(dict.fromkeys(x['phase'] for x in rows)), final_phase=last['phase'],
                        final_channel=last['channel'], rental_items=len(last['rental_inventory']),
                        native_items=len(last['native_inventory']), final_rental_selection=last['rental_set']))
assert sorted(x['final_phase'] for x in summary) == ['BATTLE', 'BATTLE', 'LOBBY', 'LOBBY']
assert all(x['final_channel'] == -1 for x in summary)
assert sorted(x['native_items'] for x in summary if x['final_phase'] == 'LOBBY') == [28, 29]

identities = [r['runner_identity']] + r['process_history'] + [dict(pid=x['pid'], creation_utc=x['start_utc']) for x in c['owned_processes']]
assert len(identities) == 4
pid_filter = ' OR '.join('ProcessId=' + str(x['pid']) for x in identities)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '" + pid_filter + "'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode == 0 and not observed.stderr
live = json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for row in live:
    for ident in identities:
        if row['ProcessId'] == ident['pid']:
            assert abs((datetime.fromisoformat(row['created']) - datetime.fromisoformat(ident['creation_utc'])).total_seconds()) > .001

out = run / 'independent-boundary'
out.mkdir()
(out / 'reader.py').write_bytes(Path(__file__).read_bytes())
(out / 'process-CIM.json').write_bytes(observed.stdout)
files = {str(p.relative_to(ROOT)).replace('\\', '/'): p for p in list(case.glob('*')) if p.is_file()}
for p in [run / 'receipt.json', run / 'runner.py', run / 'source-inputs.json', *captures,
          runtime / 'a-transferred', runtime / 'b-transferred',
          ROOT / 'LiTo381-Modern/tests/native_teleport_handoff.rs',
          ROOT / 'LiTo381-Modern/src/modern_lito/native_login.rs',
          ROOT / 'LiTo381-Modern/src/modern_lito/native_lobby.rs']:
    files[p.relative_to(ROOT).as_posix()] = p
raw = {n: sha(p) for n, p in files.items()}
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for n, p in files.items():
        z.write(p, n)
with zipfile.ZipFile(out / 'raw.zip') as z:
    for n, h in raw.items():
        with z.open(n) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == h
record = dict(status='independently_preserved_current_same_endpoint_channel_contract_failure',
    source_manifest_sha256=r['source_manifest_sha256'], reference_jar_sha256=r['reference_jar_sha256'],
    profile=r['profile'], run=str(run), case=str(case), runtime=str(runtime),
    original_session_id=a.session, original_observation_wrapper_exit_code=0, terminal_tool_chunk=a.terminal_chunk,
    actual_PowerShell_exit_code=1, actual_Rust_exit_code=101, actual_server_exit_code=0,
    actual_client_sessions=6, GAME_trace_sessions=summary, trace_rows=len(trace), capture_rows=len(frames),
    wire=wire, transferred_markers=2, final_handoff_reports=0,
    marker_scope='Both roles passed fresh GAME epoch, selected-character and nonempty inventory ownership, same peer, retired-frame and stale selector/NPC rejection assertions before channel entry.',
    failure='Both C_CA 00 requests receive a valid 13-byte C9 success envelope labelled rental-item-set; no subsequent S_CA population arrives. ChannelPending advances to AwaitingChannelInfo and the existing response deadline expires.',
    ambiguity='The C9 envelope alone cannot distinguish rental selection from channel success. No channel capacity or room state is fabricated.',
    cleanup=result, recorded_owned_identities=identities, recorded_owned_all_terminal=True, PID_reuse_candidates=live,
    raw_files=raw, raw_archive_sha256=sha(out / 'raw.zip'),
    normal_source_modified=False, launcher_artifact_modified=False, launcher_replacement_proposal_withdrawn=True,
    gameplay_qualification_granted=False, same_endpoint_verified=False, GUI_verified=False,
    physical_audio_verified=False, full_G06_verified=False, whole_goal_complete=False)
save(out / 'boundary.json', record)
print(json.dumps(dict(boundary=str(out / 'boundary.json'), sessions=6, owned_terminal=4, gameplay_qualified=False), ensure_ascii=False))
