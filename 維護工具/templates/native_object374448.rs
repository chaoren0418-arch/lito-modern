use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase,Snapshot},native_inventory::Item,native_skills as skills,native_skill_selection::Model};
const ITEM:u32=0xfedcba98;
const PEER:u32=0xf1234567;
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
    s.world.hp=Some([50,50]);s.world.mp=Some([1,200]);s.world.learned_skills.extend([37,44,48]);
    let mut peer=s.world.own_actor().unwrap().clone();peer.id=PEER;peer.x+=1;s.world.actors.insert(PEER,peer);
    s.inventory.items.insert(ITEM,Item{id:ITEM,use_type:1,charge:0,icon:1,blessing:0,count:1,identified:1,name:"server owned weapon".into(),detail:vec![0]});s
}
#[test]
fn object_spell_actor_and_inventory_identity_domains_do_not_alias() {
    let s=snapshot();let item=skills::ItemTarget::capture(&s,7,ITEM).unwrap();
    for (skill,row,col) in [(37,4,4),(44,5,3),(48,5,7)] {
        let mut self_bytes=vec![0x14,row,col];self_bytes.extend(1u32.to_le_bytes());
        let mut peer_bytes=vec![0x14,row,col];peer_bytes.extend(PEER.to_le_bytes());
        assert_eq!(skills::request(&s.world,skill,None).unwrap(),self_bytes);
        assert_eq!(skills::request(&s.world,skill,Some(PEER)).unwrap(),peer_bytes);
        assert!(skills::request(&s.world,skill,Some(ITEM)).is_err());
        assert!(skills::item_request(&s,7,skill,&item).is_err());
        assert!(skills::ground_request(&s.world,skill,[32768,32768]).is_err());
    }
}
#[test]
fn curse_removal_and_bless_weapon_preserve_original_asymmetric_screen_boundary() {
    for skill in [37,48] {
        for (dx,dy,allowed) in [(-19,0,true),(19,0,false),(0,-19,true),(0,19,false),(18,0,true),(20,0,false)] {
            let mut s=snapshot();let own=s.world.own_actor().unwrap();let x=i32::from(own.x)+dx;let y=i32::from(own.y)+dy;
            let peer=s.world.actors.get_mut(&PEER).unwrap();peer.x=x as u16;peer.y=y as u16;
            assert_eq!(skills::request(&s.world,skill,Some(PEER)).is_ok(),allowed,"skill={skill} dx={dx} dy={dy}");
        }
    }
}
#[test]
fn cancellation_has_three_cell_range_and_separate_original_reuse_clock() {
    for distance in [3,4] {let mut s=snapshot();let own=s.world.own_actor().unwrap();let x=own.x+distance;let y=own.y+distance;
        let peer=s.world.actors.get_mut(&PEER).unwrap();peer.x=x;peer.y=y;
        assert_eq!(skills::request(&s.world,44,Some(PEER)).is_ok(),distance==3);
    }
    assert_eq!(skills::reuse_millis(44),500);assert_eq!(skills::reuse_millis(37),10);assert_eq!(skills::reuse_millis(48),10);
}
#[test]
fn object_spell_selection_revokes_target_incarnation_revision_scene_and_correction() {
    use lito_modern::modern_lito::native_skill_selection::Target;
    let s=snapshot();let target=Target::capture(&s.world,PEER).unwrap();
    for kind in 0..4 {let mut stale=s.clone();let mut menu=Model::default();menu.sync(7,&s.world,true);menu.select(Some(48)).unwrap();
        match kind {0=>stale.world.actors.get_mut(&PEER).unwrap().incarnation+=1,1=>stale.world.actors.get_mut(&PEER).unwrap().target_revision+=1,2=>stale.world.generation+=1,_=>stale.world.local_corrections+=1,}
        assert!(menu.choice(7,&stale.world,Some(target)).is_err());
    }
}
#[test]
fn object_spells_do_not_predict_mp_material_weapon_stats_or_cures() {
    let mut s=snapshot();s.world.mp=Some([1,200]);
    for skill in [37,44,48] {let before=serde_json::to_value(&s).unwrap();assert!(skills::request(&s.world,skill,Some(PEER)).is_ok());assert_eq!(serde_json::to_value(&s).unwrap(),before);}
    // No local material row or equipped weapon is inferred as server eligibility.
    assert_eq!(s.inventory.items.len(),1);assert_eq!(s.inventory.items[&ITEM].detail,vec![0]);
}
#[test]
fn object_spells_obey_book_life_and_control_gates() {
    for skill in [37,44,48] {for kind in 0..6 {let mut s=snapshot();match kind {
        0=>s.world.mp=Some([0,200]),1=>s.world.hp=Some([0,50]),2=>{s.world.learned_skills.remove(&skill);},3=>s.world.actors.get_mut(&1).unwrap().dead=true,
        4=>{s.world.movement_blocks.insert(1);},_=>s.world.started=false,
    }assert!(skills::request(&s.world,skill,None).is_err());}}
}
#[test]
fn equipment79_is_scoped_without_inventing_weapon_success() {
    use std::time::{Instant,Duration};let mut s=snapshot();let now=Instant::now();
    let pending=skills::Pending{since:now,casts:s.world.own_cast_packets,messages:s.world.message_count,generation:s.world.generation,incarnation:s.world.own_actor().unwrap().incarnation};
    s.world.message_count+=1;s.world.last_message=Some((79,vec![]));
    assert_eq!(pending.observe_equipment(&s.world,now),skills::Feedback::Rejected(79));assert_eq!(pending.observe(&s.world,now),skills::Feedback::Pending);
    s.world.last_message=Some((161,vec![]));assert_eq!(pending.observe_equipment(&s.world,now),skills::Feedback::Pending);
    assert_eq!(pending.observe_equipment(&s.world,now+Duration::from_secs(2)),skills::Feedback::NoResponse);
    s.world.last_message=Some((79,vec![]));s.world.own_cast_packets+=1;assert_eq!(pending.observe_equipment(&s.world,now),skills::Feedback::CastAction);
    s.world.generation+=1;assert_eq!(pending.observe_equipment(&s.world,now),skills::Feedback::Interrupted);
}
#[test]
fn object_spell_menu_is_current_learned_actor_intent() {
    let mut s=snapshot();let mut menu=Model::default();menu.sync(7,&s.world,true);
    for (id,text) in [(37,"#37 解除詛咒（選取目標／自己）"),(44,"#44 相消術（選取目標／自己，三格內；消耗 2 顆魔法寶石）"),(48,"#48 祝福魔法武器（選取目標／自己）")] {
        assert_eq!(menu.rows().iter().find(|r|r.id==id).unwrap().label,text);menu.select(Some(id)).unwrap();assert_eq!(menu.choice(7,&s.world,None).unwrap().target,Some(1));
    }
    assert!(menu.selected_skill(8,&s.world).is_err());s.world.learned_skills.remove(&48);assert!(menu.selected_skill(7,&s.world).is_err());menu.sync(7,&s.world,true);assert_eq!(menu.selected(),None);
}
#[test]
fn object_spell_original_effect_cadence_alias_audio_and_retirement() {
    use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits,ActionKey,FrameEvent},native_skill_effects::Effects,native_world_effects::Target,world_map::{WorldCell,PixelPoint,ViewProjection},combat_animation::NativeAudioMode};
    use std::{path::Path,sync::atomic::AtomicBool};let root=Path::new(r"D:\lineage開發\0-客戶端");let raw=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
    for (gfx,bank,sound,ticks) in [(227u16,227,122,vec![4,2,2,4,4,2,2]),(870u16,232,140,vec![4,2,2,4,4,2,2]),(2176u16,2176,800,vec![4;17])] {
        assert_eq!(catalog.sprites[&u32::from(gfx)].graphics_id,bank);let animation=catalog.resolve_action(u32::from(gfx),ActionKey::new(0)).unwrap().animation;assert_eq!(animation.frames.len(),ticks.len());
        for (i,frame) in animation.frames.iter().enumerate() {assert_eq!(frame.duration_ticks,ticks[i]);assert_eq!(frame.subframe,if gfx==2176{if i==0{16}else{i as u32-1}}else{i as u32});assert_eq!(frame.events,if i==1{vec![FrameEvent::Sound(sound)]}else{vec![]});}
        let mut s=snapshot();let mut packet=vec![0x37,1,0,0,0];packet.extend(gfx.to_le_bytes());s.world.accept(&packet).unwrap();let before=serde_json::to_value(&s).unwrap();
        let own=s.world.own_actor().unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:PixelPoint{x:400,y:300}};let mut effects=Effects::default();effects.prepare(root,&s.world,&AtomicBool::new(false));let mut cues=vec![];
        for step in 0..=300u64 {let mut fragments=vec![];let(report,events)=effects.draw_to(&s.world,&mut Target::Fragments(&mut fragments),view,step*10_000_000);assert!(report["load_error"].is_null(),"{report}");cues.extend(events);if step==300{assert_eq!(report["active"],0);}}
        assert_eq!(serde_json::to_value(&s).unwrap(),before);assert_eq!(cues.len(),1);assert_eq!((cues[0].frame,cues[0].sound),(1,Some(sound)));assert_eq!(cues[0].prepared_audio.as_ref().unwrap().group().mode,NativeAudioMode::SuppressWhileSameIdPlaying);
        s.world.clear_preserving_generation();let mut fragments=vec![];let(report,events)=effects.draw_to(&s.world,&mut Target::Fragments(&mut fragments),view,u64::MAX);assert_eq!(report["active"],0);assert!(events.is_empty());
    }
}
