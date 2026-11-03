"""Read actual current75 same-peer catalog rebind, scoped inventories and movement."""
from pathlib import Path
from datetime import datetime,timezone
from collections import Counter
from PIL import Image
import argparse,hashlib,itertools,json,re,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
SCOPE='same_endpoint_LOGIN_catalog_rebound_rental_service';SERVICE_IDS={'1811939329','1811939330','1811939331'}
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def inventory(body):
    assert body[0]==0x7c;at=2;records={}
    def take(n):
        nonlocal at
        assert 0<=n<=len(body)-at;x=body[at:at+n];at+=n;return x
    def number(n):return int.from_bytes(take(n),'little')
    for _ in range(body[1]):
        identity=number(4);assert identity>0 and str(identity)not in records
        use_type,charge,icon,blessing,count,identified=[number(n)for n in [1,1,2,1,4,1]];assert count>0 and blessing<=3 and identified in [0,1]
        end=body.find(b'\0',at,at+256);assert end>=at;name=take(end-at).decode('cp950');assert take(1)==b'\0';detail=list(take(number(1)))
        records[str(identity)]=dict(id=identity,use_type=use_type,charge=charge,icon=icon,blessing=blessing,count=count,identified=bool(identified),name=name,detail=detail)
    assert at==len(body);return records
def directory(body):
    assert body[0]==0xfb;at=1
    def take(n):
        nonlocal at
        assert 0<=n<=len(body)-at;x=body[at:at+n];at+=n;return x
    def number(n):return int.from_bytes(take(n),'little')
    rooms,servers=number(4),number(4);assert rooms<=127 and servers<=10;d=dict(rooms=[],servers=[])
    for kind,count in [('rooms',rooms),('servers',servers)]:
        for _ in range(count):
            end=body.find(b'\0',at,at+257);assert end>=at;name=take(end-at).decode('cp950');assert take(1)==b'\0'
            ip='.'.join(map(str,take(4)));port=number(2);assert port>0
            row=dict(name=name,endpoint=ip+':'+str(port))
            if kind=='rooms':row.update(id=number(4),users=number(4));assert row['id']>0
            else:row.update(users=number(4),capacity=number(4));assert row['capacity']>0
            d[kind].append(row)
    assert at==len(body);return d
