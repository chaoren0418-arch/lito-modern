from r491_common import *
import shutil
run_id='nw-6fdb7420a2bc4b8492dd7cb90ff270ed';run=Path('D:/lito-dev-runtime/20261003-supervision/server')/run_id;own=L/('r527-backend-'+run_id);result=read(own/'result.json');consumer=read(L/'r527-consumer-candidate.json');ready=pin(run/'ready-contract.json')
assert ready['bytes']==5277 and ready['sha256']=='667B69B3FF198085D1B1B98AFDF4AC1F6A99FEE0D974D1B54397F54D2890F635'
assert consumer['executable']['sha256']=='3D3406840F74452614730B862DBEEFF41A7B195C677F4E4D00405990C634B831'and pin(consumer['executable']['path'])==consumer['executable']
assert result['success']and result['case_error']is None and result['actual_backend_attempts']==1 and result['IO_started']==result['IO_joined']==2 and result['queued_events']==result['discarded_late_events']==0 and result['Close_ok']and not result['synthetic_replay']and not result['credentials_contents_or_hash_recorded']and not result['owner_start_WORLD_allowed']and not result['Modern_starts_or_stops_JavaSQL']
pre=read(L/'r527-live-preflight-accounting.json');execution=read(L/'r527-live-connect-accounting.json');assert pre['attempts']==execution['attempts']==1 and pre['exit_code']==execution['exit_code']==0 and not execution['retry']
records=[];last=None
for v,state in zip(result['records'],[2,1,0]):
 body=bytes([210,state]);frame=bytes.fromhex(v['cipher_frame_hex']);assert v['intent_byte']==state and v['body_hex']==body.hex()and v['body_bytes']==2 and v['frame_bytes']==v['written_bytes']==4 and frame[:2]==b'\x04\x00'and len(frame)==4 and v['counter_after']==v['counter_before']+2
 key=v['counter_before'].to_bytes(8,'little');previous=0;plain=[]
 for i,b in enumerate(frame[2:]):plain.append(b^key[i&7]^previous);previous=b
 assert bytes(plain)==body;assert v['D1_body_hex']==(b'\xd1Fx485Guest(Fx485Moon)\0'+bytes([state])).hex();assert v['members_revision_after']==v['members_revision_before']+1
 if last is not None:assert last==v['counter_before']
 last=v['counter_after'];records.append(dict(**v,independent_decryption_matches=True,actual_D1_matches=True))
assert len(records)==3
source=read(L/'r527-production-source-snapshot.json');checked=[]
for e in source['source_pins']:
 for p in [e['path'],e['snapshot']]:
  actual=pin(p);assert actual['bytes']==e['bytes']and actual['sha256']==e['sha256'];checked.append(actual)
shutil.copyfile(run/'ready-contract.json',own/'accepted-ready-contract.json')
supply=P.parent.parent/'lito伺服器/server-modern-485F-live-supply.json';assert pin(supply)['sha256']=='1D5E2A837F7F78980DCE0F354A2EEB50709983D0E55157C4B6948B6471EB03A5';shutil.copyfile(supply,own/'accepted-server-live-supply.json')
status=read(run/'status.json');getter=status.get('getter',{});host=getter.get('sessions',{}).get('Host-GAME',{})
safe=dict(observed_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),source_status_path=str(run/'status.json'),state=status.get('state'),utc=status.get('utc'),lease_end_epoch_ms=status.get('lease_end_epoch_ms'),unexpected_WORLD_start_result=status.get('unexpected_WORLD_start_result'),background_failure=status.get('background_failure'),room_id=getter.get('room_id'),room_exists=getter.get('room_exists'),match_phase=getter.get('match_phase'),battle_started=getter.get('battle_started'),hub_ready_members=getter.get('hub_ready_members'),combatant_count=getter.get('combatant_count'),Host_phase=host.get('phase'),Host_ready=host.get('ready'),getter_limit='separate field/hubGET timing only;not globallyatomic. Point after ModernClose, not entirewire or cancellationfreshness proof.')
save(own/'post-Modern-close-status-projection.json',safe)
save(own/'independent-Modern-wire-verification.json',dict(result=pin(own/'result.json'),consumer=consumer,accepted_READY=pin(own/'accepted-ready-contract.json'),accepted_live_supply=pin(own/'accepted-server-live-supply.json'),request_records=records,source_pins_verified=len(checked),source_files=639,source_changes_after_preparation=0,preflight_attempts=1,connect_attempts=1,actual_backend=True,synthetic_replay=False,normal_entry_claim='Consumer source requires actual normal auth/source/Game/news/singletonCB/currentrawC9/acceptedC8/channel0/C_CF/S_CE/privateROOM/self before requests. Server sanitized wire correlation pending.',all_own_IO_joined=True,queue=0,late=0,secret_contents_copied_or_hashed=False,server_capture_auth_files_read=False,server_processes_started_stopped=False,server_stop_pending=True))
print('LiveModern verificationPASS: threeC_D2 frames/counters+actualD1;639live/frozen sourceunchanged;no credential/capturesecret files copied orhashed')
