use lito_modern::modern_lito::{compositor::Canvas, native_environment::State,
    native_light_mask::Source, native_render_environment::Bound, native_world::World,
    native_world_lighting::{metadata,Runtime}, sprite_actions::{ActionLimits,SpriteActionCatalog}};
use std::{collections::BTreeMap,path::Path,sync::atomic::{AtomicBool,Ordering}};
fn catalog(text:&str)->SpriteActionCatalog {
    SpriteActionCatalog::parse_for_inspection(text.as_bytes(),ActionLimits::default()).unwrap()
}
fn current() -> (World,State) {
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut world=World::default();
    world.relations.begin(2).unwrap();
    for key in ["world","status","actor","bot"] {
        if key=="status" {assert!(world.accept(&[0xd2]).unwrap());}
        let s=f["packets"][key].as_str().unwrap();
        let bytes:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        assert!(world.accept(&bytes).unwrap());
    }
    let mut env=State::default();env.begin(2).unwrap();env.map_changed(2,world.map_id.unwrap(),Some(0)).unwrap();
    (world,env)
}
fn runtime()->Runtime {Runtime::from_source(Source::parse(include_bytes!("fixtures/native-light-source.lmp")).unwrap())}
fn water_draw(runtime:&mut Runtime,world:&World,env:&State,catalog:&SpriteActionCatalog,canvas:&mut Canvas)->serde_json::Value {
    let centers=world.actors.keys().map(|id|(*id,[400,306])).collect();
    let cells=world.actors.iter().map(|(id,a)|(*id,[a.x,a.y])).collect();
    let actions=world.actors.keys().map(|id|(*id,0)).collect();
    runtime.draw_with_actions(Path::new("."),catalog,world,&Bound::capture(2,world,env).unwrap(),
        &centers,&cells,&actions,canvas,&AtomicBool::new(false)).unwrap()
}

#[test]
fn logical_metadata_uses_native_defaults_and_low_byte_without_alias_inheritance() {
    let c=catalog("#1 1\n102.type(5)\n108.subtype(258)\n#2 1=1\n#3 1\n102.type(-1)\n");
    assert_eq!(metadata(&c,1).unwrap(),[5,2]);
    assert_eq!(metadata(&c,2).unwrap(),[1,0]);
    assert_eq!(metadata(&c,3).unwrap(),[255,0]);
    assert!(metadata(&c,4).is_err());
    let invalid=catalog("#1 1\n102.type(5 6)\n");
    assert!(metadata(&invalid,1).is_err());
}

#[test]
fn full_bright_is_exact_and_does_not_open_resources_or_change_retained_selector() {
    let (world,env)=current();let bound=Bound::capture(2,&world,&env).unwrap();
    let mut runtime=Runtime::default();let mut canvas=Canvas::new(800,600).unwrap();canvas.clear([173,91,44]);
    let before=canvas.rgba().to_vec();
    let row=runtime.draw(Path::new("absent-resource-directory"),&catalog("#1 1\n"),&world,&bound,&BTreeMap::new(),&mut canvas,&AtomicBool::new(false)).unwrap();
    assert_eq!(row["reason"],"native_full_bright_mask_bypass");assert_eq!(canvas.rgba(),before);assert_eq!(runtime.remap().level(),None);
}

#[test]
fn blindness_uses_local_identity_in_signed_registration_order_and_shared_centers() {
    let (world,mut env)=current();env.accept(2,&[0x0a,1]).unwrap();
    let own=world.own_actor().unwrap().id;
    assert_ne!(world.actors.keys().next().copied(),Some(own));
    let centers=world.actors.keys().map(|id|(*id,if *id==own {[400,306]}else{[424,306]})).collect();
    let mut canvas=Canvas::new(800,600).unwrap();canvas.clear([255,255,255]);
    let row=runtime().draw(Path::new("."),&catalog("#1 1\n"),&world,&Bound::capture(2,&world,&env).unwrap(),&centers,&mut canvas,&AtomicBool::new(false)).unwrap();
    assert_eq!(row["active"],true);assert_eq!(row["actor_ids_signed_order"][0],own);
    assert_eq!(row["composition"]["blits"].as_array().unwrap().len(),1);
    assert_eq!(row["composition"]["blits"][0]["actor_index"],0);
    assert_eq!(row["composition"]["blits"][0]["center"],serde_json::json!([400,306]));
    assert_eq!(&canvas.rgba()[..4],&[0,0,0,255]);
    assert!(canvas.rgba().chunks_exact(4).any(|p|p[0]>200));
    assert!(row["pixels"]["transformed"].as_u64().unwrap()>400_000);
    assert_eq!(world.own_actor().unwrap().id,own);
}

