from r491_common import *
import re,struct,difflib
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   instances+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['sha256']==v['sha256']and cache[p]['bytes']==v['bytes'],p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
names=['r502-source-study.json','r502-source-oracle.json','r502-production-source-snapshot.json','r502-source-scope.json','r502-entry-source-contract.json','r502-player-candidate.json','r502-consumer-candidate.json','r502-consumer-exit.json','r502-event-details.json','r502-entry-callback-event-mapping.json','r502-open-gaps.json','r502-documentation-correction.json']
for n in names:walk(read(L/n))
study=read(L/'r502-source-study.json');asm=Path(study['original_disassembly']['path']).read_text(encoding='utf-8');lines=asm.splitlines();pe=Path(study['analysis_PE']['path']).read_bytes();assert len(study['fragments'])==12
for q in study['fragments']:
 p=Path(q['path'])
 if 'name'in q:
  m=re.search('^'+q['name']+r' proc near',asm,re.M);z=re.search(r'^sub_[0-9A-F]+ proc near',asm[m.end():],re.M);assert p.read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()];assert q['line_start']==asm[:m.start()].count('\n')+1
 elif p.name.startswith('PE-initial-'):
  address=int(p.stem.split('-')[-1],16);assert p.read_bytes()==pe[address-0x400000:address-0x400000+q['bytes']]
 else:
  for line in p.read_text(encoding='utf-8').splitlines():
   n,text=line.split(': ',1);assert lines[int(n)-1]==text
for q in study['direct_references']:assert lines[q['line']-1]==q['text']
assert len(study['direct_references'])==90 and not study['direct_5033C4_destination_writes']
assert [q['initial_value']for q in study['PE_values']]==[0x114F8,0xFFFFFFFF,0]
nt=struct.unpack_from('<I',pe,0x3C)[0];sect=nt+24+struct.unpack_from('<H',pe,nt+20)[0];flags=struct.unpack_from('<I',pe,sect+36)[0];assert flags==0xE0000060
assert all(q['section']['writable']and q['section']['flags']=='0xe0000060'for q in study['PE_values'])
assert all(not q['current_Native_qualified']for q in study['fields'])
assert Path(study['existing_Modern_profile']['path']).read_bytes()==Path(study['existing_Modern_profile_snapshot']['path']).read_bytes()
prod=read(L/'r502-production-source-snapshot.json');old=read(L/'r501-production-source-snapshot.json');new={q['path']:q for q in prod['source_pins']};assert len(new)==617;changed=[];diffs=[]
for q in old['source_pins']:
 walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
 if new[q['path']]['sha256']!=q['sha256']:
  changed.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
added=['native_login_events.rs','native_player_entry_commands.rs']
for n in added:
 p=P/'src/modern_lito'/n;diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
assert set(changed)=={'native_city_runtime.rs','native_city_window.rs','native_login_transport.rs'}and '\n'.join(diffs)==Path(prod['source_difference']['path']).read_text(encoding='utf-8')and prod['source_difference']['bytes']==71952
for q in prod['source_pins']:walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
candidate=read(L/'r502-consumer-candidate.json');player=read(L/'r502-player-candidate.json');assert len(candidate['source_pins'])==211 and candidate['production_library']==player['production_library']and player['candidate_executions']==0
for q in candidate['source_pins']:walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
tests=read(L/'r502-event-details.json');assert tests['flow_cases']==14 and tests['requested_cases']=='all'and tests['consumer_executions']==1 and read(L/'r502-consumer-exit.json')['exit_code']==0
assert tests['historical_PASS_reruns']==0 and not tests['formal_authentication_connected']and not tests['native_GUI_or_HWND']and tests['GUI']==0
by={}
for q in tests['trace']:by.setdefault(q['case'],[]).append(q)
assert set(by)=={'bounds','normal_local','normal_roundtrip','special_roundtrip','late_connect','late_read','late_write','unsupported59','header_eof','connect_failure'}
assert by['bounds'][0]['details']['rejections']==11 and by['normal_local'][0]['details']['workers_started']==0
final_metrics=[]
for name in ['normal_roundtrip','special_roundtrip']:
 assert len(by[name])==3 and by[name][-1]['details']['exact_source_frames']==6
 assert not by[name][0]['details']['Complete_is_authsuccess']
 m=by[name][-1]['details']['metrics'];assert m['workers_started']==m['workers_joined']==1 and len(m['partial_offsets'])>0;final_metrics.append(m)
