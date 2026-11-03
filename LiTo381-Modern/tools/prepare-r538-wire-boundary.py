import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';old=p/'target/r537-native-pet-race-return-inputs';dest=p/'target/r538-native-pet-race-wire-inputs'
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
check({'path':str(old/'new_candidate_partial_result.json'),'bytes':20828,'sha256':'C21F54225A5A62E5D4DA8C1D3FBCF805883E57732629EE0EFE536A2DA5541A55'})
check({'path':str(old/'manifest.json'),'bytes':362343,'sha256':'ECACDCAEA96A73D32A066C5144C7DA523CE3EE7562B424C2A71151ABF96887AB'})
manifest=read(old/'manifest.json');known={d['path']:d for d in manifest['files']}
result=read(old/'new_candidate_partial_result.json');dest.mkdir(exist_ok=True);assert not any(dest.iterdir());lanes=[]
for case in result['cases']:
    for role,t in enumerate(case['transport']):
        frames=t['frames']
        if not frames or not frames[0].get('raw_hex','').startswith('070001'):continue
        seed=bytes.fromhex(frames[0]['raw_hex']);assert len(seed)==7 and frames[0]['complete_flush']
        counter=(0x87546CA1<<32)|int.from_bytes(seed[3:7],'little')
        assert counter==t['initial_cipher']%(1<<64)
        logs=[pathlib.Path(s) for s in known if 'candidate\\run-final\\'+case['scenario']+'\\capture-'+str(role)+'\\' in s and s.endswith('frames.jsonl')]
        assert len(logs)==1,logs
        check(known[str(logs[0])]);records=[json.loads(s) for s in logs[0].read_text(encoding='utf-8').splitlines() if s]
        records=[r for r in records if r['direction']=='server_to_client']
        complete=[f for f in frames if f['complete_flush']];assert len(records)==len(complete)
        for r,f in zip(records,complete):assert r['wire_length']==f['bytes_written'] and r['direction']=='server_to_client'
        assert bytes.fromhex(records[0]['plain'])==seed[2:]
        data=b''.join(bytes.fromhex(f['raw_hex']) for f in frames)
        assert all(f['ordinal']==i+1 and f['bytes_written']==len(bytes.fromhex(f['raw_hex'])) for i,f in enumerate(frames))
        ident=case['scenario']+'-receiver'+str(role);wire=dest/(ident+'.wire.bin');wire.write_bytes(data)
        log=dest/(ident+'.frames.jsonl');shutil.copyfile(logs[0],log)
        lane={'id':ident,'session_binding':case['scenario']+'/receiver'+str(role)+'/5350/native-run-memory',
            'source':'5350B36DD93F2EA1FC9F00639073A1C990C759B26F424BCE704481B646BC959E','seed_frame_hex':seed.hex().upper(),
            'wire':pin(wire),'capture':pin(log),'server_complete_application_bodies':[bytes.fromhex(r['plain']).hex().upper() for r in records[1:]],
            'partial_eof':not frames[-1]['complete_flush'],'last_raw_frame_hex':frames[-1]['raw_hex'].upper(),
            'declared_last_frame_bytes':int.from_bytes(bytes.fromhex(frames[-1]['raw_hex'])[:2],'little'),
            'actual_session_key_emitted':True,'client_received':False,'transport_kind':'actual ClientSession.run on MemorySocket; not TCP loopback'}
        lanes.append(lane)
assert len(lanes)==3 and sum(len(r['server_complete_application_bodies']) for r in lanes)==2
binding={'schema':'MODERN_R538_PINNED_5350_OFFLINE_WIRE_BINDINGS_V1','source_root_sha':'66EDC23BAA24A9C98D9837E7FC73A02CB64AB37BEEEE4E62AFEF28113C4502BB',
    'lanes':lanes,'no_cross_receiver_key_reuse':True,'offline_only':True,'wire_generation_proven':False,
    'missing':['Each of six RESULT-return receiver streams lacks its own emitted S_KEY/initial wire and counter-continuity evidence; fixed memory preset counters are not emitted keys.',
        'New owner actual loopback/raw-prefix/EOF/healthy-peer streams and close/read interleaving handoff not adopted yet.',
        'Real delivery ACK, live WORLD/admitted roster, wire generation and catalog completion marker.']}
save(dest/'wire-session-bindings.json',binding)
save(l/'r538-receiver-reuse-and-source-audit.json',{'state':'EXISTING_NATIVE_INCREMENTAL_RECEIVER_SELECTED_WITH_THREE_OWNED_KEYED_TRACES',
    'receiver':'src/modern_lito/native_source_stream.rs::Receiver::next/end','existing_cipher_decoder_reimplemented':False,
    'runtime_dispatch_not_reused':'native_city_runtime::feed_source_stream_candidate is coupled to distinct actor/login/AD368 dispatcher; R537 consumer uses only the existing underlying Receiver.',
    'LTM1_transport_not_used':True,'source_bindings':pin(dest/'wire-session-bindings.json'),'keyed_memory_lanes':3,
    'actual_complete_application_bodies_with_matching_S_KEY':2,'partial_EOF_lanes':2,'missing':binding['missing'],
    'search_error_preserved':'Initial rg supplied literal filename wildcard operands under Windows and returned three invalid-filename errors; switched to exact files and rg --glob. No source mutation from that search.',
    'preparation_error_preserved':'Initial records count assertion included an actual client_to_server C_D9 record; failed before payload output. Corrected to select server_to_client records. A patch attempt including a nonexistent line also failed atomically before any edit.',
    'actual_new_Java_SQL_provider_GUI':0,'new_full_bindings':0})
print(json.dumps({'binding':pin(dest/'wire-session-bindings.json'),'lanes':3,'keyed_complete_bodies':2,'partial_lanes':2},ensure_ascii=False))
