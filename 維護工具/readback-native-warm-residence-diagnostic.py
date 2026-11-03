"""Verify measured distribution and fixed-budget failure without GUI promotion."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,math,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int)
ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-performance') and a.session_id>0 and a.terminal_exit_code==0
r=read(run/'receipt.json');assert r['status']=='quiet_Model_warm_residence_diagnostic_complete'
assert r['owned_process'] is None and len(r['processes'])==2 and all(p['exit_code']==0 for p in r['processes'])
ar=read(run/'archive-readback.json');assert sha(run/'raw.zip')==ar['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(ar['files']) and set(z.namelist())==set(ar['files'])
    for name,value in ar['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
        if name!='Release-x64-diagnostic.exe':assert sha(run/name)==value
    binary=z.read('Release-x64-diagnostic.exe');h=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[h+4:h+6]==b'\x64\x86' and binary[h+24:h+26]==b'\x0b\x02'
    assert hashlib.sha256(binary).hexdigest().upper()==r['diagnostic_sha256']
assert sha(r['broad_completion'])==r['broad_completion_sha256']
broad=read(r['broad_completion']);assert broad['observed_terminal_exit_code']==0 and broad['recorded_original_owned_processes_all_terminal']
assert broad['source_manifest_sha256']==r['source_manifest_sha256']
budget_path=ROOT/'驗證紀錄/current/native-performance/acceptance-budget-v1.json'
assert sha(budget_path)==sha(run/'acceptance-budget-v1.json')==r['budget_sha256']=='98F17500C627D9A5F18767329A7F4913CEB42F09B1226DFA6F8B3735E8CF3CBA'
before=read(run/'quiet-before-measurement.json');after=read(run/'quiet-after-measurement.json')
assert not before['heavy'] and not after['heavy']
result=read(run/'diagnostic-results.json');assert result['warmup_reloads']==10 and result['warmup_pairs']==5 and result['measured_reloads']==40
assert result['synthetic_authority_fixture'] and result['actors']==1 and not result['GUI_verified']
records=result['records'];assert len(records)==50
for i,row in enumerate(records):
    assert row['index']==i and row['phase']==('warmup' if i<10 else 'measured')
    assert row['wall_us']>0 and row['decoded_bytes']<=64<<20 and row['load_profile_us']['archive_sources_reused']
    if i:
        assert row['terrain_reload_count']==records[i-1]['terrain_reload_count']+1
        assert row['position']!=records[i-1]['position'] and row['native_terrain_center']!=records[i-1]['native_terrain_center']
values=sorted(row['wall_us'] for row in records[10:]);assert len(values)==40
stats=dict(median=(values[19]+values[20])//2,p95_nearest_rank=values[math.ceil(.95*40)-1],maximum=values[-1],sorted_samples=values)
assert stats==result['wall_us']==r['wall_us']
limits=read(budget_path)['budgets']['native_scene_load']
within=stats['p95_nearest_rank']<=limits['warm_residency_reload_p95_ms_max']*1000 and stats['maximum']<=limits['warm_residency_reload_single_ms_max']*1000
assert within==r['synthetic_warm_distribution_within_original_budget']
assert not within,'This baseline qualification records the observed over-budget run; preserve changed distributions separately.'
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,value in pins.items():assert sha(ROOT/name)==value
for info in r['libraries'].values():assert sha(info['path'])==info['sha256']
q=dict(status='independently_read_back_quiet_Release_Model_warm_distribution_over_fixed_p95_budget',
    original_session_id=a.session_id,observed_terminal_exit_code=0,source_manifest_sha256=r['source_manifest_sha256'],
    warmup_reloads=10,measured_reloads=40,wall_us=stats,cold_Model_load_plus_first_draw_us=result['cold_Model_load_plus_first_draw_us'],
    original_warm_p95_budget_ms=limits['warm_residency_reload_p95_ms_max'],original_warm_single_budget_ms=limits['warm_residency_reload_single_ms_max'],
    synthetic_warm_p95_budget_met=False,synthetic_warm_single_budget_met=stats['maximum']<=limits['warm_residency_reload_single_ms_max']*1000,
    budget_sha256=r['budget_sha256'],budget_changed=False,no_compiler_or_owned_executable_validation_observed_before_and_after=True,
    host_CPU_load_before_percent=before['CPU_load_percent'],host_CPU_load_after_percent=after['CPU_load_percent'],
    host_other_load_not_excluded=True,synthetic_authority_fixture=True,actors=1,actual_formal_sessions=0,
    GUI_verified=False,thirty_player_gameplay_measured=False,performance_accepted=False,whole_goal_complete=False,
    raw_archive_sha256=ar['archive_sha256'],raw_files=ar['files'],observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'independent-readback.json').exists()
(run/'independent-readback.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','measured_reloads','original_warm_p95_budget_ms','synthetic_warm_p95_budget_met']},ensure_ascii=False))
