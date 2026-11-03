"""Independently bind the diagnostic to original S32 and indexed Tile bytes."""
from pathlib import Path
from datetime import datetime, timezone
from ctypes import wintypes
import argparse, collections, ctypes, hashlib, json, msvcrt, os, struct, zipfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path(r'D:\lineage開發\0-客戶端')
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def locked_read(path):
    assert path.is_absolute()
    for p in [path, *path.parents]:
        assert not os.lstat(p).st_file_attributes & 0x400, p
    fn = ctypes.WinDLL('kernel32', use_last_error=True).CreateFileW
    fn.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, wintypes.LPVOID,
        wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    fn.restype = wintypes.HANDLE
    handle = fn(str(path), 0x80000000, 1, None, 3, 0x10000000, None)
    assert handle != wintypes.HANDLE(-1).value, ctypes.get_last_error()
    return os.fdopen(msvcrt.open_osfhandle(int(handle), os.O_RDONLY | os.O_BINARY), 'rb')

def identity(f):
    s = os.fstat(f.fileno())
    return dict(device=s.st_dev, file_id=s.st_ino, size=s.st_size, mtime_ns=s.st_mtime_ns)

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', type=int, required=True)
ap.add_argument('--terminal-exit-code', type=int, required=True)
a = ap.parse_args()
assert a.terminal_exit_code == 0
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-map303-assets')
r = read(run / 'receipt.json')
assert r['status'] == 'decoded_current_library_original_classic_map303_assets'
assert r['owned_process'] is None and all(p['exit_code'] == 0 for p in r['processes'])
q = read(Path(r['original_qualification']))
assert sha(r['original_qualification']) == r['original_qualification_sha256']
assert r['source_manifest_sha256'] == q['source_manifest_sha256']
pins = read(run / 'compiled-inputs.json')
assert len(pins) == 1025 and all(sha(ROOT / name) == digest for name, digest in pins.items())
archived = read(run / 'readback.json')
assert sha(run / 'raw.zip') == archived['archive_sha256']
with zipfile.ZipFile(run / 'raw.zip') as z:
    assert len(z.namelist()) == len(archived['files']) and set(z.namelist()) == set(archived['files'])
    for name, digest in archived['files'].items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
assert sha(run / 'probe.rs') == r['diagnostic_source_sha256']
assert sha(Path(r['runtime']) / 'map303-assets.exe') == r['diagnostic_sha256']
report = read(run / 'map303.json')
assert sha(run / 'map303.json') == r['report_sha256']
assert report['sector_count'] == 30 and report['namespace'] == 'BASE map/303 + Tile'
seen = set()
for s in report['sectors']:
    key = '%04x%04x.s32' % (s['sector_x'], s['sector_y'])
    assert key not in seen
    seen.add(key)
    path = SOURCE / 'map/303' / key
    with locked_read(path) as f:
        before = identity(f)
        digest = hashlib.file_digest(f, 'sha256').hexdigest().upper()
        assert before == identity(f) and before['size'] == s['bytes']
    assert digest == s['sha256'] == s['decoder_input_sha256']
assert seen == {p.name for p in (SOURCE / 'map/303').glob('*.s32')}
checks = []
types = collections.Counter()
with locked_read(SOURCE / 'Tile.idx') as index, locked_read(SOURCE / 'Tile.pak') as pak:
    index_identity, pak_identity = identity(index), identity(pak)
    data = index.read()
    count, = struct.unpack_from('<I', data)
    assert len(data) == 4 + 28 * count and count <= 250000
    entries = {}
    for offset in range(4, len(data), 28):
        position, raw_name, length = struct.unpack_from('<I20sI', data, offset)
        name_bytes, _, padding = raw_name.partition(b'\0')
        assert not padding.strip(b'\0')
        name = name_bytes.decode('ascii').lower()
        assert name and name not in entries and position + length <= pak_identity['size']
        entries[name] = (position, length)
    for member in report['tile_members']:
        name = member['member']
        position, length = entries[name]
        assert length == member['encoded_bytes']
        pak.seek(position)
        raw = pak.read(length)
        assert len(raw) == length and hashlib.sha256(raw).hexdigest().upper() == member['encoded_sha256']
        slots, = struct.unpack_from('<I', raw)
        assert 0 < slots <= 65536
        body = 4 + 4 * (slots + 1)
        offsets = struct.unpack_from('<' + 'I' * (slots + 1), raw, 4)
        assert body + offsets[-1] == length
        selected = []
        for item in member['selected']:
            slot = item['slot']
            assert 0 <= slot < slots and offsets[slot] < offsets[-1]
            kind = raw[body + offsets[slot]]
            assert kind == item['type']
            types[kind] += 1
            selected.append(dict(slot=slot, type=kind, object_reference_instances=item['object_reference_instances']))
        checks.append(dict(member=name, offset=position, length=length,
            encoded_sha256=member['encoded_sha256'], selected=selected))
    assert index_identity == identity(index) and pak_identity == identity(pak)
assert dict(types) == {int(k): v for k, v in report['selected_type_counts'].items()}
result = dict(status='independently_read_back_current_library_original_map303_asset_probe',
    original_session_id=a.session_id, observed_terminal_exit_code=0, source_files=1024, compiled_input_files=1025,
    source_manifest_sha256=r['source_manifest_sha256'], diagnostic_sha256=r['diagnostic_sha256'],
    report_sha256=r['report_sha256'], original_sector_count=30, original_source_bytes=report['source_bytes'],
    original_Tile_idx_sha256=hashlib.sha256(data).hexdigest().upper(), Tile_idx_identity=index_identity,
    Tile_pak_identity=pak_identity, complete_Tile_pak_SHA_claimed=False, member_checks=checks,
    unique_Tile_members=len(checks), unique_selected_slots=sum(types.values()), selected_type_counts=dict(types),
    raw_object_references_requiring_original_RLE_provenance=report['raw_object_references_requiring_original_RLE_provenance'],
    selected_RGBA_hashes_are_current_decoder_outputs_not_original_pixel_oracle=True,
    original_S32_full_parser_implementation_independently_reimplemented=False,
    archive_sha256=archived['archive_sha256'], normal_scene_map303_enabled=False,
    background_scene_adapter_verified=False, actual_formal_sessions=0, GUI_verified=False,
    full_map_rendering_verified=False, whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
out = run / 'independent-readback.json'
assert not out.exists()
out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
(run / 'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:result[k] for k in ['status','unique_Tile_members','unique_selected_slots','raw_object_references_requiring_original_RLE_provenance']}, ensure_ascii=False))
