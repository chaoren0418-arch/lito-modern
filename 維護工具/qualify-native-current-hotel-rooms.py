"""Read back all three hotel room destinations on both source1021 normal mains."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
from native_resident_map_reference import qualify as terrain
from native_hotel_catalog_reference import fixture_reference

ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('entry',type=Path);ap.add_argument('middle',type=Path);a=ap.parse_args()
cohorts=[a.entry.resolve(strict=True),a.middle.resolve(strict=True)]
for p in cohorts:assert p.is_relative_to(ROOT/'驗證紀錄/current/native-hotel-current')
groups=[read(p/'receipt.json') for p in cohorts]
assert [g['action_profile'] for g in groups]==['rental-hotel-entry','rental-hotel-middle-entry']
normal=Path(groups[0]['normal_qualification']);n=read(normal);pins=read(normal.parent/'source-inputs.json')
assert n['status']=='independently_qualified_green_clock_related_scope_and_unique_entry_normal_builds' and n['features']==[]
assert len(pins)==n['source_files']==1021
assert n['source_manifest_sha256']==sha(normal.parent/'source-inputs.json')=='180741A87501C13178B8521318365810B458324663F9D419758AF983099415E9'
for name,h in pins.items():assert sha(ROOT/name)==h,name
fixture_path=ROOT/'LiTo381-Modern/tests/fixtures/native-portal-current.json';fixture=read(fixture_path)
fixture_proof=fixture_reference(fixture_path,'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539')
files={'normal-qualification.json':normal,'source-inputs.json':normal.parent/'source-inputs.json','independent-qualifier.py':Path(__file__),'original-BBC-portal-fixture.json':fixture_path}
checks=[];maps=set();sessions=0;pngs=0
for p,g in zip(cohorts,groups):
    assert g['status']=='executed_source1021_both_profiles_hotel_room_entry' and not g['source_and_corresponding_QA_frozen']
    assert g['normal_qualification']==str(normal) and sha(normal)==g['normal_qualification_sha256']
    assert g['source_manifest_sha256']==n['source_manifest_sha256'] and g['source_files']==1021
    assert g['resource_profile']=='tw13081901-v1' and g['extended_item_details'] and set(g['profiles'])=={'Debug','Release'}
    for name,h in g['validation_pins'].items():assert sha(ROOT/'維護工具'/name)==sha(p/name)==h,name
    middle=g['action_profile']=='rental-hotel-middle-entry';label='middle' if middle else 'outer'
    for profile,binding in g['profiles'].items():
        case=Path(binding['case']).resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
        r=read(case/'receipt.json');runtime=Path(r['runtime']).resolve(strict=True);assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current')
        assert binding['exit_code']==0 and binding['main_sha256']==r['exe_sha256']==n['profiles'][profile]['sha256']
        assert sha(Path(n['profiles'][profile]['path']))==sha(runtime/'qualified-client.exe')==r['exe_sha256']
        assert r['jar_sha256']==fixture_proof['actual_reference_sha256'] and r['source_matches_snapshot']
        assert r['action_profile']==g['action_profile'] and r['resource_profile']==g['resource_profile'] and r['extended_item_details']
        s=r['server'];assert s['actual_client_sessions']==4 and s['game_sessions']==2 and s['memberships_empty']
        assert s['authority_mutations']==s['active_hotel_leases_after_disconnect']==s['classic_world_members_after_disconnect']==0
        assert not s['sql_access'] and not s['state_persistent'];sessions+=4
        marker=read(case/'validation-completed.json');assert marker['receipt_sha256']==sha(case/'receipt.json')
        assert set(marker['comparison_pins'])=={'rental-entry-comparison.json','rental-hotel-entry-comparison.json','attributes-comparison.json'}
        for name,h in marker['comparison_pins'].items():assert sha(case/name)==h and read(case/name)['receipt_sha256'].upper()==sha(case/'receipt.json')
        trace=[json.loads(line)['state'] for line in (case/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
        arrivals=[];expected_portals=[]
        for role,own,quantity in [('a',1,4 if middle else 8),('b',3,2 if middle else 1)]:
            c=r['clients'][role];h=c['hotel'];assert c['worker_quiescent'] and c['shutdown']=='closed'
            assert h['quantity']==quantity and h['room_entry_verified'] and not h['server_authority_injection'] and not h['gui_verified']
            before=h['before'];rented=h['rented'];arrival=h['arrival'];w=arrival['world'];actor=w['actors'][str(own)]
            destination=next(x for x in fixture['rooms'] if x['keys_min']<=quantity<=x['keys_max'])
            assert [w['map_id'],actor['x'],actor['y'],actor['heading']]==[destination[k] for k in ['map','x','y','heading']]
            assert w['generation']==before['world']['generation']+1 and w['completed_portals']==before['world']['completed_portals']+1
            assert set(w['actors'])=={str(own)} and arrival['joined_room']['map_id']==w['map_id']
            assert arrival['inventory']['items']==rented['inventory']['items']==h['repeated']['inventory']['items']
            keys=set(rented['inventory']['items'])-set(before['inventory']['items']);assert len(keys)==quantity
            assert {k:v for k,v in rented['inventory']['items'].items() if k not in keys}==before['inventory']['items']
            assert all(rented['inventory']['items'][k]['count']==1 for k in keys)
            assert all(arrival['merchant'][key] is None for key in ['owner','npc','dialog','input','listing']) and not arrival['merchant']['waiting']
            authority=[x for x in trace if x['object_id']==own and x['rental_town']]
            granted=next(i for i,x in enumerate(authority) if x['rental_hotel'] and x['rental_hotel']['keys']==quantity and {v['id'] for v in x['rental_inventory']} >= {int(k) for k in keys})
            token=authority[granted]['rental_hotel']['token']
            arrived=next(i for i in range(granted,len(authority)) if [authority[i]['rental_map'],authority[i]['x'],authority[i]['y']]==[w['map_id'],actor['x'],actor['y']] and authority[i]['rental_hotel'] and authority[i]['rental_hotel']['token']==token)
            assert all(x['rental_gems']==20 for x in authority[granted:arrived+1])
            refs=[terrain(c['render'],c['end']['world']),terrain(h['render'],rented['world']),terrain(h['room_render'],w)]
            assert h['room_render']['frame']['world_generation']==w['generation'];pngs+=3;maps.add(w['map_id'])
            portal=bytes([0x5a])+destination['map'].to_bytes(2,'little')+destination['x'].to_bytes(2,'little')+destination['y'].to_bytes(2,'little')
            assert portal.hex().upper()==destination['portal'].upper();expected_portals.append(portal)
            arrivals.append(dict(role=role,quantity=quantity,map=w['map_id'],generation=w['generation'],key_ids=sorted(keys),lease_token=token,authority_granted_index=granted,authority_arrived_index=arrived,terrain=refs))
        capture_files=list((runtime/'captures').rglob('frames.jsonl'));assert len(capture_files)==1
        frames=[json.loads(line) for file in capture_files for line in file.read_text(encoding='utf-8-sig').splitlines()]
        assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==4
        packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
        npc=(1812987995).to_bytes(4,'little');actions=[];quantities=[];portals=[];acks=[]
        for i,(row,b) in enumerate(packets):
            if row['direction']=='client_to_server':
                if b[0]==0x27:assert b[1:5]==npc and b[-2:]==b'\0\0';actions.append(b[5:-2].decode('ascii'))
                if b[0]==0x51:assert len(b)==11 and b[1:5]==npc and b[-2:]==b'\0\0';quantities.append(int.from_bytes(b[5:9],'little'))
                if b==b'\x08':acks.append(i)
            elif b[0]==0x5a:portals.append((i,b))
        assert quantities==([4,2] if middle else [8,1]) and actions==['room','room','enter']*2
        assert [b for _,b in portals]==expected_portals and len(acks)==2 and portals[0][0]<acks[0]<portals[1][0]<acks[1]
        checks.append(dict(scenario=label,profile=profile,case=str(case),receipt_sha256=sha(case/'receipt.json'),main_sha256=r['exe_sha256'],actual_formal_sessions=4,arrivals=arrivals,exact_C08=2,exact_S5A=[b.hex().upper() for _,b in portals]))
        for kind,folder in [('case',case),('runtime',runtime)]:
            for f in folder.rglob('*'):
                if not f.is_file() or f.name=='qualified-client.exe':continue
                assert f.resolve().is_relative_to(ROOT) and not f.is_symlink() and f.stat().st_size<16*1024**2
                files[label+'/'+profile+'/'+kind+'/'+str(f.relative_to(folder)).replace('\\','/')]=f
    for f in p.iterdir():
        if f.is_file() and f.suffix.lower() not in ['.zip']:files[label+'/cohort/'+f.name]=f
assert maps=={501,502,503} and sessions==16 and pngs==24
out=cohorts[1]/'all-rooms-qualified';out.mkdir(exist_ok=False)
hashes={name:sha(file) for name,file in files.items()};archive=out/'raw-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,file in files.items():z.write(file,name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(hashes)
    for name,h in hashes.items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
for name,h in pins.items():assert sha(ROOT/name)==h,name
q=dict(status='independently_qualified_source1021_all_three_hotel_rooms_both_profiles',source_files=1021,source_manifest_sha256=n['source_manifest_sha256'],
    normal_qualification=str(normal),normal_qualification_sha256=sha(normal),cohorts=[str(p) for p in cohorts],checks=checks,fixture_reference=fixture_proof,
    maps=sorted(maps),actual_formal_sessions=16,LOGIN_sessions=8,GAME_sessions=8,original_world_PNGs=24,exact_S5A=8,exact_C08=8,
    resource_profile='tw13081901-v1',extended_item_details=True,raw_archive_sha256=sha(archive),raw_archive_files=hashes,raw_readback_verified=True,
    GUI_verified=False,shared_keys_verified=False,natural_expiry_verified=False,manual_exit_verified=False,complete_framebuffer_oracle=False,
    whole_G06_3d_complete=False,whole_goal_complete=False,qualifier_sha256=sha(Path(__file__)),qualified_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(out/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(qualification=str(out/'qualified.json'),maps=q['maps'],sessions=16,PNGs=24,raw_files=len(hashes)),ensure_ascii=False))
