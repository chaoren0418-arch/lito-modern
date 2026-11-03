from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text('utf-8')
 for a,b,count in pairs:
  assert s.count(a)==count,(name,a,s.count(a));s=s.replace(a,b)
 p.write_text(s,'utf-8')
edit('native_city_runtime.rs',[
 ('#[path="native_java_room_tail.rs"]','#[path="native_java_room_settings.rs"]pub mod java_room_settings;\n#[path="native_java_room_tail.rs"]',1),
 ('enum StreamIngress {','enum StreamIngress {JavaRoomSettings(java_room_settings::Change),',1),
 ('pub struct Runtime {','pub struct Runtime {java_settings:java_room_settings::State,last_settings_checkpoint:Option<java_room_settings::Snapshot>,',1),
 ('Ok(Self{java_self:','Ok(Self{java_settings:java_room_settings::State::default(),last_settings_checkpoint:None,java_self:',1),
 ('self.last_self_checkpoint=None;','self.last_self_checkpoint=None;self.last_settings_checkpoint=None;',1),
 ('self.java_self.retire();self.java_participants.retire();','if let Some(book)=&self.java_settings.book{self.last_settings_checkpoint=Some(book.snapshot());}self.java_settings.retire();self.java_self.retire();self.java_participants.retire();',2),
 ('presentation::Key{java_self:','presentation::Key{java_settings:self.java_room_settings_snapshot(),java_self:',1),
 ('let packet=match if bytes.first()','let packet=match if self.java_room_settings_dispatch(&bytes){epoch.decoded_java_room_settings(&bytes)}else if bytes.first()',1),
 ('let ingress=if bytes.first()','let ingress=if self.java_room_settings_dispatch(&bytes){self.consume_java_room_settings_candidate(&packet).map(StreamIngress::JavaRoomSettings)}else if bytes.first()',1),
 ('match &event.ingress {StreamIngress::JavaRoomTail','match &event.ingress {StreamIngress::JavaRoomSettings(_)=>self.java_settings.note_stream(),StreamIngress::JavaRoomTail',1),
])
edit('native_city_presentation.rs',[('pub java_self:','pub java_settings:Option<super::java_room_settings::Snapshot>,\n pub java_self:',1)])
edit('native_source_connection.rs',[(' pub fn decoded_java_room_tail',' pub fn decoded_java_room_settings(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_settings::decode(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}\n pub fn decoded_java_room_tail',1)])
edit('native_java_room_entry.rs',[
 ('        let self_state=','        let settings_book=if let Reply::Accepted(room)=&reply{self.prepare_settings_joined(room.id,revision,navigation)?}else{None};\n        let self_state=',1),
 ('self.java_participants.book=participant_book;','self.java_participants.book=participant_book;self.java_settings.book=settings_book;',1),
])
edit('native_java_room_tail.rs',[
 ('  // Clone-before-commit:', '  let mut settings=self.java_settings.book.clone();\n  if self.java_room.key.room.as_ref().is_some_and(|room|room.id==receipt.room_id){if let Some(b)=settings.as_mut(){b.invalidate_joined(self)?;invalidated=true;}}\n  // Clone-before-commit:',1),
 ('self.java_participants.book=book;','self.java_participants.book=book;self.java_settings.book=settings;',1),
])
print('R520 integrated explicit settings routes/private CE scope/archive/CF invalidation')
