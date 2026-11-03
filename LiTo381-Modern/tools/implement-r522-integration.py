from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text('utf-8')
 for a,b,n in pairs:
  assert s.count(a)==n,(name,a,s.count(a));s=s.replace(a,b)
 p.write_text(s,'utf-8')
edit('native_java_room_settings.rs',[
 ('  let book=self.java_settings.book.as_ref().context("private CE settings book absent")?;','  self.admit_java_room_settings_context()?;let book=self.java_settings.book.as_ref().unwrap();\n  let mut next=book.clone();let change=next.observe(self,event)?;self.java_settings.book=Some(next);self.status="已收到房間設定資料；修改結果仍未確認".into();Ok(change)\n }\n pub(crate) fn admit_java_room_settings_context(&self)->Result<JoinedContext>{\n  ensure!(self.bootstrap_adapter.is_some()&&self.java_room.adapter.is_some()&&self.java_room.key.phase==super::java_room_entry::Phase::RoomReceipts&&self.entry_navigation.key.scene==super::entry_navigation::Scene::RoomReceipts,"settings outside explicit normal room scope");\n  let book=self.java_settings.book.as_ref().context("private CE settings book absent")?;self.source_scope.admit_epoch(&book.epoch)?;',1),
 ('  let mut next=book.clone();let change=next.observe(self,event)?;self.java_settings.book=Some(next);self.status="已收到房間設定資料；修改結果仍未確認".into();Ok(change)\n }\n}', '  Ok(c.clone())\n }\n}',1),
])
edit('native_source_send.rs',[(' pub(crate) fn prepare_exit_d0',' pub(crate) fn prepare_ordinary_room_map(&self,map:u16)->Result<Prepared>{ensure!(matches!(map,509|514|65535),"map candidate outside bounded509/514/FFFF");let mut body=vec![209,3];body.extend(map.to_le_bytes());self.prepare_body(body)}\n pub(crate) fn prepare_exit_d0',1)])
edit('native_city_runtime.rs',[
 ('#[path="native_java_room_settings.rs"]','#[path="native_java_room_map_request.rs"]pub mod java_room_map_request;\n#[path="native_java_room_settings.rs"]',1),
 ('pub struct Runtime {','pub struct Runtime {java_map_request:java_room_map_request::State,last_map_request_checkpoint:Option<java_room_map_request::Snapshot>,',1),
 ('Ok(Self{java_settings:','Ok(Self{java_map_request:java_room_map_request::State::default(),last_map_request_checkpoint:None,java_settings:',1),
 ('self.last_settings_checkpoint=None;','self.last_settings_checkpoint=None;self.last_map_request_checkpoint=None;',1),
 ('self.java_settings.retire();','self.java_settings.retire();self.archive_retire_java_room_map_request();',2),
 ('presentation::Key{java_settings:','presentation::Key{java_map_request:self.java_room_map_request_snapshot(),java_settings:',1),
 ('self.note_self_exit_write(ticket.serial(),&progress);','self.note_self_exit_write(ticket.serial(),&progress);self.note_java_room_map_write(ticket.serial(),&progress);',1),
])
edit('native_city_presentation.rs',[(' pub java_settings:',' pub java_map_request:super::java_room_map_request::Snapshot,\n pub java_settings:',1)])
edit('native_java_room_tail.rs',[('if invalidated{self.java_self.invalidate_joined();}','if invalidated{self.java_self.invalidate_joined();self.invalidate_java_room_map_request();}',1)])
print('R522 true Sender writer integrated; shared private settings context; no optimistic map/ready/ACK')
