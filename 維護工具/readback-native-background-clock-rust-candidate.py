"""Check candidate-only changes, native input pins and the bounded queue test scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def function(text,name):
    start=text.index('fn '+name+'(');body=text.index('{',start);depth=0
    for i in range(body,len(text)):
        if text[i]=='{':depth+=1
        elif text[i]=='}':
            depth-=1
            if depth==0:return text[start:i+1]
    raise AssertionError('Function end missing')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path)
origin=ap.add_mutually_exclusive_group(required=True);origin.add_argument('--session-id',type=int);origin.add_argument('--exec-chunk-id')
ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-background-clock-rust-candidate') and a.terminal_exit_code==0
assert a.session_id is None or a.session_id>0
r=read(run/'receipt.json');assert r['status']=='unapplied_background_shared_clock_candidate_passed'
assert r['owned_process'] is None and len(r['processes'])==2 and all(p['exit_code']==0 for p in r['processes'])
retained=read(run/'archive-readback.json');assert sha(run/'raw.zip')==retained['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(retained['files']) and set(z.namelist())==set(retained['files'])
    for name,h in retained['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    exe=z.read('Rust-x64-probe.exe');at=int.from_bytes(exe[60:64],'little')
    assert exe[:2]==b'MZ' and exe[at+4:at+6]==b'\x64\x86' and exe[at+24:at+26]==b'\x0b\x02'
    assert hashlib.sha256(exe).hexdigest().upper()==r['diagnostic_sha256']
native=Path(r['native_qualification']);n=read(native)
assert sha(native)==r['native_qualification_sha256']==sha(run/'native-qualification.json')
assert n['status']=='independently_qualified_original_background_offset_callback' and n['cases']==57
assert sha(run/'native-vectors.csv')==r['native_vectors_sha256']==n['raw_files']['vectors.csv']
candidate=ROOT/'維護工具/candidates/native-background-clock';meta=read(run/'metadata.json')
for name,h in r['candidate_files'].items():assert sha(run/name)==sha(candidate/name)==h
assert sha(meta['base_source'])==sha(run/'base-native_water_clock.rs')==meta['base_sha256']
base=(run/'base-native_water_clock.rs').read_text(encoding='utf-8-sig')
proposed=(run/'native_water_clock.rs').read_text(encoding='utf-8-sig')
for name in ['with_values','phase','counter','with_green_counter','green_counter','events','schedule','cancel','water_callback','green_callback','ensure_map_aux','map_filter','map_loaded','retire_scene','water_packet']:
    assert function(base,name)==function(proposed,name),name
assert function(proposed,'dispatch').replace('                Callback::Kept(6)=>next.background_callback(&mut clock)?,\n','')==function(base,'dispatch')
tail='/// Incremental scroll is independent of the water enable flag.'
assert proposed[proposed.index(tail):]==base[base.index(tail):]
assert not meta['automatic_scene_binding_added'] and not meta['Owned_loaded_and_retirement_methods_changed']
probe=ROOT/'維護工具/oracle/NativeBackgroundClockRustProbe.rs';assert sha(probe)==sha(run/'probe.rs')==r['probe_sha256']
for info in r['libraries'].values():assert sha(info['path'])==info['sha256']
result=read(run/'candidate-results.json');assert sha(run/'candidate-results.json')==r['report_sha256']
assert result['native_callback_cases']==57 and len(result['synthetic_queue_scenarios'])==7
names=[x['name'] for x in result['synthetic_queue_scenarios']]
assert names==['independent_water_and_green_values','one_dispatch_cutoff_fresh_reschedule_no_catchup',
    'cancel_owner0_then_one_immediate_rearm','explicit_stop_preserves_values_and_other_owner',
    'generic_source_filters_keep_slot6_without_advancing_values','modern_bounded_queue_rejects_before_publishing_partial_state','u32_scheduled_deadline_wrap']
assert (run/'vectors-stdout.log').read_text(encoding='utf-8-sig').strip()=='background queue candidate: 57 native callback vectors and 7 synthetic queue scenarios passed'
assert not (run/'vectors-stderr.log').read_bytes().strip()
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,h in pins.items():assert sha(ROOT/name)==h
q=dict(status='independently_read_back_unapplied_shared_background_clock_candidate',original_session_id=a.session_id,
    original_exec_tool_chunk_id=a.exec_chunk_id,observed_terminal_exit_code=0,source_manifest_sha256=r['source_manifest_sha256'],
    candidate_files=r['candidate_files'],native_qualification_sha256=sha(native),native_callback_cases=57,
    synthetic_queue_scenarios=names,native_callback_delay_ms=64,native_scheduler_body_executed=False,
    existing_water_green_and_map_methods_preserved=True,existing_Owned_and_History_methods_preserved=True,
    explicit_stop_and_rearm_API_only=True,automatic_scene_binding_added=False,real_clock_or_GUI_timing_verified=False,
    candidate_applied_to_product=False,product_source_changed=False,actual_formal_sessions=0,whole_goal_complete=False,
    raw_archive_sha256=retained['archive_sha256'],raw_files=retained['files'],observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'independent-readback.json').exists()
(run/'independent-readback.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(status=q['status'],native_callback_cases=57,synthetic_queue_scenarios=len(names),automatic_scene_binding_added=False),ensure_ascii=False))
