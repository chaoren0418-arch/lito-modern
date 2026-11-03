"""Read actual packets/authority/items/PNGs for the separate advertised rental scope."""
from pathlib import Path
from datetime import datetime,timezone
from collections import Counter
from PIL import Image
import argparse,hashlib,itertools,json,re,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
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
    def take(count):
        nonlocal at
        assert 0<=count<=len(body)-at;value=body[at:at+count];at+=count;return value
    def number(count):return int.from_bytes(take(count),'little')
    for _ in range(body[1]):
        identity=number(4);assert identity>0 and str(identity)not in records
        use_type,charge,icon,blessing,count,identified=[number(n)for n in [1,1,2,1,4,1]];assert count>0 and blessing<=3
        end=body.find(b'\0',at,at+256);assert end>=at;name=take(end-at).decode('cp950');assert take(1)==b'\0';detail=list(take(number(1)))
        records[str(identity)]=dict(id=identity,use_type=use_type,charge=charge,icon=icon,blessing=blessing,count=count,identified=identified,name=name,detail=detail)
    assert at==len(body);return records
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
assert a.session>0 and a.terminal_chunk not in ['','pending','unknown']and a.terminal_exit_code==0
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-rental-service-same-endpoint');c=read(run/'receipt.json')
assert c['status']=='executed_same_endpoint_advertised_rental_pending_independent_readback'and c['owned_processes']==[]and c['actual_sessions']==6 and c['asset_metadata_unchanged']
assert not c['source_overlays_used']and not c['diagnostic_API_extension']and not c['normal_source_modified']and c['normal_source_bodies_unchanged']
build=Path(c['compile']);b=read(build/'receipt.json');assert b['status']=='compiled_normal_public_API_same_endpoint_rental_probe'and sha(build/'receipt.json')==c['compile_receipt_sha256']
assert c['source_manifest_sha256']==b['source_manifest_sha256']and c['probe_sha256']==b['probe_sha256']==sha(run/'probe.rs')==sha(build/'probe.rs')
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1049
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for n,h in c['pins'].items():assert sha(n)==h,n
for item in b['libraries'].values():assert sha(item['path'])==item['sha256']
assert sha(c['product'])==c['product_sha256']==b['product_sha256']and sha(c['reference_jar'])==c['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(c['catalog'])==c['catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
for n,h in c['harness_class_pins'].items():assert sha(n)==h,n
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
compiled=read(run/'compile-terminal-observation.json');assert compiled['observed_terminal_exit_code']==0 and compiled['original_recorded_compile_processes_terminal']and compiled['compile_receipt_sha256']==sha(build/'receipt.json')
normal=read(Path(b['normal_cohort'])/'cohort.json')
for item in normal['profiles'].values():assert sha(item['normal']['path'])==item['normal']['sha256']
work=Path(c['runtime']);assert work.resolve(strict=True).is_relative_to(P/'target/current/native-rental-service-same-endpoint')
ready=read(work/'ready.json');server=read(work/'result.json')
assert ready['same_endpoint']and ready['town_port']==ready['b_port']==ready['c_port']and ready['jar_sha256'].upper()==c['reference_jar_sha256']
assert server['status']=='sessions_closed'and server['actual_client_sessions']==6 and server['authority_mutations']==0 and not server['sql_access']and not server['state_persistent']and server['memberships_empty']and server['staged_loadouts_empty']and server['authenticated_accounts_empty']
assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',(run/'client.stdout').read_text('utf-8-sig'))==[('1','0','0','0','0')]
captures=list((work/'captures').glob('*/frames.jsonl'));assert len(captures)==1
frames=[json.loads(s)for s in captures[0].read_text('utf-8-sig').splitlines()]
assert sum(row['plain']=='MASKED_C_LOGIN'for row in frames)==6
packets=[(i,row,bytes.fromhex(row['plain']))for i,row in enumerate(frames)if row['plain']!='MASKED_C_LOGIN']
outgoing=[(i,body)for i,row,body in packets if row['direction']=='client_to_server']
source_requests=[body for _,body in outgoing if body[0]==0xc8]
assert len(source_requests)==4 and all(body[1:].rstrip(b'\0').decode('cp950')=='測試乙'for body in source_requests), 'initial C8 confirmation or staged destination replay mismatch'

assert sum(body[0]==0xc9 for _,body in outgoing)==2,'only the two original character choices'
assert sum(body==b'\xd9'for _,body in outgoing)==6,'four rental entries and two actual transfer exits'
assert sum(body==b'\xca\x01'for _,body in outgoing)==4 and not any(body==b'\xca\0'for _,body in outgoing)
assert sum(body==b'\xcf'+int(60001).to_bytes(4,'little')+b'\0'for _,body in outgoing)==4
teleports=[(i,row,body)for i,row,body in packets if row['direction']=='server_to_client'and body[0]==0xf4 and 'rental-teleport'in row['note']];assert len(teleports)==2
assert sum(row['direction']=='server_to_client'and body[0]==0xf3 and 'rental-teleport'in row['note']for _,row,body in packets)==2
reports={role:read(work/(role+'-handoff.json'))for role in ['a','b']};action_rows={}
for role,data in reports.items():
    expected=b'\x27'+int(1812987996).to_bytes(4,'little')+data['action'].encode('ascii')+bytes(2);indices=[i for i,body in outgoing if body==expected];assert len(indices)==1;action_rows[role]=indices[0]
resume_indices=[i for i,body in outgoing if body==b'\xca\x01'and i>max(action_rows.values())];assert len(resume_indices)==2;resume_cutoff=min(resume_indices)
batches=[(i,inventory(body))for i,row,body in packets if row['direction']=='server_to_client'and body[0]==0x7c]
assert batches and any(item['detail']for _,batch in batches for item in batch.values())
trace=[json.loads(s)for s in(work/'server-trace.jsonl').read_text('utf-8-sig').splitlines()];assert len({s['session_identity']for s in trace})==4
checks={};expected_moves=[];pictures={}
for role,account,index in [('a','PeerFixture',1),('b','PeerFixture2',2)]:
    data=reports[role];before=data['before'];destination=data['destination'];battle=data['battle'];resume=data['service_resume'];continued=resume['continued']
    assert data['claimed_scope']=='same_endpoint_advertised_rental_service'and data['same_endpoint']and data['old_peer']==data['new_peer']and data['old_frame_rejected']
    assert data['new_epoch']==data['old_epoch']+1 and before['selected_character']==destination['selected_character']==battle['selected_character']==continued['selected_character']
    assert destination['inventory']['owner']['connection']==data['new_epoch']and destination['inventory']['owner']['selection']>before['inventory']['owner']['selection']
    assert destination['joined_room']is None and destination['rental_set']is None and destination['world']['actors']=={}and not destination['world']['started']
    assert destination['services']['rooms']and destination['services']['rooms'][0]['id']==60001
    assert all(not row['world_ready']and row['actors']==0 for row in data['transitions'])
    assert any(row['phase']=='handoff_closing'and row['inventory_owner']is None and row.get('services')is None for row in data['transitions'])
    assert before['world']['map_id']==battle['world']['map_id']==continued['world']['map_id']==500 and battle['world']['generation']>before['world']['generation']and continued['world']['generation']==battle['world']['generation']
    assert battle['joined_room']['id']==continued['joined_room']['id']==60001 and battle['joined_room']['mode']==6 and battle['rental_set']==before['rental_set']==resume['requested_set']==1
    assert resume['kind']=='advertised_rental_service'and not resume['normal_channel_continuation_verified']and not resume['GUI_verified']and not resume['nondefault_stat_continuity_verified']
    items=before['inventory']['items'];received=destination['inventory']['items'];assert items==received and items
    initial=[]
    for left,right in itertools.combinations([(i,batch)for i,batch in batches if i<action_rows[role]],2):
        if not(set(left[1])&set(right[1]))and left[1]|right[1]==items:initial.append([left[0],right[0]])
    assert len(initial)==1,'source full inventory must match two original disjoint wire batches'
    dest=[i for i,batch in batches if action_rows[role]<i<resume_cutoff and batch==received];assert len(dest)==1,'initial destination must match one entire original wire batch before service selection'
    endpoint=[body for _,_,body in teleports if body[1]==index];assert len(endpoint)==1 and endpoint[0][2:6]==bytes([127,0,0,2])and int.from_bytes(endpoint[0][6:8],'little')==ready['town_port']
    groups={}
    for row in trace:
        if row['test_account']==account:groups.setdefault(row['session_identity'],[]).append(row)
    assert len(groups)==2;old_rows,new_rows=list(groups.values());old=[row for row in old_rows if row['phase']=='BATTLE'and row['rental_town']and row['rental_inventory']];new=[row for row in new_rows if row['phase']=='BATTLE'and row['rental_town']and row['rental_inventory']];assert old and new
    assert old[-1]['character']==new[-1]['character']==continued['selected_character']and old[-1]['session_identity']!=new[-1]['session_identity']
    assert {str(item['id']):item['count']for item in old[-1]['rental_inventory']}=={key:item['count']for key,item in received.items()}
    assert {str(item['id']):item['count']for item in new[-1]['rental_inventory']}=={key:item['count']for key,item in continued['inventory']['items'].items()}
    steps=resume['steps'];assert steps and steps[-1]['after']!=steps[0]['before']
    assert any([row['x'],row['y']]==steps[0]['before']for row in new)and[new[-1]['x'],new[-1]['y']]==steps[-1]['after']
    own=continued['world']['actors'][str(continued['world']['status']['object_id'])];assert[own['x'],own['y']]==steps[-1]['after']
    for step in data['steps']+steps:
        x,y=step['before'];heading=step['heading'];assert 0<=heading<=7 and all(0<=n<=65535 for n in step['before']+step['after'])
        delta=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)][heading];assert step['after']==[x+delta[0],y+delta[1]]
        expected_moves.append(b'\x0a'+x.to_bytes(2,'little')+y.to_bytes(2,'little')+bytes([heading]))
    picture=Path(resume['image']).resolve(strict=True);assert picture.parent.samefile(work)and picture.name==f'{role}-same-endpoint-rental.png'
    with Image.open(picture)as image:assert image.size==(800,600)and image.mode=='RGBA';image.load()
    assert resume['frame'];pictures[role]=dict(path=str(picture),sha256=sha(picture))
    checks[role]=dict(character=continued['selected_character'],epochs=[data['old_epoch'],data['new_epoch']],source_S7C_rows=initial[0],destination_S7C_row=dest[0],all_initial_item_fields_exact=True,inventory_items=len(items),fresh_same_map_generation=battle['world']['generation'],resume_steps=len(steps),actual_final_authority_position=steps[-1]['after'],original_PNG=pictures[role])
