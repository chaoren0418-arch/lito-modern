"""Independently read phase/owner/target cases; do not infer C02 or rendering."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
assert a.session is None or a.session>0
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-cone-cold-core-runtime') and a.terminal_chunk not in ['','pending','unknown']
r=read(run/'receipt.json');assert r['status']=='executed_both_profiles_frozen_cold_core_scenarios_only' and r['owned_process'] is None
assert len(r['process_history'])==2 and all(j['exit_code']==0 and not j.get('timed_out') for j in r['process_history'])
assert sha(r['current_library_completion'])==r['current_library_completion_sha256']
terminal=read(r['current_library_completion']);assert terminal['observed_terminal_exit_code']==0 and terminal['recorded_original_owned_processes_all_terminal']
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file()};jobs=r['process_history']+[r['original_runner']];reports={}
expected=['entry_immediate_dispatch','captured_flight_arrival_current_anchor','frozen_impact_late_from_now','terminal_source0_full_dwell','old_owner_launch_and_unplayed_audio_rejected','reused_target_rejects_impact','same_map_new_generation_retires_flight']
for profile in ['Debug','Release']:
    info=r['profiles'][profile];report=Path(info['report']);s=read(report);assert sha(report)==info['report_sha256'] and info['actual_exit_code']==0 and info['cases']==7
    assert s['status']=='executed_isolated_cold_core_scenarios_only' and s['synthetic_local_projectile_cues'] and s['actual_C02_casts']==s['new_sessions']==0
    assert not s['rendering_executed'] and not s['physical_audio_verified'] and not s['normal_product_registered'] and not s['GUI_verified']
    assert [v['case']for v in s['cases']]==expected;rows=s['cases'];snap=[v['snapshot']for v in rows]
    e=snap[0]['effects'][0];assert e['phase']==dict(kind='fly',next_callback_ms=60) and e['position']==[32769,32769] and e['source_frame']==1 and e['direction']==3
    assert snap[0]['started']==1 and snap[0]['arrival_hurts']==0
    e=snap[1]['effects'][0];assert e['phase']['kind']=='impact' and e['position']==[32780,32780] and e['source_frame']==1
    assert snap[1]['arrival_hurts']==snap[1]['prepared_sound_events']==1
    e=snap[2]['effects'][0];assert e['position']==[32780,32780] and e['source_frame']==3 and e['phase']['start_ms']+e['phase']['next_callback_ms']==2166
    assert snap[2]['arrival_hurts']==1 and snap[2]['prepared_sound_events']==0
    assert snap[3]['active']==0 and snap[3]['effects'][0]['source_frame']==0 and snap[3]['effects'][0]['finished']
    assert rows[4]['sink_calls']==0 and snap[4]['active']==0 and snap[4]['started']==0
    assert snap[5]['active']==snap[5]['arrival_hurts']==snap[5]['prepared_sound_events']==0
    assert snap[6]['active']==snap[6]['started']==0 and not snap[6]['effects']
    bpath=Path(r['builds'][profile]['path']);b=read(bpath/'receipt.json');assert sha(bpath/'receipt.json')==r['builds'][profile]['receipt_sha256']
    assert b['profile']==profile and b['compiler']['exit_code']==0 and b['source_manifest_sha256']==r['source_manifest_sha256'] and not b['source_overrides_used']
    exe=Path(b['product']);assert sha(exe)==b['product_sha256']==r['builds'][profile]['product_sha256']
    pe=exe.read_bytes();offset=int.from_bytes(pe[60:64],'little');assert pe[:2]==b'MZ' and pe[offset:offset+4]==b'PE\0\0' and int.from_bytes(pe[offset+4:offset+6],'little')==0x8664
    if profile=='Release':
        for flag in ['opt-level=z','lto=thin','codegen-units=16','strip=symbols']:assert flag in b['compiler']['command']
    for n,h in read(bpath/'compiled-inputs.json').items():assert sha(ROOT/n)==h
    for name,pin in b['candidate_inputs'].items():assert sha(pin['path'])==pin['sha256']==r['candidate_inputs'][name];files['compile/'+profile+'/'+name]=bpath/name
    for name in ['receipt.json','runner.py','candidate_source.rs','compiled-inputs.json']:files['compile/'+profile+'/'+name]=bpath/name
    files['products/'+profile+'.exe']=exe;jobs.extend([b['original_runner'],b['compiler']]);reports[profile]=s
assert reports['Debug']==reports['Release']
ids=','.join(str(v['pid'])for v in jobs);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live]if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'CIM.json').write_bytes(raw);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());files.update({'reader.py':out/'qualifier.py','CIM.json':out/'CIM.json'});pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_isolated_cold_core_both_profiles_only',run=str(run),base_source_manifest_sha256=r['source_manifest_sha256'],candidate_inputs=r['candidate_inputs'],profiles=['Debug','Release'],cases_each=7,profiles_exact_outcomes_equal=True,original_session=a.session,original_tool_completed_without_session_id=a.session is None,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,recorded_original_identities=len(jobs),recorded_original_identities_all_terminal=True,scope='local synthetic cue phase transitions, current-target arrival anchor, no duplicate hurt, terminal dwell, owner/audio-latch and incarnation/generation retirement only',old_external_clock_mapping_qualified=False,normal_product_registered=False,actual_C02_casts=0,new_sessions=0,rendering_executed=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),cases_each=7,actual_C02_casts=0,normal_product_registered=False),ensure_ascii=False))
