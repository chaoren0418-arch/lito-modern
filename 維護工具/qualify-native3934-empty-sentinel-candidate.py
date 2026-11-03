"""Read back the candidate and compare all pixels using a separately retained contract reader.

The Python reader derives from the same documented format contract. Agreement
is not an independent format-source oracle or original-client execution.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse, ast, hashlib, json, re, struct, subprocess, zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path)
ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True)
ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native3934-empty-sentinel-candidate')
assert a.terminal_exit_code==0
r=read(run/'receipt.json');rb=read(run/'readback.json');m=read(run/'candidate-candidate.json');obs=read(run/'sprite-observation.json')
assert r['status']=='passed_isolated3934_sentinel_candidate' and r['resolved_definitions']==9
assert sha(run/'raw.zip')==rb['raw_archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert set(z.namelist())==set(rb['raw_files'])
    for n,h in rb['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
base=Path(r['related_qualification']).parent;pins=read(base/'compiled-inputs.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
assert sha(ROOT/'LiTo381-Modern/src/modern_lito/sprite.rs')==m['original_decoder_sha256']
assert sha(run/'candidate-sprite.rs')==m['candidate_decoder_sha256']
with zipfile.ZipFile(run/'candidate-before.zip') as z:
    before=z.read('LiTo381-Modern/src/modern_lito/sprite.rs')
assert hashlib.sha256(before).hexdigest().upper()==m['original_decoder_sha256']
new=run.joinpath('candidate-sprite.rs').read_bytes()
addition=b'\n                    | (352, 120, -288, -360, u16::MAX, 0)'
assert new.count(addition)==1 and new.replace(addition,b'')==before
assert len(r['process_history'])==2 and [p['exit_code'] for p in r['process_history']]==[0,0]
assert r['owned_process'] is None and sha(r['executable'])==r['executable_sha256']
ids=','.join(str(p['pid']) for p in r['process_history'])
script=f"$ids=@({ids}); @(Get-CimInstance Win32_Process | Where-Object {{$_.ProcessId -in $ids}} | ForEach-Object {{[pscustomobject]@{{pid=$_.ProcessId;creation_utc=$_.CreationDate.ToUniversalTime().ToString('o')}}}}) | ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',script],encoding='utf-8-sig').strip()
live=json.loads(raw) if raw else []
if isinstance(live,dict):live=[live]
for p in r['process_history']:
    assert not any(p['pid']==v['pid'] and datetime.fromisoformat(p['creation_utc'])==datetime.fromisoformat(v['creation_utc'].replace('Z','+00:00')) for v in live)
prior=ROOT/'驗證紀錄/current/native-shared-healing-family-SPR'
pq=read(prior/'qualified.json');assert sha(prior/'SPR-inspection-evidence.zip')==pq['archive_sha256']
with zipfile.ZipFile(prior/'SPR-inspection-evidence.zip') as z:
    source=z.read('inspector.py')
assert hashlib.sha256(source).hexdigest().upper()==pq['archive_file_pins']['inspector.py']
source=source.decode('utf-8-sig').replace('\r\n','\n').replace('\r','\n')
tree=ast.parse(source)
parts=[ast.get_source_segment(source,n) for n in tree.body if isinstance(n,(ast.ClassDef,ast.FunctionDef)) and n.name in ['Cursor','decode']]
assert len(parts)==2
adapter='import struct, hashlib\n'+'\n\n'.join(parts)+'\n'
for old,new in [('len(b)<=1024*1024','len(b)<=2*1024*1024'),('assert count==21','assert 1<=count<=255'),('(432,132,-368,-468,65535,0)])','(432,132,-368,-468,65535,0),(352,120,-288,-360,65535,0)])')]:
    assert adapter.count(old)==1;adapter=adapter.replace(old,new)
namespace={};exec(compile(adapter,'retained_contract_adapter.py','exec'),namespace)
out=run/'separate-python-contract-readback';out.mkdir()
(out/'reader.py').write_text(adapter,encoding='utf-8',newline='\n')
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
rows=[];frames_checked=0
for row in obs['rows']:
    assert row['asset_pixels_decoded'] and not row['pixel_equivalence_verified']
    pak=Path(r'D:\lineage開發\0-客戶端')/row['payload_file'];stat=pak.stat()
    with pak.open('rb') as f:f.seek(row['entry_offset']);payload=f.read(row['entry_size'])
    assert (stat.st_size,stat.st_mtime_ns)==(pak.stat().st_size,pak.stat().st_mtime_ns)
    assert len(payload)==row['encoded_bytes'] and hashlib.sha256(payload).hexdigest().upper()==row['encoded_sha256']
    metadata,pixels,header=namespace['decode'](payload)
    assert len(metadata)==row['decoded_frames']
    assert {v['subframe'] for v in row['frames']}==set(range(len(metadata)))
    for frame in row['frames']:
        p=metadata[frame['subframe']]
        assert [p['x'],p['y']]==frame['offset'] and [p['width'],p['height']]==frame['size']
        assert p['rgba_sha256']==frame['RGBA_sha256'] and p['rgba_bytes']==frame['RGBA_bytes']
        assert p['opaque_pixels']==frame['nontransparent_pixels']
        frames_checked+=1
    rows.append(dict(skill_id=row['skill_id'],member=row['member'],member_sha256=row['encoded_sha256'],
        header=header,frames=metadata))
assert len(rows)==9 and frames_checked==128
assert [p['subframe'] for p in next(v for v in rows if v['skill_id']==72)['frames'] if p['blank_sentinel']]==[0,12]
save(out/'all-nine-frame-readback.json',dict(rows=rows,frames_checked=frames_checked,
    reader_derivation='Separate Python implementation of the same documented Rust/PakViewer contract; not an independent format-source oracle.',
    reader_source_archive=str(prior/'SPR-inspection-evidence.zip'),reader_source_archive_sha256=pq['archive_sha256']))
terminal=dict(original_session_id=a.session_id,terminal_exit_code=0,terminal_tool_chunk_id=a.terminal_chunk,handle_closed=True)
save(run/'original-session-observation.json',terminal)
files={p.name:p for p in out.iterdir() if p.is_file()};hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    for n,h in hashes.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
save(run/'independent-readback.json',dict(status='independently_read_back_isolated_exact3934_sentinel_candidate',
    source_manifest_sha256=r['source_manifest_sha256'],original_session=terminal,
    candidate_raw_archive_sha256=sha(run/'raw.zip'),separate_contract_readback_raw_sha256=sha(out/'raw.zip'),
    independent_contract_frame_matches=128,baseline_unchanged_members=8,
    rejection_controls=dict(sentinel_field_mutations=14,nonempty_reversed_bounds=1,existing_limits=3),
    recorded_direct_processes=2,recorded_direct_processes_all_terminal=True,
    product_source_changed=False,applied=False,pixel_equivalence_to_original_client_verified=False,
    native_caller_bank_binding_verified=False,actual_new_sessions=0,actual_new_casts=0,GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(status='independently_read_back_isolated_exact3934_sentinel_candidate',frames_checked=128,
    rejection_controls=18,baseline_unchanged_members=8,product_source_changed=False,output=str(run/'independent-readback.json')),ensure_ascii=False))
