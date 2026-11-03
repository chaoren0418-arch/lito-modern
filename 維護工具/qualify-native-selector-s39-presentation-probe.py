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
assert not m['applied'] and m['after']==r['candidate_after'] and len(m['after'])==20
for n,h in m['after'].items():assert sha(candidate/'sources'/Path(n).name)==h
for v in r['libraries'].values():assert sha(v['path'])==v['sha256']
for v in r['baseline_gates'].values():assert sha(v['path'])==v['sha256'] and read(v['path'])['source_manifest_sha256']==r['source_manifest_sha256']
bank=Path(r['baseline_gates']['bank']['path']);assert read(bank)['exact_blank_source_negative_controls']==40 and read(bank)['actual_finite_producers_exercised']==9
assert read(bank.parent.parent/'receipt.json')['candidate_after']==m['after']
product=Path(r['product']).resolve(strict=True);assert product.is_relative_to(ROOT/'LiTo381-Modern/target/current') and sha(product)==r['product_sha256']
with product.open('rb') as f:
 h=f.read(64);assert h[:2]==b'MZ';f.seek(int.from_bytes(h[60:64],'little'));h=f.read(26);assert h[:4]==b'PE\0\0' and h[4:6]==b'\x64\x86' and h[24:26]==b'\x0b\x02'
o=read(run/'probe-observation.json');assert sha(run/'probe-observation.json')==r['observation_sha256']

expected={'nine_finished_no_restart_or_audio_replay','nine_active_callbacks_continue_after_source_ring_rotation','nine_new_world_generation_resets_phase','old_world_authority_unchanged','workers_quiescent','captured_S39_admitted_and_acknowledged','captured_S39_renders_correct_bank_origin_and_typed_audio','S39_callback_lifecycle_preserved'}
assert set(o['checks'])==expected and all(o['checks'].values())
assert o['status']=='executed_actual_candidate_all_nine_and_captured_S39_lifecycle'
assert o['graphics']==[2170,10701,2173,759,3936,3104,3934,2234,6527]
assert o['actual_new_sessions']==o['actual_new_casts']==0 and o['full_Model_Presentation_executed'] and not o['native_window_refresh_executed'] and not o['physical_audio_opened']
counts={2170:13,10701:15,2173:17,759:10,3936:13,3104:6,3934:13,2234:31,6527:10}
previous=ROOT/'驗證紀錄/current/native-selector-nine-presentation-probe/20261017-102503-761364/qualified/qualified.json'
assert sha(previous)=='B61107107B595144BCAF7A07B4CB9152FB61CAC448E19E551CC343E1982E610C'
previous_q=read(previous);assert previous_q['candidate_metadata_sha256']==m['parent_candidate_metadata_sha256'] and previous_q['new_same_world_audio_forwardings']==0 and previous_q['finite2170_callback_lifecycle_verified']
e=o['completed_case'];assert e['passed'] and e['worker_quiescent'] and e['world_authority_unchanged'] and e['old_sources_retained']==9
assert sorted(e['counter_calls'])==[133,801,803,807,835,1354,1355,4631]
finished={v['graphics']:v for v in e['finished_rows']};assert set(finished)==set(counts)
for gfx,row in finished.items():
 assert row['action']==0 and row['source_frame']==0 and row['subframe']==counts[gfx]-1
 assert row['sprite']==f'{2174 if gfx==10701 else gfx}-0.spr'
 assert row['callback_count']==counts[gfx]+1 and row['finished'] and not row['drawn'] and row['next_callback_ms'] is None and 'error' not in row
assert e['finished_before']['frame']['skill_effects']['active']==0
assert e['resumed']['frame']['skill_effects']['effects']==[] and e['resumed']['native_cue_ids']==[]
fresh=e['new_generation']['frame']['skill_effects']['effects'];assert len(fresh)==9 and {v['graphics'] for v in fresh}==set(counts)
for row in fresh:assert row['source_frame']==1 and row['subframe']==0 and row['callback_count']==1 and not row['finished'] and 'error' not in row
assert e['new_generation']['native_cue_ids']==[803,801,807,133,835,4631]
e=o['active_case'];assert e['passed'] and e['worker_quiescent'] and e['world_authority_unchanged']
assert e['controlled_ring_rotation_inputs']==129 and e['current_source_ring_size']==128 and e['active_source_cues_retained_independently']==9
before={v['graphics']:v for v in e['before']['frame']['skill_effects']['effects']};after={v['graphics']:v for v in e['resumed']['frame']['skill_effects']['effects']}
assert set(before)==set(after)==set(counts)
for gfx,row in after.items():
 prior=before[gfx];assert row['sequence']==prior['sequence'] and row['callback_count']>prior['callback_count']>=3
 assert row['elapsed_ns']>=prior['elapsed_ns'] and row['source_frame']!=1 and 'error' not in row
 assert row['sprite']==prior['sprite']==f'{2174 if gfx==10701 else gfx}-0.spr'
 assert row['subframe']==(counts[gfx]-1 if row['source_frame']==0 else row['source_frame']-1)
