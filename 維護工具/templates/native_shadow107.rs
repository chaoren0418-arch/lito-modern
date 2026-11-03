use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase,Snapshot},native_inventory::Item,native_skills as skills,native_skill_selection::Model};
const ITEM:u32=0xfedcba98;
fn snapshot()->Snapshot {
    let mut s=LoginController::default().snapshot().clone();
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    for key in ["world","start","status","actor","bot"] {
        let hex=f["packets"][key].as_str().unwrap();
        let bytes:Vec<u8>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
        s.world.accept(&bytes).unwrap();
    }
    let mut own=s.world.actors.remove(&ITEM).unwrap();own.id=1;s.world.actors.insert(1,own);s.world.status.as_mut().unwrap().object_id=1;
    s.phase=LoginPhase::BattlePending;s.game_connected=true;s.inventory.begin(7,"PeerFixture").unwrap();
    s.world.hp=Some([50,50]);s.world.mp=Some([1,200]);s.world.learned_skills.insert(107);
    s.inventory.items.insert(ITEM,Item{id:ITEM,use_type:1,charge:0,icon:1,blessing:0,count:1,identified:1,name:"server owned weapon".into(),detail:vec![0]});s
}
#[test]
fn shadow_fang_requires_inventory_intent_even_when_commit_path_is_named_self() {
    let s=snapshot();let item=skills::ItemTarget::capture(&s,7,ITEM).unwrap();
    assert_eq!(skills::item_request(&s,7,107,&item).unwrap(),[0x14,13,2,0x98,0xba,0xdc,0xfe]);
    assert!(skills::request(&s.world,107,None).is_err());
    for actor in [1,2,ITEM] {assert!(skills::request(&s.world,107,Some(actor)).is_err());}
    assert!(skills::ground_request(&s.world,107,[32768,32768]).is_err());
    assert!(skills::ItemTarget::capture(&s,7,1).is_err());
}
#[test]
fn shadow_fang_uses_server_mp_material_eligibility_and_inventory_result() {
    let mut s=snapshot();let item=skills::ItemTarget::capture(&s,7,ITEM).unwrap();
    // One displayed MP and no local material row must not override server INT /
    // consumption or infer eligibility from icon, name, identified flag or detail.
    let before=serde_json::to_value(&s).unwrap();assert!(skills::item_request(&s,7,107,&item).is_ok());
    assert_eq!(serde_json::to_value(&s).unwrap(),before);assert_eq!(skills::reuse_millis(107),0);
    assert_eq!(s.inventory.items.len(),1);assert_eq!(s.inventory.items[&ITEM].detail,vec![0]);
    for kind in 0..5 {let mut invalid=s.clone();match kind {0=>invalid.world.mp=Some([0,200]),1=>invalid.world.hp=Some([0,50]),2=>{invalid.world.learned_skills.remove(&107);},3=>invalid.observer=true,_=>{invalid.world.movement_blocks.insert(1);}}assert!(skills::item_request(&invalid,7,107,&item).is_err());}
    s.inventory.items.get_mut(&ITEM).unwrap().detail=vec![1];s.inventory.revision+=1;
    assert!(skills::item_request(&s,7,107,&item).is_err());
}
#[test]
fn shadow_fang_revokes_captured_item_after_transfer_control_or_inventory_ABA() {
    let s=snapshot();let item=skills::ItemTarget::capture(&s,7,ITEM).unwrap();
    for kind in 0..8 {let mut stale=s.clone();match kind {
        0=>stale.inventory.owner.as_mut().unwrap().selection+=1,
        1=>stale.world.generation+=1,2=>stale.world.control_interruptions+=1,
        3=>stale.world.local_corrections+=1,4=>stale.world.actors.get_mut(&1).unwrap().incarnation+=1,
        5=>{let same=stale.inventory.items.remove(&ITEM).unwrap();stale.inventory.revision+=1;stale.inventory.items.insert(ITEM,same);stale.inventory.revision+=1;},
        6=>stale.game_connected=false,_=>stale.world.actors.get_mut(&1).unwrap().dead=true,
    }assert!(skills::item_request(&stale,7,107,&item).is_err());}
    assert!(skills::item_request(&s,8,107,&item).is_err());
}
#[test]
fn shadow_fang_menu_needs_learned_current_owner_and_explicit_item_selection() {
    let mut s=snapshot();let mut menu=Model::default();menu.sync(7,&s.world,true);menu.select(Some(107)).unwrap();
    assert_eq!(menu.selected_skill(7,&s.world).unwrap(),107);
    assert_eq!(menu.rows().iter().find(|r|r.id==107).unwrap().label,"#107 暗影之牙（選取背包武器；消耗 1 顆二級黑魔石）");
    assert!(menu.choice(7,&s.world,None).is_err());assert!(menu.selected_skill(8,&s.world).is_err());
    s.world.learned_skills.remove(&107);assert!(menu.selected_skill(7,&s.world).is_err());menu.sync(7,&s.world,true);assert_eq!(menu.selected(),None);
}
#[test]
fn shadow_fang_messages_do_not_fake_success_or_cross_into_ordinary_skill_rejection() {
    use std::time::{Instant,Duration};let mut s=snapshot();let now=Instant::now();
    let pending=skills::Pending{since:now,casts:s.world.own_cast_packets,messages:s.world.message_count,generation:s.world.generation,incarnation:s.world.own_actor().unwrap().incarnation};
    for code in [161,308] {s.world.message_count=pending.messages+1;s.world.last_message=Some((code,vec![]));assert_eq!(pending.observe_item(&s.world,now),skills::Feedback::Pending);}
    s.world.last_message=Some((79,vec![]));assert_eq!(pending.observe_item(&s.world,now),skills::Feedback::Rejected(79));assert_eq!(pending.observe(&s.world,now),skills::Feedback::Pending);
    s.world.message_count=pending.messages;assert_eq!(pending.observe_item(&s.world,now),skills::Feedback::Pending);
    s.world.last_message=Some((161,vec![]));assert_eq!(pending.observe_item(&s.world,now+Duration::from_secs(4)),skills::Feedback::NoResponse);
    s.world.generation+=1;assert_eq!(pending.observe_item(&s.world,now),skills::Feedback::Interrupted);
}

