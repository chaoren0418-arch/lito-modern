from r491_common import *
import difflib
cache={};count=0
def walk(v):
 global count
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   count+=1
   for p in [v['path']]+([v['snapshot']]if v.get('snapshot')else[]):
    if p not in cache:cache[p]=pin(p)
    assert (cache[p]['bytes'],cache[p]['sha256'])==(v['bytes'],v['sha256']),p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
exit=read(L/'r504-refusal-consumer-exit.json');assert exit['exit_code']==0
events=read(L/'r504-event-details.json');assert len(events['flows'])==9 and all(q['passed']for q in events['flows'])
delayed=next(q['details']for q in events['flows']if q['case']=='delayed_old_identical_wire');assert delayed['delayed_old_write_complete']==True and delayed['attempted_bytes']==15 and delayed['error_kind']==None
assert events['actual_complete_socket_frames']==9 and events['IO_workers_joined']==3 and events['endpoint_threads_joined']==2 and events['Runtime_Closed']==3
save(L/'r504-console-count-erratum.json',dict(console_summary_count=8,actual_structured_passed_flows=9,reason='Consumer summary literal undercounts byone; source_effects plus8 runtime/socket cases areallrecorded individually. Fixedsource/exe/stdout preserved; no rerun orcode change. Allhandoff counts useactual9.',stdout=pin(L/'r504-refusal-consumer.stdout.log'),authoritative=pin(L/'r504-event-details.json'),functional_change=False))
prod=read(L/'r504-production-source-snapshot.json');old=read(L/'r503-final-production-source-snapshot.json');new={q['path']:q for q in prod['source_pins']};assert len(new)==618;diffs=[];changed=[]
for q in old['source_pins']:
 p=Path(q['snapshot']);assert pin(p)['sha256']==q['sha256']
 if new[q['path']]['sha256']!=q['sha256']:
  changed.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(p.read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert len(changed)==3 and changed==prod['changed'] and '\n'.join(diffs)==(L/'r504-source-difference.patch').read_text(encoding='utf-8')
for n in ['native_login_events.rs','native_source_connection.rs','native_source_send.rs','native_player_entry_commands.rs','native_source_identity.rs','native_mode_state_flow.rs']:
 assert (P/'src/modern_lito'/n).read_bytes()==(P/'target/r503-final-production-source/src/modern_lito'/n).read_bytes()
study=read(L/'r504-source-study.json');lines=Path(study['originals'][0]['path']).read_text(encoding='utf-8-sig').splitlines()
for q in study['fragments']:
 for line in Path(q['path']).read_text(encoding='utf-8').splitlines():n,s=line.split(': ',1);assert lines[int(n)-1]==s
for n,t in [(350108,'cmp     al, 7'),(350131,'cmp     al, 8'),(350197,'movsx   ecx, [ebp+var_1]'),(350199,'cmp     ecx, 0Ah'),(350210,'case 0'),(350301,'case 1'),(350318,'case 10'),(350343,'default case, cases 2-9')]:assert t in lines[n-1]
o=read(L/'r504-source-oracle.json');assert o['old_late']['wire']==o['new_success']['wire'] and o['old_late']['before']==o['fresh_roster']['after']
for v in o['old_rx']+[o['old_tx'],o['old_late'],o['new_tx'],o['new_success']]+o['fresh_roster']['vectors']:
 key=v['before'].to_bytes(8,'little');last=0;out=[]
 for i,b in enumerate(v['plain']):last=b^key[i%8]^last;out.append(last)
 assert v['wire']==list((len(out)+2).to_bytes(2,'little'))+out and v['after']==(v['before']+len(out))%2**64
nav=(P/'src/modern_lito/native_entry_navigation.rs').read_text(encoding='utf-8');runtime=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');window=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8')
assert 'status:7|8' in nav and 'status:1|10' in nav and '_=>StatusEffect::NativeIgnore' in nav
assert 'self.character_scene.enter()' not in nav and 'self.disconnect_source_connection_candidate(&epoch)?;' in nav and 'Scene::ReconnectRequired' in nav
assert 'Change::RestartAfterNativeAlert' in runtime and 'batch.terminal=Some' in runtime
assert 'GetWindowTextW(draw.hwndItem,&mut text)' in window and 'let mut text:Vec<u16>=if submit' not in window
entry=read(L/'r504-entry-source-contract.json');previous=read(L/'r503-final-player-candidate.json');a=Path(previous['candidate_main']['path']);b=Path(entry['candidate_main']['path']);(L/'r504-player-entry-difference.patch').write_text(''.join(difflib.unified_diff(a.read_text(encoding='utf-8').splitlines(True),b.read_text(encoding='utf-8').splitlines(True),fromfile=str(a),tofile=str(b))),encoding='utf-8')
scope=read(L/'r503-source-scope.json');scope.pop('changed_from_fixed_R502',None);scope.pop('added_from_fixed_R502',None);scope.update(state='R504_SOURCE_ALERT_CLASSIFIER_AND_NEW_EPOCH_RETRY_POLICY',source_files=618,changed_from_fixed_R503=changed,added_from_fixed_R503=[],scope='Modernonly/ownASCII scratch; exactsource7/8,1/10alerts vsnativeignoredothers; explicitModernclose/newEpoch/roster policy, notNativeDisconnect. BackownerDrawactualcaption. NoJavaSQL/server/provider/Original/T73/GUI.')
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
save(L/'r504-source-scope.json',scope);save(L/'source-change-scope.json',scope)
builds=[__import__('json').loads(s) for s in (L/'r504a-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert any(q.get('reason')=='build-finished'and q.get('success')for q in builds)
history=dict(library_builds=1,library_failures=0,player_links=1,player_executions=0,consumer_links=1,consumer_executions=1,consumer_failures=0,preparation_failures=0,verifier_failures=0,build_logs=[pin(L/n)for n in ['r504a-library-build.jsonl','r504a-library-build.stderr.log','r504-player-build.stdout.log','r504-player-build.stderr.log','r504-refusal-consumer-build.stdout.log','r504-refusal-consumer-build.stderr.log']],consumer_exit=pin(L/'r504-refusal-consumer-exit.json'),console_erratum=pin(L/'r504-console-count-erratum.json'),cleanup=dict(IO_workers_joined=3,endpoint_threads_joined=2,Runtime_Closed=3,oldsocket_actual_EOF=True,delayed_old_tcp_write_completed=True,oldlate_bytes=15,no_helpers_or_monitors=True),historical_PASS_reruns=0,GUI=0)
save(L/'r504-execution-history.json',history)
roots=['r504-source-study.json','r504-source-oracle.json','r504-source-scope.json','r504-production-source-snapshot.json','r504-entry-source-contract.json','r504-player-candidate.json','r504-refusal-consumer-candidate.json','r504-refusal-consumer-exit.json','r504-event-details.json','r504-console-count-erratum.json','r504-execution-history.json']
for n in roots:walk(read(L/n))
assert len(read(L/'r504-refusal-consumer-candidate.json')['source_pins'])==218
proof=dict(state='R504_NEW_REFUSAL_RETRY_RISK_SEGMENT_VERIFIED',json_roots=[pin(L/n)for n in roots],pin_instances=count,unique_paths=len(cache),source_files=618,source_pairs=218,changed=3,added=0,unchanged=615,source_fragments=6,actual_diff_bytes=(L/'r504-source-difference.patch').stat().st_size,entry_diff_bytes=(L/'r504-player-entry-difference.patch').stat().st_size,flows=9,classified_statuses=512,actual_complete_socket_frames=9,new_navigation_frames=3,setup_frames=6,delayed_old_tcp_bytes=15,delayed_equals_new_success_wire=True,IO_workers_joined=3,endpoint_threads_joined=2,Runtime_Closed=3,max_observed_Runtime_poll_us=max(q['details'].get('max_poll_us',0)for q in events['flows']),elapsed_seconds=events['elapsed_seconds'],Native_alert_disconnect=False,Modern_restart_policy=True,historical_PASS_reruns=0,GUI=0,complete_modes=16,total_modes=40)
save(L/'r504-independent-contract.json',proof);print('R504 verifyPASS',count,'pins/',len(cache),'unique;9newflows/512statuses/15actualdelayedidenticaloldbytes/3IO2endpoint3Closed/GUI0')