assert final_metrics[0]['max_receiver_tail']==64 and final_metrics[1]['actual_read_events']==1027
for name in ['late_connect','late_read','late_write','unsupported59','header_eof','connect_failure']:
 m=by[name][0]['details'];assert m['workers_started']==m['workers_joined']==1 and m['max_event_queue']<=4;final_metrics.append(m)
assert by['late_read'][0]['details']['max_event_queue']==4 and by['late_read'][0]['details']['late_events_discarded']==4
assert by['late_write'][0]['details']['late_events_discarded']==1 and by['late_write'][0]['details']['actual_write_reports']==1
assert by['late_connect'][0]['details']['actual_read_events']==0
max_poll=max(m['max_poll_us']for m in final_metrics);assert max_poll<200000
window=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8');events=(P/'src/modern_lito/native_login_events.rs').read_text(encoding='utf-8');commands=(P/'src/modern_lito/native_player_entry_commands.rs').read_text(encoding='utf-8');runtime=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');main=Path(player['candidate_main']['path']).read_text(encoding='utf-8')
for name in ['OpenLogin','EditLogin','SubmitLogin','CancelLogin','ChooseCharacter','SubmitCharacter','BackCharacter','Close']:assert 'EntryCommand::'+name in window and 'Command::'+name in commands
for forbidden in ['TcpStream','connect_source_loopback_fixture','read_source_loopback_step','write_source_loopback_step','thread::sleep','connect_timeout']:assert forbidden not in window
assert 'const EVENTS: usize = 4;'in events and 'const POLL_BUDGET: usize = 8;'in events and 'mpsc::sync_channel(1)'in events and 'mpsc::sync_channel(EVENTS)'in events and 'socket.set_nonblocking(true)'in events
assert 'JoinHandle::is_finished'in events and 'self.login_events.close()'in runtime and 'self.login_events.idle()'in runtime and 'self.poll_player_fixture_events()'in runtime
worker=events[events.index('fn io_worker('):events.index('impl Runtime {')]
for forbidden in ['prepare_source_login','login_form','feed_source_stream','source_actor','account','password']:assert forbidden not in worker
assert '--candidate-synthetic-fixture'in main and 'FixtureProfile::load(&path)'in main and 'None => Ok(Route::Pick)'in main and 'run_synthetic_fixture'in main
assert '收到尚未支援的 59 清單回包'in runtime
assert pin(P/'src/main.rs')['sha256']=='F445E1B028DD51483BB11442C72B2A266CC47B89121B6627B0C4A5D1C7085335'
o=read(L/'r502-source-oracle.json')
for profile in o['profiles']:
 for v in profile['send_vectors']:
  raw=bytes(v['wire']);plain=bytes(v['plain']);assert struct.unpack_from('<H',raw)[0]==len(plain)+2;key=v['before'].to_bytes(8,'little');decoded=bytes(c^key[i&7]^(raw[1+i]if i else 0)for i,c in enumerate(raw[2:]));assert decoded==plain;assert v['after']==(v['before']+len(plain))&0xFFFFFFFFFFFFFFFF
save(L/'r502-candidate.json',dict(state='R502_FIXED_LIBRARY_PLAYER_SHARED_COMMAND_EVENT_CONSUMER_SAME_SOURCE',source_pins=candidate['source_pins'],production_library=player['production_library'],player=player['executable'],consumer=dict(path=candidate['path'],bytes=candidate['bytes'],sha256=candidate['sha256']),source_pairs=211,GUI=0,LOADED=False))
save(L/'r502-independent-contract.json',dict(state='R502_SOURCE_PROFILE_DIFF_ENTRY_CALLBACK_BOUNDED_EVENT_AND_CLEANUP_STATIC_CONTRACT_PASS',pin_instances=instances,unique_paths=len(cache),verified_json=[pin(L/n)for n in names],source_pairs=211,production_files=617,source_difference=prod['source_difference'],changed=changed,added=added,unchanged_previous_sources=612,source_fragments=12,direct_source_references=90,flows=14,profile_rejections=11,exact_socket_frames=16,partial_compatibility_byte=1,IO_workers_joined=8,resource_workers_retired=9,endpoint_threads_joined=6,late_event_rejections=5,max_observed_poll_us=max_poll,worker_protocol_controller=False,worker_plaintext_credentials=False,consumer_executions=1,player_executions=0,historical_PASS_reruns=0,GUI=0,LOADED=False))
print('R502 independentPASS',instances,'pins',len(cache),'unique/211pairs617source/14flows/71952bytediff;maxpoll',max_poll,'us')
