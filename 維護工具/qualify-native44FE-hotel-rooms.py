"""Independently read current hotel authority, exact wire, terrain and owned-process closure."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,time,zipfile
from native_hotel_catalog_reference import fixture_reference
from native_resident_map_reference import qualify as terrain

ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',required=True,type=int);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native44FE-hotel')
g=read(run/'cohort.json');assert g['status']=='executed_current_normal_hotel_room_entry_independent_cohort_readback_pending' and g['owned_process'] is None
assert len(g['cases'])==4 and all(p['exit_code']==0 for p in g['process_history'])
pins=read(run/'source-inputs.json');assert len(pins)==1040
assert g['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
for n,h in pins.items():assert sha(ROOT/n)==h,n
for n,h in g['validation_pins'].items():assert sha(n)==h,n
fixture_path=ROOT/'LiTo381-Modern/tests/fixtures/native-portal-current.json';fixture=read(fixture_path)
fixture_proof=fixture_reference(fixture_path,g['reference_jar_sha256'])
files={'cohort.json':run/'cohort.json','source-inputs.json':run/'source-inputs.json','qualifier.py':Path(__file__),'fixture.json':fixture_path}
for p in run.iterdir():
    if p.is_file():files['driver/'+p.name]=p
checks=[];maps=set();sessions=0;pngs=0
identities=[g['runner_identity'],*g.get('previous_runner_identities',[]),*g['process_history']]
for profile in ['Debug','Release']:
    for action,label in [('rental-hotel-entry','outer'),('rental-hotel-middle-entry','middle')]:
        binding=g['cases'][profile+'-'+action];case=Path(binding['directory']).resolve(strict=True)
        assert case.is_relative_to(V/'native-real-peers') and binding['PowerShell_exit_code']==0
        r=read(case/'receipt.json');runtime=Path(r['runtime']).resolve(strict=True)
        assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current/native-real-peers')
        assert sha(case/'receipt.json')==binding['receipt_sha256'] and sha(case/'rental-hotel-entry-comparison.json')==binding['comparison_sha256']
        normal=ROOT/('LiTo381-Modern/target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/LiToModern.exe')
        assert sha(normal)==sha(runtime/'qualified-client.exe')==r['exe_sha256']==g['normal_mains'][profile]
        assert r['jar_sha256']==fixture_proof['actual_reference_sha256'] and r['source_matches_snapshot']
        assert r['action_profile']==action and r['resource_profile']=='tw13081901-v1' and r['extended_item_details']
        for n,h in read(case/'source-pins.json').items():assert sha(ROOT/'LiTo381-Modern'/n)==h,n
        s=r['server'];assert s['actual_client_sessions']==4 and s['game_sessions']==2 and s['memberships_empty']
        assert s['authority_mutations']==s['active_hotel_leases_after_disconnect']==s['classic_world_members_after_disconnect']==0
        assert not s['sql_access'] and not s['state_persistent'];sessions+=4
        marker=read(case/'validation-completed.json');assert marker['receipt_sha256']==sha(case/'receipt.json')
        assert set(marker['comparison_pins'])=={'rental-entry-comparison.json','rental-hotel-entry-comparison.json','attributes-comparison.json'}
        for n,h in marker['comparison_pins'].items():assert sha(case/n)==h and read(case/n)['receipt_sha256'].upper()==sha(case/'receipt.json')
        for role in ['server','a','b']:
            identity=read(case/(role+'-launch-identity.json'));assert identity['exe_sha256']==g['normal_mains'][profile]
            identities.append(identity)
        trace=[json.loads(line)['state']for line in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
        arrivals=[];expected_portals=[];middle=label=='middle'
        for role,own,quantity in [('a',1,4 if middle else 8),('b',3,2 if middle else 1)]:
            c=r['clients'][role];h=c['hotel'];assert c['worker_quiescent'] and c['shutdown']=='closed'
            assert h['quantity']==quantity and h['room_entry_verified'] and not h['server_authority_injection'] and not h['gui_verified']
            before=h['before'];rented=h['rented'];arrival=h['arrival'];w=arrival['world'];actor=w['actors'][str(own)]
            destination=next(x for x in fixture['rooms']if x['keys_min']<=quantity<=x['keys_max'])
            assert [w['map_id'],actor['x'],actor['y'],actor['heading']]==[destination[k]for k in ['map','x','y','heading']]
            assert w['generation']==before['world']['generation']+1 and w['completed_portals']==before['world']['completed_portals']+1
            assert set(w['actors'])=={str(own)} and arrival['joined_room']['map_id']==w['map_id']
            assert arrival['inventory']['items']==rented['inventory']['items']==h['repeated']['inventory']['items']
            keys=set(rented['inventory']['items'])-set(before['inventory']['items']);assert len(keys)==quantity
            assert {k:v for k,v in rented['inventory']['items'].items()if k not in keys}==before['inventory']['items']
            assert all(rented['inventory']['items'][k]['count']==1 for k in keys)
            assert all(arrival['merchant'][k]is None for k in ['owner','npc','dialog','input','listing']) and not arrival['merchant']['waiting']
            authority=[x for x in trace if x['object_id']==own and x['rental_town']]
            granted=next(i for i,x in enumerate(authority)if x['rental_hotel']and x['rental_hotel']['keys']==quantity and {v['id']for v in x['rental_inventory']}>={int(k)for k in keys})
            token=authority[granted]['rental_hotel']['token']
            arrived=next(i for i in range(granted,len(authority))if [authority[i]['rental_map'],authority[i]['x'],authority[i]['y']]==[w['map_id'],actor['x'],actor['y']]and authority[i]['rental_hotel']and authority[i]['rental_hotel']['token']==token)
            assert all(x['rental_gems']==20 for x in authority[granted:arrived+1])
            refs=[terrain(c['render'],c['end']['world']),terrain(h['render'],rented['world']),terrain(h['room_render'],w)]
            assert h['room_render']['frame']['world_generation']==w['generation'];pngs+=3;maps.add(w['map_id'])
            portal=bytes([0x5a])+destination['map'].to_bytes(2,'little')+destination['x'].to_bytes(2,'little')+destination['y'].to_bytes(2,'little')
            assert portal.hex().upper()==destination['portal'].upper();expected_portals.append(portal)
            arrivals.append(dict(role=role,quantity=quantity,map=w['map_id'],generation=w['generation'],key_ids=sorted(keys),lease_token=token,authority_granted_index=granted,authority_arrived_index=arrived,terrain=refs,room_png=h['room_render']['png']))
        capture_files=list((runtime/'captures').glob('*/frames.jsonl'));assert len(capture_files)==1
        frames=[json.loads(line)for line in capture_files[0].read_text('utf-8-sig').splitlines()]
        assert sum(x['plain']=='MASKED_C_LOGIN'for x in frames)==4
        packets=[(x,bytes.fromhex(x['plain']))for x in frames if x['plain']!='MASKED_C_LOGIN']
        npc=(1812987995).to_bytes(4,'little');actions=[];quantities=[];portals=[];acks=[]
        for i,(row,b)in enumerate(packets):
            if row['direction']=='client_to_server':
                if b[0]==0x27:assert b[1:5]==npc and b[-2:]==b'\0\0';actions.append(b[5:-2].decode('ascii'))
                if b[0]==0x51:assert len(b)==11 and b[1:5]==npc and b[-2:]==b'\0\0';quantities.append(int.from_bytes(b[5:9],'little'))
                if b==b'\x08':acks.append(i)
            elif b[0]==0x5a:portals.append((i,b))
        assert quantities==([4,2]if middle else [8,1])and actions==['room','room','enter']*2
        assert [b for _,b in portals]==expected_portals and len(acks)==2 and portals[0][0]<acks[0]<portals[1][0]<acks[1]
        checks.append(dict(scenario=label,profile=profile,case=str(case),receipt_sha256=sha(case/'receipt.json'),normal_sha256=r['exe_sha256'],actual_sessions=4,arrivals=arrivals,exact_C08=2,exact_S5A=[b.hex().upper()for _,b in portals]))
        for kind,folder in [('case',case),('runtime',runtime)]:
            for f in folder.rglob('*'):
                if not f.is_file()or f.name=='qualified-client.exe':continue
                assert f.resolve().is_relative_to(ROOT)and not f.is_symlink()and f.stat().st_size<16*1024**2
                files[label+'/'+profile+'/'+kind+'/'+f.relative_to(folder).as_posix()]=f
assert maps=={501,502,503}and sessions==16 and pngs==24
ids=sorted({int(x['pid'])for x in identities});pid_filter=' OR '.join('ProcessId='+str(pid)for pid in ids)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+pid_filter+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert result.returncode==0 and not result.stderr
live=json.loads(result.stdout.decode('utf-8-sig'))if result.stdout.strip()else []
if isinstance(live,dict):live=[live]
for row in live:
    for item in identities:
        if row['ProcessId']==item['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(item['creation_utc'])).total_seconds())>.001
out=run/'qualified';out.mkdir();(out/'process-CIM.json').write_bytes(result.stdout)
hashes={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    assert set(z.namelist())==set(hashes)
    for n,h in hashes.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
q=dict(status='independently_qualified_current_normal_all_three_hotel_rooms_both_profiles',source_manifest_sha256=g['source_manifest_sha256'],reference_jar_sha256=g['reference_jar_sha256'],normal_mains=g['normal_mains'],
    original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,cohort=str(run),cases=checks,maps=sorted(maps),actual_cases=4,actual_sessions=16,
    original_world_PNGs=24,exact_S5A=8,exact_C08=8,fixture_reference=fixture_proof,recorded_owned_identities=len(identities),recorded_owned_all_terminal=True,PID_reuse_candidates=live,
    retained_without_rerun=1 if g.get('previous_runner_identities')else 0,initial_reader_failure=g.get('initial_reader_failure_terminal'),
    normal_source_modified=False,launcher_artifact_modified=False,raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'),
    GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,shared_keys_verified=False,natural_expiry_verified=False,manual_exit_verified=False,
    full_G06_verified=False,whole_goal_complete=False,qualified_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=16,maps=sorted(maps),recorded_terminal=len(identities)),ensure_ascii=False))