assert e['resumed']['native_cue_ids']==[]
fixture=read(run/'current-world-packets.json');assert sha(run/'current-world-packets.json')==o['fixture_sha256']
assert fixture['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(fixture['source_receipt'])==fixture['source_receipt_sha256'] and sha(fixture['source_capture'])==fixture['source_capture_sha256']
assert sha(Path(fixture['source_receipt']).parent/'visibility-comparison.json')==fixture['source_case_comparison_sha256']
wire=Path(fixture['source_capture']).read_text(encoding='utf-8-sig').splitlines();assert len(fixture['packets'])==18 and len(fixture['expected']['actors'])==13
for packet in fixture['packets']:
 row=json.loads(wire[packet['capture_line']-1]);assert row['direction']=='server_to_client' and re.fullmatch('[0-9a-fA-F ]*',row['plain']) and bytes.fromhex(row['plain']).hex()==packet['hex'] and packet['capture_line']<fixture['source_cast_boundary_line']


changed={Path(n).name for n,h in m['after'].items() if m['parent_after'][n]!=h};assert changed=={'native_attack_many.rs','native_world_scene.rs','native_world_view.rs'}
match=re.search(r'const CASTER_JSON:&\[u8\]=br####"(.*?)"####;', (run/'probe.rs').read_text(encoding='utf-8-sig'),re.S);assert match
caster_bytes=match.group(1).encode('ascii');assert hashlib.sha256(caster_bytes).hexdigest().upper()==o['caster_fixture_sha256'];caster=json.loads(caster_bytes)
for key in ['source_capture','source_client_report','source_raw_archive']:assert sha(caster[key])==caster[key+'_sha256']
assert caster['source_raw_archive_sha256']=='B45663FB5CB14589E11067A14D1D55C503434C89B4A92AD1B568440663F55061'
wire=Path(caster['source_capture']).read_text(encoding='utf-8-sig').splitlines();assert len(caster['packets'])==19
for packet in caster['packets']+[caster['s39']]:
 row=json.loads(wire[packet['capture_line']-1]);assert row['direction']=='server_to_client' and bytes.fromhex(row['plain']).hex()==packet['hex']
b=bytes.fromhex(caster['s39']['hex']);assert b.hex()=='391303000000d97f1d800021aa44005e0f00000001000100000020'
assert (b[1],int.from_bytes(b[2:6],'little'),int.from_bytes(b[15:17],'little'),b[17])==(19,3,3934,0)
assert int.from_bytes(b[20:22],'little')==1 and int.from_bytes(b[22:26],'little')==1 and b[26]==32
assert caster['expected']['hp']==caster['expected']['mp']==[1000,1000] and caster['expected']['own_id']==3 and len(caster['expected']['actors'])==13
case=o['s39_case'];assert case['passed'] and all(case[k] for k in ['captured_packet_acknowledged','S39_remains_many_attack_not_S37','S39_decoder_does_not_spend_authority','new_generation_emits','world_authority_unchanged','worker_quiescent'])
assert case['same_world_resume_new_audio_groups']==0 and case['counter_calls']==[1355,1355]
controls=case['controls'];assert controls['legacy170_profile_still_admitted'] and set(controls['rejected'])=={'heading','wrong-action','wrong-type','extra-byte','target-marker','unreviewed-graphics','zero-wire-sequence','zero-target','tail','truncated','target-budget'}
attack_record=controls['captured_attack'];assert attack_record['action']==19 and attack_record['caster']==3 and attack_record['position']==[32729,32797] and attack_record['graphics']==3934 and attack_record['targets']==[{'id':1,'hit':True}]
first=case['before']['frame']['many_effects']['effects'];resumed=case['resumed']['frame']['many_effects']['effects'];assert len(first)==len(resumed)==1
assert first[0]['graphics']==resumed[0]['graphics']==3934 and first[0]['sprite']==resumed[0]['sprite']=='3934-0.spr'
assert first[0]['position']==resumed[0]['position']==[32729,32797] and first[0]['source_frame']==2 and first[0]['subframe']==1 and first[0]['drawn']
assert resumed[0]['source_frame']!=1 and resumed[0]['callback_count']>=first[0]['callback_count'] and resumed[0]['elapsed_ns']>=first[0]['elapsed_ns']
assert case['before']['native_cue_ids']==[1355] and case['resumed']['native_cue_ids']==[] and case['fresh']['native_cue_ids']==[1355]
assert case['resumed']['frame']['native_finite_audio_history']['retained_keys']>=1
assert case['finished']['frame']['many_effects']['effects']==[] and case['finished']['native_cue_ids']==[]

visual=read(run/'visual-observations.json');assert len(visual['images'])==9
snapshots=[o['completed_case']['finished_before'],o['completed_case']['resumed'],o['completed_case']['new_generation'],o['active_case']['before'],o['active_case']['resumed'],o['s39_case']['before'],o['s39_case']['resumed'],o['s39_case']['finished'],o['s39_case']['fresh']]
assert {Path(x['png']).resolve() for x in snapshots}=={Path(x['path']).resolve() for x in visual['images']}
for row in visual['images']:
 path=Path(row['path']).resolve(strict=True);assert path.is_relative_to(run) and row['viewed'] and sha(path)==row['sha256']
 with Image.open(path) as im:assert im.size==(800,600) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest().upper()==row['rgba_sha256']
ids=[(p['pid'],p['creation_utc']) for p in r['process_history']]
command="$ids=@("+','.join(str(pid) for pid,_ in ids)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
 for pid,created in ids:
  if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
out=run/'qualified';assert not out.exists();out.mkdir();(out/'CIM.json').write_bytes(p.stdout);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())

