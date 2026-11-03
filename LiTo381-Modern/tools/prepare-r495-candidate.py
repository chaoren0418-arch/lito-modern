from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
for a,b in [
 ('pub struct Runtime {source_stream:source_stream::Receiver,','pub struct Runtime {source_send:source_send::Sender,source_stream:source_stream::Receiver,'),
 ('Ok(Self{source_stream:source_stream::Receiver::default(),','Ok(Self{source_send:source_send::Sender::default(),source_stream:source_stream::Receiver::default(),'),
 ('self.source_stream=source_stream::Receiver::default();','self.source_send=source_send::Sender::default();self.source_stream=source_stream::Receiver::default();')]:
 assert s.count(a)==(2 if a.startswith('self.')else 1),a;s=s.replace(a,b)
p.write_text(s,encoding='utf-8')
O=P/'target/r495-player-entry-candidate';O.mkdir(exist_ok=False)
old=read(L/'r494-player-candidate.json');before=Path(old['candidate_main']['path']).read_text(encoding='utf-8');main=O/'LiToModern-R495.rs';main.write_text(before.replace('R494','R495').replace('r494','r495'),encoding='utf-8')
entry=read(L/'r494-entry-source-contract.json');entry['candidate_main']=pin(main);entry['state']='R495_TYPED_SOURCE_CHARACTER_SEND_PARTIAL_WRITE_CANDIDATE_NORMAL_AUTH_SOCKET_OPEN';entry['UI_assets']='SameRuntime/sourceepoch source-seededindependentsendcounter;typedreceivedslot->cs201framing/chainedencrypt->guardedwriteviews/partialreports/retirement. Full480560/auth/socket/UIOPEN.';entry.pop('previous_R493_player_record',None);entry['previous_R494_player_record']=pin(L/'r494-player-candidate.json');save(L/'r495-entry-source-contract.json',entry)
for oldn,newn in [('build-r494-player-candidate.ps1','build-r495-player-candidate.ps1'),('build-r494-source-stream-consumer.ps1','build-r495-source-send-consumer.ps1')]:
 s=(P/'tools'/oldn).read_text(encoding='utf-8').replace('R494','R495').replace('r494','r495')
 if 'consumer'in newn:
  s=s.replace('r493-candidate.json','r494-candidate.json').replace('R493 source snapshot drift','R494 source snapshot drift').replace('186','189').replace('source-stream-consumer','source-send-consumer').replace('source_stream_consumer','source_send_consumer').replace("native_source_stream.rs'))","native_source_send.rs'))")
 (P/'tools'/newn).write_text(s,encoding='utf-8')
progress('R495_active_480560_chdch_auth_gaps_and_typed_source_character_cs201_sender','R495 source480560 sendschdch opcode0/word8033/cache636584/FromCD/5033C4lowword,externalregistry/versionauthorityOPEN;no credentials/auth success fabricated. Ordinary4928C0 cs201 usesreceivedselectedrecordname. NewRuntime-ownedSender seededfromsame1/d event,sendcounterindependentofreceive,typedselectedslot->boundedLE16/chainedencrypt/prewriteadvance,guardedtickets/views/partialwrite/end/failretirement. NoNativeSocket/provider/JavaSQL/GUI/phase6;build/oracle/newconsumerpending.1atomicRuntimepatchhunkorderfailure0Runtimewrites,newmodulealreadywritten;correctedbeforebuild.')
print('R495 prepared:189consumerpairs609production expected;seededindependentsender,fullauthOPEN')
