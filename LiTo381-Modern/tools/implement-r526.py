from r491_common import *
s=(P/'src/modern_lito/native_java_room_mode_request.rs').read_text('utf-8')
s=s[:s.index('#[cfg(test)]mod tests')]+s[s.index('#[derive(Clone,Debug,Default,PartialEq,Eq)]pub struct Snapshot'):]
s=s.replace('java_room_mode_request','java_room_ready').replace('java_mode_request','java_ready').replace('last_mode_request_checkpoint','last_ready_checkpoint').replace('mode_request','ready_request').replace('mode writer','ready writer').replace('mode intent','ready intent').replace('mode context','ready context').replace('mode control','ready control').replace('mode request','ready request').replace('relative mode writer','participant ready writer').replace('ordinary Java C_D1/2 writer','ordinary Java participant C_D2 writer')
s=s.replace('14B6DA5301356A26D038A97A7D0D42FD0F8C2DA3D68DD7ED0A0D13EB9409945E','3F31E26FF006B132C4531BA53811597D3D8EF8581531AD04E8599B0C2B170947').replace('1BB37D61C8E25EC195EA2084941EFDC89D2E6F2760119583357A088A04C3EC8F','F3867EC1B86DA3DF66A7493E92CD239E40AEC59B658EC0B588F463EFF13A2667').replace('modern_synthetic_485F_relative_ready_request_v1','modern_synthetic_485F_participant_ready_v1')
start=s.index('#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Direction');end=s.index('#[derive(Clone,Debug,PartialEq,Eq)]pub struct ContextKey')
s=s[:start]+'''#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ParticipantIntent {NotReady,Ready,ShopList}
impl ParticipantIntent {pub fn byte(self)->u8{match self{Self::NotReady=>0,Self::Ready=>1,Self::ShopList=>2}}pub fn from_wire(b:u8)->Result<Self>{match b{0=>Ok(Self::NotReady),1=>Ok(Self::Ready),2=>Ok(Self::ShopList),_=>anyhow::bail!("participant state must be0/1/2")}}}
pub fn encode(intent:ParticipantIntent)->Vec<u8>{vec![210,intent.byte()]}
'''+s[end:]
s=s.replace('pub joined:settings::JoinedContext}','pub joined:settings::JoinedContext,pub participants_revision:u64}')
s=s.replace('direction:Direction','intent:ParticipantIntent').replace('c.direction','c.intent').replace('direction})','intent})').replace('(direction:Direction)','(intent:ParticipantIntent)').replace('direction:c.intent','intent:c.intent')
s=s.replace('prepare_ordinary_room_mode_step(c.intent.byte())','prepare_participant_ready(c.intent.byte())').replace('==5','==4').replace('five bytes','four bytes').replace('frame_bytes:5','frame_bytes:4')
s=s.replace('self.java_settings.mode_map_adapter.as_ref().is_some_and(|a|a.producer_sha256()==PRODUCER_SHA)&&','')
s=s.replace('Ok(ContextKey{room_id:room.id,joined})','''let own=self.java_room_self_snapshot();let alias=self.qualified_singleton_self_alias().context("ready current singleton self absent")?;
  ensure!(own.current_alias_qualified&&own.joined.as_ref().is_some_and(|c|c.room_id==room.id&&c.ce_revision==joined.ce_revision&&c.navigation_revision==joined.navigation_revision)&&self.java_room_participants_route_enabled(),"ready current self/ROOM context absent");
  let members=self.java_room_participants_snapshot().context("ready private participants absent")?;
  ensure!(members.authenticated_singleton_self_qualified&&members.room_id==room.id&&members.ce_revision==Some(joined.ce_revision)&&!members.joined_room_invalidated&&members.rows.iter().filter(|r|r.participant.display_alias==alias).count()==1,"ready requires unique observed current self; not hub membership proof");
  Ok(ContextKey{room_id:room.id,joined,participants_revision:members.revision})''')
