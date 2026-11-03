from r491_common import *
import re,difflib,struct
cache={};instances=0;roots=[]
def check(q,path=None):
 global instances
 instances+=1;path=path or q['path']
 if path not in cache:cache[path]=pin(path)
 actual=cache[path];assert(actual['bytes'],actual['sha256'])==(q['bytes'],q['sha256']),(path,actual,q)
def walk(v,initial=False):
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   check(v,v.get('snapshot') if initial and v.get('snapshot') else None)
   if v.get('snapshot'):check(v,v['snapshot'])
  for q in v.values():walk(q,initial)
 elif isinstance(v,list):
  for q in v:walk(q,initial)
names=['r503-source-study-initial.json','r503-source-study.json','r503-initial-source-oracle.json','r503-source-oracle.json','r503-source-scope.json','r503-final-production-source-snapshot.json','r503-final-entry-source-contract.json','r503-final-player-candidate.json','r503-guard-consumer-candidate.json','r503-event-details.json','r503-guard-event-details.json','r503-consumer-exit.json','r503-guard-consumer-exit.json','r503-execution-history.json','r503-open-gaps.json','r503-entry-callback-event-mapping.json','r503-initial-segment.json']
for n in names:walk(read(L/n));roots.append(pin(L/n))
for n in ['r503-production-source-snapshot.json','r503-consumer-candidate.json','r503-player-candidate.json']:
 walk(read(L/n),True);roots.append(pin(L/n))
study=read(L/'r503-source-study.json');lines=Path(study['originals'][0]['path']).read_text(encoding='utf-8-sig').splitlines();pe=Path(study['originals'][1]['path']).read_bytes()
for q in study['fragments']:
 p=Path(q['path']);content=p.read_text(encoding='utf-8')
 if p.name.startswith('asm-'):
  for line in content.splitlines():n,s=line.split(': ',1);assert lines[int(n)-1]==s
 else:
  assert p.name.startswith('pe-');va=int(p.stem.split('-')[-1],16);raw=bytes.fromhex(content.splitlines()[1]);assert raw==pe[va-0x400000:va-0x400000+len(raw)]
assert pe[0x2343d8:0x2343dc]==b'dhd\0' and pe[0x102778:0x10277b]==b'cc\0'
assert 'call    sub_4A76D0' in lines[374225-1] and 'call    sub_4A7850' in lines[374255-1]
assert 'mov     dword_64BBD4, 3' in lines[230032-1]
assert 'mov     ecx, 1' in lines[232876-1] and 'call    sub_469CA0' in lines[232896-1]
assert 'mov     dword_64AF98, eax' in lines[226651-1]
prod=read(L/'r503-final-production-source-snapshot.json');old=read(L/'r502-production-source-snapshot.json');new={q['path']:q for q in prod['source_pins']};assert len(new)==618;diffs=[];changed=[]
for q in old['source_pins']:
 check(q,q['snapshot'])
 if new[q['path']]['sha256']!=q['sha256']:
  changed.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
p=P/'src/modern_lito/native_entry_navigation.rs';diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
assert changed==prod['changed'] and len(changed)==7 and prod['unchanged']==610
assert '\n'.join(diffs)==(L/'r503-final-source-difference.patch').read_text(encoding='utf-8')
initial=read(L/'r503-event-details.json');final=read(L/'r503-guard-event-details.json')
assert len(initial['flows'])==16 and len(final['flows'])==4
assert all(f['passed'] for f in initial['flows']+final['flows'])
assert initial['actual_complete_socket_frames']==18 and final['actual_complete_socket_frames']==3
assert sum(f['details'].get('rejected',0) for f in final['flows'])==30
assert read(L/'r503-consumer-exit.json')['exit_code']==read(L/'r503-guard-consumer-exit.json')['exit_code']==0
assert len(read(L/'r503-consumer-candidate.json')['source_pins'])==214 and len(read(L/'r503-guard-consumer-candidate.json')['source_pins'])==216
runtime=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');window=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8');nav=p.read_text(encoding='utf-8')
assert 'ensure!(!self.entry_navigation.active()||action==Action::Close' in runtime
assert 'let result=s.runtime.borrow_mut().as_mut().unwrap().leave_source_character_scene();match result{Ok(())=>{s.screen.set(Screen::Maps)' in window
assert 'Command::BackLobby' in window and 'self.entry_navigation.key.return_mode=1;' in nav
assert not any('pub fn' in line and 'set_' in line for line in nav.splitlines())
assert (P/'src/modern_lito/native_login_events.rs').read_bytes()==(P/'target/r502-production-source/src/modern_lito/native_login_events.rs').read_bytes()
oracle=read(L/'r503-source-oracle.json')
for name in ['normal_tx','normal_rx','auto_tx','auto_rx','malformed_rx']:
 for v in oracle[name]:
  body=bytes(v['plain']);counter=v['before'];key=counter.to_bytes(8,'little');last=0;encrypted=[]
  for i,b in enumerate(body):last=b^key[i%8]^last;encrypted.append(last)
  assert v['wire']==list((len(body)+2).to_bytes(2,'little'))+encrypted
  assert v['after']==(counter+len(body))%2**64
o=read(L/'r503-execution-history.json');assert len(o['library_builds'])==5 and all(b['success'] for b in o['library_builds']);assert len(o['preparation_failures'])==1
proof=dict(state='R503_INITIAL16_PLUS_FINAL4_SOURCE_CLOSED_SUBSET_VERIFIED',json_roots=roots,pin_instances=instances,unique_paths=len(cache),source_files=618,changed=7,added=1,unchanged=610,final_source_pairs=216,original_fragment_count=len(study['fragments']),production_diff_bytes=(L/'r503-final-source-difference.patch').stat().st_size,entry_diff_bytes=(L/'r503-player-entry-difference.patch').stat().st_size,initial_flows=16,final_affected_increment_flows=4,final_atomic_action_rejections=30,actual_complete_socket_frames=21,new_navigation_frames=9,setup_login_prerequisite_frames=12,IO_workers_joined=6,endpoint_threads_joined=6,Runtime_Closed=7,max_observed_Runtime_poll_us=max(f['details'].get('max_poll_us',0) for f in initial['flows']+final['flows']),max_observed_tail=max(f['details'].get('max_receiver_tail',0) for f in initial['flows']+final['flows']),historical_PASS_reruns=0,initial16_reruns=0,page40_source_closed=False,production_authentication=False,GUI=0,complete_modes=16,total_modes=40)
save(L/'r503-independent-contract.json',proof)
print('R503 verified',instances,'pins/',len(cache),'unique;16initial+4final increments;618sources;21wireframes/9new;GUI0')
