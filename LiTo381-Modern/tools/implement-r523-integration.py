from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text('utf-8')
 for a,b,n in pairs:
  assert s.count(a)==n,(name,a,s.count(a));s=s.replace(a,b)
 p.write_text(s,'utf-8')
edit('native_source_send.rs',[(' pub(crate) fn prepare_ordinary_room_map',' pub(crate) fn prepare_ordinary_room_settings(&self,input:&super::java_room_settings_request::Inputs)->Result<Prepared>{self.prepare_body(super::java_room_settings_request::encode(input)?)}\n pub(crate) fn prepare_ordinary_room_map',1)])
edit('native_city_runtime.rs',[
 ('#[path="native_java_room_map_request.rs"]','#[path="native_java_room_settings_request.rs"]pub mod java_room_settings_request;\n#[path="native_java_room_map_request.rs"]',1),
 ('pub struct Runtime {','pub struct Runtime {java_settings_request:java_room_settings_request::State,last_settings_request_checkpoint:Option<java_room_settings_request::Snapshot>,',1),
 ('Ok(Self{java_map_request:','Ok(Self{java_settings_request:java_room_settings_request::State::default(),last_settings_request_checkpoint:None,java_map_request:',1),
 ('self.last_map_request_checkpoint=None;','self.last_map_request_checkpoint=None;self.last_settings_request_checkpoint=None;',1),
 ('self.archive_retire_java_room_map_request();','self.archive_retire_java_room_map_request();self.archive_retire_java_room_settings_request();',2),
 ('presentation::Key{java_map_request:','presentation::Key{java_settings_request:self.java_room_settings_request_snapshot(),java_map_request:',1),
 ('self.note_java_room_map_write(ticket.serial(),&progress);','self.note_java_room_map_write(ticket.serial(),&progress);self.note_java_room_settings_write(ticket.serial(),&progress);',1),
])
edit('native_city_presentation.rs',[(' pub java_map_request:',' pub java_settings_request:super::java_room_settings_request::Snapshot,\n pub java_map_request:',1)])
edit('native_java_room_tail.rs',[('self.invalidate_java_room_map_request();','self.invalidate_java_room_map_request();self.invalidate_java_room_settings_request();',1)])
print('R523 compact ASCII Sender writer integrated; shared owned CE scope/CF/archive, no optimistic state/ACK')
