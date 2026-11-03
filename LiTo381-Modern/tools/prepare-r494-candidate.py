from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
for a,b in [
 ('pub struct Runtime {source_identity:source_identity::State,','pub struct Runtime {source_stream:source_stream::Receiver,source_identity:source_identity::State,'),
 ('Ok(Self{source_identity:source_identity::State::default(),','Ok(Self{source_stream:source_stream::Receiver::default(),source_identity:source_identity::State::default(),'),
 ('self.source_identity=source_identity::State::default();self.source_flow=SourceFlow::default();self.source_scope=next;','self.source_stream=source_stream::Receiver::default();self.source_identity=source_identity::State::default();self.source_flow=SourceFlow::default();self.source_scope=next;'),
 ('self.source_scope.retire();self.source_identity=source_identity::State::default();','self.source_scope.retire();self.source_stream=source_stream::Receiver::default();self.source_identity=source_identity::State::default();')]:
 assert s.count(a)==1,a;s=s.replace(a,b)
p.write_text(s,encoding='utf-8')
O=P/'target/r494-player-entry-candidate';O.mkdir(exist_ok=False)
old=read(L/'r493-player-candidate.json');before=Path(old['candidate_main']['path']).read_text(encoding='utf-8');main=O/'LiToModern-R494.rs';main.write_text(before.replace('R493','R494').replace('r493','r494'),encoding='utf-8')
entry=read(L/'r493-entry-source-contract.json');entry['candidate_main']=pin(main);entry['state']='R494_SOURCE_SEEDED_BOUNDED_LE16_STREAM_RUNTIME_CANDIDATE_NORMAL_SOCKET_OPEN';entry['UI_assets']='Runtime-ownedNativeLE16stream/1dseed/chainedcipher->existingepochdecoder/roster/modeentrypoints;normalNativeprofile/socket/auth/login480560OPEN.';entry.pop('previous_R492_player_record',None);entry['previous_R493_player_record']=pin(L/'r493-player-candidate.json');save(L/'r494-entry-source-contract.json',entry)
for oldn,newn in [('build-r493-player-candidate.ps1','build-r494-player-candidate.ps1'),('build-r493-source-identity-consumer.ps1','build-r494-source-stream-consumer.ps1')]:
 s=(P/'tools'/oldn).read_text(encoding='utf-8').replace('R493','R494').replace('r493','r494')
 if 'consumer'in newn:
  s=s.replace('r492-candidate.json','r493-candidate.json').replace('R492 source snapshot drift','R493 source snapshot drift').replace('183','186').replace('source-identity-consumer','source-stream-consumer').replace('source_identity_consumer','source_stream_consumer').replace("native_source_identity.rs'))","native_source_stream.rs'))")
 (P/'tools'/newn).write_text(s,encoding='utf-8')
old=read(L/'r493-event-input.json')
def frame(body):return [*len(body).__add__(2).to_bytes(2,'little'),*body]
def enc(body,counter):
 key=counter.to_bytes(8,'little');out=[];previous=0
 for i,x in enumerate(body):x=x^key[i&7]^previous;out.append(x);previous=x
 return out,(counter+len(body))&((1<<64)-1)
seed=0xffff_fffe;counter=0x87546ca100000000|seed
plain=[old['mode206'],[3,1],old['replacement'],[213,2,*777 .to_bytes(4,'little'),1]];wire=[];steps=[]
for body in plain:
 c=counter;cipher,counter=enc(body,counter);wire+=frame(cipher);steps.append(dict(before=c,after=counter,plain=body,cipher=cipher))
save(L/'r494-event-input.json',dict(state='R494_EXPLICIT_OFFLINE_NATIVE_STREAM_VECTORS_NO_SOCKET_AUTH_ASSERTION',plain_seed_body=[1,*seed.to_bytes(4,'little')],plain_seed_frame=frame([1,*seed.to_bytes(4,'little')]),seed_high=0x87546ca1,initial_counter=0x87546ca100000000|seed,wire=wire,steps=steps,roster=old['replacement'],mode206=old['mode206'],input_context_fields=[],counter_injection=False,Native_socket_executed=False,GUI=0,LOADED=False))
progress('R494_active_Native_opcode1_seed_counter_bypass_reset_and_Runtime_owned_LE16_stream','R494 source:4C62A0 flag8317B8=0 beforeoldclose;Native1/d ->4AE050 low32seed/high87546CA1 sets8317B8=1 andsend/receivecounters then480560loginhookOPEN.4B38D0 headerremaining2/pointer/bodyflaginit;4B37C0 LE16 fullbody chaineddecrypt->64bitaddadc bodylength BEFOREdispatch. Runtimeownedboundedframer/cipher sameepoch decoder/roster/mode,EOF/errorretire/batch8backpressure compiledcandidatepending;no guessedseed/nativeSocket/LTM1alias/GUI/phase6/fullcountupgrade.')
print('R494 prepared:186 consumerpairs/608production expected;no arbitrarycounter setter')
