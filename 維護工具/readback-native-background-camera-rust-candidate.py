"""Bind the Rust candidate, compiled probe and complete qualified native vector domain."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,csv,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int);ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-background-camera-rust-candidate') and a.session_id>0 and a.terminal_exit_code==0
r=read(run/'receipt.json');assert r['status']=='isolated_Rust_camera_increment_candidate_passed'
assert r['owned_process'] is None and len(r['processes'])==2 and all(p['exit_code']==0 for p in r['processes'])
retained=read(run/'archive-readback.json');assert sha(run/'raw.zip')==retained['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(retained['files']) and set(z.namelist())==set(retained['files'])
    for name,h in retained['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    exe=z.read('Rust-x64-candidate-probe.exe');at=int.from_bytes(exe[60:64],'little')
    assert exe[:2]==b'MZ' and exe[at+4:at+6]==b'\x64\x86' and exe[at+24:at+26]==b'\x0b\x02'
    assert hashlib.sha256(exe).hexdigest().upper()==r['diagnostic_sha256']
native=Path(r['native_qualification']);q=read(native)
assert sha(native)==r['native_qualification_sha256']==sha(run/'native-qualification.json')
assert q['status']=='independently_qualified_original_camera_increment_selector_fragment' and q['cases']==192
assert q['original_exec_tool_chunk_id']=='6cefd6' and q['original_session_id'] is None
assert sha(run/'native-vectors.csv')==r['native_vectors_sha256']==q['raw_files']['vectors.csv']
assert sha(run/'candidate.rs')==sha(r['candidate_path'])==r['candidate_sha256']
probe=ROOT/'維護工具/oracle/NativeBackgroundCameraRustProbe.rs'
assert sha(run/'probe.rs')==sha(probe)==r['probe_sha256']
assert sha(r['compiler_path'])==r['compiler_sha256']
rows=list(csv.DictReader((run/'native-vectors.csv').open(encoding='ascii',newline='')))
domain={(int(v['count']),int(v['heading']),int(v['actor13']),int(v['before_x']),int(v['before_y'])) for v in rows}
assert len(rows)==len(domain)==192
assert domain=={(n,h,p,x,y) for n in [4,6,8,12] for h in range(8) for p in [1,2] for x,y in [(48,48),(0,0),(-4096,4096)]}
result=read(run/'candidate-results.json');assert sha(run/'candidate-results.json')==r['report_sha256']
assert result['cases']==192 and result['checked_host_domain_rejections']==10
assert (run/'native-vectors-stdout.log').read_text(encoding='utf-8-sig').strip()=='Rust camera candidate: 192 native vector comparisons, 10 host domain rejections'
assert not (run/'native-vectors-stderr.log').read_bytes().strip()
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,h in pins.items():assert sha(ROOT/name)==h
qualification=dict(status='independently_read_back_isolated_Rust_camera_increment_candidate',
    original_session_id=a.session_id,observed_terminal_exit_code=0,source_manifest_sha256=r['source_manifest_sha256'],
    candidate_sha256=r['candidate_sha256'],actual_probe_sha256=r['diagnostic_sha256'],native_qualification_sha256=sha(native),
    native_vectors_sha256=r['native_vectors_sha256'],native_vector_cases=192,checked_host_domain_rejections=10,
    host_rejections_are_modern_bounds_not_original_overflow_or_fallback_semantics=True,
    native_instruction_bytes_linked_into_Rust_probe=0,raw_archive_sha256=retained['archive_sha256'],raw_files=retained['files'],
    candidate_applied_to_product=False,product_source_changed=False,clock_or_owner_binding_verified=False,
    initial_or_final_callback_paths_verified=False,whole_callback_executed=False,modern_Pose_binding_verified=False,
    normal_scene_map303_enabled=False,actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'independent-readback.json').exists()
(run/'independent-readback.json').write_text(json.dumps(qualification,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:qualification[k] for k in ['status','native_vector_cases','checked_host_domain_rejections','candidate_applied_to_product']},ensure_ascii=False))
