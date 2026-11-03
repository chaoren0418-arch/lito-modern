import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';dest=p/'target/r538-native-pet-race-loopback-wire-inputs'
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
root=p.parent.parent/'lito伺服器/Native-PetRace-5350-loopback交錯交接.json'
check({'path':str(root),'bytes':5305,'sha256':'EC96F1E4D388A4D9DCDC634537A191842B4D02606EF706677A071475EBDB947A'})
v=read(root);refs=[d for d in v.values() if isinstance(d,dict) and {'path','bytes','sha256'}<=d.keys()]+v['parent_263A_64A40_5ABC_unchanged']
assert len(refs)==14
for d in refs:check(d)
m=read(pathlib.Path(v['manifest']['path']));assert len(m['files'])==642;owner=pathlib.Path(v['root']).resolve()
for d in m['files']:assert pathlib.Path(d['path']).resolve().is_relative_to(owner);check(d)
dest.mkdir();shutil.copyfile(root,l/'r538-loopback-server-root.json');copied=[]
for key in ['manifest','version_runtime_contract','verification','actual_result','actual_received_consumer_index','actual_readback','process_ownership','listener_postcheck']:
    to=dest/(key+'.json');shutil.copyfile(v[key]['path'],to);copied.append(pin(to))
idx=read(pathlib.Path(v['actual_received_consumer_index']['path']));lanes=[];total=0
for s in idx['streams']:
    check(s['raw_received']);wire=pathlib.Path(s['raw_received']['path']).read_bytes();seed=bytes.fromhex(s['S_KEY_raw_hex']);assert len(seed)==7 and seed[:3]==b'\x07\x00\x01' and wire.startswith(seed)
    ident=s['mode']+'-'+s['role'];to=dest/(ident+'.wire.bin');shutil.copyfile(s['raw_received']['path'],to);copied.append(pin(to))
    offset=7;plain=[];frame_rows=[]
    for f in s['complete_application_frames']:
        check(f['application_pin']);check(f['wire_pin']);body=pathlib.Path(f['application_file']).read_bytes();raw=pathlib.Path(f['wire_file']).read_bytes()
        assert f['offset']==offset and f['ordinal']==len(plain)+2 and wire[offset:offset+len(raw)]==raw
        assert body==bytes.fromhex(f['application_hex']) and len(raw)==len(body)+2 and f['received_complete'] and f['client_ACK'] is False
        app=dest/(ident+f'-body{len(plain):02}.bin');shutil.copyfile(f['application_file'],app);copied.append(pin(app))
        frame_rows.append({'offset':offset,'ordinal':f['ordinal'],'body':pin(app),'body_hex':body.hex().upper()});plain.append(body.hex().upper());offset+=len(raw)
    tail=wire[offset:];assert bool(tail)==bool(s['partial_tail'])
    if tail:
        check(s['partial_tail']['raw_pin']);assert tail==pathlib.Path(s['partial_tail']['raw_pin']['path']).read_bytes() and len(tail)==5
        assert s['partial_tail']['actual_EOF'] and s['partial_tail']['application_body_available'] is False
    else:assert offset==len(wire)
    total+=len(plain)
    lanes.append({'id':ident,'session_binding':s['mode']+'/'+s['role']+'/5350/actual-loopback-received',
        'source':v['candidate_source']['sha256'],'seed_frame_hex':seed.hex().upper(),'wire':pin(to),
        'server_complete_application_bodies':plain,'complete_received_frames':frame_rows,'partial_eof':bool(tail),
        'last_raw_frame_hex':tail.hex().upper(),'owner_actual_EOF_before_healthy_teardown':s['actual_EOF_before_healthy_owned_teardown'],
        'role':s['role'],'mode':s['mode'],'actual_session_key_emitted':True,'owner_actual_received':True,
        'Modern_delivery_during_replay_assumed':True,'full_player_ACK':False})
assert len(lanes)==6 and total==13
binding={'schema':'MODERN_R538_PINNED_5350_OFFLINE_WIRE_BINDINGS_V1','source_root_sha':'EC96F1E4D388A4D9DCDC634537A191842B4D02606EF706677A071475EBDB947A',
    'lanes':lanes,'offline_only':True,'owner_received_application_bodies':13,'partial_tails':2,'cross_socket_total_order':False,
    'original_66ED_RESULT_lane_keys_supplied':False}
save(dest/'wire-session-bindings.json',binding)
save(l/'r538-loopback-wire-adoption.json',{'state':'SAME_5350_ACTUAL_LOOPBACK_RECEIVED_STREAMS_PINNED_FOR_OFFLINE_DECODER_REPLAY',
    'root':pin(root),'references_checked':14,'manifest_files_checked':642,'copied':copied,'binding':pin(dest/'wire-session-bindings.json'),
    'streams':6,'owner_actual_received_bodies':13,'partial_tails':2,'new_server_or_socket_execution':0,
    'old_66ED_RESULT_keys_not_supplied':True,'healthypeer_EOF_before_teardown_proven':False,'no_process_or_listener_reuse':True,
    'SQL_TLS_provider_GUI':0,'new_full_bindings':0})
print(json.dumps({'binding':pin(dest/'wire-session-bindings.json'),'streams':6,'bodies':13,'partial_tails':2},ensure_ascii=False))