def counts(items):return {str(row['id']):row['count']for row in items}
def item_counts(items):return {key:row['count']for key,row in items.items()}

ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--wrapper-exit-code',type=int,choices=[0,1],required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-catalog-handoff')and a.session>0 and a.terminal_chunk not in ['','pending','unknown']
c=read(run/'receipt.json');assert not c['owned_processes']and c['actual_sessions']==6 and c['asset_metadata_unchanged']
assert not c['source_overlays_used']and not c['diagnostic_API_extension']and not c['normal_source_modified']and c['normal_source_bodies_unchanged']
build=Path(c['compile']);b=read(build/'receipt.json');assert b['status']=='compiled_normal_public_API_same_endpoint_rental_probe'and sha(build/'receipt.json')==c['compile_receipt_sha256']
assert c['source_manifest_sha256']==b['source_manifest_sha256']and c['probe_sha256']==b['probe_sha256']==sha(run/'probe.rs')==sha(build/'probe.rs')
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for n,h in c['pins'].items():assert sha(n)==h,n
for info in b['libraries'].values():assert sha(info['path'])==info['sha256']
assert sha(c['product'])==c['product_sha256']==b['product_sha256']
assert sha(c['reference_jar'])==c['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(c['catalog'])==c['catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
for n,h in c['harness_class_pins'].items():assert sha(n)==h,n
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
nq=read(b['normal_qualification']);assert sha(b['normal_qualification'])==b['normal_qualification_sha256']and nq['source_manifest_sha256']==c['source_manifest_sha256']and nq['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles'
assert nq['passed_per_profile']==16 and nq['recorded_owned_processes_all_terminal']and not nq['source_overlays_used']
for info in nq['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
compiled=read(run/'compile-terminal-observation.json');assert compiled['observed_terminal_exit_code']==0 and compiled['original_recorded_compile_processes_terminal']and compiled['compile_receipt_sha256']==sha(build/'receipt.json')
for row in c['process_history']+b['process_history']:assert row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle')
work=Path(c['runtime']).resolve(strict=True);assert work.is_relative_to(P/'target/current/native-catalog-handoff')
ready=read(work/'ready.json');server=read(work/'result.json')
assert ready['same_endpoint']and ready['town_port']==ready['b_port']==ready['c_port']and ready['jar_sha256'].upper()==c['reference_jar_sha256']
assert server['status']=='sessions_closed'and server['actual_client_sessions']==6 and server['authority_mutations']==0 and not server['sql_access']and not server['state_persistent']and server['memberships_empty']and server['staged_loadouts_empty']and server['authenticated_accounts_empty']
assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',(run/'client.stdout').read_text('utf-8-sig'))==[('1','0','0','0','0')]
reports={role:read(work/(role+'-handoff.json'))for role in ['a','b']};assert all(data['claimed_scope']==SCOPE for data in reports.values())
reader_failure=None
if a.wrapper_exit_code==1:
    assert c['profile']=='Debug'and c['status']=='failed_preserved'and c['error']=='AssertionError()'
    f=read(run/'initial-reader-failure/failure.json');assert f['original_wrapper_session']==a.session and f['original_wrapper_terminal_chunk']==a.terminal_chunk and f['original_wrapper_exit_code']==1
    assert f['actual_client_exit_code']==f['actual_server_exit_code']==0 and not f['runtime_reexecuted']
    assert sha(run/'runner.py')==sha(run/'initial-reader-failure/runner.before.py')==f['before_runner_sha256']
    assert "['claimed_scope']=='same_endpoint_advertised_rental_service'"in(run/'runner.py').read_text('utf-8-sig')
    reader_failure=dict(path=str(run/'initial-reader-failure/failure.json'),sha256=sha(run/'initial-reader-failure/failure.json'),original_wrapper_exit_code=1,actual_child_runtime_exit_code=0,retained_without_runtime_replay=True)
else:assert c['status']=='executed_same_endpoint_advertised_rental_pending_independent_readback'
captures=list((work/'captures').glob('*/frames.jsonl'));assert len(captures)==1;frames=[json.loads(s)for s in captures[0].read_text('utf-8-sig').splitlines()]
assert sum(row['plain']=='MASKED_C_LOGIN'for row in frames)==6
packets=[(i,row,bytes.fromhex(row['plain']))for i,row in enumerate(frames)if row['plain']!='MASKED_C_LOGIN'];outgoing=[(i,body)for i,row,body in packets if row['direction']=='client_to_server']
assert sum(body[0]==0xc9 for _,body in outgoing)==2
assert sum(body==b'\xd9'for _,body in outgoing)==6
assert sum(body==b'\xca\x01'for _,body in outgoing)==4 and not any(body==b'\xca\0'for _,body in outgoing)
assert sum(body==b'\xcf'+int(60001).to_bytes(4,'little')+b'\0'for _,body in outgoing)==4
teleports=[(i,row,body)for i,row,body in packets if row['direction']=='server_to_client'and body[0]==0xf4 and 'rental-teleport'in row['note']];assert len(teleports)==2
assert sum(row['direction']=='server_to_client'and body[0]==0xf3 and 'rental-teleport'in row['note']for _,row,body in packets)==2
directories=[(i,directory(body))for i,row,body in packets if row['direction']=='server_to_client'and body[0]==0xfb];assert len(directories)==2 and max(i for i,_ in directories)<min(i for i,_,_ in teleports)
actions={}
for role,data in reports.items():
    request=b'\x27'+int(1812987996).to_bytes(4,'little')+data['action'].encode('ascii')+bytes(2);indices=[i for i,body in outgoing if body==request];assert len(indices)==1;actions[role]=indices[0]
resumes=[i for i,body in outgoing if body==b'\xca\x01'and i>max(actions.values())];assert len(resumes)==2;cutoff=min(resumes)
batches=[(i,inventory(body))for i,row,body in packets if row['direction']=='server_to_client'and body[0]==0x7c];assert any(row['detail']for _,batch in batches for row in batch.values())
trace=[json.loads(s)for s in(work/'server-trace.jsonl').read_text('utf-8-sig').splitlines()];assert len({row['session_identity']for row in trace})==4
checks={};expected_moves=[];pictures={}
for role,account,own,backend,index in [('a','PeerFixture',1,'TestKnight',1),('b','PeerFixture2',3,'TestMage',2)]:
    data=reports[role];before=data['before'];destination=data['destination'];battle=data['battle'];resume=data['service_resume'];continued=resume['continued'];alias=before['selected_character']
    assert data['role']==role and data['same_endpoint']and data['old_peer']==data['new_peer']and data['old_frame_rejected']
    assert data['new_epoch']==data['old_epoch']+1 and alias==destination['selected_character']==battle['selected_character']==continued['selected_character']
    assert before['inventory']['owner']['character']==destination['inventory']['owner']['character']==alias and destination['inventory']['owner']['connection']==data['new_epoch']and destination['inventory']['owner']['selection']>before['inventory']['owner']['selection']
    assert any(body==b'\xc9'+alias.encode('cp950')+b'\0'for _,body in outgoing)
    assert destination['joined_room']is None and destination['rental_set']is None and destination['world']['actors']=={}and not destination['world']['started']
    assert before['services']in[entry for _,entry in directories]
    source_rooms=[room for room in before['services']['rooms']if room['endpoint']==data['old_peer']];assert len(source_rooms)==1 and source_rooms[0]['id']==60001
    assert destination['services']==dict(rooms=source_rooms,servers=[])
    assert all(not row['world_ready']and row['actors']==0 for row in data['transitions'])
    assert any(row['phase']=='handoff_closing'and row['inventory_owner']is None and row['services']is None for row in data['transitions'])
    assert before['world']['map_id']==battle['world']['map_id']==continued['world']['map_id']==500 and battle['world']['generation']>before['world']['generation']and continued['world']['generation']==battle['world']['generation']
    assert battle['joined_room']['id']==continued['joined_room']['id']==60001 and battle['joined_room']['mode']==6 and battle['rental_set']==before['rental_set']==resume['requested_set']==1
    assert resume['kind']=='LOGIN_catalog_rebound_to_same_endpoint_GAME'and not resume['normal_channel_continuation_verified']and not resume['GUI_verified']and not resume['nondefault_stat_continuity_verified']
    endpoint=[body for _,_,body in teleports if body[1]==index];assert len(endpoint)==1 and endpoint[0][2:6]==bytes([127,0,0,2])and int.from_bytes(endpoint[0][6:8],'little')==ready['town_port']
    groups={}
    for row in trace:
        if row['test_account']==account:groups.setdefault(row['session_identity'],[]).append(row)
    assert len(groups)==2;old_rows,new_rows=list(groups.values());old=[row for row in old_rows if row['phase']=='BATTLE'and row['rental_town']and row['rental_inventory']];new=[row for row in new_rows if row['phase']=='BATTLE'and row['rental_town']and row['rental_inventory']];assert old and new
    assert all(row['character']==backend and row['object_id']==own and row['source']==before['selected_source']for row in old+new)
    items=before['inventory']['items'];received=destination['inventory']['items'];carry=counts(old[-1]['rental_inventory']);native=counts(old[-1]['native_inventory'])
    assert set(carry).isdisjoint(native)and(set(carry)|set(native)).isdisjoint(SERVICE_IDS)
    assert set(items)==set(carry)|set(native)|SERVICE_IDS and item_counts({key:items[key]for key in carry})==carry and item_counts({key:items[key]for key in native})==native
    assert received=={key:items[key]for key in carry},'all staged rental fields must survive exactly; account/service inventory is a separate scope'
    initial=[(left[0],right[0])for left,right in itertools.combinations([(i,batch)for i,batch in batches if i<actions[role]],2)if not(set(left[1])&set(right[1]))and left[1]|right[1]==items];assert len(initial)==1
    dest=[i for i,batch in batches if actions[role]<i<cutoff and batch==received];assert len(dest)==1
    staged=[j for j,row in enumerate(new_rows)if row['character']==backend and counts(row['native_inventory'])==item_counts(received)and not row['rental_town']];assert staged
    current=continued['inventory']['items'];assert battle['inventory']['items']==current
    grants=[(i,batch)for i,batch in batches if i>cutoff and set(batch).isdisjoint(received)and received|batch==current];assert len(grants)==1;grant_index,grant=grants[0];assert SERVICE_IDS<=set(grant)
    current_gear={key:item for key,item in grant.items()if key not in SERVICE_IDS};assert len(current_gear)==len(carry)
    steps=resume['steps'];assert steps and steps[-1]['after']!=steps[0]['before']
    authority=[j for j,row in enumerate(new_rows)if row['phase']=='BATTLE'and row['rental_town']and row['rental_map']==500 and counts(row['native_inventory'])==item_counts(received)and counts(row['rental_inventory'])==item_counts(current_gear)and[row['x'],row['y']]==steps[-1]['after']];assert authority,'bind inventory to live pre-close gameplay authority, not restored account inventory after disconnect'
    assert any([row['x'],row['y']]==steps[0]['before']for row in new)
    actor=continued['world']['actors'][str(own)];assert[actor['x'],actor['y']]==steps[-1]['after']and str(1812987996)in continued['world']['actors']and str(3 if own==1 else 1)in continued['world']['actors']
    for step in data['steps']+steps:
        x,y=step['before'];heading=step['heading'];assert 0<=heading<=7 and all(0<=n<=65535 for n in step['before']+step['after']);dx,dy=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)][heading];assert step['after']==[x+dx,y+dy]
        expected_moves.append(b'\x0a'+x.to_bytes(2,'little')+y.to_bytes(2,'little')+bytes([heading]))
    picture=Path(resume['image']).resolve(strict=True);assert picture.parent.samefile(work)and picture.name==f'{role}-same-endpoint-rental.png'
    with Image.open(picture)as image:
        assert image.size==(800,600)and image.mode=='RGBA';image.load();rgba=hashlib.sha256(image.tobytes()).hexdigest().upper()
    frame=resume['frame'];assert frame['world_generation']==battle['world']['generation']and frame['map_id']==500 and frame['local_object_id']==own and frame['width']==800 and frame['height']==600 and frame['rgba_sha256'].upper()==rgba
    pictures[role]=dict(path=str(picture),sha256=sha(picture),rgba_sha256=rgba)
    checks[role]=dict(client_wire_alias=alias,underlying_character=backend,epochs=[data['old_epoch'],data['new_epoch']],source_S7C_rows=initial[0],source_native_items=len(native),source_rental_items=len(carry),separate_service_ids=sorted(SERVICE_IDS),destination_S7C_row=dest[0],all_staged_item_fields_exact=True,destination_staged_items=len(received),destination_staged_authority_indices=staged,resume_rental_S7C_row=grant_index,resume_native_items=len(received),resume_rental_items=len(current_gear),resume_total_items=len(current),pre_close_authority_indices=authority,resume_steps=len(steps),final_authority_position=steps[-1]['after'],fresh_world_generation=battle['world']['generation'],old_frame_rejected=True,original_PNG=pictures[role])
assert Counter(body for _,body in outgoing if body[0]==0x0a)==Counter(expected_moves)
identities=[c['runner'],*c['process_history'],*compiled['identities']];assert len(identities)==6
filters=' OR '.join('ProcessId='+str(pid)for pid in sorted({row['pid']for row in identities}));command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress";z=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert z.returncode==0 and not z.stderr;live= json.loads(z.stdout.decode('utf-8-sig'))if z.stdout.strip()else[];live=[live]if isinstance(live,dict)else live
for row in live:
    for identity in identities:
        if row['ProcessId']==identity['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(identity['creation_utc'])).total_seconds())>.001
out=run/'qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'terminal-CIM.json').write_bytes(z.stdout)
with zipfile.ZipFile(out/'product.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as archive:archive.write(c['product'],'probe.exe')
with zipfile.ZipFile(out/'product.zip')as archive:
    with archive.open('probe.exe')as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==c['product_sha256']
files={f'run/{p.relative_to(run).as_posix()}':p for p in run.iterdir()if p.is_file()};files.update({f'compile/{p.name}':p for p in build.iterdir()if p.is_file()});files['normal-related-qualified.json']=Path(b['normal_qualification'])
for p in(run/'initial-reader-failure').glob('*'):
    if p.is_file():files['run/initial-reader-failure/'+p.name]=p
for p in work.rglob('*'):
    if p.is_file()and p.suffix.lower()not in ['.exe','.zip']and'classes'not in p.relative_to(work).parts:
        assert p.resolve().is_relative_to(work)and not p.is_symlink();files['runtime/'+p.relative_to(work).as_posix()]=p
pins={name:sha(p)for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as archive:
    for name,p in files.items():archive.write(p,name)
with zipfile.ZipFile(out/'raw.zip')as archive:
    assert set(archive.namelist())==set(pins)
    for name,h in pins.items():
        with archive.open(name)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
q=dict(status='independently_qualified_current75_same_endpoint_LOGIN_catalog_rebind_rental_scope',profile=c['profile'],source_manifest_sha256=c['source_manifest_sha256'],probe_sha256=c['probe_sha256'],reference_jar_sha256=c['reference_jar_sha256'],original_session=a.session,terminal_tool_chunk=a.terminal_chunk,actual_original_wrapper_exit_code=a.wrapper_exit_code,actual_client_exit_code=0,actual_server_exit_code=0,wrapper_reader_failure=reader_failure,normal_qualification=b['normal_qualification'],normal_qualification_sha256=b['normal_qualification_sha256'],normal_mains=nq['normal_mains'],actual_sessions=6,LOGIN_sessions=2,GAME_sessions=4,LOGIN_service_directories=2,GAME_service_directories=0,checks=checks,original_PNGs=pictures,exact_C_MOVE=len(expected_moves),recorded_original_identities=6,recorded_original_owned_processes_terminal=True,product_sha256=c['product_sha256'],product_archive_sha256=sha(out/'product.zip'),raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),source_overlays_used=False,diagnostic_API_extension=False,normal_channel_continuation_verified=False,old_CA00_failure_preserved=True,nondefault_stat_continuity_verified=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False,qualified_utc=datetime.now(timezone.utc).isoformat())
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=6,source_inventory_scopes={role:dict(native=x['source_native_items'],rental=x['source_rental_items'],staged=x['destination_staged_items'],resumed=x['resume_total_items'])for role,x in checks.items()},retained_outer_reader_failure=reader_failure is not None),ensure_ascii=False))