#[test]
fn water_outer_bypass_updates_native_selector_and_cancel_or_wrong_map_never_changes_pixels() {
    let (world,mut env)=current();env.map_changed(2,world.map_id.unwrap(),Some(1)).unwrap();
    let mut runtime=runtime();let mut canvas=Canvas::new(800,600).unwrap();canvas.clear([177,88,33]);let before=canvas.rgba().to_vec();
    let bound=Bound::capture(2,&world,&env).unwrap();let cancel=AtomicBool::new(false);
    let row=water_draw(&mut runtime,&world,&env,&catalog("#0 1\n#61 1\n"),&mut canvas);
    assert_eq!(row["reason"],"native_unblinded_water_mask_bypass");assert_eq!(runtime.remap().level(),Some(3));assert_eq!(canvas.rgba(),before);
    let retained=*runtime.remap().row();
    env.map_changed(2,world.map_id.unwrap(),Some(0)).unwrap();
    runtime.draw(Path::new("."),&catalog("#1 1\n"),&world,&Bound::capture(2,&world,&env).unwrap(),&BTreeMap::new(),&mut canvas,&cancel).unwrap();
    assert_eq!(*runtime.remap().row(),retained);
    cancel.store(true,Ordering::Release);
    assert!(runtime.draw(Path::new("."),&catalog("#1 1\n"),&world,&bound,&BTreeMap::new(),&mut canvas,&cancel).is_err());
    let mut other=world.clone();other.map_id=Some(1);cancel.store(false,Ordering::Release);
    assert!(runtime.draw(Path::new("."),&catalog("#1 1\n"),&other,&bound,&BTreeMap::new(),&mut canvas,&cancel).is_err());
    assert_eq!(canvas.rgba(),before);
}

#[test]
fn selector_history_survives_scene_replacement_but_retires_with_the_game_owner() {
    let (world,mut env)=current();env.map_changed(2,world.map_id.unwrap(),Some(1)).unwrap();
    let mut first=runtime();let mut canvas=Canvas::new(800,600).unwrap();let c=catalog("#1 1\n");let centers=BTreeMap::new();let cancel=AtomicBool::new(false);
    water_draw(&mut first,&world,&env,&catalog("#0 1\n#61 1\n"),&mut canvas);
    let retained=first.remap();assert_eq!(retained.level(),Some(3));
    let mut next=Runtime::default();assert!(next.restore_remap(0,retained.clone()).is_err());
    next.restore_remap(2,retained.clone()).unwrap();
    env.map_changed(2,world.map_id.unwrap(),Some(0)).unwrap();
    next.draw(Path::new("absent"),&c,&world,&Bound::capture(2,&world,&env).unwrap(),&centers,&mut canvas,&cancel).unwrap();
    assert_eq!(next.remap().level(),Some(3));assert_eq!(next.remap().row(),retained.row());
    env.retire();env.begin(4).unwrap();env.map_changed(4,world.map_id.unwrap(),Some(0)).unwrap();
    next.draw(Path::new("absent"),&c,&world,&Bound::capture(4,&world,&env).unwrap(),&centers,&mut canvas,&cancel).unwrap();
    assert_eq!(next.remap().level(),None);assert_eq!(*next.remap().row(),[0;32]);
}

#[test]
fn blind_two_does_not_query_catalog_for_own_or_outside_sources() {
    let (mut world,mut env)=current();env.accept(2,&[0x0a,2]).unwrap();
    let own=world.own_actor().unwrap().id;let position=[world.actors[&own].x,world.actors[&own].y];
    world.actors.get_mut(&own).unwrap().graphics=u16::MAX;
    let remote=*world.actors.keys().find(|id|**id!=own).unwrap();
    let actor=world.actors.get_mut(&remote).unwrap();actor.graphics=u16::MAX-1;actor.x=position[0]+18;actor.y=position[1];
    let centers=world.actors.keys().map(|id|(*id,[400,306])).collect();let cancel=AtomicBool::new(false);
    let mut canvas=Canvas::new(800,600).unwrap();canvas.clear([255,255,255]);
    let row=runtime().draw(Path::new("."),&catalog("#1 1\n"),&world,&Bound::capture(2,&world,&env).unwrap(),&centers,&mut canvas,&cancel).unwrap();
    assert_eq!(row["composition"]["outside_bounds"],1);
    assert_eq!(row["composition"]["blits"].as_array().unwrap().len(),1);
    assert_eq!(row["actor_ids_signed_order"][row["composition"]["blits"][0]["actor_index"].as_u64().unwrap() as usize],own);
    world.actors.get_mut(&remote).unwrap().x=position[0]+17;canvas.clear([177,88,33]);let before=canvas.rgba().to_vec();
    assert!(runtime().draw(Path::new("."),&catalog("#1 1\n"),&world,&Bound::capture(2,&world,&env).unwrap(),&centers,&mut canvas,&cancel).is_err());
    assert_eq!(canvas.rgba(),before);
}

