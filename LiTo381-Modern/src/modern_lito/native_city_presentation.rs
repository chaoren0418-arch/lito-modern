//! Immutable visible-state key;unchanged40ms polls do not republish controls.
use super::{UiPage,source_send,super::native_mode_menu::{Filter,Hit}};
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Key {
 pub java_ready:super::java_room_ready::Snapshot,
 pub java_mode_request:super::java_room_mode_request::Snapshot,
 pub java_settings_request:super::java_room_settings_request::Snapshot,
 pub java_map_request:super::java_room_map_request::Snapshot,
 pub java_settings:Option<super::java_room_settings::Snapshot>,
 pub java_self:super::java_room_self::Snapshot,
 pub java_catalog:Option<super::java_room_tail::CatalogSnapshot>,
 pub java_participants:Option<super::java_room_participants::Snapshot>,
 pub java_room:super::java_room_entry::Key,
 pub java_return:super::java_return::Key,
 pub java_character:super::java_character_flow::Key,
 pub bootstrap:super::login_bootstrap::Key,
 pub login_form:super::login_form::Snapshot,
 pub awaiting_roster:bool,
 pub character_ui:super::character_selection::Key,
 pub navigation:super::entry_navigation::Key,
 pub transfer:super::entry_transfer::Key,
 pub startup_host_fixture:Option<std::net::SocketAddrV4>,
 pub epoch:Option<u64>,pub roster_revision:u64,pub pending:source_send::Snapshot,
 pub page:UiPage,pub generation:u64,pub frame_revision:u64,pub skin:bool,pub status:String,
 pub phase:u32,pub mode:u16,pub identifier:u32,pub version:Option<i32>,pub selected_slot:Option<usize>,
 pub selected_map:Option<usize>,pub pan_pending:bool,pub menu:Option<(Filter,usize,usize,Option<usize>,Option<Hit>)>,
}
#[derive(Default)]pub struct Gate {last:Option<(Key,bool)>}
impl Gate {
 pub fn changed(&mut self,key:Key,entry_scene:bool)->bool{let next=(key,entry_scene);if self.last.as_ref()==Some(&next){false}else{self.last=Some(next);true}}
}
