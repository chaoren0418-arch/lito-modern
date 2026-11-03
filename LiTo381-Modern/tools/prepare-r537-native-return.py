import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';dest=p/'target/r537-native-pet-race-return-inputs'
def read(f):return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
    h=hashlib.sha256()
    with f.open('rb') as s:
        for b in iter(lambda:s.read(1024*1024),b''):h.update(b)
    return {'path':str(f),'bytes':f.stat().st_size,'sha256':h.hexdigest().upper()}
def check(d):
    a=pin(pathlib.Path(d['path']));assert (a['bytes'],a['sha256'])==(d['bytes'],d['sha256'].upper()),a
def save(f,v):
    with f.open('x',encoding='utf-8') as s:json.dump(v,s,ensure_ascii=False,indent=2);s.write('\n')
root=p.parent.parent/'lito伺服器/Native-PetRace-caller部分寫入與RESULT返回交接.json'
check({'path':str(root),'bytes':4990,'sha256':'66EDC23BAA24A9C98D9837E7FC73A02CB64AB37BEEEE4E62AFEF28113C4502BB'})
v=read(root);refs=[d for d in v.values() if isinstance(d,dict) and {'path','bytes','sha256'}<=d.keys()]+v['parents_unchanged']
assert len(refs)==13
for d in refs:check(d)
m=read(pathlib.Path(v['manifest']['path']));assert len(m['files'])==1036
owner=pathlib.Path(v['root']).resolve()
for d in m['files']:
    assert pathlib.Path(d['path']).resolve().is_relative_to(owner);check(d)
dest.mkdir();shutil.copyfile(root,l/'r537-caller-return-server-root.json');copied=[]
for key in ['manifest','source_diff_from_263A','version_lifecycle_contract','partial_caller_evidence','new_candidate_partial_result','RESULT_return_result','RESULT_consumer_index']:
    f=pathlib.Path(v[key]['path']);to=dest/(key+f.suffix);shutil.copyfile(f,to);copied.append(pin(to))
index=read(pathlib.Path(v['RESULT_consumer_index']['path']))
assert index['per_player_sequences_count']==6 and index['successful_application_bodies']==39
assert index['RESULT_return_bodies']==22 and index['necessary_active_D9_setup_bodies']==17
sequences=[]
for seq in index['per_player_sequences']:
    frames=[];aliases=set();rooms=set()
    for n,f in enumerate(seq['frames'],1):
        assert f['ordinal']==n and f['complete_flush'] and f['client_ACK'] is False
        check(f['application_pin']);check(f['wire_pin'])
        b=pathlib.Path(f['application_file']).read_bytes();assert b==bytes.fromhex(f['application_hex'])
        capture=json.loads(f['capture_json_line']);assert bytes.fromhex(capture['plain'])==b and capture['wire_length']==len(b)+2
        own_name=None
        if b[0]==0xD0:
            name=b[5:-1];assert b[-1]==0
            if name.startswith(seq['account_alias'].encode('ascii')):
                aliases.add(name.hex().upper());rooms.add(int.from_bytes(b[1:5],'little'));own_name=name.hex().upper()
        to=dest/f'{seq["scenario"]}-{seq["role"]}-{n:03}-op{b[0]:02X}.bin';shutil.copyfile(f['application_file'],to);copied.append(pin(to))
        frames.append({'ordinal':n,'stage':f['stage'],'body_hex':b.hex().upper(),'body_file':pin(to),
            'RESULT_return_output':f['RESULT_return_output'],'own_exit_name_hex':own_name,
            'complete_server_flush':True,'client_delivery_proven':False,'source_application':f['application_pin']})
    assert len(aliases)<=1 and len(rooms)<=1
    room=2 if seq['scenario']=='default_room_last_member_reset' else 3
    assert not rooms or rooms=={room}
    sequences.append({'scenario':seq['scenario'],'role':seq['role'],'account_alias':seq['account_alias'],
        'origin_room_id':room,'self_alias_hex':next(iter(aliases)) if aliases else seq['account_alias'].encode('ascii').hex().upper(),
        'self_display_alias_observed_in_own_D0':bool(aliases),'frames':frames})
partial=[]
for c in read(pathlib.Path(v['partial_caller_evidence']['path']))['cases']:
    if c['version']!='new_candidate':continue
    check(c['raw_partial_bytes']);b=pathlib.Path(c['raw_partial_bytes']['path']).read_bytes()
    assert len(b)==c['actual_bytes_written']<c['declared_wire_length'] and int.from_bytes(b[:2],'little')==c['declared_wire_length']
    assert c['complete_frame'] is False and c['application_body_available'] is False and c['client_ACK'] is False
    assert c['post_failure_output_acquisitions']==0
    to=dest/(c['scenario']+'.partial.bin');shutil.copyfile(c['raw_partial_bytes']['path'],to);copied.append(pin(to))
    partial.append({'scenario':c['scenario'],'failed_role':c['failed_role'],'raw_prefix':pin(to),
        'declared_wire_length':c['declared_wire_length'],'actual_bytes_written':c['actual_bytes_written'],
        'application_body_available':False,'client_delivery_proven':False,'post_failure_output_acquisitions':0,
        'server_poststates_separate':c['true_poststates']})
assert len(partial)==2
replay={'schema':'MODERN_R537_RESULT_RETURN_AND_PARTIAL_OFFLINE_INPUT_V1','candidate_source':v['candidate_source'],
    'per_receiver_ordinal_only':True,'cross_socket_total_order':False,'client_ACK':False,'sequences':sequences,'partial_failures':partial,
    'successful_bodies':39,'necessary_setup_bodies':17,'RESULT_return_bodies':22,'actual_handle_calls':10,
    'zero_total_catalog_marker_available':False,'zero_row_gap':index['zero_row_gap'],
    'assumption':'Only fixed offline delivery is assumed. Server poststates/flush and failed prefixes are not client receipt, occupancy or membership proof.'}
save(dest/'actual-return-replay.json',replay)
save(l/'r537-caller-return-adoption.json',{'state':'EXACT_NEW_RETURN_PROFILE_AND_PARTIAL_INPUTS_ADOPTED_READ_ONLY','root':pin(root),
    'references_checked':13,'manifest_files_checked':1036,'copied':copied,'replay':pin(dest/'actual-return-replay.json'),
    'actual_new_Java_calls':0,'SQL_TLS_provider_GUI':0,'old_profiles_retained':True,'deployed_485F_adoption':False})
print(json.dumps({'sequences':6,'successful_bodies':39,'setup':17,'return':22,'partial_frames':2,'replay':pin(dest/'actual-return-replay.json')},ensure_ascii=False))