#[test]
fn blind_two_queries_predicate_only_inside_the_native_inclusive_bounds() {
    let (mut world,mut env)=current();env.accept(2,&[0x0a,2]).unwrap();
    let own=world.own_actor().unwrap().id;let position=[world.actors[&own].x,world.actors[&own].y];
    world.actors.get_mut(&own).unwrap().graphics=u16::MAX;
    let remote=*world.actors.keys().find(|id|**id!=own).unwrap();
    let c=catalog("#1 1\n102.type(5)\n");let cancel=AtomicBool::new(false);let centers=world.actors.keys().map(|id|(*id,[400,306])).collect();
    for (dx,dy,inside) in [(17,15,true),(18,15,false),(17,16,false)] {
        let actor=world.actors.get_mut(&remote).unwrap();actor.graphics=if inside {1}else{u16::MAX-1};actor.x=position[0]+dx;actor.y=position[1]+dy;
        let mut canvas=Canvas::new(800,600).unwrap();canvas.clear([255,255,255]);
        let row=runtime().draw(Path::new("."),&c,&world,&Bound::capture(2,&world,&env).unwrap(),&centers,&mut canvas,&cancel).unwrap();
        assert_eq!(row["composition"]["blits"].as_array().unwrap().len(),if inside {2}else{1});
        assert_eq!(row["composition"]["outside_bounds"],usize::from(!inside));
    }
}

#[test]
fn water_uses_real_master_war_death_mode_and_current_display_action_fields() {
    let (mut world,mut env)=current();env.map_changed(2,world.map_id.unwrap(),Some(1)).unwrap();
    let own=world.own_actor().unwrap().id;let remote=2u32;
    let own_name=world.actors[&own].relations.name.clone();
    world.actors.get_mut(&own).unwrap().relations.flag_1f.reset(false);
    world.actors.get_mut(&own).unwrap().light=0;
    let c=catalog("#0 1\n102.type(0)\n#10 1\n102.type(10)\n108.subtype(2)\n#11 1\n102.type(11)\n#6 1\n102.type(6)\n");
    let centers=world.actors.keys().map(|id|(*id,[400,306])).collect();
    let cells=world.actors.iter().map(|(id,a)|(*id,[a.x,a.y])).collect();
    // gfx, master kind,1F,50, action, mode23, class, range. Class overrides
    // must still light with raw255 (signed negative); ordinary classes skip.
    for (gfx,master,flag,dead,action,mode,class,range) in [
        (10,0,false,false,0,0,10,0),(10,1,false,false,0,0,6,0),
        (10,2,false,false,0,0,5,14),(10,0,true,false,0,0,5,14),
        (10,0,true,true,0,0,3,5),(10,0,true,false,8,0,3,5),
        (10,0,false,false,8,0,2,0),(10,0,false,false,0,0xa2,1,0),
        (11,2,true,true,0,0,11,0),(11,0,false,false,28,0,1,0),
        (11,0,false,false,37,0,1,0),(11,0,false,false,128,0,11,0),
        (6,0,false,true,0,0,3,5),(6,0,false,false,8,0,3,5)] {
        let a=world.actors.get_mut(&remote).unwrap(); a.graphics=gfx;a.light=255;a.unknown=mode;
        a.relations.master=match master {1=>own_name.clone(),2=>b"Other".to_vec(),_=>Vec::new()};
        a.relations.flag_1f.reset(flag);a.draw_state.native_light_dead=dead;
        // Deliberately disagree with generic authority/death and retained cues.
        a.dead=!dead; a.actions.clear();
        let actions=[(own,0),(remote,action)].into_iter().collect();
        let mut canvas=Canvas::new(800,600).unwrap();canvas.clear([177,88,33]);let before=canvas.rgba().to_vec();
        let row=runtime().draw_with_actions(Path::new("."),&c,&world,&Bound::capture(2,&world,&env).unwrap(),
            &centers,&cells,&actions,&mut canvas,&AtomicBool::new(false)).unwrap();
        assert_eq!(row["water_source_classes"][remote.to_string()],class,"gfx{gfx} master{master} action{action}");
        let blits=row["composition"]["blits"].as_array().unwrap();
        assert_eq!(blits.len(),usize::from(range!=0));
        if range!=0 {assert_eq!(blits[0]["range"],range);}
        assert_eq!(row["source_classification_complete"],true);assert_eq!(row["active"],false);
        assert_eq!(canvas.rgba(),before); // separate extra water pass is still pending
    }
}

