"""Combine retained positive Debug/Release cases without rerunning players."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest().upper()

ap = argparse.ArgumentParser()
ap.add_argument('Debug',type=Path)
ap.add_argument('Release',type=Path)
a = ap.parse_args()
profiles = {}
pins = {}
common = None
for profile in ['Debug','Release']:
    path = getattr(a,profile).resolve(strict=True)
    assert path.is_relative_to(ROOT / '驗證紀錄/current/native-startup-siege563')
    q = read(path)
    assert q['status']=='independently_qualified_current_selected_map563_thirty_player_TW_extended'
    assert q['selected_profiles']==[profile] and set(q['profiles'])=={profile}
    assert q['actual_formal_sessions']==60 and q['actual_test_executables']==1
    assert q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_terminal']
    identity = {k:q[k] for k in ['source_manifest_sha256','source_files','compiled_input_files',
        'normal_mains','resource_profile','extended_item_details','jar_sha256','common_catalog_sha256']}
    if common is None:
        common=identity
    else:
        assert common==identity
    raw=path.parent/'raw.zip'
    assert sha(raw)==q['raw_archive_sha256']
    with zipfile.ZipFile(raw) as z:
        assert len(z.namelist())==len(q['raw_files']) and set(z.namelist())==set(q['raw_files'])
        for name,h in q['raw_files'].items():
            with z.open(name) as f:
                assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    v=q['profiles'][profile]
    assert v['independent_wire']['map']==v['independent_terrain']['map']==563
    assert all(x['original_tag']==102 for x in v['independent_wire']['checks'])
    assert v['independent_wire']['C73_requests']==4 and v['independent_wire']['S5A_replies']==2 and v['independent_wire']['C08_acks']==2
    profiles[profile]=dict(qualification=str(path),qualification_sha256=sha(path),
        original_session_id=q['original_session_id'],actual_formal_sessions=60,actual_test_sha256=v['actual_test_sha256'],
        C_MOVE_requests=v['independent_terrain']['C_MOVE_requests'],
        original_world_PNGs=sum(len(x['images']) for x in v['independent_terrain']['checks']),
        max_decoded_terrain_bytes=max(n for x in v['independent_terrain']['checks'] for n in x['decoded_bytes']),
        original_owned_identities=q['recorded_owned_identities'],raw_archive_sha256=q['raw_archive_sha256'])
    source=read(path.parent.parent/'source-inputs.json')
    if not pins:
        pins=source
    else:
        assert pins==source
assert len(pins)==1024 and all(sha(ROOT/name)==h for name,h in pins.items())
out=ROOT/'驗證紀錄/current/native-startup-siege563'/('combined-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out/'aggregator.py').write_bytes(Path(__file__).read_bytes())
result=dict(status='independently_qualified_both_current_map563_thirty_player_TW_extended',**common,
    profiles=profiles,actual_formal_sessions=120,actual_test_executables=2,
    C_MOVE_requests=sum(v['C_MOVE_requests'] for v in profiles.values()),
    original_world_PNGs=sum(v['original_world_PNGs'] for v in profiles.values()),
    max_decoded_terrain_bytes=max(v['max_decoded_terrain_bytes'] for v in profiles.values()),
    recorded_owned_identities=sum(v['original_owned_identities'] for v in profiles.values()),
    recorded_owned_processes_terminal=True,gameplay_rerun=False,GUI_verified=False,
    whole_G06_complete=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(qualification=str(out/'qualified.json'),sessions=120,moves=result['C_MOVE_requests'],PNGs=result['original_world_PNGs']),ensure_ascii=False))