#[test]
fn shadow_fang_original_2951_variable_cadence_and_sound_retire_without_local_duration_state() {
    use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits,ActionKey,FrameEvent},native_skill_effects::Effects,native_world_effects::Target,world_map::{WorldCell,PixelPoint,ViewProjection},combat_animation::NativeAudioMode};
    use std::{path::Path,sync::atomic::AtomicBool};
    let root=Path::new(r"D:\lineage開發\0-客戶端");
    let raw=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
    let animation=catalog.resolve_action(2951,ActionKey::new(0)).unwrap().animation;
    assert_eq!(animation.frames.len(),13);
    let durations=[4,2,2,2,2,3,6,5,4,4,4,4,4];
    for (index,frame) in animation.frames.iter().enumerate() {
        assert_eq!((frame.image_id,frame.subframe,frame.duration_ticks),(0,if index==0{12}else{index as u32-1},durations[index]));
        assert_eq!(frame.events,if index==1{vec![FrameEvent::Sound(1510)]}else{vec![]});
    }
    let mut s=snapshot();let mut packet=vec![0x37,1,0,0,0];packet.extend(2951u16.to_le_bytes());s.world.accept(&packet).unwrap();
    let own=s.world.own_actor().unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:PixelPoint{x:400,y:300}};
    let mut effects=Effects::default();effects.prepare(root,&s.world,&AtomicBool::new(false));let mut cues=vec![];
    for step in 0..=250u64 {
        let mut fragments=vec![];let (report,events)=effects.draw_to(&s.world,&mut Target::Fragments(&mut fragments),view,step*10_000_000);
        assert!(report["load_error"].is_null(),"{report}");cues.extend(events);
        if step==250 {assert_eq!(report["active"],0);}
    }
    assert_eq!(cues.len(),1);assert_eq!((cues[0].frame,cues[0].sound),(1,Some(1510)));
    assert_eq!(cues[0].prepared_audio.as_ref().unwrap().group().mode,NativeAudioMode::SuppressWhileSameIdPlaying);
    s.world.clear_preserving_generation();let mut fragments=vec![];
    let (report,events)=effects.draw_to(&s.world,&mut Target::Fragments(&mut fragments),view,u64::MAX);
    assert_eq!(report["active"],0);assert!(events.is_empty());
}
