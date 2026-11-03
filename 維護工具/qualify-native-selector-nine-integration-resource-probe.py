"""Read back coherent candidate bank pixels and the observed Effects boundary."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def archive(path,digest,pins):
 assert sha(path)==digest
 with zipfile.ZipFile(path) as z:
  assert len(z.namelist())==len(pins) and set(z.namelist())==set(pins)
  for n,h in pins.items():
   with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);ap.add_argument('--tool-chunk-id',required=True);a=ap.parse_args()
assert a.session_id>0 and a.terminal_exit_code==0 and a.tool_chunk_id
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-nine-integration-resource-probe')
r=read(run/'receipt.json');rb=read(run/'readback.json');assert r['status']==rb['status']=='executed_coherent_candidate_resource_and_Effects_component_probe'
assert r['owned_process'] is None and len(r['process_history'])==2 and all(q['exit_code']==0 for q in r['process_history'])
archive(run/'raw.zip',rb['raw_archive_sha256'],rb['raw_files'])
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for n,h in pins.items():assert sha(ROOT/n)==h,n
candidate=Path(r['candidate']).resolve(strict=True);assert candidate.is_relative_to(ROOT/'維護工具/candidates')
m=read(candidate/'candidate.json');assert not m['applied'] and m['base_source_manifest_sha256']==r['source_manifest_sha256']
assert m['after']==r['candidate_after'] and len(m['after'])==19
for n,h in m['after'].items():assert sha(candidate/'sources'/Path(n).name)==h
for v in r['libraries'].values():assert sha(v['path'])==v['sha256']
for v in r['baseline_gates'].values():assert sha(v['path'])==v['sha256'] and read(v['path'])['source_manifest_sha256']==r['source_manifest_sha256']
product=Path(r['product']).resolve(strict=True);assert product.is_relative_to(ROOT/'LiTo381-Modern/target/current') and sha(product)==r['product_sha256']
with product.open('rb') as f:
 h=f.read(64);assert h[:2]==b'MZ';f.seek(int.from_bytes(h[60:64],'little'));h=f.read(26);assert h[:4]==b'PE\0\0' and h[4:6]==b'\x64\x86' and h[24:26]==b'\x0b\x02'
observation=read(run/'probe-observation.json');assert sha(run/'probe-observation.json')==r['observation_sha256']
reference=ROOT/'驗證紀錄/current/native3934-empty-sentinel-candidate/20261017-071716-373733'
reference_q=read(reference/'independent-readback.json');assert reference_q['independent_contract_frame_matches']==128 and reference_q['source_manifest_sha256']==r['source_manifest_sha256']
reference_archive=reference/'separate-python-contract-readback/raw.zip';assert sha(reference_archive)==reference_q['separate_contract_readback_raw_sha256']
with zipfile.ZipFile(reference_archive) as z:
 expected=read(reference/'separate-python-contract-readback/all-nine-frame-readback.json')
 assert z.read('all-nine-frame-readback.json')==(reference/'separate-python-contract-readback/all-nine-frame-readback.json').read_bytes()
assert expected['frames_checked']==128
members={row['member']:row for row in expected['rows']};assert len(members)==9
mapping={2170:'2170-0.spr',10701:'2174-0.spr',2173:'2173-0.spr',759:'759-0.spr',3936:'3936-0.spr',3104:'3104-0.spr',3934:'3934-0.spr',2234:'2234-0.spr',6527:'6527-0.spr'}
frames=observation['frames'];assert len(frames)==128
seen=set()
for row in frames:
 gfx=row['graphics'];assert gfx in mapping and row['sprite']==mapping[gfx]
 key=(gfx,row['source_frame']);assert key not in seen;seen.add(key)
 count=len(members[row['sprite']]['frames'])
 assert row['subframe']==(count-1 if row['source_frame']==0 else row['source_frame']-1), 'Native terminal slot0 precedes source slots1..N-1 in the catalog'
 target=members[row['sprite']]['frames'][row['subframe']]
 assert row['subframe']==target['subframe']
 assert [row[k] for k in ['width','height','x_offset','y_offset','rgba_sha256']]==[target[k] for k in ['width','height','x','y','rgba_sha256']]
assert set(seen)=={(gfx,i) for gfx,member in mapping.items() for i in range(len(members[member]['frames']))}
assert observation['blank_frames']==[[3936,0,12],[3936,1,0],[3934,0,12],[3934,1,0]]
flat=sum(f['rgba_bytes'] for member in members.values() for f in member['frames'])
assert observation['flat_frame_rgba_bytes']==flat==sum(row['rgba_bytes'] for row in frames)
assert observation['retained_fragment_bytes']>0
assert observation['bank_retained_rgba_bytes']==flat+observation['retained_fragment_bytes']
assert 0<observation['bank_encoded_bytes']<=64*1024*1024 and observation['bank_retained_rgba_bytes']<=64*1024*1024
assert 0<observation['sound_file_bytes']<=8*1024*1024 and 0<observation['sound_pcm_bytes']<=8*1024*1024
first=observation['first_component']['new_forwardings'];second=observation['reconstructed_component']['new_forwardings'];assert first==1 and second in [0,1]
assert observation['reconstruction_counter_forwardings']==[803]*(first+second)
controls=observation['reviewed_blank_source_controls']
assert controls['positive_sources']==['3936-0.spr','3934-0.spr'] and len(controls['rejects'])==len(set(controls['rejects']))==40
expected_audio={2170:(803,'SuppressWhileSameIdPlaying'),10701:(801,'SuppressWhileSameIdPlaying'),2173:(807,'SuppressWhileSameIdPlaying'),759:(133,'SuppressWhileSameIdPlaying'),3936:(1354,'AllowOverlappingVoices'),3104:(None,'none'),3934:(1355,'AllowOverlappingVoices'),2234:(835,'SuppressWhileSameIdPlaying'),6527:(4631,'AllowOverlappingVoices')}
producers=observation['all_nine_finite_producers'];assert len(producers)==9 and {p['graphics'] for p in producers}==set(expected_audio)
for producer in producers:
 sound,mode=expected_audio[producer['graphics']];groups=producer['emitted_groups']
 assert producer['receipt']['new_forwardings']==(0 if sound is None else 1)
 assert len(groups)==(0 if sound is None else 1)
 if sound is not None:assert groups==[dict(sound=sound,mode=mode,candidates=[sound],selection=0)]
assert observation['counter_forwardings']==[803]*(first+second)+[sound for sound,_ in expected_audio.values() if sound is not None]
assert observation['component_reconstruction_reemitted']==(second>0)
assert observation['actual_new_sessions']==observation['actual_new_casts']==0
assert not observation['audio_device_opened'] and not observation['full_Model_or_Presentation_executed']
identities=[(q['pid'],q['creation_utc']) for q in r['process_history']]
command="$ids=@("+','.join(str(pid) for pid,_ in identities)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for q in live:
 for pid,created in identities:
  if q['ProcessId']==pid:assert abs((datetime.fromisoformat(q['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
out=run/'qualified';assert not out.exists();out.mkdir();(out/'CIM.json').write_bytes(p.stdout);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files={'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json','receipt.json':run/'receipt.json','probe-observation.json':run/'probe-observation.json','reference-readback.json':reference/'independent-readback.json','reference-frames.json':reference/'separate-python-contract-readback/all-nine-frame-readback.json'}
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
archive(out/'raw.zip',sha(out/'raw.zip'),hashes)
q=dict(status='independently_qualified_candidate_resource_bank_and_observed_Effects_boundary',
 original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,
 source_manifest_sha256=r['source_manifest_sha256'],candidate_files=19,actual_product_sha256=r['product_sha256'],
 original_runner_raw_archive_sha256=rb['raw_archive_sha256'],original_same_format_contract_reference_archive_sha256=sha(reference_archive),
 independently_compared_RGBA_frames=128,explicit_alias='10701->2174',exact_blank_frames=observation['blank_frames'],
 independently_summed_flat_RGBA_bytes=flat,reported_retained_fragment_bytes=observation['retained_fragment_bytes'],
 reported_full_retained_bytes=observation['bank_retained_rgba_bytes'],retained_byte_budget=64*1024*1024,
 compact_atlas_allocation_bytes_independently_recomputed=False,
 first_Effects_component_forwardings=first,reconstructed_Effects_component_forwardings=second,
 exact_blank_source_negative_controls=40,actual_finite_producers_exercised=9,typed_audio_groups_emitted=8,
 actual_overlap_groups=3,actual_suppress_groups=5,silent_finite_producers=1,
 component_reconstruction_reemitted=second>0,full_Model_Presentation_replay_suppression_verified=False,
 actual_new_sessions=0,actual_new_casts=0,recorded_direct_processes=2,recorded_direct_processes_all_terminal=True,
 candidate_whole_runtime_qualified=False,product_source_changed=False,applied=False,
 independent_original_client_pixel_oracle=False,GUI_verified=False,physical_audio_verified=False,
 raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
save(out/'qualified.json',q);print(json.dumps({k:q[k] for k in ['status','independently_compared_RGBA_frames','component_reconstruction_reemitted','full_Model_Presentation_replay_suppression_verified']},ensure_ascii=False))
