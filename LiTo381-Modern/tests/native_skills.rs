use lito_modern::modern_lito::{native_skills as skills, native_world::World};
fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(data).unwrap();
    let hex = v["packets"][key].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn p(key: &str) -> Vec<u8> {
    packet(include_bytes!("fixtures/native-skills-current.json"), key)
}
fn world() -> World {
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(
            include_bytes!("fixtures/native-world-current.json"),
            key,
        ))
        .unwrap();
    }
    let mut own = w.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    w
}

#[test]
fn current_book_is_atomic_add_remove_and_survives_map_but_not_new_world_owner() {
    let mut w = World::default();
    w.accept(&p("add")).unwrap();
    assert_eq!(
        w.learned_skills.iter().copied().collect::<Vec<_>>(),
        vec![1, 3, 4, 43, 220]
    );
    w.accept(&p("remove")).unwrap();
    assert_eq!(
        w.learned_skills.iter().copied().collect::<Vec<_>>(),
        vec![1, 4, 43]
    );
    w.accept(&p("empty")).unwrap();
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert_eq!(w.learned_skills.len(), 3);
    let before = serde_json::to_value(&w).unwrap();
    let b = p("add");
    for end in 0..b.len() {
        assert!(w.accept(&b[..end]).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    let mut bad = b.clone();
    *bad.last_mut().unwrap() = 0x10;
    assert!(w.accept(&bad).is_err());
    let mut bad = b;
    bad.push(0);
    assert!(w.accept(&bad).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
}
#[test]
fn spell_requests_use_formal_selectors_and_never_predict_mana() {
    let mut w = world();
    w.accept(&p("add")).unwrap();
    w.accept(&p("mana")).unwrap();
    let before = serde_json::to_value(&w).unwrap();
    assert_eq!(skills::request(&w, 1, None).unwrap(), p("heal"));
    assert_eq!(skills::request(&w, 3, None).unwrap(), p("shield"));
    assert_eq!(skills::request(&w, 43, None).unwrap(), vec![0x14, 5, 2, 1, 0, 0, 0]);
    assert_eq!(skills::request(&w, 43, Some(2)).unwrap(), vec![0x14, 5, 2, 2, 0, 0, 0]);
    // Formal native screen is asymmetric at distance nineteen. Requests remain
    // pure: effective MP cost, equipment discounts and speed are server-owned.
    for (dx, accepted) in [(18i32, true), (19, false), (-19, true), (-20, false)] {
        let mut edge = w.clone();
        let own_x = edge.actors[&1].x;
        let own_y = edge.actors[&1].y;
        let actor = edge.actors.get_mut(&2).unwrap();
        actor.x = u16::try_from(i32::from(own_x) + dx).unwrap();
        actor.y = own_y;
        let snapshot = serde_json::to_value(&edge).unwrap();
        assert_eq!(skills::request(&edge, 43, Some(2)).is_ok(), accepted);
        assert_eq!(serde_json::to_value(&edge).unwrap(), snapshot);
    }
    for change in 0..6 {
        let mut refused = w.clone();
        match change {
            0 => { refused.learned_skills.remove(&43); },
            1 => refused.mp = Some([0, 10]),
            2 => refused.hp = Some([0, 50]),
            3 => refused.actors.get_mut(&2).unwrap().dead = true,
            4 => { refused.actors.remove(&2); },
            _ => { refused.movement_blocks.insert(2); },
        }
        assert!(skills::request(&refused, 43, Some(2)).is_err());
    }
    w.learned_skills.insert(8);
    assert_eq!(skills::request(&w, 8, Some(2)).unwrap(), vec![0x14,0,7,2,0,0,0]);
    w.learned_skills.remove(&8);
    assert!(skills::request(&w, 1, Some(999)).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    w.mp = Some([0, 10]);
    assert!(skills::request(&w, 1, None).is_err());
    w.mp = Some([10, 10]);
    w.learned_skills.clear();
    assert!(skills::request(&w, 1, None).is_err());
}
#[test]
fn effects_preserve_formal_unsigned_ids_and_are_bounded_atomic_and_map_scoped() {
    let mut w = world();
    w.accept(&p("heal_effect")).unwrap();
    assert_eq!(w.effects[0].graphics, 744);
    assert_eq!(w.effects[0].target, Some((1, w.actors[&1].incarnation)));
    w.accept(&p("location_effect")).unwrap();
    assert_eq!(w.effects[1].position, [32729, 32797]);
    assert_eq!(w.effects[1].target, None);
    let before = serde_json::to_value(&w).unwrap();
    let b = p("heal_effect");
    for end in 0..b.len() {
        assert!(w.accept(&b[..end]).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    for _ in 0..128 {
        w.accept(&b).unwrap();
    }
    assert_eq!(w.effects.len(), 128);
    assert_eq!(w.effects_dropped, 2);
    w.accept(&p("effect_boundary")).unwrap();
    assert!(w.orphan_updates > 0);
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert!(w.effects.is_empty());
}
#[test]
fn skill_feedback_requires_authority_and_expires_without_guessing_success() {
    use std::time::{Duration, Instant};
    let mut w = world();
    let now = Instant::now();
    let pending = skills::Pending {
        since: now,
        casts: 0,
        messages: 0,
        generation: w.generation,
        incarnation: w.actors[&1].incarnation,
    };
    assert_eq!(pending.observe(&w, now), skills::Feedback::Pending);
    assert_eq!(
        pending.observe(&w, now + Duration::from_secs(2)),
        skills::Feedback::NoResponse
    );
    w.accept(&[0x10, 22, 1, 0]).unwrap();
    assert_eq!(pending.observe(&w, now), skills::Feedback::Rejected(278));
    w.accept(&[0x10, 1, 0, 0]).unwrap();
    assert_eq!(pending.observe(&w, now), skills::Feedback::Rejected(278));
    let pending = skills::Pending {
        messages: w.message_count,
        ..pending
    };
    w.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(pending.observe(&w, now), skills::Feedback::CastAction);
    w.actors.get_mut(&1).unwrap().incarnation += 1;
    assert_eq!(pending.observe(&w, now), skills::Feedback::Interrupted);
}
#[test]
fn source_heal_and_shield_effects_play_once_independently_of_actor_replacement() {
    use lito_modern::modern_lito::native_world_scene::Model;
    use std::{path::Path, sync::atomic::AtomicBool};
    let mut w = world();
    w.accept(&p("add")).unwrap();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    let first = model.draw_at(&w, &cancel, 0).unwrap();
    assert!(
        first.receipt["skill_effects"]["load_error"].is_null(),
        "{}",
        first.receipt["skill_effects"]
    );
    w.accept(&p("heal_effect")).unwrap();
    w.accept(&p("shield_effect")).unwrap();
    let before = serde_json::to_value(&w).unwrap();
    let mut frames = Vec::new();
    let mut sound_keys = std::collections::BTreeSet::new();
    // Poll at the actual original frame deadlines as well as the scene samples.
    // Positive-delay native callbacks reschedule from dispatch, so 100ms polling
    // must not be mistaken for an uninterrupted native timer queue.
    use lito_modern::modern_lito::{resources::{SharedArchive, SharedArchiveKind},
        sprite_actions::{ActionKey, ActionLimits, SpriteActionCatalog}};
    let bytes = SharedArchive::open(Path::new(r"D:\lineage開發\0-客戶端"), SharedArchiveKind::Text)
        .unwrap().read_entry("list.spr").unwrap();
    let catalog = SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
    let mut times: std::collections::BTreeSet<u64> = (0..=3000).step_by(100).collect();
    for graphics in [221, 744] {
        let animation = catalog.resolve_action(graphics, ActionKey::new(0)).unwrap().animation;
        let mut due = 0;
        for source in (1..animation.frames.len()).chain(std::iter::once(0)) {
            due += u64::from(animation.frames[source].duration_ticks) * 1000 / 24;
            if due <= 3000 { times.insert(due); }
        }
    }
    for ms in times {
        let f = model.draw_at(&w, &cancel, ms * 1_000_000).unwrap();
        let mut r = f.receipt;
        assert!(r["skill_effects"]["load_error"].is_null());
        for e in r["skill_effects"]["entered_events"].as_array().unwrap() {
            assert!(sound_keys.insert(e.to_string()));
        }
        if [0, 500, 1500, 2500].contains(&ms) {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "target/current/verification/native-skills-{ms}.png"
            ));
            let mut enc = png::Encoder::new(std::fs::File::create(&path).unwrap(), 800, 600);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .unwrap()
                .write_image_data(f.canvas.rgba())
                .unwrap();
            r["png"] = serde_json::json!(path);
        }
        frames.push(r);
    }
    assert_eq!(frames.last().unwrap()["skill_effects"]["started"], 2);
    assert_eq!(frames.last().unwrap()["skill_effects"]["active"], 0);
    assert!(sound_keys.len() >= 2);
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    w.accept(&p("heal_effect")).unwrap();
    let active = model.draw_at(&w, &cancel, 3_100_000_000).unwrap();
    assert_eq!(active.receipt["skill_effects"]["active"], 1);
    let mut replacement = packet(
        include_bytes!("fixtures/native-world-current.json"),
        "actor",
    );
    replacement[5..9].copy_from_slice(&1u32.to_le_bytes());
    w.accept(&replacement).unwrap();
    let detached = model.draw_at(&w, &cancel, 3_200_000_000).unwrap();
    assert_eq!(detached.receipt["skill_effects"]["active"], 1);
    assert_eq!(detached.receipt["skill_effects"]["entered_events"], serde_json::json!([]));
    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/current/verification/native-skills.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
}

#[test]
fn original_tw_basic_heal_frames_sound_clock_and_retirement_preserve_authority() {
    use lito_modern::modern_lito::{native_skill_effects::Effects, native_world_effects::{Target,Mix},
        resources::{SharedArchive,SharedArchiveKind}, sprite,
        world_map::{WorldCell,PixelPoint,ViewProjection}};
    use sha2::{Digest,Sha256};
    use std::{path::Path,sync::atomic::AtomicBool};
    let root=Path::new(r"D:\lineage開發\0-客戶端");
    let mut w=world();w.accept(&p("add")).unwrap();
    w.accept(&[0x37,1,0,0,0,0xca,0x2d]).unwrap();
    assert_eq!(w.effects.last().unwrap().graphics,11722);
    let position=w.effects.last().unwrap().position;
    let before=serde_json::to_value(&w).unwrap();
    let mut archive=SharedArchive::open(root,SharedArchiveKind::SpriteShard(13)).unwrap();
    let bytes=archive.read_entry("11722-0.spr").unwrap();
    assert_eq!(format!("{:X}",Sha256::digest(&bytes)),"85F6872052FA335CAB1EE565C3E184E2D4E718A6137C87D28850C88ECCA6F8E5");
    let reference=sprite::decode_classic_spr_with_limits(&bytes,Default::default()).unwrap();
    assert_eq!(reference.frames.len(),21);
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-TW11722-original-frame-metadata.json"),
        serde_json::to_vec_pretty(&reference.frames.iter().enumerate().map(|(i,f)|serde_json::json!({"subframe":i,"x":f.x_offset,"y":f.y_offset,"width":f.width,"height":f.height,"unknown1":f.unknown1,"unknown2":f.unknown2,"rgba_bytes":f.rgba.len(),"rgba_sha256":format!("{:X}",Sha256::digest(&f.rgba)),"opaque_pixels":f.rgba.chunks_exact(4).filter(|p|p[3]!=0).count()})).collect::<Vec<_>>()).unwrap()).unwrap();
    let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(position[0]),y:i32::from(position[1])},anchor_pixel:PixelPoint{x:400,y:300}};
    let mut fx=Effects::default();fx.prepare(root,&w,&AtomicBool::new(false));
    let mut observations=Vec::new();
    for step in 0..=21u64 {
        let mut queue=Vec::new();
        let (r,events)=fx.draw_to(&w,&mut Target::Fragments(&mut queue),view,step*125_000_000);
        assert!(r["load_error"].is_null(),"{r}");
        assert_eq!(r["started"],1,"accepted11722 must use its original TW definition and SPR: {r}");
        assert_eq!(r["omitted"],0);
        assert_eq!(events.iter().filter_map(|e|e.sound).collect::<Vec<_>>(),if step==0{vec![109]}else{vec![]});
        let frame=&r["effects"][0];
        assert_eq!(frame["graphics"],11722);assert_eq!(frame["sprite"],"11722-0.spr");
        assert_eq!(frame["callback_count"],step+1);
        assert_eq!(frame["native_clock_elapsed_ms"],step*125);
        let source=if step<20{step+1}else{0};
        assert_eq!(frame["source_frame"],source);
        let sub=if source==0{20}else{source as usize-1};assert_eq!(frame["subframe"],sub);
        if step<21 {
            assert_eq!(r["active"],1);
            let expected=&reference.frames[sub];
            if matches!(sub,0|13..=20) {
                assert_eq!((expected.x_offset,expected.y_offset,expected.width,expected.height,expected.unknown1,expected.unknown2),(192,120,0,0,u16::MAX,0));
                assert!(expected.rgba.is_empty());assert!(queue.is_empty());assert_eq!(frame["drawn"],false);
            } else {
                assert_eq!(queue.len(),1);assert_eq!(frame["drawn"],true);
                assert!(matches!(queue[0].mix,Mix::Screen));assert_eq!(queue[0].position,view.anchor_cell);
                let actual=queue[0].frame;assert!(actual.fragments.is_some());
                assert_eq!((actual.x_offset,actual.y_offset,actual.width,actual.height,actual.unknown1,actual.unknown2),
                    (expected.x_offset,expected.y_offset,expected.width,expected.height,expected.unknown1,expected.unknown2));
                assert_eq!(actual.rgba,expected.rgba,"original source subframe{sub}");
            }
            assert_eq!(frame["next_callback_ms"],(step+1)*125);
        } else {
            assert_eq!(r["active"],0);assert!(queue.is_empty());assert_eq!(frame["drawn"],false);assert_eq!(frame["finished"],true);assert!(frame["next_callback_ms"].is_null());
        }
        assert_eq!(serde_json::to_value(&w).unwrap(),before);
        observations.push(r);
    }
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-shared-basic-heal-frames.json"),serde_json::to_vec_pretty(&observations).unwrap()).unwrap();
}

#[test]
fn tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority() {
    use lito_modern::modern_lito::native_skill_selection;
    let mut w=world();w.accept(&p("add")).unwrap();w.accept(&p("mana")).unwrap();
    for skill in [19,35,57] {w.learned_skills.insert(skill);}
    let before=serde_json::to_value(&w).unwrap();
    assert!(skills::request(&w,1,Some(2)).is_ok(),"existing live-target fixture must remain valid");
    let mut menu=native_skill_selection::Model::default();assert!(menu.sync(11,&w,true));
    for (skill,row,column,clock) in [(19,2,2,100),(35,4,2,200),(57,7,0,500)] {
        let self_packet=vec![0x14,row,column,1,0,0,0];
        assert_eq!(skills::request(&w,skill,None).unwrap(),self_packet,"higher heal selector must be usable from its actual learned ID");
        assert_eq!(skills::request(&w,skill,Some(2)).unwrap(),vec![0x14,row,column,2,0,0,0]);
        assert_eq!(native_skill_selection::choice(&w,skill,None).unwrap().skill,skill);
        assert_eq!(skills::reuse_millis(skill),clock);
        assert_eq!(skills::client_clock_millis(skill),clock);
        assert!(!menu.rows().iter().find(|r|r.id==skill).unwrap().label.contains("尚未支援"));
        assert!(skills::request(&w,skill,Some(999)).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
    w.actors.get_mut(&2).unwrap().dead=true;
    let dead=serde_json::to_value(&w).unwrap();
    for skill in [19,35,57] {assert!(skills::request(&w,skill,Some(2)).is_err());}
    assert_eq!(serde_json::to_value(&w).unwrap(),dead);
    w.learned_skills.remove(&35);assert!(skills::request(&w,35,None).is_err());
}

#[test]
fn tw_higher_heal_original_slots_clothes_sound_and_retirement_preserve_authority() {
    use lito_modern::modern_lito::{native_skill_effects::Effects,native_world_effects::{Target,Mix},
        resources::{SharedArchive,SharedArchiveKind},sprite,
        world_map::{WorldCell,PixelPoint,ViewProjection}};
    use sha2::{Digest,Sha256};use std::{path::Path,sync::atomic::AtomicBool,collections::BTreeMap};
    let root=Path::new(r"D:\lineage開發\0-客戶端");
    // Pinned against a separate bounded Python reader of the same documented
    // SPR contract. These are source RGBA hashes, not a native-exe screenshot oracle.
    let originals: &[(u16,u8,usize,&str,&[(i32,i32,u32,u32,u16,u16,&str)])]=&[
        (11723,14,63949,"1534C34569664D816C2578CC692BE3FD66EBB1A3489B9F7B8B11870A39A34CFA",&[
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (0,-96,48,144,64766,265,"6413E72BB4922EB4C8400FAF99AD80689CD64C94451EA8DAE2495AB2CA1E1CBD"),
            (0,-120,48,168,64510,266,"E507F6F45904A504BBA28189BF695B5CBB0C1A276C696EB111258A38FC78A0FB"),
            (-24,-120,96,168,64510,266,"C267809BFB015E047BEF311492B8EB78A2F449DD756D660771986AFC4FFFA574"),
            (0,-120,72,168,64510,266,"437B1C7B12C194C0E1AD3C3EC79707AE4233FFE38AF0A782E61A0B0891E5323B"),
            (0,-120,72,168,64510,266,"BC1AA07E5A2CB39E8B4F3C7E2BDAFED419DFF96E5EC24ABFEA47B5B7BB1D914F"),
            (-24,-120,96,168,64510,266,"E11C533A13C9FC288302CF72A43834640A2A12ABB39C50F8A0C0B398F27C1A11"),
            (-24,-108,96,156,64510,266,"5ACCC553A35EE00BFCA6B886BD3445554A0DA987FC17F23E3CA827528869BDEB"),
            (-24,-108,96,156,64510,266,"6A6EC483FCCEBE9CED338B3DB534EDF3D708DA6A94E7E5805DC5231249144328"),
            (0,-108,72,156,64510,266,"D49CDCA24FC1783C955FDD885A92C2D1BCBBAB83BA6A4D9C5CAB520E4CDB3578"),
            (0,-108,72,156,64510,266,"0A41979BD423092C166BB35946FB46D310313A62C303E87DD1CF613C5C5748CD"),
            (0,-108,48,156,64510,266,"CC7EEFC20239C4D3832A6BB4342548CED533B9BE266ABCE54E48314090327EF3"),
            (0,-108,48,156,64510,266,"C0B1C64AC4AE818CF6C2B581BCB51405A8A8E2417BF40D388687BF7EE185B684"),
            (0,-108,48,156,64510,265,"66B6266D928B7D54F6D5BB26516E322341FB544E5B206CE56911EC894E29E90B"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
        ]),
        (11724,15,98415,"FAEF47D9ED96FEB9773D0883C9D46899F0A9FF51546894BA3EA577EBFF93AA40",&[
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (0,-120,48,168,64510,267,"1E7C1FCA3468CCAB8748D6D128118077DF4DE504B84E74D1AA7FCDB22B35A00C"),
            (0,-120,48,168,64510,267,"DB2F374ED5294A3AE5E7CD29B8E4EF3B42ED634131457CD6BDA1F102B88AE0E5"),
            (-24,-120,96,168,64510,267,"5BDB0AEF7F2367D5749134B6F3A4D448895766E15435D386A4F615CC97137706"),
            (-24,-120,96,168,64510,267,"D3577F8CD97D886680D68F973196A8925FF1E4D974FBB6FA30383F88CBFE62BA"),
            (-24,-120,96,168,64510,267,"4397E6159AB7A48A8C6E558FD1597E83E8E7D83F05485B59520E6E9315398100"),
            (-24,-120,96,168,64510,267,"E76402E302E6D411B6277B3256BB34B7AE758B95F0E634FA6474200E8A8F558D"),
            (-24,-120,96,168,64510,267,"B7EE178B92066BAD1D117795D78547C1864773B45B321C249DB82EE52E9EB9DC"),
            (-24,-120,96,168,64510,267,"B8BAB218EE6600F1940B393C9CD830EEA8190FB6F11EC3BE2509FF604DFF94BE"),
            (-24,-120,96,168,64510,267,"E898AB70F1359A2C321672EC603CB13B73D31F17E74E9960D68D66B5F767CDAD"),
            (0,-120,72,168,64510,267,"0DD24CB963BEBF1549CB74D3C0FE8843AA8E1AF44590FF9DA22423AC813E3A9E"),
            (0,-120,48,168,64510,267,"2F424C624241F61C09F4D41AD5581FF7EF77EE248D5B13D602C81404AE35CA5F"),
            (0,-120,48,168,64510,267,"C0480B2C3E555D9D74135D644759AB8FD69379B0F2EF72874005A834CD996291"),
            (0,-120,48,168,64510,267,"94A084240234A97C3DB9F4E3A3DA9239BECEF4567DA8F8058FA518E85A519356"),
            (0,-108,48,156,64510,265,"2BDA9837CA5A3EF42DE4FE0169B9CE073DB026FB78C7E6CCE1443F77635C9772"),
            (24,-72,24,24,64775,64775,"3A3C54901D80CF18C03FD00E387772AA27BF36CFFAB4ABB32EEF84F216E66FD2"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
        ]),
        (11725,0,129975,"72656534C480225926B4B721EC13C088A5AC8816A0E0FA164BAC7994691816D4",&[
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (0,-120,48,168,64510,267,"9E090A2CADA793201201AE570CD08A8A7F80D017897CBCB2E728276EEDC053AA"),
            (-24,-120,96,168,64510,267,"F65B5E398E48E5B399A5037AD8209A025147CCF3112041939D888BF610C737D2"),
            (-24,-120,96,168,64510,267,"8E322051A8383C17CE6CEFCC5375E051165DB2B536FC80C0413AE6FB41D41C95"),
            (-24,-120,96,168,64510,267,"774AFD62DA1FEEB832D1FB0599F6B37DC88A59A9EF2C2A52A972143DB05D80B5"),
            (-24,-120,96,168,64510,267,"00CA3204AA05BA7F45E27F59A69D5F9E76C895A4A79CDBF973119AF263E09C39"),
            (-24,-120,96,168,64510,267,"604571B2599A82ED21C5B6A40C7DC45820064939E61DF4E70006D2F002301B60"),
            (-24,-120,96,180,64509,267,"1D4365FD60806C8D6C3B531DA612113605A0F1FC6FCBE8504C9202CCE5CEF0B5"),
            (-24,-120,96,168,64510,267,"94ED6B259441D0BB3F74C674DF4DE71BCDB61F753B43DC9919887D8EE6027090"),
            (-24,-120,96,168,64510,267,"04C334D8904BC27706FA2BA2AEE7ED7893ABD1CC685506BA72E4D1DD486A6E70"),
            (0,-120,72,168,64510,267,"D55B0A5492436482E6A0646934D55E6606712E318D287DC9D2DB7C53749F88FF"),
            (0,-132,72,180,64254,267,"BB7FA2AFBDE96AAB1E14FBA25BBC9106E4F03E96CCBC2667D9F1EF9CB513C913"),
            (0,-132,72,180,64254,267,"5CF01AEB1030D90AB5FE4B0ECCFEB013C6A2018C53503B990D486D5BD6CEB44C"),
            (0,-120,72,168,64510,267,"3E60BA844EC02970DAAA0FB687CCAFE16E2F8699B6568EA53E0A6465F11D58A7"),
            (0,-132,72,180,64510,268,"5AB9A77D06D1B8B6B38B327661FBE2DA403C34446CDC68C4CD2D56F117047E71"),
            (0,-132,48,180,64510,268,"CCD6CB4BF688AE343C34054EA7DD0BCE24CD3C6C2770B76B9E408B2C966CC619"),
            (0,-120,48,168,64510,267,"DCDB759D63509C59B88DADFE171AEC0DA2571195CC3031065C738966E8322C31"),
            (0,-120,48,168,64510,267,"802A998B94DA57E1B266EDE948017102F3AA2ACEF0C4D5D128D907BF75725EDE"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
        ]),
        (11726,1,44329,"EFC16BA8E0778312AB474AA269915D3B6D3551B2475C02D8A3FAE26FFFFDE4AA",&[
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (-24,-24,96,72,65534,259,"5FF787D7C883219C70535355A662D20F048C5C2A151DFD36D5D017ED0A5FA039"),
            (-24,-24,96,84,65532,515,"B22D1E2BC4F345A8178DF772AF6E1B290ABB800755D3D4A60D891C9A35F8B5A6"),
            (-24,-24,96,84,65532,515,"DB9907863B9687CAFBFFF46F31956FD6099550AE851E7F6C445761ACFC9EEF20"),
            (-24,-24,96,84,65532,515,"7AC6E9C72C99E86DF3E03964D43A775358845965A6439C57FEB69A1082ABBFDC"),
            (-24,-24,96,84,65532,515,"3A21A1FBBCC38A51FEB6C99DFE726DD96888BA32D30E57D5F781F56E528857EC"),
            (-24,-24,96,84,65532,515,"CB3136373431BC0CCDBB61EDB9995A2288C449928E1584690EC472A7DB83FF1C"),
            (-24,-24,96,84,65532,515,"F91385AE75F2012AD36D6551235B7B4E51D121FF9CCA5E03CCEA00EC277798BB"),
            (-24,-24,96,84,65532,515,"5DB6286E3CA04A65BD16FEB91852F44B0DC295A7F7DF153423DDF4BC26DC4079"),
            (-24,-24,96,84,65532,515,"7038B0DE7221C5F8492D97A3BFD927482FFE54AA3FB3EC5AD341A877629C2FAC"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
            (192,120,0,0,65535,0,"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
        ]),
    ];
    let mut reference=BTreeMap::new();
    for &(gfx,shard,size,digest,expected) in originals {
        let mut archive=SharedArchive::open(root,SharedArchiveKind::SpriteShard(shard)).unwrap();
        let bytes=archive.read_entry(&format!("{gfx}-0.spr")).unwrap();
        assert_eq!(bytes.len(),size);assert_eq!(format!("{:X}",Sha256::digest(&bytes)),digest);
        let decoded=sprite::decode_classic_spr_with_limits(&bytes,Default::default()).unwrap();
        assert_eq!(decoded.frames.len(),21);assert_eq!(expected.len(),21);
        for (frame,&(x,y,width,height,unknown1,unknown2,rgba_sha)) in decoded.frames.iter().zip(expected) {
            assert_eq!((frame.x_offset,frame.y_offset,frame.width,frame.height,frame.unknown1,frame.unknown2),(x,y,width,height,unknown1,unknown2));
            assert_eq!(format!("{:X}",Sha256::digest(&frame.rgba)),rgba_sha);
        }
        reference.insert(gfx,decoded.frames);
    }
    let mut groups=Vec::new();
    for (gfx,sound) in [(11723u16,114u32),(11724,115),(11725,126)] {
        let mut w=world();let mut packet=vec![0x37,1,0,0,0];packet.extend(gfx.to_le_bytes());w.accept(&packet).unwrap();
        let position=w.effects.last().unwrap().position;let before=serde_json::to_value(&w).unwrap();
        let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(position[0]),y:i32::from(position[1])},anchor_pixel:PixelPoint{x:400,y:300}};
        let mut fx=Effects::default();fx.prepare(root,&w,&AtomicBool::new(false));let mut observations=Vec::new();
        for step in 0..=21u64 {
            let mut queue=Vec::new();let (r,events)=fx.draw_to(&w,&mut Target::Fragments(&mut queue),view,step*125_000_000);
            assert!(r["load_error"].is_null(),"{r}");assert_eq!(r["started"],1,"actual wire gfx{gfx} omitted: {r}");assert_eq!(r["omitted"],0);
            assert_eq!(events.iter().filter_map(|e|e.sound).collect::<Vec<_>>(),if step==0{vec![sound]}else{vec![]});
            let row=&r["effects"][0];let source=if step<20{step+1}else{0};let sub=if source==0{20}else{source as usize-1};
            assert_eq!(row["graphics"],gfx);assert_eq!(row["sprite"],format!("{gfx}-0.spr"));assert_eq!(row["callback_count"],step+1);assert_eq!(row["source_frame"],source);assert_eq!(row["subframe"],sub);
            let mut visible=Vec::new();
            if step<21 {
                for layer in std::iter::once(gfx).chain((gfx==11725).then_some(11726)) {
                    let f=&reference[&layer][sub];if !f.rgba.is_empty(){visible.push(f);}
                }
            }
            assert_eq!(queue.len(),visible.len(),"full-heal clothes11726 uses the same source subframe; no omitted layer, looping or fake empty pixels");
            for (actual,expected) in queue.iter().zip(&visible) {
                assert!(matches!(actual.mix,Mix::Screen));assert_eq!(actual.position,view.anchor_cell);assert!(actual.frame.fragments.is_some());
                assert_eq!((actual.frame.x_offset,actual.frame.y_offset,actual.frame.width,actual.frame.height,actual.frame.unknown1,actual.frame.unknown2),
                    (expected.x_offset,expected.y_offset,expected.width,expected.height,expected.unknown1,expected.unknown2));
                assert_eq!(actual.frame.rgba,expected.rgba);
            }
            assert_eq!(row["drawn"],!visible.is_empty());
            if step<21 {assert_eq!(r["active"],1);assert_eq!(row["next_callback_ms"],(step+1)*125);}
            else {assert_eq!(r["active"],0);assert_eq!(row["finished"],true);assert!(row["next_callback_ms"].is_null());}
            assert_eq!(serde_json::to_value(&w).unwrap(),before);observations.push(r);
        }
        groups.push(serde_json::json!({"wire_graphics":gfx,"sound":sound,"observations":observations}));
    }
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-shared-higher-heal-frames.json"),serde_json::to_vec_pretty(&groups).unwrap()).unwrap();
}

#[test]
fn tw_higher_full_heal_group_budget_rejects_both_layers_atomically() {
    use lito_modern::modern_lito::{native_skill_effects::Effects,native_world_effects::{Target,Effect,Mix},
        resources::{SharedArchive,SharedArchiveKind},sprite,
        world_map::{WorldCell,PixelPoint,ViewProjection}};
    use std::{path::Path,sync::atomic::AtomicBool};
    let root=Path::new(r"D:\lineage開發\0-客戶端");let mut w=world();w.accept(&[0x37,1,0,0,0,0xcd,0x2d]).unwrap();
    let pos=w.effects.last().unwrap().position;let before=serde_json::to_value(&w).unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(pos[0]),y:i32::from(pos[1])},anchor_pixel:PixelPoint{x:400,y:300}};
    let mut fx=Effects::default();fx.prepare(root,&w,&AtomicBool::new(false));
    {let mut empty=Vec::new();let (first,events)=fx.draw_to(&w,&mut Target::Fragments(&mut empty),view,0);assert_eq!(first["started"],1,"{first}");assert!(empty.is_empty());assert_eq!(events.iter().filter_map(|e|e.sound).collect::<Vec<_>>(),vec![126]);}
    let mut archive=SharedArchive::open(root,SharedArchiveKind::SpriteShard(0)).unwrap();let bytes=archive.read_entry("11725-0.spr").unwrap();let seed=sprite::decode_world_spr_with_limits(&bytes,Default::default()).unwrap();
    let mut queue=(0..511).map(|_|Effect{frame:&seed.frames[1],position:view.anchor_cell,mix:Mix::Screen}).collect::<Vec<_>>();
    let (result,events)=fx.draw_to(&w,&mut Target::Fragments(&mut queue),view,125_000_000);
    assert_eq!(queue.len(),511,"neither body nor clothes may be submitted when the complete group exceeds512");assert_eq!(result["omitted"],1);assert_eq!(result["active"],0);assert!(result["effects"][0]["error"].as_str().unwrap().contains("effect submission budget"));assert!(events.is_empty());assert_eq!(serde_json::to_value(&w).unwrap(),before);
}

#[test]
fn legacy_higher_heal_aliases_keep_original744_pixels_wire_ids_and_individual_sounds() {
    use lito_modern::modern_lito::{native_skill_effects::Effects,native_world_effects::{Target,Mix},
        sprite_actions::{ActionKey,ActionLimits,SpriteActionCatalog},
        resources::{SharedArchive,SharedArchiveKind},sprite,
        world_map::{WorldCell,PixelPoint,ViewProjection}};
    use std::{path::Path,sync::atomic::AtomicBool};use sha2::{Digest,Sha256};
    let root=Path::new(r"D:\lineage開發\0-客戶端");let cancel=AtomicBool::new(false);let mut text_archive=SharedArchive::open(root,SharedArchiveKind::Text).unwrap();let catalog_bytes=text_archive.read_entry("list.spr").unwrap();let catalog=SpriteActionCatalog::parse_for_inspection(&catalog_bytes,ActionLimits{max_bytes:8*1024*1024,..Default::default()}).unwrap();
    assert_eq!(catalog_bytes.len(),4_067_417);
    assert_eq!(format!("{:X}",Sha256::digest(&catalog_bytes)),"523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127");
    let mut definitions=Vec::new();
    for gfx in [829,830,831] {
        let row=&catalog.sprites[&gfx];let resolved=catalog.resolve_action(gfx,ActionKey::new(0)).unwrap();let a=resolved.animation;
        definitions.push(serde_json::json!({"wire_graphics":gfx,"graphics_id":row.graphics_id,"definition_sprite":resolved.definition_sprite_id,"image_count":row.image_count,"header_issue":format!("{:?}",row.header_issue),"direction":format!("{:?}",a.directionality),"FPS":a.frames_per_second,
            "attributes":row.attributes.iter().map(|v|serde_json::json!({"key":format!("{:?}",v.key),"body":v.body_bytes})).collect::<Vec<_>>(),
            "frames":a.frames.iter().map(|f|serde_json::json!({"image":f.image_id,"subframe":f.subframe,"ticks":f.duration_ticks,"events":format!("{:?}",f.events)})).collect::<Vec<_>>() }));
    }
    let output=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");std::fs::create_dir_all(&output).unwrap();std::fs::write(output.join("native-legacy-higher-heal-original-definitions.json"),serde_json::to_vec_pretty(&definitions).unwrap()).unwrap();
    let mut archive=SharedArchive::open(root,SharedArchiveKind::Sprite).unwrap();let bytes=archive.read_entry("744-0.spr").unwrap();assert_eq!(bytes.len(),7259);assert_eq!(format!("{:X}",Sha256::digest(&bytes)),"5532E36EDCD31152777B044F7F15CC1424C0D178BDE875C31B35358A99A2819C");let reference=sprite::decode_classic_spr_with_limits(&bytes,Default::default()).unwrap();assert_eq!(reference.frames.len(),21);
    let mut observations=Vec::new();
    for (gfx,sound) in [(829u16,114u32),(830,115),(831,126)] {
        let mut w=world();w.accept(&p("add")).unwrap();let mut packet=vec![0x37,1,0,0,0];packet.extend(gfx.to_le_bytes());w.accept(&packet).unwrap();assert_eq!(w.effects.last().unwrap().graphics,gfx);let pos=w.effects.last().unwrap().position;let before=serde_json::to_value(&w).unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(pos[0]),y:i32::from(pos[1])},anchor_pixel:PixelPoint{x:400,y:300}};
        let mut fx=Effects::default();fx.prepare(root,&w,&cancel);let mut rows=Vec::new();
        for step in 0..=21u64 {
            let mut queue=Vec::new();let (r,events)=fx.draw_to(&w,&mut Target::Fragments(&mut queue),view,step*125_000_000);assert!(r["load_error"].is_null(),"{r}");assert_eq!(r["started"],1,"legacy alias{gfx} must use its original744 bank without rewriting wire IDs: {r}");assert_eq!(r["omitted"],0);
            assert_eq!(events.iter().filter_map(|e|e.sound).collect::<Vec<_>>(),if step==0{vec![sound]}else{vec![]});let row=&r["effects"][0];let source=if step<20{step+1}else{0};let sub=if source==0{20}else{source as usize-1};assert_eq!(row["graphics"],gfx);assert_eq!(row["sprite"],"744-0.spr");assert_eq!(row["subframe"],sub);assert_eq!(row["callback_count"],step+1);
            if step<21 {assert_eq!(r["active"],1);assert_eq!(queue.len(),1);assert_eq!(queue[0].frame.rgba,reference.frames[sub].rgba);assert!(matches!(queue[0].mix,Mix::Screen));assert_eq!(queue[0].position,view.anchor_cell);assert_eq!(row["next_callback_ms"],(step+1)*125);}
            else {assert_eq!(r["active"],0);assert!(queue.is_empty());assert_eq!(row["finished"],true);assert!(row["next_callback_ms"].is_null());}
            assert_eq!(serde_json::to_value(&w).unwrap(),before);rows.push(r);
        }
        observations.push(serde_json::json!({"wire_graphics":gfx,"sound":sound,"observations":rows}));
    }
    std::fs::write(output.join("native-legacy-higher-heal-frames.json"),serde_json::to_vec_pretty(&observations).unwrap()).unwrap();
}

// Append to tests/native_skills.rs only when the product proposal is applied.
#[test]
fn detection_selector_menu_guards_and_cooldown_leave_authority_unchanged() {
    use lito_modern::modern_lito::native_skill_selection;
    let mut w=world();w.accept(&p("mana")).unwrap();w.learned_skills.insert(13);
    let before=serde_json::to_value(&w).unwrap();
    assert_eq!(skills::request(&w,13,None).unwrap(),[0x14,1,4]);
    assert!(skills::request(&w,13,Some(2)).is_err());assert_eq!(skills::reuse_millis(13),500);
    let target=native_skill_selection::Target::capture(&w,2).unwrap();
    assert_eq!(native_skill_selection::choice(&w,13,Some(target)).unwrap().target,None);
    let mut menu=native_skill_selection::Model::default();assert!(menu.sync(11,&w,true));
    assert_eq!(menu.rows().iter().find(|r|r.id==13).unwrap().label,"#13 偵測術（自己）");
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    for kind in 0..6 {
        let mut bad=w.clone();match kind {
            0=>{bad.learned_skills.remove(&13);},1=>bad.mp=Some([0,10]),2=>bad.hp=Some([0,10]),
            3=>bad.actors.get_mut(&1).unwrap().dead=true,4=>bad.started=false,
            _=>{bad.movement_blocks.insert(1);},
        }
        let before=serde_json::to_value(&bad).unwrap();assert!(skills::request(&bad,13,None).is_err());
        assert_eq!(serde_json::to_value(&bad).unwrap(),before);
    }
}

#[test]
fn detection_original749_frames_sound_once_and_generation_retirement() {
    use lito_modern::modern_lito::{native_skill_effects::Effects,native_world_effects::{Target,Mix},
        resources::{SharedArchive,SharedArchiveKind},sprite,sprite_actions::{ActionKey,ActionLimits,SpriteActionCatalog},
        world_map::{WorldCell,PixelPoint,ViewProjection}};
    use std::{path::Path,sync::atomic::AtomicBool};
    use sha2::{Digest,Sha256};
    let root=Path::new(r"D:\lineage開發\0-客戶端");let cancel=AtomicBool::new(false);
    let bytes=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();
    let a=catalog.resolve_action(749,ActionKey::new(0)).unwrap().animation;assert_eq!(a.frames.len(),23);
    let name="749-0.spr";
    let bytes=SharedArchive::open(root,SharedArchiveKind::Sprite).unwrap().read_entry(name).unwrap();
    assert_eq!(format!("{:X}",Sha256::digest(&bytes)),"01D399D0D7F4ACC94AFD4A5A03E4373E101C57BDEF078EF8EAFD6492B792A8B7");
    let reference=sprite::decode_classic_spr_with_limits(&bytes,Default::default()).unwrap();
    let mut w=world();w.learned_skills.insert(13);w.accept(&[0x37,1,0,0,0,0xed,2]).unwrap();
    let position=w.effects.last().unwrap().position;let before=serde_json::to_value(&w).unwrap();
    let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(position[0]),y:i32::from(position[1])},anchor_pixel:PixelPoint{x:400,y:300}};
    let mut effects=Effects::default();effects.prepare(root,&w,&cancel);let mut due=0u64;let mut sounds=Vec::new();
    for step in 0..=a.frames.len() {
        let source=if step<22 {step+1}else{0};let mut fragments=Vec::new();
        let (r,events)=effects.draw_to(&w,&mut Target::Fragments(&mut fragments),view,due*1_000_000);
        assert!(r["load_error"].is_null(),"{r}");assert_eq!(r["started"],1);assert_eq!(r["omitted"],0);
        sounds.extend(events.into_iter().filter_map(|e|e.sound));
        if step==23 {assert_eq!(r["active"],0);assert!(fragments.is_empty());}
        else {
            assert_eq!(r["effects"][0]["source_frame"],source);assert_eq!(fragments.len(),1);
            let expected=&reference.frames[a.frames[source].subframe as usize];let actual=fragments[0].frame;
            assert_eq!((actual.x_offset,actual.y_offset,actual.width,actual.height),(expected.x_offset,expected.y_offset,expected.width,expected.height));
            assert_eq!(actual.rgba,expected.rgba);assert!(matches!(fragments[0].mix,Mix::Screen));
            due+=u64::from(a.frames[source].duration_ticks)*1000/u64::from(a.frames_per_second);
        }
        assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
    assert_eq!(sounds,[108]);
    w.accept(&[0x37,1,0,0,0,0xed,2]).unwrap();
    assert_eq!(effects.draw_to(&w,&mut Target::Fragments(&mut Vec::new()),view,(due+1)*1_000_000).0["active"],1);
    w.accept(&[0x28,0x34,2,0]).unwrap();
    assert_eq!(effects.draw_to(&w,&mut Target::Fragments(&mut Vec::new()),view,(due+2)*1_000_000).0["active"],0);
}

#[test]
fn invisibility_tick_envelope_guards_clock_and_menu_preserve_authority() {
    use lito_modern::modern_lito::native_skill_selection;
    let mut w=world();w.accept(&p("mana")).unwrap();w.learned_skills.insert(60);
    let before=serde_json::to_value(&w).unwrap();
    for tick in [0u64,2,0x12345678,0x7fffffff,0x80000000,0xffffffff,0x100000000,u64::MAX] {
        let b=skills::self_tick_request(&w,60,None,skills::ClientTick::from_wrapping_millis(tick)).unwrap();
        let mut expected=vec![0x14,7,3];expected.extend_from_slice(&(tick as u32).to_le_bytes());assert_eq!(b,expected);
    }
    // The native 4D69F0 clock is initialized by controller creation, as in 480A60.
    let _controller=lito_modern::modern_lito::native_login::LoginController::default();
    let start=skills::ClientTick::now().unwrap();let actual=skills::request(&w,60,None).unwrap();let end=skills::ClientTick::now().unwrap();
    assert_eq!(&actual[..3],[0x14,7,3]);assert_eq!(actual.len(),7);
    let sent=u32::from_le_bytes(actual[3..].try_into().unwrap());
    assert!(sent.wrapping_sub(start.as_millis())<=end.as_millis().wrapping_sub(start.as_millis()));
    assert!(skills::request(&w,60,Some(2)).is_err());
    assert!(skills::self_tick_request(&w,60,Some(2),start).is_err());
    assert!(skills::self_tick_request(&w,13,None,start).is_err());assert_eq!(skills::reuse_millis(60),0);
    let target=native_skill_selection::Target::capture(&w,2).unwrap();
    assert_eq!(native_skill_selection::choice(&w,60,Some(target)).unwrap().target,None);
    let mut menu=native_skill_selection::Model::default();assert!(menu.sync(11,&w,true));
    assert_eq!(menu.rows().iter().find(|row|row.id==60).unwrap().label,"#60 隱身術（自己，消耗 1 顆魔法寶石）");
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    for kind in 0..6 {
        let mut bad=w.clone();match kind {
            0=>{bad.learned_skills.remove(&60);},1=>bad.mp=Some([0,10]),2=>bad.hp=Some([0,10]),
            3=>bad.actors.get_mut(&1).unwrap().dead=true,4=>bad.started=false,
            _=>{bad.movement_blocks.insert(1);},
        }
        let before=serde_json::to_value(&bad).unwrap();
        assert!(skills::self_tick_request(&bad,60,None,start).is_err());assert!(skills::request(&bad,60,None).is_err());
        assert_eq!(serde_json::to_value(&bad).unwrap(),before);
    }
}
