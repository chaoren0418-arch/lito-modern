"""Read actual candidate bank against retained member hashes and independent WAV."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,struct,wave,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def digest(b):return hashlib.sha256(b).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
assert a.session is None or a.session>0
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-cone-cold-assets-runtime') and a.terminal_chunk not in ['','pending','unknown']
r=read(run/'receipt.json');assert r['status']=='executed_isolated_cold_assets_Debug_only' and r['owned_process'] is None and set(r['profiles'])=={'Debug'}
assert len(r['process_history'])==2 and all(v['exit_code']==0 for v in r['process_history'])
assert r['resource_bank_loaded'] and r['resource_pack_read'] and not r['audio_played'] and r['new_sessions']==r['new_casts']==0
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
assert sha(run/'cone_cold_assets.rs')==r['candidate_sha256']==sha(ROOT/'維護工具/candidates/native-cone-cold/cone_cold_assets.rs')
assert sha(run/'assets_probe.rs')==r['profiles']['Debug']['probe_sha256']==sha(ROOT/'維護工具/candidates/native-cone-cold/assets_probe.rs')
p=r['profiles']['Debug'];exe=Path(p['artifact']);assert sha(exe)==p['artifact_sha256']
pe=exe.read_bytes();e=struct.unpack_from('<I',pe,60)[0];assert pe[:2]==b'MZ' and pe[e:e+4]==b'PE\0\0' and struct.unpack_from('<H',pe,e+4)[0]==0x8664
for pin in p['library_pins'].values():assert sha(pin['path'])==pin['sha256']
report=read(run/'asset-bank.json');assert report['status']=='loaded_isolated_canonical_cold_assets_only'
assert all(report[k] for k in ['pre_cancel_refused','invalid_direction_refused','all_direction_pairs_checked','impact_pair_checked'])
assert not any(report[k] for k in ['audio_device_opened','rendering_executed','normal_product_registered'])
old=ROOT/'驗證紀錄/current/native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json';oq=read(old.parent/'qualified/qualified.json');assert sha(old)==oq['raw_files'][old.name]
tw=next(v for v in read(old)['explicit_TW_namespace_inspections'] if v['namespace']=='launcher/TW13081901.pak');reference={v['name']:v for v in tw['members']}
assert report['pack_sha256']==tw['pack_sha256'] and report['text_sha256']==tw['text_sha256']
expected={f'{g}-{i}.spr' for g in [11738,11739] for i in range(8)}|{'11740-0.spr','11741-0.spr'}
assert {v['name'] for v in report['members']}==expected and len(report['members'])==18
frames=0;empty=[];retained=0;encoded=0
for member in report['members']:
    ref=reference[member['name']];assert member['raw_sha256']==ref['raw_sha256'] and member['raw_bytes']==ref['raw_bytes'];encoded+=member['raw_bytes']
    assert len(member['frames'])==len(ref['frames'])==ref['declared_headers']['frame_count']
    for f,g in zip(member['frames'],ref['frames']):
        for key in ['index','offset','size','unknown','frame_type','rgba_sha256','fragment_references']:assert f[key]==g[key],(member['name'],f['index'],key)
        assert f['rgba_bytes']==f['size'][0]*f['size'][1]*4;retained+=f['rgba_bytes'];frames+=1
        if not f['rgba_bytes']:empty.append((member['name'],f['index'],f['offset'],f['unknown']))
assert frames==64 and empty==[('11741-0.spr',7,[192,120],[65535,0])]
assert encoded==report['encoded_sprite_bytes']==166500 and retained==1655424
# The world decoder retains compact fragment atlases in addition to flat RGBA.
# This is the actual bank's memory observation, bounded separately from the
# independently summed source-frame pixels; do not equate the two measures.
assert retained<=report['retained_rgba_bytes']==1918784<=32*1024*1024
sound=ROOT.parents[1]/'0-客戶端/Sound/715.wav';s=report['sound'];assert sha(sound)==s['wav_sha256'] and sound.stat().st_size==s['wav_bytes']==88696
with wave.open(str(sound),'rb') as w:
    assert (w.getnchannels(),w.getframerate(),w.getsampwidth(),w.getcomptype())==(1,22050,2,'NONE');pcm=w.readframes(w.getnframes());assert len(pcm)==s['pcm_bytes']==88576 and digest(pcm)==s['pcm_sha256']
assert report['sound_group']=='NativeAudioGroup { mode: SuppressWhileSameIdPlaying, candidates: [715] }'
jobs=r['process_history']+[r['original_runner']];ids=','.join(str(v['pid']) for v in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'CIM.json').write_bytes(raw);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file()};files.update({'product/Debug.exe':exe,'reference/asset-inventory.json':old,'reference/qualified.json':old.parent/'qualified/qualified.json','reference/715.wav':sound});pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(digest(z.read(n))==h for n,h in pins.items())
q=dict(status='independently_qualified_isolated_cold_asset_bank_Debug_only',source_manifest_sha256=r['source_manifest_sha256'],candidate_sha256=r['candidate_sha256'],run=str(run),original_session=a.session,original_tool_completed_without_session_id=a.session is None,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,recorded_original_identities=3,recorded_original_identities_all_terminal=True,profiles=['Debug'],members=18,source_frames=64,nonempty_frames=63,reviewed_empty_frames=1,encoded_sprite_bytes=encoded,flattened_frame_rgba_bytes=retained,retained_rgba_including_fragment_atlases=report['retained_rgba_bytes'],all_member_raw_and_decoded_hashes_match_retained_reference=True,independent_WAV_PCM_readback=True,sound=s,normal_product_registered=False,new_sessions=0,new_casts=0,rendering_executed=False,renderer_qualified=False,audio_device_opened=False,physical_audio_verified=False,GUI_verified=False,whole_goal_complete=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins)
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),members=18,source_frames=64,PCM_bytes=len(pcm),renderer_qualified=False),ensure_ascii=False))
