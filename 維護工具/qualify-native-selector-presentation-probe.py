"""Read actual candidate Presentation delivery results without claiming a GUI."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def member(z,n):
 with z.open(n) as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def verify_zip(path,digest,pins):
 assert sha(path)==digest
 with zipfile.ZipFile(path) as z:
  assert len(z.namelist())==len(pins) and set(z.namelist())==set(pins)
  for n,h in pins.items():assert member(z,n)==h,n
def function(text,name):
 start=text.index('fn '+name+'(');return text[start:text.index('\n}\n',start)+3]
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);ap.add_argument('--tool-chunk-id',required=True);a=ap.parse_args();assert a.session_id>0 and a.terminal_exit_code==0
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-nine-presentation-probe')
r=read(run/'receipt.json');rb=read(run/'readback.json');assert r['status']==rb['status']=='executed_coherent_candidate_Presentation_lifecycle_probe'
assert sha(run/'probe.rs')==r['probe_sha256']
verify_zip(run/'raw.zip',rb['raw_archive_sha256'],rb['raw_files'])
assert r['owned_process'] is None and len(r['process_history'])==2 and [p['exit_code'] for p in r['process_history']]==[0,0]
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1025
for n,h in inputs.items():assert sha(ROOT/n)==h,n
candidate=Path(r['candidate']).resolve(strict=True);assert candidate.is_relative_to(ROOT/'維護工具/candidates');m=read(candidate/'candidate.json')
assert not m['applied'] and m['after']==r['candidate_after'] and len(m['after'])==19
for n,h in m['after'].items():assert sha(candidate/'sources'/Path(n).name)==h
for v in r['libraries'].values():assert sha(v['path'])==v['sha256']
for v in r['baseline_gates'].values():assert sha(v['path'])==v['sha256'] and read(v['path'])['source_manifest_sha256']==r['source_manifest_sha256']
bank=Path(r['baseline_gates']['bank']['path']);assert read(bank)['exact_blank_source_negative_controls']==40 and read(bank)['actual_finite_producers_exercised']==9
assert read(bank.parent.parent/'receipt.json')['candidate_after']==m['after']
product=Path(r['product']).resolve(strict=True);assert product.is_relative_to(ROOT/'LiTo381-Modern/target/current') and sha(product)==r['product_sha256']
with product.open('rb') as f:
 h=f.read(64);assert h[:2]==b'MZ';f.seek(int.from_bytes(h[60:64],'little'));h=f.read(26);assert h[:4]==b'PE\0\0' and h[4:6]==b'\x64\x86' and h[24:26]==b'\x0b\x02'
o=read(run/'probe-observation.json');assert sha(run/'probe-observation.json')==r['observation_sha256']
expected={'environment_deferred_exactly_once','root_ABA_late_context_retired','old_Frame_payload_and_input_fenced','new_S37_and_new_generation_still_emit','same_world_resume_does_not_replay','pending_snapshot_history_retained','bounded_history_guard_controls'}
assert set(o['checks'])==expected and all(o['checks'].values())
assert o['full_Model_Presentation_executed'] and not o['native_window_refresh_executed'] and not o['physical_audio_opened']
assert o['actual_new_sessions']==o['actual_new_casts']==0 and not o['visual_phase_continuity_verified']
e=o['environment_case'];assert e['passed'] and e['deferred_groups']==1 and e['counter_calls']==[803] and e['worker_quiescent'] and e['world_unchanged_by_renderer']
e=o['root_aba_case'];assert e['passed'] and e['retired_successes']+e['retired_errors']>0 and e['counter_calls']==[803] and e['worker_quiescent'] and e['world_unchanged_by_renderer']
e=o['resume_case'];assert e['same_world_resume_player_forwardings']==0 and e['same_world_resume_native_cue_ids']==[] and e['all_counter_calls']==[803,133,803]
assert e['old_Frame_cancel_fenced'] and e['old_generation_input_retired'] and e['old_generation_Frame_payload_fenced'] and e['worker_quiescent']
assert e['genuine_new_S37_sound']==133 and e['new_generation_sound']==803
pending=o['pending_ring_case'];assert pending['passed'] and pending['controlled_extra_S37_inputs']==129 and pending['current_ring_size']==128 and pending['pending_duplicate_suppressed']==1 and pending['counter_calls']==[803,133] and pending['retained_keys_after_new_event']<=2 and pending['old_pending_input_retired'] and pending['worker_quiescent']
guards=o['history_guard_case'];assert guards['hard_bound']==4096 and guards['passed'] and all(guards[k] for k in ['overflow_rejected_transactionally','missing_provenance_rejected','fresh_payload_duplicate_consumed','new_sequence_allowed','pending_union_retained','expired_sequences_pruned','world_connection_map_reset','discarded_event_cannot_replay'])
fixture=read(run/'current-world-packets.json');assert sha(run/'current-world-packets.json')==o['fixture_sha256']
assert fixture['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(fixture['source_receipt'])==fixture['source_receipt_sha256'] and sha(fixture['source_capture'])==fixture['source_capture_sha256']
assert sha(Path(fixture['source_receipt']).parent/'visibility-comparison.json')==fixture['source_case_comparison_sha256']
wire=Path(fixture['source_capture']).read_text(encoding='utf-8-sig').splitlines();assert len(fixture['packets'])==18 and len(fixture['expected']['actors'])==13
for packet in fixture['packets']:
 row=json.loads(wire[packet['capture_line']-1]);assert row['direction']=='server_to_client' and re.fullmatch('[0-9a-fA-F ]*',row['plain']) and bytes.fromhex(row['plain']).hex()==packet['hex'] and packet['capture_line']<fixture['source_cast_boundary_line']
# Preserve the original failing test predicates: the repair changes the candidate.
failed=ROOT/'驗證紀錄/current/native-selector-nine-presentation-probe/20261017-094626-956043'
failure=read(failed/'failure-independent-readback.json');assert failure['observed_terminal_exit_code']==1 and failure['resumed_player_forwardings']==3
assert sha(failed/'raw.zip')==failure['raw_archive_sha256']
assert read(failed/'current-world-packets.json')==fixture
# The retained original is CRLF; this probe's pinned copy is LF. Both parser
# inputs are hashed separately and every packet is checked against capture.
for name in ['environment_case','root_aba_case','resume_case']:
 assert function((failed/'probe.rs').read_text(encoding='utf-8-sig'),name)==function((run/'probe.rs').read_text(encoding='utf-8-sig'),name),name
old=read(failed/'receipt.json')['candidate_after'];changed={Path(n).name for n,h in m['after'].items() if old[n]!=h};assert changed=={'native_audio_delivery.rs','native_skill_effects.rs','native_world_view.rs'}
visual=read(run/'visual-observations.json');assert len(visual['images'])==5
snapshots=[o['environment_case']['snapshot'],o['root_aba_case']['snapshot'],o['resume_case']['before'],o['resume_case']['resumed'],o['pending_ring_case']['snapshot']]
assert {Path(x['png']).resolve() for x in snapshots}=={Path(x['path']).resolve() for x in visual['images']}
for row in visual['images']:
 path=Path(row['path']).resolve(strict=True);assert path.is_relative_to(run) and row['viewed'] and sha(path)==row['sha256']
 with Image.open(path) as im:
  assert im.size==(800,600) and im.mode=='RGBA';assert hashlib.sha256(im.tobytes()).hexdigest().upper()==row['rgba_sha256']
ids=[(p['pid'],p['creation_utc']) for p in r['process_history']]
command="$ids=@("+','.join(str(pid) for pid,_ in ids)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
 for pid,created in ids:
  if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
out=run/'qualified';assert not out.exists();out.mkdir();(out/'CIM.json').write_bytes(p.stdout);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files={'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json','receipt.json':run/'receipt.json','probe-observation.json':run/'probe-observation.json','visual-observations.json':run/'visual-observations.json','original-failure-readback.json':failed/'failure-independent-readback.json','packet-fixture.json':run/'current-world-packets.json'}
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
verify_zip(out/'raw.zip',sha(out/'raw.zip'),hashes)
q=dict(status='independently_qualified_actual_candidate_Presentation_finite_audio_lifecycle',original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,source_manifest_sha256=r['source_manifest_sha256'],candidate_files=19,candidate=str(candidate),candidate_metadata_sha256=sha(candidate/'candidate.json'),actual_product_sha256=r['product_sha256'],runner_raw_archive_sha256=rb['raw_archive_sha256'],bank_qualification=str(bank),bank_qualification_sha256=sha(bank),original_failure_raw_archive_sha256=failure['raw_archive_sha256'],original_three_case_assertions_unchanged=True,packet_fixture_semantically_identical_to_original_failure=True,original_failure_fixture_sha256=sha(failed/'current-world-packets.json'),actual_Model_Presentation_executed=True,old_same_world_audio_forwardings=3,new_same_world_audio_forwardings=0,pending_snapshot_rotated_inputs=129,history_key_bound=4096,history_overflow_and_missing_provenance_rejected=True,genuine_new_event_and_world_generation_preserved=True,worker_cases_quiescent=True,diagnostic_PNGs_viewed=5,packet_fixture_sha256=o['fixture_sha256'],captured_fixture_jar_sha256=fixture['jar_sha256'],controlled_renderer_packets_not_actual_casts=True,actual_new_sessions=0,actual_new_casts=0,recorded_direct_children=2,recorded_direct_children_all_terminal=True,native_window_refresh_executed=False,physical_audio_verified=False,GUI_verified=False,visual_phase_continuity_verified=False,all_candidate_gameplay_verified=False,product_source_changed=False,applied=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:q[k] for k in ['status','old_same_world_audio_forwardings','new_same_world_audio_forwardings','pending_snapshot_rotated_inputs','history_key_bound','native_window_refresh_executed']},ensure_ascii=False))
