"""Read back a terminal real-time hotel lease case against its immutable products."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, sys, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

def verify_zip(path, digest, pins=None):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as archive:
        assert len(set(archive.namelist())) == len(archive.namelist())
        if pins is not None:
            assert set(archive.namelist()) == set(pins)
        for name in archive.namelist():
            with archive.open(name) as stream:
                actual = hashlib.file_digest(stream, 'sha256').hexdigest().upper()
            if pins is not None:
                assert actual == pins[name], name

ap = argparse.ArgumentParser()
ap.add_argument('case', type=Path)
ap.add_argument('--session-id', type=int, required=True)
ap.add_argument('--terminal-exit-code', type=int, required=True)
ap.add_argument('--client-identities', type=Path, required=True)
a = ap.parse_args()
case = a.case.resolve(strict=True)
assert case.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers') and a.terminal_exit_code == 0
out = case/'independent-natural-qualified'
assert not out.exists()
r = read(case/'receipt.json'); start = read(case/'natural-expiry-start.json')
runtime = Path(r['runtime']).resolve(strict=True)
assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current/native-real-peers')
assert r['status'] == 'verified_real_peer_action' and r['profile'] == 'Release'
assert r['action_profile'] in ['rental-hotel-expiry', 'rental-hotel-middle-expiry']
assert r['resource_profile'] == 'tw13081901-v1' and r['extended_item_details']
assert r['source_matches_snapshot'] and not start['clock_or_authority_injection']
assert r['exe_sha256'] == start['exe_sha256'] == sha(start['immutable_exe']) == sha(runtime/'qualified-client.exe')
assert r['exe_sha256'] == '139B5247429B719B5F188CE16EC65FD5036A290E2FAFCDBCAEA3CD165E43C51F'
assert r['jar_sha256'] == start['jar_sha256'] == r['source_sha256_at_end'] == sha(ROOT/'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar')
assert r['jar_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert r['common_catalog_sha256'] == sha(ROOT/'LiTo381-Modern/target/current/shared-profile-inputs/common_catalog-B64ED920.bin')
s = r['server']
assert s == read(case/'result.json') and s['status'] == 'sessions_closed'
assert s['actual_client_sessions'] == 4 and s['game_sessions'] == 2 and s['memberships_empty']
assert s['authority_mutations'] == s['active_hotel_leases_after_disconnect'] == s['classic_world_members_after_disconnect'] == 0
assert not s['sql_access'] and not s['state_persistent']
snapshot = case/'validation-snapshot'
assert len(list(snapshot.iterdir())) == 10
for file in snapshot.iterdir():
    key = '維護工具/'+file.name if file.suffix == '.py' else 'LiTo381-Modern/tests/fixtures/'+file.name
    assert sha(file) == start['development_inputs'][key], key
sys.path.insert(0, str(snapshot))
from native_resident_map_reference import qualify as terrain
from native_hotel_catalog_reference import fixture_reference
fixture_path = snapshot/'native-portal-current.json'
fixture = read(fixture_path); fixture_proof = fixture_reference(fixture_path, r['jar_sha256'])
marker = read(case/'validation-completed.json')
assert marker['receipt_sha256'] == sha(case/'receipt.json') and not marker['runtime_receipt_rewritten']
assert set(marker['comparison_pins']) == {'rental-entry-comparison.json', 'rental-hotel-entry-comparison.json', 'rental-hotel-expiry-comparison.json', 'attributes-comparison.json'}
for name, digest in marker['comparison_pins'].items():
    assert sha(case/name) == digest and read(case/name)['receipt_sha256'].upper() == sha(case/'receipt.json')

# Link the retained normal image to its archived compiler inputs. Live developer
# inputs changed in a cfg-test-only correction; do not claim they stayed frozen.
normal_path = ROOT/'驗證紀錄/current/native-actor-green-clock/related-20261016-161430-077487/qualified.json'
n = read(normal_path); pins = read(normal_path.parent/'source-inputs.json')
assert n['status'] == 'independently_qualified_green_clock_related_scope_and_unique_entry_normal_builds'
assert len(pins) == n['source_files'] == 1021 and n['features'] == []
assert sha(normal_path.parent/'source-inputs.json') == n['source_manifest_sha256'] == '180741A87501C13178B8521318365810B458324663F9D419758AF983099415E9'
assert n['profiles']['Release']['sha256'] == r['exe_sha256']
verify_zip(normal_path.parent/'compiled-source.zip', n['compiled_source_sha256'], pins)
verify_zip(normal_path.parent/'normal-Release/product.zip', n['profiles']['Release']['archive']['sha256'], n['profiles']['Release']['archive']['files'])
invariance_path = ROOT/'驗證紀錄/current/native-teleport-wire/TX-retirement-20261016-172642-785425/normal-invariance.json'
invariance = read(invariance_path)
assert invariance['prior_source_manifest_sha256'] == n['source_manifest_sha256']
assert invariance['declared_source_files_plus_additional_resource'] == 1022
verify_zip(invariance_path.parent/'normal-invariance-raw.zip', invariance['raw_archive_sha256'], invariance['raw_files'])

trace = [json.loads(line) for line in (case/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
log = (case/'server.log').read_text(encoding='utf-8-sig')
middle = r['action_profile'] == 'rental-hotel-middle-expiry'
checks = []; expected_portals = []; key_ids = set(); png_paths = set()
for role, own, quantity in [('a', 1, 4 if middle else 8), ('b', 3, 2 if middle else 1)]:
    c = r['clients'][role]; h = c['hotel']; natural = h['natural_expiry']
    assert c['worker_quiescent'] and c['shutdown'] == 'closed'
    assert h['quantity'] == quantity and h['room_entry_verified'] and not h['server_authority_injection'] and not h['gui_verified']
    seconds = 7200 if quantity >= 5 else 3600
    assert natural['expected_seconds'] == seconds and not natural['clock_or_authority_injection']
    assert (seconds-2)*1000 <= natural['elapsed_ms'] <= (seconds+120)*1000
    before = h['before']; rented = h['rented']; arrival = h['arrival']; returned = natural['returned']
    w = arrival['world']; actor = w['actors'][str(own)]
    destination = next(x for x in fixture['rooms'] if x['keys_min'] <= quantity <= x['keys_max'])
    assert [w['map_id'], actor['x'], actor['y'], actor['heading']] == [destination[k] for k in ['map','x','y','heading']]
    assert set(w['actors']) == {str(own)} and arrival['joined_room']['map_id'] == w['map_id']
    assert w['generation'] == before['world']['generation']+1 and w['completed_portals'] == before['world']['completed_portals']+1
    items = before['inventory']['items']; rented_items = rented['inventory']['items']
    keys = set(rented_items)-set(items); key_ids |= {int(k) for k in keys}
    assert len(keys) == quantity and all(rented_items[k]['count'] == 1 for k in keys)
    assert {k:v for k,v in rented_items.items() if k not in keys} == items
    assert arrival['inventory']['items'] == rented_items == h['repeated']['inventory']['items']
    assert returned['inventory']['items'] == items
    rw = returned['world']; ra = rw['actors'][str(own)]
    assert [rw['map_id'],ra['x'],ra['y'],ra['heading']] == [500,32779,32832,2]
    assert rw['generation'] == before['world']['generation']+2 and rw['completed_portals'] == before['world']['completed_portals']+2
    assert rw['portal'] is None and rw['status']['object_id'] == own and '1812987995' in rw['actors']
    for state in [arrival, returned]:
        assert all(state['merchant'][k] is None for k in ['owner','npc','dialog','input','listing']) and not state['merchant']['waiting']
    authority = [v for v in trace if v['state']['object_id'] == own and v['state']['rental_town']]
    granted = next(i for i,v in enumerate(authority) if v['state']['rental_hotel'] and v['state']['rental_hotel']['keys'] == quantity and {item['id'] for item in v['state']['rental_inventory']} >= {int(k) for k in keys})
    token = authority[granted]['state']['rental_hotel']['token']
    arrived = next(i for i in range(granted,len(authority)) if authority[i]['state']['rental_map'] == w['map_id'] and authority[i]['state']['rental_hotel'] and authority[i]['state']['rental_hotel']['token'] == token)
    expired = next(i for i in range(arrived+1,len(authority)) if authority[i]['state']['rental_map'] == 500 and authority[i]['state']['rental_hotel'] is None and not ({v['id'] for v in authority[i]['state']['rental_inventory']} & {int(k) for k in keys}))
    elapsed = authority[expired]['ms']-authority[granted]['ms']
    assert (seconds-3)*1000 <= elapsed <= (seconds+120)*1000
    assert all(v['state']['rental_gems'] == 20 for v in authority[granted:expired+1])
    assert f'rental-hotel lease invalidated token={token} reason=rental-hotel-expiry keys={quantity}' in log
    captures = [(c['render'], c['end']['world']), (h['render'], rented['world']), (h['room_render'], w), (natural['render'], rw)]
    refs = []
    for capture, world in captures:
        # Native PNG writers use the Win32 extended path namespace. Compare
        # ancestry in the same namespace while retaining the original raw path.
        raw_path = capture['png']
        path = Path(raw_path[4:] if raw_path.startswith('\\\\?\\') else raw_path).resolve(strict=True)
        assert path.is_relative_to(runtime) and path.samefile(Path(raw_path))
        assert capture['frame']['world_generation'] == world['generation']
        png_paths.add(str(path)); refs.append(terrain(capture,world))
    expected_portals.append(bytes([0x5a])+b''.join(destination[k].to_bytes(2,'little') for k in ['map','x','y']))
    checks.append(dict(role=role,map=w['map_id'],keys=quantity,key_ids=sorted(keys),lease_token=token,expected_seconds=seconds,client_ms=natural['elapsed_ms'],server_ms=elapsed,granted_index=granted,arrived_index=arrived,expired_index=expired,terrain=refs))
expected_portals += [bytes([0x5a])+b''.join(v.to_bytes(2,'little') for v in [500,32779,32832])]*2
assert len(png_paths) == 8
capture_files = list((runtime/'captures').rglob('frames.jsonl')); assert len(capture_files) == 1
frames = [json.loads(line) for line in capture_files[0].read_text(encoding='utf-8-sig').splitlines()]
assert sum(v['plain'] == 'MASKED_C_LOGIN' for v in frames) == 4
packets = [(v,bytes.fromhex(v['plain'])) for v in frames if v['plain'] != 'MASKED_C_LOGIN']
actions=[]; quantities=[]; portals=[]; acks=[]; removals=[]; npc=(1812987995).to_bytes(4,'little')
for i,(row,b) in enumerate(packets):
    if row['direction'] == 'client_to_server':
        if b[0] == 0x27:
            assert b[1:5] == npc and b[-2:] == b'\0\0'; actions.append(b[5:-2].decode('ascii'))
        if b[0] == 0x51:
            assert len(b) == 11 and b[1:5] == npc and b[-2:] == b'\0\0'; quantities.append(int.from_bytes(b[5:9],'little'))
        if b == b'\x08': acks.append(i)
    else:
        if b[0] == 0x5a: portals.append((i,b))
        if b[0] == 0x17 and len(b) == 5 and int.from_bytes(b[1:5],'little') in key_ids: removals.append(int.from_bytes(b[1:5],'little'))
assert actions == ['room','room','enter']*2 and quantities == ([4,2] if middle else [8,1])
assert [b for _,b in portals] == expected_portals and len(acks) == 4
assert all(portals[i][0] < acks[i] and (i == 3 or acks[i] < portals[i+1][0]) for i in range(4))
assert len(removals) == len(key_ids) and set(removals) == key_ids

identities = read(a.client_identities)
assert identities['case'] == str(case) and identities['original_session_id'] == a.session_id
assert len(identities['clients']) == 2 and {v['role'] for v in identities['clients']} == {'a','b'}
server = read(case/'server-launch-identity.json')
processes = [(server['pid'],server['creation_utc'])]+[(v['pid'],v['creation_utc']) for v in identities['clients']]
ids = ','.join(str(pid) for pid,_ in processes)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p = subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode == 0 and not p.stderr
live = json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live,dict): live=[live]
for v in live:
    for pid,created in processes:
        if v['ProcessId'] == pid:
            assert abs((datetime.fromisoformat(v['created'])-datetime.fromisoformat(created)).total_seconds()) > 0.001

files={'independent-qualifier.py':Path(__file__),'client-identities.json':a.client_identities,'normal-qualification.json':normal_path,'normal-source-inputs.json':normal_path.parent/'source-inputs.json','normal-invariance.json':invariance_path}
for label,folder in [('case',case),('runtime',runtime)]:
    for file in folder.rglob('*'):
        if not file.is_file() or file.name == 'qualified-client.exe': continue
        assert file.resolve().is_relative_to(ROOT) and not file.is_symlink() and file.stat().st_size < 32*1024**2
        files[label+'/'+file.relative_to(folder).as_posix()]=file
hashes={name:sha(file) for name,file in files.items()}
out.mkdir(); (out/'completion-CIM.json').write_bytes(p.stdout)
archive=out/'raw.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,file in files.items(): z.write(file,name)
verify_zip(archive,sha(archive),hashes)
q=dict(status='independently_qualified_immutable_Release_real_time_hotel_expiry',case=str(case),original_session_id=a.session_id,observed_terminal_exit_code=0,checks=checks,maps=sorted({v['map'] for v in checks}),actual_formal_sessions=4,LOGIN_sessions=2,GAME_sessions=2,original_world_PNGs=8,exact_S5A=4,exact_C08=4,exact_expired_key_removals=len(removals),exe_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],resource_profile=r['resource_profile'],extended_item_details=True,normal_qualification=str(normal_path),normal_qualification_sha256=sha(normal_path),normal_source_manifest_sha256=n['source_manifest_sha256'],normal_declared_sources=1021,additional_Windows_resource_input=1,normal_invariance_qualification=str(invariance_path),normal_invariance_sha256=sha(invariance_path),all_live_development_inputs_unchanged_claimed=False,fixture_reference=fixture_proof,exact_original_owned_processes_all_terminal=True,observed_PID_reuse_candidates=live,raw_archive_sha256=sha(archive),raw_files=hashes,raw_readback_verified=True,clock_or_authority_injection=False,GUI_verified=False,physical_input_verified=False,manual_guest_exit_verified=False,complete_framebuffer_oracle=False,whole_G06_3d_complete=False,whole_goal_complete=False,qualifier_sha256=sha(Path(__file__)),qualified_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(qualification=str(out/'qualified.json'),maps=q['maps'],sessions=4,PNGs=8,raw_files=len(hashes)),ensure_ascii=False))
