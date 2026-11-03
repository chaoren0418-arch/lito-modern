import hashlib,json,pathlib,re,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';dest=p/'target/r540-native-pet-race-start-inputs'
def read(f):return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
 h=hashlib.sha256()
 with f.open('rb') as s:
  for b in iter(lambda:s.read(1024*1024),b''):h.update(b)
 return {'path':str(f),'bytes':f.stat().st_size,'sha256':h.hexdigest().upper()}
def check(d):
 a=pin(pathlib.Path(d['path']));assert (a['bytes'],a['sha256'])==(d['bytes'],d['sha256'].upper()),a
def pins(v):
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():yield v
  else:
   for x in v.values():yield from pins(x)
 elif isinstance(v,list):
  for x in v:yield from pins(x)
def save(f,v):
 with f.open('x',encoding='utf-8') as s:json.dump(v,s,ensure_ascii=False,indent=2);s.write('\n')
root=p.parent.parent/'lito伺服器/Native-PetRace-開始clock與roster-component交接.json';assert pin(root)['sha256']=='3AD211071920145A5F86F17FD7CD033AF17D2BF728D81E47B280C7C30C2F6C2B'
v=read(root);refs=list(pins(v));assert len(refs)==15
for d in refs:check(d)
m=read(pathlib.Path(v['manifest']['path']));assert len(m['files'])==1027;owner=pathlib.Path(v['root']).resolve()
for d in m['files']:assert pathlib.Path(d['path']).resolve().is_relative_to(owner);check(d)
supp=p.parent.parent/'lito伺服器/Native-PetRace-開始component-BattleRules版本補註-r2.json';assert pin(supp)['sha256']=='1DC4F7DADD4612D9B63D09C9C6B66E3AE49129C85FE66469943DBB5F0D47C93C'
b=read(supp);br=list(pins(b));assert len(br)==18
for d in br:check(d)
assert b['recompile_class_counts']=={'all':183,'byte_equal':180,'different':3} and b['whole_class_equivalence'] is False
norm=lambda s:'\n'.join(re.sub(r'#\d+','#CP',' '.join(x.split())) for x in s.splitlines() if x.strip())
assert len(b['selected_start_methods'])==4
for x in b['selected_start_methods']:assert x['normalized_equal'] and norm(pathlib.Path(x['runtime']['path']).read_text(encoding='utf-8'))==norm(pathlib.Path(x['recompiled']['path']).read_text(encoding='utf-8'))
dest.mkdir();copied=[]
for k in ['manifest','contract','consumer_index','verification','actual_result','actual_own_key_readback','ownership','listener_postcheck','source_class_audit']:
 to=dest/(k+'.json');shutil.copyfile(v[k]['path'],to);copied.append(pin(to))
shutil.copyfile(root,l/'r540-start-server-root.json');shutil.copyfile(supp,dest/'BattleRules-r2.json')
for i,x in enumerate(b['selected_start_methods']):
 for direction in ['runtime','recompiled']:shutil.copyfile(x[direction]['path'],dest/f'method-{i}-{direction}.txt')
c=read(pathlib.Path(v['contract']['path']));a=read(pathlib.Path(v['actual_result']['path']));proof=a['proof'];idx=read(pathlib.Path(v['consumer_index']['path']))
for e in c['method_source_excerpts']:
 check(e['source']);lines=pathlib.Path(e['source']['path']).read_text(encoding='utf-8').splitlines()
 for x in e['lines']:assert lines[x['line']-1]==x['text']
proto=owner/'source-audit/Protocol.java';pr=next(d for d in m['files'] if pathlib.Path(d['path'])==proto);check(pr);lines=proto.read_text(encoding='utf-8').splitlines()
save(dest/'protocol-actor-source-excerpt.json',{'source':pr,'first_line':7440,'last_line':7478,'lines':[{'line':n,'text':lines[n-1]} for n in range(7440,7479)],'Protocol_source_class_is_in_180_exact_byte_matches':True})
initial=c['actual_initial'];begin=c['actual_begin'];prep=proof['after_actual_all_member_prepare']['sessions_and_target_object_diagnostics'];cleanup=proof['cleanup_actual_endBattle_before_FIN']['sessions_and_target_object_diagnostics'];ticks=proof['actual_countdownWork_calls'];movement=c['actual_movement']
assert (initial['battleGeneration'],begin['battleGeneration'],cleanup['battleGeneration'])==(11,12,13)
assert begin['battle_roster']==begin['match_participants']==begin['race_runner_ids']==[101,102,103]
assert begin['race_deadlineNanos']==begin['match_deadlineNanos']==begin['race_startedNanos']+300_000_000_000
assert all(x['phase']=='ROOM' for x in initial['sessions']+begin['sessions']+prep['sessions']+cleanup['sessions'])
members=[{'object_id':x['objectId'],'alias_hex':x['alias'].encode('ascii').hex().upper(),'prepared_x':x['positionX'],'prepared_y':x['positionY'],'prepared_gfx':x['petRaceBaseGfx']} for x in prep['sessions']]
events=[{'kind':'CountdownStarted','after_rx_offset':107,'value':3,'source_now':None,'source_next':proof['actual_READY_countdown_started']['sessions_and_target_object_diagnostics']['countdownNextNanos']}]
for index,tick in enumerate(ticks):
 d=tick['after']['sessions_and_target_object_diagnostics'];events.append({'kind':'Begin' if index==2 else 'Tick','after_rx_offset':115 if index else 111,'value':d['countdownValue'],'source_now':tick['now_argument_actual_System_nanoTime'],'source_next':d['countdownNextNanos']})
