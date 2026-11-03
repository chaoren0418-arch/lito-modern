use lito_modern::modern_lito::{native_skills as skills,native_login::{LoginController,LoginPhase,Snapshot},native_inventory::Item,native_skill_selection::Model};
const ID:u32=0xfedcba98;
fn state()->Snapshot {
 let mut s=LoginController::default().snapshot().clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
 for key in ["world","start","status","actor","bot"] {let hex=f["packets"][key].as_str().unwrap();let bytes:Vec<u8>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();s.world.accept(&bytes).unwrap();}
 let mut own=s.world.actors.remove(&ID).unwrap();own.id=1;s.world.actors.insert(1,own);s.world.status.as_mut().unwrap().object_id=1;
 s.phase=LoginPhase::BattlePending;s.game_connected=true;s.inventory.begin(7,"PeerFixture").unwrap();s.world.hp=Some([50,50]);s.world.mp=Some([100,100]);s.world.learned_skills.extend([12,21]);
 s.inventory.items.insert(ID,Item{id:ID,use_type:1,charge:0,icon:1,blessing:0,count:1,identified:1,name:"owned weapon".into(),detail:vec![0]});s
}
#[test]
fn item_spells_encode_inventory_identity_without_actor_or_ground_fallback() {
 let s=state();let target=skills::ItemTarget::capture(&s,7,ID).unwrap();
 for (id,row,col) in [(12,1,3),(21,2,4)] {let before=serde_json::to_value(&s).unwrap();assert_eq!(skills::item_request(&s,7,id,&target).unwrap(),[0x14,row,col,0x98,0xba,0xdc,0xfe]);assert!(skills::request(&s.world,id,Some(2)).is_err());assert!(skills::ground_request(&s.world,id,[0,0]).is_err());assert_eq!(serde_json::to_value(&s).unwrap(),before);assert_eq!(skills::reuse_millis(id),10);}
 assert!(skills::ItemTarget::capture(&s,7,2).is_err());assert!(skills::item_request(&s,7,73,&target).is_err());
}
#[test]
fn item_spells_reject_changed_owner_item_inventory_revision_and_same_bytes_ABA() {
 let s=state();let target=skills::ItemTarget::capture(&s,7,ID).unwrap();
 for kind in 0..9 {let mut bad=s.clone();match kind {
  0=>bad.inventory.owner.as_mut().unwrap().selection+=1,1=>bad.inventory.owner.as_mut().unwrap().connection=8,
  2=>bad.inventory.owner.as_mut().unwrap().character="Other".into(),3=>{bad.inventory.items.remove(&ID);},
  4=>bad.inventory.items.get_mut(&ID).unwrap().count=2,5=>bad.inventory.items.get_mut(&ID).unwrap().name="updated".into(),
  6=>bad.inventory.items.get_mut(&ID).unwrap().detail.push(1),7=>bad.inventory.revision+=1,
  _=>{let old=bad.inventory.items.remove(&ID).unwrap();bad.inventory.revision+=1;bad.inventory.items.insert(ID,old);bad.inventory.revision+=1;},
 }let before=serde_json::to_value(&bad).unwrap();assert!(skills::item_request(&bad,7,12,&target).is_err());assert_eq!(serde_json::to_value(&bad).unwrap(),before);}
 assert!(skills::item_request(&s,8,12,&target).is_err());
}
#[test]
fn item_spells_revoke_on_source_scene_actor_life_or_control_interruption() {
 let s=state();let target=skills::ItemTarget::capture(&s,7,ID).unwrap();
 for kind in 0..7 {let mut bad=s.clone();match kind {0=>bad.world.generation+=1,1=>bad.world.local_corrections+=1,2=>bad.world.control_interruptions+=1,3=>bad.world.movement_locks+=1,4=>bad.world.actors.get_mut(&1).unwrap().incarnation+=1,5=>bad.world.map_id=Some(bad.world.map_id.unwrap().wrapping_add(1)),_=>bad.world.clear_preserving_generation(),}assert!(skills::item_request(&bad,7,21,&target).is_err());}
}
#[test]
fn item_spells_keep_eligibility_cost_and_1800_second_effect_server_owned() {
 let mut s=state();s.world.mp=Some([1,100]);let target=skills::ItemTarget::capture(&s,7,ID).unwrap();let before=serde_json::to_value(&s).unwrap();assert!(skills::item_request(&s,7,12,&target).is_ok());assert_eq!(serde_json::to_value(&s).unwrap(),before);
 for kind in 0..10 {let mut bad=s.clone();match kind {0=>{bad.world.learned_skills.remove(&12);},1=>bad.world.mp=Some([0,100]),2=>bad.world.hp=Some([0,50]),3=>bad.world.actors.get_mut(&1).unwrap().dead=true,4=>{bad.world.movement_blocks.insert(2);},5=>bad.game_connected=false,6=>bad.phase=LoginPhase::LobbyReady,7=>bad.observer=true,8=>bad.world.observer_mode=true,_=>bad.world.started=false,}assert!(skills::item_request(&bad,7,12,&target).is_err());}
 // A changed server item is a new authority snapshot, not a local prediction.
 s.inventory.items.get_mut(&ID).unwrap().name="server changed weapon".into();s.inventory.revision+=1;assert!(skills::item_request(&s,7,12,&target).is_err());
}
#[test]
fn item_spell_menu_is_learned_and_scoped_without_requiring_an_actor_target() {
 let mut s=state();let mut menu=Model::default();menu.sync(7,&s.world,true);menu.select(Some(12)).unwrap();assert_eq!(menu.selected_skill(7,&s.world).unwrap(),12);assert_eq!(menu.rows().iter().find(|r|r.id==12).unwrap().label,"#12 武器附魔（選取背包物品）");assert!(menu.choice(7,&s.world,None).is_err());
 menu.select(Some(21)).unwrap();assert_eq!(menu.selected_skill(7,&s.world).unwrap(),21);assert!(menu.selected_skill(8,&s.world).is_err());s.world.learned_skills.remove(&21);assert!(menu.selected_skill(7,&s.world).is_err());menu.sync(7,&s.world,true);assert_eq!(menu.selected(),None);
}
#[test]
fn item_spell_invalid79_feedback_is_scoped_and_does_not_invent_a_success_ACK() {
 use std::time::{Instant,Duration};let mut s=state();let now=Instant::now();let p=skills::Pending{since:now,casts:s.world.own_cast_packets,messages:s.world.message_count,generation:s.world.generation,incarnation:s.world.own_actor().unwrap().incarnation};
 s.world.message_count+=1;s.world.last_message=Some((79,vec![]));assert_eq!(p.observe_item(&s.world,now),skills::Feedback::Rejected(79));assert_eq!(p.observe(&s.world,now),skills::Feedback::Pending);
 s.world.last_message=Some((161,vec![]));assert_eq!(p.observe_item(&s.world,now+Duration::from_secs(2)),skills::Feedback::NoResponse);s.world.own_cast_packets+=1;assert_eq!(p.observe_item(&s.world,now),skills::Feedback::CastAction);s.world.generation+=1;assert_eq!(p.observe_item(&s.world,now),skills::Feedback::Interrupted);
}
#[test]
fn item_spell_original747_748_frames_and_sound_groups_are_exact() {
 use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{ActionKey,ActionLimits,SpriteActionCatalog,FrameEvent}};use std::path::Path;
 let bytes=SharedArchive::open(Path::new(r"D:\lineage開發\0-客戶端"),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let c=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();
 for(gfx,n,ticks,sound)in[(747,19,4,113),(748,20,3,137)]{let a=c.resolve_action(gfx,ActionKey::new(0)).unwrap().animation;assert_eq!(a.frames.len(),n);assert_eq!(a.frames[0].subframe,n as u32-1);assert_eq!(a.frames[1].events,[FrameEvent::Sound(sound)]);assert!(a.frames.iter().all(|f|f.duration_ticks==ticks));for(i,f)in a.frames.iter().enumerate().skip(1){assert_eq!(f.subframe,i as u32-1);if i>1{assert!(f.events.is_empty());}}}
}
#[test]
fn item_spell_finite_effects_emit_one_original_cue_then_retire() {
 use lito_modern::modern_lito::{native_skill_effects::Effects,native_world_effects::Target,world_map::{WorldCell,PixelPoint,ViewProjection},combat_animation::NativeAudioMode};use std::{path::Path,sync::atomic::AtomicBool};
 for(gfx,n,ms,sound)in[(747u16,19,166,113),(748u16,20,125,137)]{let mut s=state();let mut packet=vec![0x37,1,0,0,0];packet.extend(gfx.to_le_bytes());s.world.accept(&packet).unwrap();let own=s.world.own_actor().unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:PixelPoint{x:400,y:300}};let mut e=Effects::default();e.prepare(Path::new(r"D:\lineage開發\0-客戶端"),&s.world,&AtomicBool::new(false));let mut all=Vec::new();
 for step in 0..=n {let mut fragments=Vec::new();let(r,events)=e.draw_to(&s.world,&mut Target::Fragments(&mut fragments),view,step*ms*1_000_000);assert!(r["load_error"].is_null(),"{r}");all.extend(events);if step==n{assert_eq!(r["active"],0);}}
 assert_eq!(all.len(),1);assert_eq!((all[0].frame,all[0].sound),(1,Some(sound)));assert_eq!(all[0].prepared_audio.as_ref().unwrap().group().mode,NativeAudioMode::SuppressWhileSameIdPlaying);
 s.world.clear_preserving_generation();let mut fragments=Vec::new();let(r,events)=e.draw_to(&s.world,&mut Target::Fragments(&mut fragments),view,u64::MAX);assert_eq!(r["active"],0);assert!(events.is_empty());}
}
