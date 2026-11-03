"""Independently read native x64 spatial candidate outputs; never infer playback."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-cone-cold-motion-candidate') and a.terminal_chunk not in ['','pending','unknown']
r=read(run/'receipt.json');assert r['status']=='executed_both_profiles_isolated_spatial_candidate_only' and r['owned_process'] is None
assert not r['normal_product_registered'] and not r['renderer_qualified'] and not r['world_cell_conversion_qualified'] and not r['timer_units_qualified']
assert r['new_sessions']==r['new_casts']==0 and len(r['process_history'])==6 and all(p['exit_code']==0 for p in r['process_history'])
assert sha(run/'motion.rs')==r['candidate_sha256']==sha(ROOT/'維護工具/candidates/native-cone-cold/motion.rs')
assert sha(r['primary_motion_binding'])==r['primary_motion_binding_sha256'];binding=read(r['primary_motion_binding']);assert binding['status']=='primary_type5_motion_and_arrival_control_flow_bound_only'
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1035
for n,h in inputs.items():assert sha(ROOT/n)==h
expected={'tests::eight_native_directions_reach_on_the_third_dispatch','tests::asymmetric_positive_rounding_preserves_native_even_x','tests::asymmetric_negative_rounding_preserves_native_bias','tests::y_dominant_motion_has_the_native_repeated_x_samples','tests::odd_destination_is_not_silently_snapped_or_clamped','tests::fixed16_wrap_is_retained_in_the_nonarrival_boundary_case','tests::coincident_points_arrive_once_on_the_immediate_dispatch','tests::retirement_stops_late_callbacks_without_an_arrival'}
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file()};counts={}
for profile in ['Debug','Release']:
    info=r['profiles'][profile];exe=Path(info['product']);assert sha(exe)==info['product_sha256'];pe=exe.read_bytes();e=int.from_bytes(pe[0x3c:0x40],'little');assert pe[:2]==b'MZ' and pe[e:e+4]==b'PE\0\0' and int.from_bytes(pe[e+4:e+6],'little')==0x8664
    d=run/profile;inventory={s.split(': test')[0] for s in (d/'inventory/stdout.log').read_text(encoding='utf-8-sig').splitlines() if s.endswith(': test')};assert inventory==expected==set(info['inventory'])
    log=(d/'runtime/stdout.log').read_text(encoding='utf-8-sig');assert set(re.findall(r'^test (\S+) \.\.\. ok$',log,re.M))==expected
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)==[('8','0','0','0','0')]
    compiler=next(j for j in r['process_history'] if Path(j['logs'])==d/'compile');assert compiler['command'][:2]==[compiler['command'][0],str(run/'motion.rs')]
    assert compiler['command'][compiler['command'].index('--target')+1]=='x86_64-pc-windows-msvc'
    if profile=='Release':
        for flag in ['opt-level=z','lto=thin','codegen-units=16','strip=symbols']:assert flag in compiler['command']
    files['products/'+profile+'.exe']=exe;counts[profile]=dict(passed=8,failed=0,ignored=0,product_sha256=sha(exe))
ids=','.join(str(j['pid']) for j in r['process_history']);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress";raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in r['process_history']:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'CIM.json').write_bytes(raw);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());files.update({'reader.py':out/'qualifier.py','CIM.json':out/'CIM.json'});pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_isolated_type5_spatial_candidate_both_profiles_only',source_manifest_sha256=r['source_manifest_sha256'],candidate_sha256=r['candidate_sha256'],run=str(run),profiles=counts,original_session=a.session,observed_original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,recorded_child_identities=6,recorded_children_all_terminal=True,original_runner_birth_recorded=False,original_runner_terminal_tool_observed=True,scope='captured unsigned16 native points, spatial stepping/termination only',world_cell_conversion_qualified=False,timer_units_qualified=False,actor_life_owner_fences_qualified=False,asset_rendering_qualified=False,normal_product_registered=False,new_sessions=0,new_casts=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins)
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),passed_each=8,new_casts=0,normal_product_registered=False),ensure_ascii=False))
