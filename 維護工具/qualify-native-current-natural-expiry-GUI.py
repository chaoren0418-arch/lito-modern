"""Qualify real-duration normal GUI expiry from UI, exact wire and owner trace."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(j):return(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
utc=lambda s:datetime.fromisoformat(s.replace('Z','+00:00'))
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);ap.add_argument('--plan',type=Path,required=True);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(V/'native-installed-resource-profile-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json');plan=read(a.plan);source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']==plan['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert c['scenario']in ['rental-hotel-expiry','rental-hotel-middle-expiry']and c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'
assert c['recorded_owned_processes_terminal']and all(x['exit_code']==0 and not x.get('terminated_during_failure_cleanup')for x in c['processes'])
assert c['runtime_deadline_seconds']==8000 and c['allowed_cleanup_seconds']==30 and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']and w['current_full_default_library_verified']
assert sha(case/'runner.py')==c['launcher_sha256']and sha(plan['prior_plan'])==plan['prior_plan_sha256']=='153F249337DF06C9E0BDB92100C46FC78056D1CAB4ACC7C719569B16E8311AB8'and plan['numeric_submit_key']=='Return'and plan['original_catalogue_durations_preserved']
assert plan['cases']==read(plan['prior_plan'])['cases']and a.plan.resolve(strict=True).is_relative_to(V/'native-quantity-enter-failure-tooling')
matrix=[x for x in plan['cases']if(x['profile'],x['scenario'])==(c['profile'],c['scenario'])];assert len(matrix)==1;matrix=matrix[0]
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']and p['source_manifest_sha256']==source
assert len(p['compiled_inputs'])==1049 and all(sha(R/n)==h for n,h in p['compiled_inputs'].items())
for group in ['fixture_sources','fixture_origins']:
    for n,h in p[group].items():assert sha(n)==h,n
for n,h in p['class_files'].items():assert sha(Path(p['classes'])/n)==h,n
assert sha(p['jar'])==p['jar_sha256']==plan['formal_jar_sha256']and sha(p['catalog'])==p['catalog_sha256']
assert sha(plan['rooms_fixture'])==plan['rooms_fixture_sha256']
from native_hotel_catalog_reference import fixture_reference
room_proof=fixture_reference(Path(plan['rooms_fixture']),p['jar_sha256'])
policy=p['operator_wait_policy'];assert policy['rental_natural_expiry_seconds']==8000 and not policy['application_protocol_deadlines_changed']and not policy['gameplay_expiry_clocks_changed']
observer=p['passive_hotel_lease_registry_observation'];assert observer['readonly_getters_verified']and sha(observer['contract'])==observer['contract_sha256']
normal=read(p['normal_qualification']);assert sha(p['normal_qualification'])==p['normal_qualification_sha256']and normal['source_manifest_sha256']==source and normal['passed_per_profile']==19
lib=Path(p['library'])/'completion-observation.json';assert sha(lib)==p['library_completion_sha256']and read(lib)['source_manifest_sha256']==source
terminals=list(case.glob('checkpoint-terminal-*/evidence.json'));assert len(terminals)==1;t=read(terminals[0]);assert t['observed_actual_exit_code']==0 and t['original_identities_terminal']and t['owned_processes']==c['processes']
result=read(case/'result.json');assert result['actual_client_sessions']==4 and result['game_sessions']==2 and result['memberships_empty']and not result['sql_access']and result['authority_mutations']==result['active_hotel_leases_after_disconnect']==result['classic_world_members_after_disconnect']==0
events=read(case/'Sky-events.json');assert [utc(e['observed_utc'])for e in events]==sorted(utc(e['observed_utc'])for e in events)
trace=[json.loads(l)for l in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()];log=(case/'server.stdout.log').read_text('utf-8-sig')
frames=next((case/'captures').glob('*/frames.jsonl'));wire=[json.loads(l)for l in frames.read_text('utf-8-sig').splitlines()];assert sum(x['plain']=='MASKED_C_LOGIN'for x in wire)==4
packets=[(x,bytes.fromhex(x['plain']))for x in wire if x['plain']!='MASKED_C_LOGIN'];outgoing=[b for x,b in packets if x['direction']=='client_to_server']
assert [b[1:].rstrip(b'\0').decode('cp950')for b in outgoing if b[0]==0xc8]==['測試乙']*4
assert [b[1:].rstrip(b'\0').decode('cp950')for b in outgoing if b[0]==0xc9]==['PeerFixture(測試乙)','PeerFixture2(測試乙)']
assert not any(b[0]in [0x10,0xd9]for b in outgoing)
route=read(R/'LiTo381-Modern/tests/fixtures/native-gui/hotel-route.json');moves=[b'\x0a'+int(step['before'][0]).to_bytes(2,'little')+int(step['before'][1]).to_bytes(2,'little')+bytes([step['heading']])for step in route['steps']];assert len(moves)==13 and [b for b in outgoing if b[0]==0x0a]==moves*2
NPC=(1812987995).to_bytes(4,'little');action_bodies=[b for b in outgoing if b[0]==0x27];assert all(len(b)>=8 and b[1:5]==NPC and b[-2:]==b'\0\0'for b in action_bodies)
actions=[b[5:-2].decode('ascii')for b in action_bodies];assert actions==['room','enter']*2
replies=[(row,b)for row,b in packets if row['direction']=='client_to_server'and b[0]==0x51];assert len(replies)==2
roles={};all_keys=set();tokens=set()
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
    spec=matrix['roles'][role];quantity=spec['quantity'];seconds=spec['expected_seconds'];assert seconds==(7200 if quantity>=5 else 3600)
    product=p['products'][c['profile']][role];assert sha(product['path'])==sha(product['original_path'])==product['sha256']==normal['normal_mains'][c['profile']]
    own=[e for e in events if e['role']==role];by={}
    for e in own:
        if e['label']in by:assert by[e['label']]['action']is None and e['action']is None
        by[e['label']]=e
        assert e['window']['app'].casefold()==('process:'+product['path']).casefold()
    assert len({e['window']['id']for e in own})==1
    labels=['account','login','source-choice','commit-source','get-characters','character-response','select-character','confirm-loadout','enter-town','world-focus',*['hotel-walk-'+str(i)for i in range(1,14)],'hotel-talk','hotel-rent-choice','hotel-rent-request','hotel-quantity-requested','hotel-quantity-value','hotel-quantity-submit','hotel-keys-granted','hotel-enter-choice','hotel-enter-request','hotel-room-loaded','hotel-expiry-wait-start','hotel-expiry-ready','close-normal'];assert set(labels)<=set(by)
    assert [own.index(by[label])for label in labels]==sorted(own.index(by[label])for label in labels)
    assert by['account']['action']['args']['value']==account and by['source-choice']['action']['args']['value']=='測試乙'
    assert account+'(測試乙)'in by['character-response']['tree']and all(account+'(測試甲)'not in e['tree']for e in own)
    for label in ['login','commit-source','get-characters','select-character','confirm-loadout','enter-town','hotel-talk','hotel-rent-request','hotel-enter-request']:assert by[label]['action']==dict(method='press_key',args={'key':'space'})
    for i,step in enumerate(route['steps'],1):assert by['hotel-walk-'+str(i)]['action']==dict(method='press_key',args={'key':{3:'KP_3',5:'KP_1'}[step['heading']]})
    value=by['hotel-quantity-value'];assert value['action']['method']=='set_value'and value['action']['args']['value']==str(quantity)and 'ID: 222'in value['focused_element']
    assert by['hotel-quantity-submit']['action']==dict(method='press_key',args={'key':'Return'})and own[own.index(by['hotel-quantity-submit'])-1]is value
    for label in ['hotel-quantity-requested','hotel-room-loaded','hotel-expiry-wait-start','hotel-expiry-ready']:assert by[label]['action']is None
    assert by['hotel-rent-choice']['action']['args']['value']=='租借房間'and by['hotel-enter-choice']['action']['args']['value']=='進入房間'
    assert f"地圖 {spec['map_id']}"in by['hotel-room-loaded']['tree']and 'ID: 318'in by['hotel-room-loaded']['tree']and f"背包（{spec['expected_initial_items']+quantity}）"in by['hotel-room-loaded']['tree']
    after=by['hotel-expiry-ready']['tree'];assert '地圖 500'in after and 'ID: 318'in after and 'ID: 208'in after and f"背包（{spec['expected_initial_items']}）"in after and not re.search(r'\bID:\s*220\b',after)and '世界資源載入失敗'not in after
    assert by['close-normal']['action']==dict(method='press_key',args={'key':'Alt_L+F4'})
    authority=[r for r in trace if r['state']['test_account']==account and r['state']['source']=='測試乙'and r['state']['phase']=='BATTLE'and r['state']['rental_town']];assert len({r['state']['session_identity']for r in authority})==1
    before=next(r for r in authority if r['state']['rental_map']==500 and r['state']['rental_hotel']is None and [r['state']['x'],r['state']['y']]==[32779,32831])
    grant=next(r for r in authority if r['state']['rental_hotel']is not None and r['state']['rental_hotel']['keys']==quantity);lease=grant['state']['rental_hotel'];assert lease['map']==spec['map_id']and lease['token']not in tokens;tokens.add(lease['token'])
    arrival=next(r for r in authority if r['state']['rental_map']==spec['map_id']);assert arrival['state']['rental_hotel']==lease and arrival['state']['rental_hotel_owner_lease_active']is True
    keyed=lambda r,field:{x['id']:x for x in r['state'][field]};initial=keyed(before,'rental_inventory');rented=keyed(grant,'rental_inventory');keys=set(rented)-set(initial);assert len(keys)==quantity and not keys&all_keys;all_keys|=keys
    assert {k:v for k,v in rented.items()if k not in keys}==initial and all(rented[k]['catalog_id']==40312 and rented[k]['count']==1 and not rented[k]['equipped']for k in keys)
    expired=next(r for r in authority if r['ms']>arrival['ms']and r['state']['rental_map']==500 and r['state']['rental_hotel']is None and r['state']['rental_hotel_owner_lease_active']is False and not keys&set(keyed(r,'rental_inventory')))
    elapsed=expired['ms']-grant['ms'];assert(seconds-3)*1000<=elapsed<=(seconds+120)*1000
    assert [expired['state'][k]for k in ['x','y','heading']]==[*spec['expiry_return_xy'],2]
    assert keyed(expired,'rental_inventory')==initial and keyed(expired,'native_inventory')==keyed(before,'native_inventory')
    assert all(expired['state'][k]==before['state'][k]for k in ['rental_set','character','rental_gems','shop_points'])
    assert f"rental-hotel lease invalidated token={lease['token']} reason=rental-hotel-expiry keys={quantity}"in log
    match=[(r,b)for r,b in replies if int.from_bytes(b[5:9],'little')==quantity];assert len(match)==1;reply_row,reply=match[0];assert len(reply)==11 and reply[1:5]==NPC and reply[-2:]==b'\0\0'
    requests=[(r,b)for r,b in packets if r['direction']=='server_to_client'and b[0]==0x5b and utc(r['time'])<utc(reply_row['time'])];request_row,request=requests[-1];assert request[1:5]==NPC and [int.from_bytes(request[i:i+4],'little')for i in [9,13,17]]==[1,1,8]and request[21]==0 and 0<=(utc(reply_row['time'])-utc(request_row['time'])).total_seconds()<30
    assert utc(by['hotel-expiry-wait-start']['observed_utc'])<utc(by['hotel-expiry-ready']['observed_utc'])<utc(by['close-normal']['observed_utc'])
    roles[role]=dict(account=account,map_id=spec['map_id'],quantity=quantity,lease_token=lease['token'],key_ids=sorted(keys),original_seconds=seconds,actual_grant_to_expiry_ms=elapsed,base_inventory_all_fields_preserved=True,registry_lease_inactive_before_disconnect=True)
portals=[(i,b)for i,(r,b)in enumerate(packets)if r['direction']=='server_to_client'and b[0]==0x5a];acks=[i for i,(r,b)in enumerate(packets)if r['direction']=='client_to_server'and b==b'\x08'];assert len(portals)==len(acks)==4
assert [b for _,b in portals]==[bytes.fromhex(matrix['roles'][r]['portal'])for r in ['a','b']]+[bytes([0x5a])+b''.join(x.to_bytes(2,'little')for x in [500,32779,32832])]*2
assert all(portals[i][0]<acks[i]and(i==3 or acks[i]<portals[i+1][0])for i in range(4))
removed=[int.from_bytes(b[1:5],'little')for r,b in packets if r['direction']=='server_to_client'and len(b)==5 and b[0]==0x17 and int.from_bytes(b[1:5],'little')in all_keys];assert len(removed)==len(all_keys)and set(removed)==all_keys
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=case/'qualified-natural-expiry-keyboard';out.mkdir();files={x.relative_to(case).as_posix():x for x in case.iterdir()if x.is_file()};files.update({'captures/frames.jsonl':frames,'terminal.json':terminals[0],'prepared.json':Path(c['prepared'])/'prepared.json','plan.json':a.plan,'normal-qualification.json':Path(p['normal_qualification']),'library-completion.json':lib,'observer-contract.json':Path(observer['contract']),'qualifier.py':Path(__file__)})
for role,product in p['products'][c['profile']].items():files[role+'.exe']=Path(product['path'])
hashes={n:sha(path)for n,path in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,path in files.items():z.write(path,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_normal_GUI_original_duration_hotel_expiry_limited',source_manifest_sha256=source,profile=c['profile'],scenario=c['scenario'],case=str(case),actual_formal_sessions=4,original_session_id=t['original_session_id'],observed_actual_exit_code=0,original_identities_terminal=True,roles=roles,room_catalogue_reference=room_proof,clock_or_authority_injection=False,merged_capture_individual_socket_delivery_claimed=False,GUI_full_flow_verified=False,physical_audio_verified=False,pixel_verified=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
(out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'));(out/'evidence.json').write_bytes(data(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='當版'+c['profile']+' '+c['scenario']+'正常GUI原租期到期限定核對：實等3600／7200秒依房型，原S17／S5A／C08、有效租約退役、原物品全欄與45／43背包及技能恢復皆在正常退出前核對。只授此情境，不移記其他組態／房型或像素與完整G01–G12，不交人工。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j.setdefault('current_normal_GUI_natural_expiry',{}).setdefault(c['profile'],{})[c['scenario']]=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=c['profile'],scenario=c['scenario'],actual_formal_sessions=4,GUI_full_flow_verified=False),ensure_ascii=False))