assert Counter(body for _,body in outgoing if body[0]==0x0a)==Counter(expected_moves)
identities=[c['runner']]+c['process_history']+compiled['identities'];assert len(identities)==6
for row in c['process_history']+b['process_history']:assert row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle')
ids=','.join(str(row['pid'])for row in identities);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);current=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];current=[current]if isinstance(current,dict)else current
for actual in current:
    for row in identities:
        if actual['ProcessId']==row['pid']:assert abs((datetime.fromisoformat(actual['created'])-datetime.fromisoformat(row['creation_utc'])).total_seconds())>.001,'original owned process still present'
out=run/'qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'terminal-CIM.json').write_bytes(raw)
with zipfile.ZipFile(out/'product.zip','x',zipfile.ZIP_DEFLATED)as z:z.write(c['product'],'probe.exe')
with zipfile.ZipFile(out/'product.zip')as z:
    with z.open('probe.exe')as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==c['product_sha256']
files={f'run/{p.name}':p for p in run.iterdir()if p.is_file()};files.update({f'compile/{p.name}':p for p in build.iterdir()if p.is_file()})
for p in work.rglob('*'):
    if p.is_file()and p.suffix.lower()not in ['.exe','.zip']and'classes'not in p.relative_to(work).parts:
        assert p.resolve(strict=True).is_relative_to(work.resolve(strict=True))and not p.is_symlink();files['runtime/'+p.relative_to(work).as_posix()]=p
pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
save(out/'qualified.json',dict(status='normal_public_API_same_endpoint_advertised_rental_scope_qualified',profile=c['profile'],source_manifest_sha256=c['source_manifest_sha256'],probe_sha256=c['probe_sha256'],reference_jar_sha256=c['reference_jar_sha256'],original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,actual_sessions=6,checks=checks,original_PNGs=pictures,recorded_original_identities=6,recorded_original_owned_processes_terminal=True,product_sha256=c['product_sha256'],product_archive_sha256=sha(out/'product.zip'),raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),source_overlays_used=False,diagnostic_API_extension=False,normal_source_modified=False,normal_channel_continuation_verified=False,old_CA00_failure_preserved=True,nondefault_stat_continuity_verified=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False))
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=6,players=2,scope='same-endpoint advertised rental service only'),ensure_ascii=False))