#[test]
fn water_war_end_uses_historical_graphics_metadata_after_actor_morph() {
    let (mut world,mut env)=current();env.map_changed(2,world.map_id.unwrap(),Some(1)).unwrap();
    let own=world.own_actor().unwrap().id;
    let clan=world.actors[&own].relations.clan.clone();
    let mut b=vec![0x36,1];b.extend(&clan);b.push(0);b.extend(b"Enemy\0");
    let remote=world.actors.get_mut(&2).unwrap();remote.relations.clan=b"Enemy".to_vec();remote.graphics=6;
    world.accept(&b).unwrap();b[1]=2;world.accept(&b).unwrap();
    world.actors.get_mut(&2).unwrap().graphics=10;
    let c=catalog("#0 1\n#6 1\n102.type(6)\n#10 1\n102.type(10)\n");
    let mut canvas=Canvas::new(800,600).unwrap();
    let snapshot=serde_json::to_value(&world).unwrap();
    let row=water_draw(&mut runtime(),&world,&env,&c,&mut canvas);
    assert_eq!(row["water_source_classes"]["2"],10);
    assert_eq!(serde_json::to_value(&world).unwrap(),snapshot);
    let missing=catalog("#0 1\n#10 1\n102.type(10)\n");
    let centers=world.actors.keys().map(|id|(*id,[400,306])).collect();
    let cells=world.actors.iter().map(|(id,a)|(*id,[a.x,a.y])).collect();
    let actions=world.actors.keys().map(|id|(*id,0)).collect();
    canvas.clear([177,88,33]);let before=canvas.rgba().to_vec();
    assert!(runtime().draw_with_actions(Path::new("."),&missing,&world,&Bound::capture(2,&world,&env).unwrap(),
        &centers,&cells,&actions,&mut canvas,&AtomicBool::new(false)).is_err());
    assert_eq!(canvas.rgba(),before);
}

#[test]
fn water_budget_stops_metadata_and_action_queries_at_exact_native_700_limit() {
    let (mut world,mut env)=current();env.map_changed(2,world.map_id.unwrap(),Some(1)).unwrap();
    let own=world.own_actor().unwrap().id; let template=world.actors[&own].clone();world.actors.clear();
    for id in std::iter::once(own).chain(1..=50) {
        let mut a=template.clone();a.id=id;a.graphics=if id==50 {u16::MAX}else{0};
        a.relations.flag_1f.reset(true);world.actors.insert(id,a);
    }
    let c=catalog("#0 1\n");let centers=world.actors.keys().map(|id|(*id,[400,306])).collect();
    let cells=world.actors.iter().map(|(id,a)|(*id,[a.x,a.y])).collect();
    let actions=world.actors.keys().filter(|id|**id!=50).map(|id|(*id,0)).collect();
    let mut canvas=Canvas::new(800,600).unwrap();
    let row=runtime().draw_with_actions(Path::new("."),&c,&world,&Bound::capture(2,&world,&env).unwrap(),
        &centers,&cells,&actions,&mut canvas,&AtomicBool::new(false)).unwrap();
    assert_eq!(row["composition"]["accumulated_range"],700);
    assert_eq!(row["composition"]["blits"].as_array().unwrap().len(),50);
    assert!(row["water_source_classes"]["50"].is_null());
}

#[test]
fn water_requires_live_relation_owner_and_shared_action_before_mutating_pixels() {
    let (mut world,mut env)=current();env.map_changed(2,world.map_id.unwrap(),Some(1)).unwrap();
    let centers=world.actors.keys().map(|id|(*id,[400,306])).collect();
    let mut canvas=Canvas::new(800,600).unwrap();canvas.clear([177,88,33]);let before=canvas.rgba().to_vec();
    let c=catalog("#0 1\n#61 1\n");
    assert!(runtime().draw(Path::new("."),&c,&world,&Bound::capture(2,&world,&env).unwrap(),&centers,&mut canvas,&AtomicBool::new(false)).is_err());
    assert_eq!(canvas.rgba(),before);
    world.relations.retire();let cells=world.actors.iter().map(|(id,a)|(*id,[a.x,a.y])).collect();
    let actions=world.actors.keys().map(|id|(*id,0)).collect();
    assert!(runtime().draw_with_actions(Path::new("."),&c,&world,&Bound::capture(2,&world,&env).unwrap(),
        &centers,&cells,&actions,&mut canvas,&AtomicBool::new(false)).is_err());
    assert_eq!(canvas.rgba(),before);
}