s=s.replace('正在送出相對模式要求；結果尚未確認','正在送出準備狀態要求；結果尚未確認')
s=s.replace('pub(crate) fn invalidate_java_room_ready_request(&mut self){self.java_ready.invalidate("joined room invalidated");}','pub(crate) fn invalidate_java_room_ready_request(&mut self){self.java_ready.invalidate("joined room invalidated");if let Some(c)=self.java_ready.countdown.as_mut(){c.count=None;c.invalidated=true;}}')
s=s.replace('pub requests:Vec<Request>}','pub requests:Vec<Request>,pub countdown:Option<Countdown>}').replace('requests:Vec<OwnedRequest>}','requests:Vec<OwnedRequest>,countdown:Option<Countdown>}').replace('requests:self.requests.iter().map(|v|v.request.clone()).collect()}','requests:self.requests.iter().map(|v|v.request.clone()).collect(),countdown:self.countdown.clone()}').replace('self.requests.clear();self.revision=0;','self.requests.clear();self.revision=0;self.countdown=None;')
s=s.replace('if !self.java_ready.requests.is_empty(){','if !self.java_ready.requests.is_empty()||self.java_ready.countdown.is_some(){')
s+='''
/// E1 is an unsigned BYTE receipt with no room, generation or sequence on wire.
/// The local context labels where it was observed, never proves wire freshness.
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Countdown {pub epoch:u64,pub room_id:u32,pub joined:settings::JoinedContext,pub revision:u64,pub count:Option<u8>,pub receipts:Vec<u8>,pub invalidated:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct CountdownChange {pub count:u8,pub cancelled:bool}
pub fn decode_countdown(body:&[u8])->Result<u8>{ensure!(body.len()==2&&body[0]==225,"E1 requires exact opcode+unsignedBYTE");Ok(body[1])}
impl Runtime {
 /// No independent owner authority exists in this adapter. CE/D5 display,
 /// selected alias, Native flags and D1 state cannot mint an owner command.
 pub fn java_room_owner_start_unavailable_reason(&self)->&'static str{"independent current owner qualification absent; opcode-only C_D2 refused"}
 pub(crate) fn java_room_countdown_dispatch(&self,body:&[u8])->bool{self.java_ready.adapter.is_some()&&body.first()==Some(&225)}
 pub fn consume_java_room_countdown_candidate(&mut self,event:&super::SourceDecodedPacket)->Result<CountdownChange>{
  self.source_scope.admit(event)?;ensure!(self.java_ready.adapter.is_some(),"E1 explicit485F adapter absent");let count=decode_countdown(event.body())?;
  let context=self.current_ready_request_context()?;
  let mut next=self.java_ready.countdown.clone().filter(|c|!c.invalidated&&c.epoch==event.epoch_serial()&&c.room_id==context.room_id&&c.joined==context.joined).unwrap_or(Countdown{epoch:event.epoch_serial(),room_id:context.room_id,joined:context.joined,revision:0,count:None,receipts:vec![],invalidated:false});
  ensure!(next.receipts.len()<1024,"E1 receipt budget reached");next.revision=next.revision.checked_add(1).context("E1 revision exhausted")?;next.count=if count==0{None}else{Some(count)};next.receipts.push(count);self.java_ready.countdown=Some(next);
  self.status=if count==0{"已收到倒數取消通知"}else{"已收到倒數資料；等待伺服器後續通知"}.into();
  // E100 never changes D1 readiness. No local clock or WORLD grant exists.
  Ok(CountdownChange{count,cancelled:count==0})
 }
}
'''
assert 'direction' not in s
(P/'src/modern_lito/native_java_room_ready.rs').write_text(s,'utf-8')
def edit(n,old,new,count=None):
 p=P/'src/modern_lito'/n;x=p.read_text('utf-8');assert old in x,(n,old);assert count is None or x.count(old)==count;(p).write_text(x.replace(old,new),'utf-8')
edit('native_city_runtime.rs','#[path="native_java_room_mode_request.rs"]','#[path="native_java_room_ready.rs"]pub mod java_room_ready;\n#[path="native_java_room_mode_request.rs"]',1)
edit('native_city_runtime.rs','pub struct Runtime {','pub struct Runtime {java_ready:java_room_ready::State,last_ready_checkpoint:Option<java_room_ready::Snapshot>,',1)
edit('native_city_runtime.rs','Ok(Self{java_mode_request:','Ok(Self{java_ready:java_room_ready::State::default(),last_ready_checkpoint:None,java_mode_request:',1)
edit('native_city_runtime.rs','last_mode_request_checkpoint=None;Ok(epoch)','last_mode_request_checkpoint=None;self.last_ready_checkpoint=None;Ok(epoch)',1)
edit('native_city_runtime.rs','self.archive_retire_java_room_mode_request();','self.archive_retire_java_room_mode_request();self.archive_retire_java_room_ready_request();',2)
edit('native_city_runtime.rs','presentation::Key{java_mode_request:','presentation::Key{java_ready:self.java_room_ready_snapshot(),java_mode_request:',1)
edit('native_city_runtime.rs','pub enum StreamIngress {','pub enum StreamIngress {JavaCountdown(java_room_ready::CountdownChange),',1)
edit('native_city_runtime.rs','if self.java_room_mode_map_dispatch(&bytes){epoch','if self.java_room_countdown_dispatch(&bytes){epoch.decoded_java_room_countdown(&bytes)}else if self.java_room_mode_map_dispatch(&bytes){epoch',1)
edit('native_city_runtime.rs','let ingress=if self.java_room_mode_map_dispatch(&bytes)','let ingress=if self.java_room_countdown_dispatch(&bytes){self.consume_java_room_countdown_candidate(&packet).map(StreamIngress::JavaCountdown)}else if self.java_room_mode_map_dispatch(&bytes)',1)
edit('native_city_runtime.rs','self.note_java_room_mode_write(ticket.serial(),&progress);','self.note_java_room_mode_write(ticket.serial(),&progress);self.note_java_room_ready_write(ticket.serial(),&progress);',1)
edit('native_city_presentation.rs','pub struct Key {','pub struct Key {\n pub java_ready:super::java_room_ready::Snapshot,',1)
edit('native_source_connection.rs',' pub fn decoded_java_room_mode_map',' pub fn decoded_java_room_countdown(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_ready::decode_countdown(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}\n pub fn decoded_java_room_mode_map',1)
edit('native_source_send.rs',' pub(crate) fn prepare_ordinary_room_mode_step',' pub(crate) fn prepare_participant_ready(&self,state:u8)->Result<Prepared>{ensure!(state<=2,"participant state must be0/1/2");self.prepare_body(vec![210,state])}\n pub(crate) fn prepare_ordinary_room_mode_step',1)
edit('native_java_room_tail.rs','self.invalidate_java_room_mode_request();','self.invalidate_java_room_mode_request();self.invalidate_java_room_ready_request();',1)
print('R526 new scoped ready/countdown module and six integrations saved')