events.extend([{'kind':'Prepare','after_rx_offset':115,'value':0,'source_now':None,'source_next':0},{'kind':'DirectGate','after_rx_offset':423,'value':0,'source_now':movement['now_actual_System_nanoTime'],'source_next':0},{'kind':'Teardown','after_rx_offset':423,'value':0,'source_now':None,'source_next':0}])
common={'room_id':2,'map_id':566,'members':members,'generation_initial':11,'generation_begin':12,'generation_teardown':13,'source_start_nanos':begin['race_startedNanos'],'source_deadline_nanos':begin['race_deadlineNanos'],'milestones':events,'direct_gate':{'object_id':101,'from':movement['from'],'to':movement['to'],'elapsed_nanos':movement['now_actual_System_nanoTime']-begin['race_startedNanos']},'minimum_three_is_fixture_precondition':True,'scene_prerequisites_qualified':False}
lanes=[];replays=[];count=0
for s in idx['streams']:
 for k in ['raw_received','raw_sent','own_S_KEY']:check(s[k])
 rx=pathlib.Path(s['raw_received']['path']).read_bytes();tx=pathlib.Path(s['raw_sent']['path']).read_bytes();seed=pathlib.Path(s['own_S_KEY']['path']).read_bytes();assert seed==rx[:7] and seed[:3]==b'\x07\x00\x01' and s['own_S_KEY_offset']==0
 counter=(0x87546ca1<<32)|s['seed_unsigned'];offset=7;rows=[];ident=s['role'];file=dest/(ident+'.wire.bin');shutil.copyfile(s['raw_received']['path'],file);sent=dest/(ident+'.sent.bin');shutil.copyfile(s['raw_sent']['path'],sent)
 for f in s['received_application_frames']:
  check(f['application']);check(f['wire']);body=pathlib.Path(f['application_file']).read_bytes();raw=pathlib.Path(f['wire_file']).read_bytes();assert body==bytes.fromhex(f['application_hex']) and rx[offset:offset+len(raw)]==raw and f['raw_offset']==offset and f['cipher_before']%(1<<64)==counter and f['cipher_after']%(1<<64)==counter+len(body)
  app=dest/f'{ident}-body{len(rows):02}.bin';shutil.copyfile(f['application_file'],app)
  rows.append({'offset':offset,'length':len(raw),'body_hex':body.hex().upper(),'counter_before':counter,'counter_after':counter+len(body),'teardown':len(rows)>=12});counter+=len(body);offset+=len(raw)
 assert offset==len(rx) and not s['partial_frames'] and s['FULL'] and s['component_body_count_before_teardown']==12
 f=s['sent_client_frames'][0];assert len(s['sent_client_frames'])==1 and f['application_hex']=='d201' and len(tx)==4 and tx==pathlib.Path(f['wire_file']).read_bytes()
 source_start=common|{'local_object_id':101+int(ident[-1]),'ready_frame_hex':tx.hex().upper()}
 lanes.append({'id':ident,'session_binding':s['connection_id']+'/5350/ROOM-start-component-own-key','source':v['same5350_source']['sha256'],'seed_frame_hex':seed.hex().upper(),'wire':pin(file),'start':source_start})
 replays.append({'id':ident,'server_frames':rows,'sent':pin(sent),'source_component_bodies':12,'actual_source_EOF':True,'live_endpoints':s['live_runtime_endpoints']});count+=len(rows)
assert count==42 and len(lanes)==3
save(dest/'wire-session-bindings.json',{'schema':'MODERN_R540_PINNED_5350_ROOM_START_COMPONENT_V1','offline_only':True,'lanes':lanes,'BattleRules_equivalence_scope':'Only four recorded normalized countdown/phase/begin method blocks; runtime baseline classes differ from source recompile for three classes.'})
save(dest/'per-frame-replay.json',{'lanes':replays,'source_bodies':42,'start_bodies':36,'teardown_bodies':6,'actual_READY_inputs':3,'local_generation_is_not_wire_generation':True})
save(l/'r540-start-source-adoption.json',{'root':pin(root),'root_refs_checked':15,'manifest_files_checked':1027,'supplement':pin(supp),'supplement_refs_checked':18,'normalized_method_pairs':4,'recompile_counts':b['recompile_class_counts'],'whole_BattleRules_equivalence':False,'binding':pin(dest/'wire-session-bindings.json'),'replay':pin(dest/'per-frame-replay.json'),'copied':copied,'source_excerpts_checked':len(c['method_source_excerpts']),'streams':3,'own_keys':3,'start_bodies':36,'teardown_bodies':6,'client_READY_inputs':3,'no_new_server_or_Modern_socket':True,'clock_domain':'saved server System.nanoTime values are source-only metadata, no local/UTC synchronization','full_WORLD_and_BATTLE':False})
print(json.dumps({'binding':pin(dest/'wire-session-bindings.json'),'source_checked':[15,1027,18,4],'bodies':[36,6],'clock_duration':300_000_000_000},ensure_ascii=False))
