"""Bind both normal hotel key/return runs to wire, authority, S32 and raw bytes."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,struct,zipfile
from native_resident_map_reference import qualify as terrain

ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('cohort',type=Path);a=ap.parse_args()
p=a.cohort.resolve(strict=True);assert p.is_relative_to(ROOT/'驗證紀錄/current/native-hotel-current')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
g=read(p/'receipt.json');assert g['status']=='executed_source1021_both_profiles_hotel_keys_and_return' and not g['source_and_corresponding_QA_frozen']
normal=Path(g['normal_qualification']);n=read(normal);pins=read(normal.parent/'source-inputs.json')
assert sha(normal)==g['normal_qualification_sha256'] and len(pins)==n['source_files']==g['source_files']==1021
assert n['source_manifest_sha256']==g['source_manifest_sha256']==sha(normal.parent/'source-inputs.json')
assert n['status']=='independently_qualified_green_clock_related_scope_and_unique_entry_normal_builds'
for name,h in pins.items():assert sha(ROOT/name)==h,name
for name,h in g['validation_pins'].items():assert sha(ROOT/'維護工具'/name)==sha(p/name)==h,name
assert set(g['profiles'])=={'Debug','Release'} and g['resource_profile']=='tw13081901-v1' and g['extended_item_details']
files={};cases={};total_sessions=0;pngs=0
for profile,case_binding in g['profiles'].items():
    assert case_binding['exit_code']==0
    case=Path(case_binding['case']).resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
    r=read(case/'receipt.json');runtime=Path(r['runtime']).resolve(strict=True);assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current')
    assert r['exe_sha256']==case_binding['main_sha256']==n['profiles'][profile]['sha256']==sha(Path(n['profiles'][profile]['path']))==sha(runtime/'qualified-client.exe')
    assert r['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539' and r['source_matches_snapshot']
    assert r['action_profile']=='rental-hotel' and r['resource_profile']=='tw13081901-v1' and r['extended_item_details']
    server=r['server'];assert server['actual_client_sessions']==4 and server['game_sessions']==2 and server['memberships_empty']
    assert server['classic_world_members_after_disconnect']==server['authority_mutations']==server['active_hotel_leases_after_disconnect']==0
    assert not server['sql_access'] and not server['state_persistent'];total_sessions+=4
    marker=read(case/'validation-completed.json');assert marker['receipt_sha256']==sha(case/'receipt.json')
    for name,h in marker['comparison_pins'].items():
        assert sha(case/name)==h and read(case/name)['receipt_sha256'].upper()==sha(case/'receipt.json')
    assert set(marker['comparison_pins'])=={'rental-entry-comparison.json','rental-hotel-comparison.json','attributes-comparison.json'}
    trace=[json.loads(line)['state'] for line in (case/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
    checks=[]
    for role,own,quantity in [('a',1,8),('b',3,1)]:
        c=r['clients'][role];h=c['hotel'];assert c['worker_quiescent'] and c['shutdown']=='closed'
        assert h['quantity']==quantity and not h['server_authority_injection'] and not h['gui_verified'] and not h['room_entry_verified']
        before=h['before']['inventory']['items'];rented=h['rented']['inventory']['items'];returned=h['returned']['inventory']['items']
        keys=set(rented)-set(before);assert len(keys)==quantity and returned==before and h['repeated']['inventory']['items']==rented
        assert {k:v for k,v in rented.items() if k not in keys}==before and all(rented[k]['count']==1 for k in keys)
        states=[s for s in trace if s['object_id']==own and s['rental_town']]
        granted=next(i for i,s in enumerate(states) if s['rental_hotel'] and s['rental_hotel']['keys']==quantity and {v['id'] for v in s['rental_inventory']} >= {int(k) for k in keys})
        released=next(i for i in range(granted+1,len(states)) if not states[i]['rental_hotel'] and not ({v['id'] for v in states[i]['rental_inventory']} & {int(k) for k in keys}))
        assert all(s['rental_gems']==20 for s in states[granted:released+1])
        refs=[terrain(c['render'],c['end']['world']),terrain(h['render'],h['rented']['world'])];pngs+=len(refs)
        checks.append(dict(role=role,quantity=quantity,key_ids=sorted(keys),authority_granted_index=granted,authority_released_index=released,terrain=refs))
    capture_files=sorted((runtime/'captures').rglob('frames.jsonl'));assert len(capture_files)==1
    frames=[json.loads(line) for f in capture_files for line in f.read_text(encoding='utf-8-sig').splitlines()]
    assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==4
    packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
    npc=1812987995;talks=[];actions=[];quantities=[];requests=[]
    for row,b in packets:
        if row['direction']=='client_to_server':
            if b[0]==0x29:assert b==bytes([0x29])+npc.to_bytes(4,'little');talks.append(b)
            if b[0]==0x27:
                assert b[1:5]==npc.to_bytes(4,'little') and b[-2:]==b'\0\0';actions.append(b[5:-2].decode('ascii'))
            if b[0]==0x51:
                assert len(b)==11 and b[1:5]==npc.to_bytes(4,'little') and b[-2:]==b'\0\0';quantities.append(int.from_bytes(b[5:9],'little'))
        elif b[0]==0x5b:
            assert len(b)>22 and struct.unpack_from('<5I',b,1)==(npc,0,1,1,8) and b[21]==0
            requests.append(b)
    assert len(talks)==len(requests)==2 and actions==['room','room','return']*2 and quantities==[8,1]
    cases[profile]=dict(case=str(case),receipt_sha256=sha(case/'receipt.json'),main_sha256=r['exe_sha256'],actual_sessions=4,
        quantity_requests=quantities,actions=actions,invalid_or_repeated_quantity_packets=0,checks=checks)
    for prefix,folder in [('case',case),('runtime',runtime)]:
        for f in folder.rglob('*'):
            if not f.is_file() or f.name=='qualified-client.exe':continue
            assert f.resolve().is_relative_to(ROOT) and not f.is_symlink() and f.stat().st_size<16*1024**2
            files[profile+'/'+prefix+'/'+str(f.relative_to(folder)).replace('\\','/')]=f
for file in p.iterdir():
    if file.is_file():files['cohort/'+file.name]=file
files['normal-qualification.json']=normal;files['source-inputs.json']=normal.parent/'source-inputs.json';files['independent-qualifier.py']=Path(__file__)
archive=p/'raw-evidence.zip';assert not archive.exists() and not (p/'qualified.json').exists()
hashes={name:sha(file) for name,file in files.items()}
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,file in files.items():z.write(file,name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(hashes)
    for name,h in hashes.items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
for name,h in pins.items():assert sha(ROOT/name)==h,name
q=dict(status='independently_qualified_source1021_hotel_keys_return_both_profiles',source_files=1021,source_manifest_sha256=g['source_manifest_sha256'],
    normal_qualification=str(normal),normal_qualification_sha256=sha(normal),profiles=cases,actual_formal_sessions=total_sessions,
    LOGIN_sessions=4,GAME_sessions=4,original_world_PNGs=pngs,resource_profile='tw13081901-v1',extended_item_details=True,
    raw_archive_sha256=sha(archive),raw_archive_files=hashes,raw_readback_verified=True,qualifier_sha256=sha(Path(__file__)),
    room_entry_verified=False,shared_keys_verified=False,natural_expiry_verified=False,GUI_verified=False,physical_input_verified=False,
    complete_framebuffer_oracle=False,whole_G06_3d_complete=False,whole_goal_complete=False,qualified_utc=datetime.now(timezone.utc).isoformat())
(p/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(p/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(status=q['status'],sessions=total_sessions,world_PNGs=pngs,raw_files=len(hashes)),ensure_ascii=False))
