"""Fresh current-source normal hotel readback; deep wire and terrain checks retained."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,struct,subprocess,time,zipfile
from native_hotel_catalog_reference import fixture_reference
from native_resident_map_reference import qualify as terrain
from native_catalog_hotel_gate import bind,case_binding,ROOT,V,read,sha
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
p=a.run.resolve(strict=True);g,normal_root,n,inputs=bind(p);normal=normal_root/'cohort.json';pins=read(normal_root/'source-inputs.json')
profiles={}
for profile in ['Debug','Release']:
    case,r,runtime=case_binding(g,profile,'rental-hotel-share');profiles[profile]=dict(case=str(case),exit_code=0,main_sha256=r['exe_sha256'])
out=p/'share-qualified';out.mkdir()
files={'normal-qualification.json':normal,'source-inputs.json':normal.parent/'source-inputs.json','independent-qualifier.py':Path(__file__)}
checks=[];pngs=0
for profile,binding in profiles.items():
    assert binding['exit_code']==0
    case=Path(binding['case']).resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
    r=read(case/'receipt.json');runtime=Path(r['runtime']).resolve(strict=True);assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current')
    assert r['exe_sha256']==binding['main_sha256']==n['profiles'][profile]['normal']['sha256']==sha(Path(n['profiles'][profile]['normal']['path']))==sha(runtime/'qualified-client.exe')
    assert r['jar_sha256']==g['reference_jar_sha256'] and r['source_matches_snapshot']
    assert r['action_profile']=='rental-hotel-share' and r['resource_profile']=='tw13081901-v1' and r['extended_item_details']
    s=r['server'];assert s['actual_client_sessions']==4 and s['game_sessions']==2 and s['memberships_empty']
    assert s['authority_mutations']==s['active_hotel_leases_after_disconnect']==s['classic_world_members_after_disconnect']==0 and not s['sql_access'] and not s['state_persistent']
    marker=read(case/'validation-completed.json');assert marker['receipt_sha256']==sha(case/'receipt.json')
    assert set(marker['comparison_pins'])=={'rental-entry-comparison.json','rental-hotel-share-comparison.json','attributes-comparison.json'}
    for name,h in marker['comparison_pins'].items():assert sha(case/name)==h and read(case/name)['receipt_sha256'].upper()==sha(case/'receipt.json')
    owner=r['clients']['a']['hotel_share'];guest=r['clients']['b']['hotel_share'];transfer=read(runtime/'hotel-key-dropped.json')
    before_a=owner['before']['inventory']['items'];before_b=guest['before']['inventory']['items'];rented=owner['rented']['inventory']['items']
    keys=set(rented)-set(before_a);assert len(keys)==2 and {int(k) for k in keys}==set(transfer['keys'])
    transferred=keys-set(owner['dropped']['inventory']['items']);assert transferred=={str(transfer['inventory_key'])}
    assert set(guest['picked_up']['inventory']['items'])-set(before_b)==transferred
    assert owner['returned']['inventory']['items']==before_a and guest['returned']['inventory']['items']==before_b
    assert owner['returned']['world']['generation']==owner['before']['world']['generation'] and owner['returned']['world']['map_id']==500
    assert guest['entered']['world']['map_id']==502 and set(guest['entered']['world']['actors'])=={'3'}
    w=guest['returned']['world'];assert w['map_id']==500 and w['generation']==guest['before']['world']['generation']+2
    assert w['completed_portals']==guest['before']['world']['completed_portals']+2 and [w['actors']['3'][k] for k in ['x','y','heading']]==[32779,32832,2]
    assert '1' in w['actors'] and '1812987995' in w['actors']
    assert all(guest['returned']['merchant'][key] is None for key in ['owner','npc','dialog','input','listing'])
    ground_id=str(transfer['ground_id']);ground=owner['dropped']['world']['actors'][ground_id]
    assert ground_id not in guest['picked_up']['world']['actors'] and ground_id not in w['actors']
    item_rows=[x for x in guest['pickup_render']['frame']['actors'] if x['id']==int(ground_id)]
    assert len(item_rows)==1 and item_rows[0]['rendered'] and item_rows[0]['phase']=='ground_item'
    trace=[json.loads(line)['state'] for line in (case/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
    tokens=[];authority_checks=[];render_checks=[]
    for role,own,h,before,stages in [('a',1,owner,before_a,['rented','dropped','returned']),('b',3,guest,before_b,['picked_up','entered','returned'])]:
        c=r['clients'][role];assert c['worker_quiescent'] and c['shutdown']=='closed' and not h['server_authority_injection'] and not h['owner_restart_requested']
        states=[x for x in trace if x['object_id']==own and x['rental_town']];cursor=0
        for step in h['steps']:cursor=next(i for i in range(cursor,len(states)) if [states[i]['x'],states[i]['y']]==step['after'])
        rental_base={v['id']:v['count'] for v in states[cursor]['rental_inventory']};assert rental_base
        for stage in stages:
            snapshot=h[stage];items=snapshot['inventory']['items'];actor=snapshot['world']['actors'][str(own)]
            assert {k:v for k,v in items.items() if k not in keys}==before
            expected=dict(rental_base);expected.update({int(k):items[k]['count'] for k in keys if k in items})
            cursor=next(i for i in range(cursor,len(states)) if states[i]['rental_map']==snapshot['world']['map_id'] and [states[i]['x'],states[i]['y']]==[actor['x'],actor['y']] and {v['id']:v['count'] for v in states[i]['rental_inventory']}==expected)
            state=states[cursor];assert state['rental_gems']==20
            if stage=='returned':assert state['rental_hotel'] is None
            else:assert state['rental_hotel']['map']==502 and state['rental_hotel']['keys']==2;tokens.append(state['rental_hotel']['token'])
            authority_checks.append(dict(role=role,stage=stage,index=cursor))
        render_checks.append(terrain(c['render'],c['end']['world']));pngs+=1
    for capture,world in [(owner['drop_render'],owner['dropped']['world']),(guest['pickup_render'],guest['picked_up']['world']),(guest['entry_render'],guest['entered']['world']),(guest['return_render'],w)]:render_checks.append(terrain(capture,world));pngs+=1
    assert len(set(tokens))==1 and tokens[0]>0
    capture_files=list((runtime/'captures').rglob('frames.jsonl'));assert len(capture_files)==1
    frames=[json.loads(line) for f in capture_files for line in f.read_text(encoding='utf-8-sig').splitlines()];assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==4
    packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
    drops=[];pickups=[];actions=[];quantities=[];portals=[];acks=[];moves=0;npc=(1812987995).to_bytes(4,'little')
    for i,(row,b) in enumerate(packets):
        if row['direction']=='client_to_server':
            if b[0]==0x0c:drops.append((i,b))
            if b[0]==0x0b:pickups.append((i,b))
            if b[0]==0x27:assert b[1:5]==npc and b[-2:]==b'\0\0';actions.append(b[5:-2].decode('ascii'))
            if b[0]==0x51:assert len(b)==11 and b[1:5]==npc and b[-2:]==b'\0\0';quantities.append(int.from_bytes(b[5:9],'little'))
            if b==b'\x08':acks.append(i)
            if b[0]==0x0a:moves+=1
        elif b[0]==0x5a:portals.append((i,b))
    actor=owner['dropped']['world']['actors']['1']
    expected_drop=b'\x0c'+actor['x'].to_bytes(2,'little')+actor['y'].to_bytes(2,'little')+transfer['inventory_key'].to_bytes(4,'little')+(1).to_bytes(4,'little')
    wire_x=32768+(ground['x']-32768)//2
    expected_pickup=b'\x0b'+wire_x.to_bytes(2,'little')+ground['y'].to_bytes(2,'little')+transfer['ground_id'].to_bytes(4,'little')+(1).to_bytes(4,'little')+b'\0'
    assert len(drops)==len(pickups)==1 and drops[0][1]==expected_drop and pickups[0][1]==expected_pickup
    assert drops[0][0]<pickups[0][0]<portals[0][0]<acks[0]<portals[1][0]<acks[1]
    assert [b.hex().upper() for _,b in portals]==['5AF60120801980','5AF4010B804080'] and len(acks)==2
    assert actions==['room','enter','return'] and quantities==[2] and moves==len(owner['steps'])+len(guest['steps'])
    checks.append(dict(profile=profile,case=str(case),receipt_sha256=sha(case/'receipt.json'),main_sha256=r['exe_sha256'],key_ids=sorted(keys),transferred_key=transfer['inventory_key'],ground_id=transfer['ground_id'],shared_lease_token=tokens[0],authority=authority_checks,terrain=render_checks,exact_C_MOVE=moves,exact_C_DROP=1,exact_C_PICKUP=1,exact_S5A=2,exact_C08=2,pickup_wire_x=wire_x,ground_world_x=ground['x']))
    for kind,folder in [('case',case),('runtime',runtime)]:
        for f in folder.rglob('*'):
            if not f.is_file() or f.name=='qualified-client.exe':continue
            assert f.resolve().is_relative_to(ROOT) and not f.is_symlink() and f.stat().st_size<16*1024**2
            files[profile+'/'+kind+'/'+str(f.relative_to(folder)).replace('\\','/')]=f
for f in p.iterdir():
    if f.is_file() and f.suffix.lower()!='.zip':files['cohort/'+f.name]=f
assert pngs==12
hashes={name:sha(file) for name,file in files.items()};archive=out/'raw-evidence.zip';assert not archive.exists() and not (out/'qualified.json').exists()
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,file in files.items():z.write(file,name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(hashes)
    for name,h in hashes.items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
for name,h in pins.items():assert sha(ROOT/name)==h,name
q=dict(status='independently_qualified_source1047_shared_hotel_keys_and_owner_forced_guest_return',source_files=1047,source_manifest_sha256=n['source_manifest_sha256'],normal_qualification=str(normal),profiles=checks,
    actual_formal_sessions=8,LOGIN_sessions=4,GAME_sessions=4,original_world_PNGs=12,exact_C_DROP=2,exact_C_PICKUP=2,exact_S5A=4,exact_C08=4,resource_profile='tw13081901-v1',extended_item_details=True,
    raw_archive_sha256=sha(archive),raw_archive_files=hashes,raw_readback_verified=True,GUI_verified=False,natural_expiry_verified=False,manual_guest_exit_verified=False,complete_framebuffer_oracle=False,whole_G06_3d_complete=False,whole_goal_complete=False,qualifier_sha256=sha(Path(__file__)),qualified_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(out/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(status=q['status'],sessions=8,PNGs=12,raw_files=len(hashes)),ensure_ascii=False))
