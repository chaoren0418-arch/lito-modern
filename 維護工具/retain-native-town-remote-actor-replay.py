"""Seal the offline 14/15-actor comparison without inferring timing causation."""
from pathlib import Path
import json,hashlib,zipfile
ROOT=Path(__file__).resolve().parents[1]
RUN=ROOT/'驗證紀錄/current/native-G06-controller/20261017-034558-407944/town-remote-actor-replay-20261017-040540-289846'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
c=read(RUN/'cohort.json');assert c['status']=='executed_offline_14_versus_15_actor_town_replay' and c['owned_process'] is None
assert len(c['process_history'])==2 and all(p['exit_code']==0 for p in c['process_history'])
obs=read(RUN/'independent-process-observation.json');assert len(obs)==2 and not any(p['same_recorded_identity_alive'] for p in obs)
fixture=read(RUN/'fixture.json');assert sha(fixture['additional_capture'])==fixture['additional_capture_sha256']
assert sha(fixture['baseline_fixture'])==fixture['baseline_fixture_sha256']
assert fixture['worlds']['b']['expected_actor_fields']['3']['hp_ratio']==100
files={'run/'+p.relative_to(RUN).as_posix():p for p in RUN.rglob('*') if p.is_file()}
summary={}
for profile,info in c['profiles'].items():
    assert sha(info['executable'])==info['executable_sha256']
    result=RUN/profile/'results.json';assert sha(result)==info['results_sha256']
    records=read(result)['records'];assert [r['actors'] for r in records]==[14,15]
    summary[profile]=[{k:r[k] for k in ['actors','load_us','first_draw_us','load_profile_us','render_profile_us']} for r in records]
    files['binaries/'+profile+'.exe']=Path(info['executable'])
out=RUN/'independent-readback';out.mkdir();(out/'reader.py').write_bytes(Path(__file__).read_bytes());files['reader.py']=out/'reader.py'
pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
record=dict(status='retained_offline_14_versus_15_actor_comparison',original_session_id=96616,observed_terminal_exit_code=0,
    terminal_tool_chunk_id='323895',source_manifest_sha256=c['source_manifest_sha256'],results=summary,
    observed_first_Debug_archive_pin_us=summary['Debug'][0]['load_profile_us']['archive_pins_us'],
    finding='The first 14-actor Debug Model spent 23.65s observing/hashing archive sources; the following 15-actor Model spent 0.88s in that stage. Release first draws were about 0.20s for both. This is ordered, unflushed-cache data, not an isolated actor-count causal experiment.',
    source_pool_reuse_implemented=False,product_sources_changed=False,OS_cache_not_flushed=True,other_host_load_not_excluded=True,
    recorded_processes_all_terminal=True,actual_new_sessions=0,actual_qualified_formal_sessions=0,
    original_eight_second_failure_root_cause_proven=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,
    performance_accepted=False,GUI_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(readback=str(out/'readback.json'),archive_sha256=record['raw_archive_sha256']),ensure_ascii=False))
