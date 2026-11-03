from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text('utf-8')
 for a,b,count in pairs:
  assert s.count(a)==count,(name,a,s.count(a));s=s.replace(a,b)
 p.write_text(s,'utf-8')
edit('native_city_runtime.rs',[
 ('enum StreamIngress {','enum StreamIngress {JavaRoomModeMap(java_room_settings::Change),',1),
 ('let packet=match if self.java_room_settings_dispatch','let packet=match if self.java_room_mode_map_dispatch(&bytes){epoch.decoded_java_room_mode_map(&bytes)}else if self.java_room_settings_dispatch',1),
 ('let ingress=if self.java_room_settings_dispatch','let ingress=if self.java_room_mode_map_dispatch(&bytes){self.consume_java_room_mode_map_candidate(&packet).map(StreamIngress::JavaRoomModeMap)}else if self.java_room_settings_dispatch',1),
 ('StreamIngress::JavaRoomSettings(_)=>self.java_settings.note_stream()','StreamIngress::JavaRoomSettings(_)|StreamIngress::JavaRoomModeMap(_)=>self.java_settings.note_stream()',1),
])
edit('native_source_connection.rs',[(' pub fn decoded_java_room_settings',' pub fn decoded_java_room_mode_map(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_settings::decode_mode_map(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}\n pub fn decoded_java_room_settings',1)])
print('R521 explicit independent mode/map routes; shared private CE scope/lifecycle unchanged')
