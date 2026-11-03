import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';dest=p/'target/r539-native-pet-race-result-wire-inputs'
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
root=p.parent.parent/'lito伺服器/Native-PetRace-RESULT同session-key交接.json'
assert pin(root)['sha256']=='CA3C890B1FD8664428A2825916EEE4F3574D1CC4A0F47BEC4723747B84F6B782'
v=read(root);refs=[d for d in v.values() if isinstance(d,dict) and {'path','bytes','sha256'}<=d.keys()];assert len(refs)==13
for d in refs:check(d)
m=read(pathlib.Path(v['manifest']['path']));assert len(m['files'])==617;owner=pathlib.Path(v['root']).resolve()
for d in m['files']:assert pathlib.Path(d['path']).resolve().is_relative_to(owner);check(d)
ep=p.parent.parent/'lito伺服器/Native-PetRace-RESULT-endpoint來源補註.json'
assert pin(ep)['sha256']=='225994CD3397014C9303C63371DD5367A94F9DD36F40F0C15950CCCC85CBA8CA'
e=read(ep);er=[d for d in e.values() if isinstance(d,dict) and {'path','bytes','sha256'}<=d.keys()];assert len(er)==6
for d in er:check(d)
harness=pathlib.Path(e['frozen_executed_harness_source']['path']).read_text(encoding='utf-8').splitlines()
for x in e['source_excerpts']:assert harness[x['line']-1]==x['text']
dest.mkdir();copied=[]
for key in ['manifest','consumer_index','verification','source_class_runtime_binding','actual_result','actual_own_key_readback','ownership','listener_postcheck','legacy_key_audit']:
    f=dest/(key+'.json');shutil.copyfile(v[key]['path'],f);copied.append(pin(f))
shutil.copyfile(root,l/'r539-result-server-root.json');shutil.copyfile(ep,dest/'endpoint-provenance.json')
idx=read(pathlib.Path(v['consumer_index']['path']));result=read(pathlib.Path(v['actual_result']['path']));lanes=[];rxn=txn=0
for s in idx['streams']:
    ident=s['connection_id'].replace('/','-');check(s['raw_received']);check(s['raw_sent']);check(s['actual_own_S_KEY']['wire_pin'])
    rx=pathlib.Path(s['raw_received']['path']).read_bytes();tx=pathlib.Path(s['raw_sent']['path']).read_bytes();seed=pathlib.Path(s['actual_own_S_KEY']['wire_pin']['path']).read_bytes()
    assert seed==rx[:7] and seed[:3]==b'\x07\x00\x01' and s['actual_own_S_KEY']['offset']==0 and s['first_encrypted_server_frame_offset']==7
    initial=(0x87546ca1<<32)|s['actual_own_S_KEY']['seed_unsigned'];rows=[];offset=7;counter=initial;alias=b''
    for f in s['server_to_counterpart_frames']:
        for k in ['application_pin','wire_pin','same_application_as_66ED']:check(f[k])
        body=pathlib.Path(f['application_file']).read_bytes();raw=pathlib.Path(f['wire_file']).read_bytes()
        assert body==bytes.fromhex(f['application_hex'])==pathlib.Path(f['same_application_as_66ED']['path']).read_bytes()
        assert f['full_frame'] and f['raw_offset']==offset and rx[offset:offset+len(raw)]==raw and f['ordinal']==len(rows)+2
        assert f['cipher_before']%(1<<64)==counter and f['cipher_after']%(1<<64)==counter+len(body)
        if body[0]==0xd0 and s['role'] in body[5:-1].decode('ascii'):alias=body[5:-1]
        app=dest/(ident+f'-rx-body{len(rows):02}.bin');shutil.copyfile(f['application_file'],app);copied.append(pin(app))
        rows.append({'offset':offset,'length':len(raw),'body_hex':body.hex().upper(),'counter_before':counter,'counter_after':counter+len(body)})
        offset+=len(raw);counter+=len(body)
    assert offset==len(rx) and s['actual_EOF'] and not s['partial_frames'];client=[];offset=0;counter=initial
    for f in s['client_to_server_frames']:
        check(f['application_pin']);check(f['wire_pin']);body=pathlib.Path(f['application_file']).read_bytes();raw=pathlib.Path(f['wire_file']).read_bytes()
        assert body in [b'\xd9',b'\xd0'] and tx[offset:offset+len(raw)]==raw and f['raw_offset']==offset and f['cipher_before']%(1<<64)==counter and f['cipher_after']%(1<<64)==counter+1
        # Chosen replay positions before the corresponding response, derived from source stages;
        # no claim about a Modern socket receive/write schedule or global cross-socket ordering.
        response=next(x for x in rows if (x['body_hex']=='F000' if body==b'\xd9' else x['body_hex'].startswith('D0') and bytes.fromhex(x['body_hex'])[5:-1]==alias))
        client.append({'opcode':body[0],'frame_hex':raw.hex().upper(),'counter_before':counter,'counter_after':counter+1,'replay_after_rx_offset':response['offset']})
        offset+=len(raw);counter+=1
    assert offset==len(tx);case=next(c for c in result['cases'] if c['scenario']==s['scenario']);state=next(x for x in case['proof']['fixture_before_run']['sessions_and_target_object_diagnostics']['sessions'] if x['role']==s['role'])
    assert state['phase']==('CHANNEL' if s['role']=='quitter' else 'RESULT') and state['currentRoomId']==(0 if s['role']=='quitter' else (3 if 'nondefault' in s['scenario'] else 2))
    assert not state['in_room_members'] if s['role']=='quitter' else state['in_room_members']
    to=dest/(ident+'.wire.bin');shutil.copyfile(s['raw_received']['path'],to);copied.append(pin(to));out=dest/(ident+'.sent.bin');shutil.copyfile(s['raw_sent']['path'],out);copied.append(pin(out))
    lanes.append({'id':ident,'session_binding':s['connection_id']+'/5350/new-RESULT-start-own-key','source':v['candidate_source']['sha256'],
        'seed_frame_hex':seed.hex().upper(),'wire':pin(to),'fixture':{'room_id':state['currentRoomId'],'alias_hex':alias.hex().upper(),'initial_result':state['phase']=='RESULT','client_frames':client},
        'server_frames':rows,'sent':pin(out),'scenario':s['scenario'],'role':s['role'],'actual_EOF':True})
    rxn+=len(rows);txn+=len(client)
assert (len(lanes),rxn,txn)==(6,22,8)
binding={'schema':'MODERN_R539_PINNED_5350_RESULT_START_WIRE_V1','source_root_sha':pin(root)['sha256'],'lanes':lanes,'offline_only':True,'synthetic_previous_combat_or_D3':False,'historical_66ED_rekeyed':False,'replay_positions_are_offline_assumptions':True}
save(dest/'wire-session-bindings.json',binding)
save(l/'r539-result-wire-adoption.json',{'root':pin(root),'references_checked':13,'manifest_files_checked':617,'endpoint_supplement':pin(ep),'endpoint_refs_checked':6,'exact_source_excerpts_checked':9,'binding':pin(dest/'wire-session-bindings.json'),'copied':copied,'streams':6,'server_bodies':22,'client_commands':8,'own_keys':6,'old_66ED_repaired':False,'new_Modern_socket_or_server_runs':0,'endpoint_provenance':'server_endpoint is accepted s.actual post-FIN/join/EOF accessor output; 127.0.0.1:port derived from source172/177 plus previously saved runtime listener port; no new live bind observation.'})
print(json.dumps({'binding':pin(dest/'wire-session-bindings.json'),'counts':[6,22,8],'checked':[13,617,6,9]},ensure_ascii=False))
