from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
for a,b in [
 ('pub struct Runtime {source_scope:SourceScope,','pub struct Runtime {source_identity:source_identity::State,source_scope:SourceScope,'),
 ('Ok(Self{source_scope:SourceScope::default(),','Ok(Self{source_identity:source_identity::State::default(),source_scope:SourceScope::default(),'),
 ('self.source_flow=SourceFlow::default();self.source_scope=next;','self.source_identity=source_identity::State::default();self.source_flow=SourceFlow::default();self.source_scope=next;'),
 ('self.source_scope.retire();self.source_flow=SourceFlow::default();','self.source_scope.retire();self.source_identity=source_identity::State::default();self.source_flow=SourceFlow::default();')]:
 assert s.count(a)==1,a;s=s.replace(a,b)
p.write_text(s,encoding='utf-8')
O=P/'target/r493-player-entry-candidate';O.mkdir(exist_ok=False)
old=read(L/'r492-player-candidate.json');before=Path(old['candidate_main']['path']).read_text(encoding='utf-8');main=O/'LiToModern-R493.rs';main.write_text(before.replace('R492','R493').replace('r492','r493'),encoding='utf-8')
entry=read(L/'r492-entry-source-contract.json');entry['candidate_main']=pin(main);entry['state']='R493_TYPED_ROSTER_SELECTED_IDENTITY_CANDIDATE_NORMALWIRE_OPEN';entry['UI_assets']='Native3/203 parsed roster and selected0..3 identity, separate class/bound and actor alias. Visible selected-control/native transport producer OPEN.';entry.pop('previous_R491_player_record',None);entry['previous_R492_player_record']=pin(L/'r492-player-candidate.json');save(L/'r493-entry-source-contract.json',entry)
for oldn,newn in [('build-r492-player-candidate.ps1','build-r493-player-candidate.ps1'),('build-r492-source-connection-consumer.ps1','build-r493-source-identity-consumer.ps1')]:
 s=(P/'tools'/oldn).read_text(encoding='utf-8').replace('R492','R493').replace('r492','r493')
 if 'consumer'in newn:
  s=s.replace('r491-candidate.json','r492-candidate.json').replace('R491 source snapshot drift','R492 source snapshot drift').replace('180','183').replace('source-connection-consumer','source-identity-consumer').replace('source_connection_consumer','source_identity_consumer').replace("native_source_connection.rs'))","native_source_identity.rs'))")
 (P/'tools'/newn).write_text(s,encoding='utf-8')
def roster(name,title,cls,variant,guard,bound,tail):
 fmt='sscchhhcccccccccddddddddddcdddddc';out=[203,*name,0,*title,0];values={2:cls,3:variant,8:guard,26:bound,32:tail}
 for i,f in enumerate(fmt[2:],2):out.extend(values.get(i,i).to_bytes({'c':1,'h':2,'d':4}[f],'little'))
 return out
full=[206,0,*777 .to_bytes(4,'little'),83,0,0,0,0,0,0,0,3,255,255,0,0,0,0,0,0,0,0,0,0,0]
save(L/'r493-event-input.json',dict(state='R493_OFFLINE_SOURCE_BODY_VECTORS_NO_NATIVE_CONNECTION_ASSERTION',records=[roster(b'Alpha',b'Title',2,1,7,4,90),roster(b'B',b'',255,1,7,0,91),roster(b'C',b'',1,255,7,0,92),roster(b'D',b'',1,1,0,0,93)],replacement=roster(b'NewAlias',b'',4,1,2,1,94),mode206=full,input_context_fields=[],native_socket=False,fixture_selected_identity=False,GUI=0,LOADED=False))
progress('R493_active_source_roster_tail_saved_ESI_and_typed_selected_identity','R493 active:PE/ASM203 finalc uses savedincomingESI ->opcode address,not record+94;3signedcount/four-slot initializer and0..3selectedcontrol closed. ImplementedNative3/203 boundedparsedroster andselected-record actoralias/class+2D versusbound+5C,ownerqualifiedlifecycle.58writesversion63D54C inopaqueinline branch;comparison tables dynamic externalCP/fold authorityOPEN. RealNative4B37C0 LE16 framing->4C66C0 chainedcounterdecrypt->4B2970 sourcepinned,separatefromLTM1;normalwire/GUI/phase6 stillOPEN. Build/newbehaviorconsumer/seal pending.')
print('R493 candidate prepared:183 consumer pairs/607 production sources expected')
