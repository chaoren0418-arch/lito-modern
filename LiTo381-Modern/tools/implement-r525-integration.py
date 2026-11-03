from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text('utf-8')
 for a,b,n in pairs:
  assert s.count(a)==n,(name,a,s.count(a));s=s.replace(a,b)
 p.write_text(s,'utf-8')
edit('native_source_send.rs',[(' pub(crate) fn prepare_ordinary_room_map',' pub(crate) fn prepare_ordinary_room_mode_step(&self,direction:u8)->Result<Prepared>{ensure!(matches!(direction,1|2),"relative direction must be NEXT1/PREVIOUS2");self.prepare_body(vec![209,2,direction])}\n pub(crate) fn prepare_ordinary_room_map',1)])
edit('native_city_runtime.rs',[
 ('#[path="native_java_room_settings_request.rs"]','#[path="native_java_room_mode_request.rs"]pub mod java_room_mode_request;\n#[path="native_java_room_settings_request.rs"]',1),
 ('pub struct Runtime {','pub struct Runtime {java_mode_request:java_room_mode_request::State,last_mode_request_checkpoint:Option<java_room_mode_request::Snapshot>,',1),
 ('Ok(Self{java_settings_request:','Ok(Self{java_mode_request:java_room_mode_request::State::default(),last_mode_request_checkpoint:None,java_settings_request:',1),
 ('self.last_settings_request_checkpoint=None;','self.last_settings_request_checkpoint=None;self.last_mode_request_checkpoint=None;',1),
 ('self.archive_retire_java_room_settings_request();','self.archive_retire_java_room_settings_request();self.archive_retire_java_room_mode_request();',2),
 ('presentation::Key{java_settings_request:','presentation::Key{java_mode_request:self.java_room_mode_request_snapshot(),java_settings_request:',1),
 ('self.note_java_room_settings_write(ticket.serial(),&progress);','self.note_java_room_settings_write(ticket.serial(),&progress);self.note_java_room_mode_write(ticket.serial(),&progress);',1),
])
edit('native_city_presentation.rs',[(' pub java_settings_request:',' pub java_mode_request:super::java_room_mode_request::Snapshot,\n pub java_settings_request:',1)])
edit('native_java_room_tail.rs',[('self.invalidate_java_room_settings_request();','self.invalidate_java_room_settings_request();self.invalidate_java_room_mode_request();',1)])
p=P/'src/modern_lito/native_java_room_settings.rs';s=p.read_text('utf-8');s=s.replace('#[derive(Clone)]pub struct ModeMapAdapter;','#[derive(Clone)]pub struct ModeMapAdapter {producer:&\'static str}');start=s.index('impl ModeMapAdapter {');end=s.index('#[derive(Clone,Debug,PartialEq,Eq)]pub struct Summary',start)
s=s[:start]+'''impl ModeMapAdapter {
 pub fn producer_sha256(&self)->&'static str{self.producer}
 pub fn parse(bytes:&[u8])->Result<Self>{ensure!(bytes.len()<=1024,"mode/map adapter exceeds bound");let v:serde_json::Value=serde_json::from_slice(bytes)?;ensure!(v["layout_sha256"]==LAYOUT_SHA&&v["native_sha256"]==MODE_MAP_SHA&&v["scalar_boundary_sha256"]==SCALAR_BOUNDARY_SHA,"pinned independent2/3 layout required");let producer=if v["kind"]=="modern_synthetic_AD368_room_mode_map_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA{super::login_bootstrap::PRODUCER_SHA}else{ensure!(v["kind"]=="modern_synthetic_485F_room_mode_map_v1"&&v["producer_sha256"]==super::java_room_mode_request::PRODUCER_SHA&&v["network_sha256"]==super::java_room_mode_request::NETWORK_SHA&&v["mode_step_sha256"]==super::java_room_mode_request::NATIVE_SHA,"explicit pinned485F mode/map receipt adapter required");super::java_room_mode_request::PRODUCER_SHA};Ok(Self{producer})}
}
''' +s[end:];p.write_text(s,'utf-8')
edit('native_java_room_mode_request.rs',[('"map frame must be five bytes"','"relative mode frame must be five bytes"',1)])
print('R525 relative mode Sender integrated, explicit485F2/3 receipts version, CF/archive hooks; no optimistic state/ACK')