files={'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json','receipt.json':run/'receipt.json','probe-observation.json':run/'probe-observation.json','visual-observations.json':run/'visual-observations.json','previous-Presentation-qualified.json':previous,'packet-fixture.json':run/'current-world-packets.json'}
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
verify_zip(out/'raw.zip',sha(out/'raw.zip'),hashes)
q=dict(status='independently_qualified_actual_candidate_all_nine_finite_callback_lifecycle',original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,source_manifest_sha256=r['source_manifest_sha256'],candidate_files=20,candidate=str(candidate),candidate_metadata_sha256=sha(candidate/'candidate.json'),actual_product_sha256=r['product_sha256'],runner_raw_archive_sha256=rb['raw_archive_sha256'],bank_qualification=str(bank),bank_qualification_sha256=sha(bank),previous_Presentation_qualification=str(previous),previous_Presentation_qualification_sha256=sha(previous),actual_Model_Presentation_executed=True,finite_graphics=list(counts),finite_profiles_tested=9,finished_source_events_retained=9,finished_callbacks_do_not_restart=True,active_callbacks_continue_after_source_ring_rotation=True,controlled_ring_rotation_inputs=129,current_source_ring_size=128,world_generation_resets_all_nine=True,initial_and_later_counter_audio_groups=8,silent_finite_profiles=1,world_authority_unchanged=True,worker_cases_quiescent=True,diagnostic_PNGs_viewed=9,packet_fixture_sha256=o['fixture_sha256'],captured_fixture_jar_sha256=fixture['jar_sha256'],controlled_renderer_packets_not_actual_casts=True,actual_new_sessions=0,actual_new_casts=0,recorded_direct_children=2,recorded_direct_children_all_terminal=True,native_window_refresh_executed=False,physical_audio_verified=False,GUI_verified=False,all_nine_visual_callback_lifecycle_verified=True,captured_C02_S39_3934_acknowledgement_verified=True,captured_S39_profile_and_lifecycle_verified=True,captured_S39_malformed_controls=11,original_client_visual_oracle=False,all_candidate_gameplay_verified=False,product_source_changed=False,applied=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:q[k] for k in ['status','finite_profiles_tested','initial_and_later_counter_audio_groups','actual_new_casts','native_window_refresh_executed']},ensure_ascii=False))
