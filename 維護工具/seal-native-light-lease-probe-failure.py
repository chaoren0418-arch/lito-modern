"""Retain an unsuccessful rental-town probe without changing its runtime receipt."""
import hashlib
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CASE = ROOT / '驗證紀錄/current/native-light-lease-gui/20261015-221753-691'

def sha(data):
    return hashlib.sha256(data).hexdigest().upper()

def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

receipt = read(CASE / 'receipt.json')
result = read(CASE / 'result.json')
assert receipt['status'] == 'failed'
assert receipt['error'] == 'Timeout: original3600s light activation in rental town'
assert result['status'] == 'sessions_closed' and result['actual_client_sessions'] == 4
assert result['memberships_empty'] and result['classic_world_members_after_disconnect'] == 0
log = (CASE / 'server.log').read_text(encoding='utf-8-sig')
rejection = 'rental-town rejected unsupported gameplay C opcode=0x14 length=3 plain=14 00 01'
assert rejection in log
normal = read(ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json')
inputs = read(CASE / 'source-inputs.json')
assert len(inputs) == 970
pins = {}
archive = CASE / 'failed-probe-source-image-raw.zip'
assert not archive.exists(), 'Retained archive already exists; do not overwrite'
raw_files = [p for p in CASE.rglob('*') if p.is_file()]
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as z:
    for relative, expected in inputs.items():
        data = (ROOT / relative).read_bytes()
        assert sha(data) == expected, relative
        name = 'compiled-source/' + relative
        z.writestr(name, data)
        pins[name] = expected
    image = Path(normal['profiles']['Release']['path'])
    data = image.read_bytes()
    assert sha(data) == receipt['exe_sha256'] == normal['profiles']['Release']['sha256']
    z.writestr('Release/LiToModern.exe', data)
    pins['Release/LiToModern.exe'] = sha(data)
    for path in raw_files:
        name = 'raw/' + path.relative_to(CASE).as_posix()
        data = path.read_bytes()
        z.writestr(name, data)
        pins[name] = sha(data)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(pins)
    for name, expected in pins.items():
        assert sha(z.read(name)) == expected, name
evidence = {
    'status': 'failed_probe_preserved_not_gameplay_qualified',
    'runtime_receipt_unchanged_sha256': sha((CASE / 'receipt.json').read_bytes()),
    'error': receipt['error'],
    'server_observation': rejection,
    'actual_client_sessions': 4, 'game_sessions': 2,
    'memberships_empty_after_disconnect': True,
    'wire_capture_files': len(list((CASE / 'raw-capture').rglob('frames.jsonl'))),
    'independent_wire_replay_verified': False,
    'light_activation_verified': False,
    'natural3600s_expiry_run': False,
    'source_files_unchanged': 970,
    'release_sha256': receipt['exe_sha256'],
    'source_manifest_sha256': receipt['source_manifest_sha256'],
    'archive': {'path': str(archive), 'sha256': sha(archive.read_bytes()),
                'files': len(pins), 'file_pins': pins, 'readback_verified': True},
    'project_scope': '重建版與登入器為獨立專案與產物；來源唯讀，登入器替換提案已撤回。',
    'launcher_or_server_replacement_requested': False,
    'whole_goal_complete': False,
}
write(CASE / 'failure-preserved.json', evidence)
tracking = {
    'evidence': '驗證紀錄/current/native-light-lease-gui/20261015-221753-691/failure-preserved.json',
    'status': evidence['status'], 'actual_sessions': 4,
    'server_rejected_rental_town_C14': True,
    'natural3600s_expiry_run': False,
    'pending_process': None,
    'client_source_files_unchanged': 970,
    'launcher_replacement_dependency': False,
}
for relative in ('工作佇列.json', '監製進度.json',
                 '驗證紀錄/current/native-teleport-wire/progress.json',
                 '驗證紀錄/current/native-terrain-residence/continuation.json'):
    path = ROOT / relative
    document = read(path)
    document['native_light_lease_probe'] = tracking
    write(path, document)
history = ROOT / 'docs/HISTORY.md'
with history.open('a', encoding='utf-8') as stream:
    stream.write('\n\n2026-10-16：租裝村map500的Release日光術短探針已結束失敗；原C02明確記錄rental-town拒絕三byte C14（14 00 01），未啟用日光期限，未執行3600秒自然到期。4實際sessions／2 GAME及退出成員回收由原result確認，已保存1份frames.jsonl，尚未獨立重播核對。970份來源、當次Release EXE、對應QA與原log／PNG均封存逐檔讀回；正常912D／FB94產品來源未改。登入器與重建版獨立、登入器唯讀及替換提案撤回的邊界持續適用，這個場景限制不轉成外部部署依賴。見[失敗保留](../驗證紀錄/current/native-light-lease-gui/20261015-221753-691/failure-preserved.json)。\n')
print(json.dumps({k: evidence[k] for k in ('status', 'actual_client_sessions', 'wire_capture_files', 'natural3600s_expiry_run')}, ensure_ascii=False))
print('Archive readback verified:', len(pins), 'files')
