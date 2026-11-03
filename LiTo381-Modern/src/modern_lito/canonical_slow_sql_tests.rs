//! Production Main + owned real SQL + two v47 controllers + shared514/SPR/752.
//! Exact painted inputs, not a native HWND/click or release-delivery claim.
use super::*;
use crate::modern_lito::{battle_slow as slow, spell_protocol::SkillsState};
#[path = "canonical_source_window_tests.rs"]
mod source_window;
#[path = "canonical_slow_window_tests.rs"]
mod window;

pub(super) fn saved(c: &Client, context: SelectionContext) {
    saved_with_expired_vip(c, context, false);
}

pub(super) fn saved_with_expired_vip(c: &Client, context: SelectionContext, expired_vip: bool) {
    let v = c.view();
    let CharacterDataState::Ready(s) = &v.character_data else {
        panic!("SQL character missing")
    };
    let sun = context.key.source_id == 7;
    assert_eq!(s.context, context);
    assert_eq!(s.stats.intelligence, if sun { 5_000_000_000 } else { 12 });
    assert_eq!(
        s.stats.experience,
        if sun {
            9_007_199_254_740_993
        } else {
            9_007_199_254_740_995
        }
    );
    assert_eq!((s.stats.max_hp, s.stats.max_mp), (1000, 300));
    assert_eq!(s.stats.food225, if sun { 225 } else { 180 });
    let mut expected_ids = if sun {
        vec![1001, 1002, 1003]
    } else {
        vec![1001, 1002]
    };
    if expired_vip {
        expected_ids.push(3_000_086);
        let vip = s.items.last().unwrap();
        assert_eq!((vip.catalog_id, vip.quantity), (300086, 1));
        assert!(vip.equipped, "source SQL flag is preserved");
        assert_eq!(
            gear_snapshot(c).equipped(3_000_086),
            Some(false),
            "expired VIP is not effective equipment"
        );
    }
    assert_eq!(
        s.items
            .iter()
            .map(|item| item.object_id)
            .collect::<Vec<_>>(),
        expected_ids
    );
    assert_eq!(s.items[1].quantity, if sun { 2 } else { 5 });
    assert_eq!(gear_snapshot(c).source.context, context);
    assert_eq!(gear_snapshot(c).equipped(1001), Some(true));
    let SkillsState::Ready(skills) = &v.skills else {
        panic!("SQL learned catalog missing")
    };
    assert_eq!(skills.context, context);
    assert_eq!(skills.slow().is_some(), sun);
}

pub(super) fn select(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    source_id: u32,
) -> SelectionContext {
    select_with_expired_vip(cs, i, d, source_id, false)
}

pub(super) fn select_with_expired_vip(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    source_id: u32,
    expired_vip: bool,
) -> SelectionContext {
    send(cs, i, d, || LoginCommand::SelectSource(source_id));
    wait(cs, d, "Slow29 SQL roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
    let v = cs[i].view();
    assert_eq!(v.selected_source, Some(source_id));
    assert_eq!(v.characters.len(), 1);
    assert_eq!(v.characters[0].object_id, 3333);
    let context = SelectionContext {
        key: CharacterKey {
            source_id,
            object_id: 3333,
        },
        epoch: v.roster_epoch,
    };
    send(cs, i, d, || LoginCommand::SelectCharacter(context));
    wait(cs, d, "Slow29 SQL source qualification", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && !v.record_pending
            && !v.spell_pending
            && matches!(v.equipment, equipment::State::Ready(_))
    });
    saved_with_expired_vip(&cs[i], context, expired_vip);
    context
}

fn lane(c: &Client) -> data::Spell {
    own_actor(c)
        .spells
        .histories
        .0
        .into_iter()
        .find(|s| s.lane == data::Lane::Slow)
        .expect("v47 Slow29 ledger")
}

fn target(c: &Client) -> crate::modern_lito::battle_window::Target {
    let a = own_actor(c);
    crate::modern_lito::battle_window::Target::Player {
        entity: a.entity,
        life: a.resources.as_ref().unwrap().life + 1,
    }
}
fn proof(c: &Client) -> slow::Intent {
    slow::Intent::from_observation(c.view().canonical.observation().unwrap(), target(c)).unwrap()
}
pub(super) fn gems(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    gems_count(cs, i, d, context, 4);
}
pub(super) fn gems_count(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    context: SelectionContext,
    quantity: i64,
) {
    let read = shop::Request::read(context, CHANNEL, None);
    send(cs, i, d, || LoginCommand::Shop(read));
    wait(cs, d, "Slow29 shop READ", |cs| !cs[i].view().shop_pending);
    let before = cs[i].view().shop.snapshot().unwrap().clone();
    assert!(before.materials_enabled && before.purchasable(40318));
    let current = cs[i].view();
    let shop_view = crate::modern_lito::shop_window::View {
        request: read,
        state: current.shop.clone(),
        pending: current.shop_pending,
        status: current.status,
        labels: std::collections::BTreeMap::from([(40318, "魔法寶石".into())]),
    };
    let detail = crate::modern_lito::shop_window::selected_text(&shop_view, Some(40318));
    assert!(detail.contains("施法材料") && detail.contains("不能在背包直接使用"));
    assert!(!detail.contains("尚未實作"));
    assert!(
        crate::modern_lito::shop_window::requested_icons(&shop_view, Some(40318)).contains(&235)
    );
    let q = crate::modern_lito::shop_window::transaction_request(
        &shop_view,
        Some(40318),
        shop::Action::Buy,
        &quantity.to_string(),
    )
    .unwrap();
    assert_eq!(
        q,
        read.mutation(&before, shop::Action::Buy, 40318, quantity)
    );
    send(cs, i, d, || LoginCommand::Shop(q));
    assert_eq!(
        cs[i].view().shop.snapshot(),
        Some(&before),
        "no optimistic gem purchase"
    );
    wait(cs, d, "Slow29 buy gems", |cs| !cs[i].view().shop_pending);
    let v = cs[i].view();
    assert_eq!(v.shop.reply().unwrap().status, shop::Status::Ok);
    let after = v.shop.snapshot().unwrap();
    assert_eq!(after.balance, before.balance - quantity * 100);
    assert_eq!(after.stack(40318).unwrap().quantity, quantity);
}
pub(super) fn material_inventory(
    c: &Client,
    uses: i64,
) -> crate::modern_lito::battle_inventory::Model {
    use crate::modern_lito::battle_inventory::{self, Key};
    let view = c.view();
    let surface = c.surface.as_ref().unwrap();
    let model = battle_inventory::model(surface, &view, |token| {
        if token == "$508" {
            "魔法寶石".into()
        } else {
            token.into()
        }
    })
    .unwrap();
    if let Some(source) = surface.observation.personal().source() {
        for item in &source.items {
            assert_eq!(
                model.rows.iter().find(|row| row.key == Key::Source(item.object_id))
                    .and_then(|row| row.graphic),
                Some(item.inventory_graphic),
                "visible grid icon must be the selected source's confirmed inventory graphic",
            );
        }
    }
    assert_inventory_details(surface, &model);
    let mut pending = view.clone();
    pending.world_pending = true;
    let pending_model = battle_inventory::model(surface, &pending, str::to_owned).unwrap();
    assert!(pending_model
        .rows
        .iter()
        .all(|r| r.equip.is_none() && r.unequip.is_none() && r.local.is_none()));
    assert!(pending_model.selection(Key::Material(40318)).is_some());
    let mut detached = view.clone();
    detached.navigation = None;
    assert!(battle_inventory::model(surface, &detached, str::to_owned).is_none());
    let initial = surface
        .observation
        .data()
        .personal
        .materials
        .as_ref()
        .unwrap()
        .stocks
        .0[0]
        .initial;
    let row = model
        .rows
        .iter()
        .find(|r| r.key == Key::Material(40318))
        .unwrap();
    assert_eq!(
        row.label,
        format!(
            "本場材料 · 魔法寶石 · 剩餘 {}／已消耗 {uses}",
            initial - uses
        )
    );
    assert!(row.equip.is_none() && row.unequip.is_none() && row.local.is_none());
    assert!(model.local(row.key).is_none());
    let m = surface
        .observation
        .data()
        .personal
        .materials
        .as_ref()
        .unwrap();
    assert_eq!(
        (m.uses, m.stocks.0[0].initial, m.stocks.0[0].remaining),
        (uses, initial, initial - uses)
    );
    model
}

fn slow_landed(chance: i32, roll: i32, haste_items: i64, missing_effect: bool) -> bool {
    !missing_effect && roll <= chance && haste_items == 0
}

#[test]
fn slow_accepted_receipt_can_miss_without_a_target_icon() {
    assert!(slow_landed(90, 90, 0, false));
    assert!(!slow_landed(90, 91, 0, false));
    assert!(!slow_landed(90, 1, 1, false));
    assert!(!slow_landed(90, 1, 0, true));
}

fn cast(cs: &mut [Client; 2], d: &Deadline, target_index: usize, count: i64) -> bool {
    wait(cs, d, "Slow29 current shared pixels", |cs| {
        cs.iter().all(|c| c.rendered())
    });
    let before = own_actor(&cs[0]);
    let inventory_before = material_inventory(&cs[0], count - 1);
    material_inventory(&cs[1], 0);
    let t = target(&cs[target_index]);
    let surface = cs[0].surface.as_ref().unwrap();
    let Target::Player { entity, .. } = t else {
        unreachable!()
    };
    let actor = surface
        .geometry
        .actors
        .iter()
        .find(|a| a.entity_id == entity as u64)
        .unwrap();
    let viewport = crate::modern_lito::compositor::Viewport {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    };
    let point = (actor.top_left.y.max(0)..(actor.top_left.y + actor.height as i32).min(600))
        .find_map(|y| {
            (actor.top_left.x.max(0)..(actor.top_left.x + actor.width as i32).min(800)).find_map(
                |x| {
                    let p = crate::modern_lito::world_map::PixelPoint { x, y };
                    battle_window::hit(surface, viewport, p, battle_window::Skill::Slow)
                        .filter(|h| h.intent.kind == Kind::Slow(t))
                        .map(|_| p)
                },
            )
        })
        .expect("visible actor pixels must select the real Slow target");
    for (scale, x, y) in [(1, 0, 0), (2, 17, 29)] {
        let view = crate::modern_lito::compositor::Viewport {
            x,
            y,
            width: 800 * scale,
            height: 600 * scale,
        };
        let p = crate::modern_lito::world_map::PixelPoint {
            x: x + point.x * scale,
            y: y + point.y * scale,
        };
        let h = battle_window::hit(surface, view, p, battle_window::Skill::Slow).unwrap();
        assert_eq!(
            h.intent.kind,
            Kind::Slow(t),
            "DPI/offset cannot swap a target"
        );
        assert!(h.name.unwrap().0.contains(if target_index == 0 {
            "太陽神"
        } else {
            "月亮"
        }));
        assert!(battle_window::hit(
            surface,
            view,
            crate::modern_lito::world_map::PixelPoint { x: x - 1, y: y - 1 },
            battle_window::Skill::Slow
        )
        .is_none());
    }
    let old =
        slow::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), t).unwrap();
    let v = loop {
        let v = input(cs, 0, d, Kind::Slow(t));
        if v.canonical.outcome() != Some(Outcome::NotSent) {
            break v;
        }
        assert_eq!(
            own_actor(&cs[0]).spells,
            before.spells,
            "NotSent cannot charge or consume"
        );
        d.pause("Slow29 fresh painted user action after confirmed NotSent");
    };
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Slow(slow::Reason::None))
    );
    let accepted = own_actor(&cs[0]);
    let h = lane(&cs[0]);
    assert_eq!(
        (h.ledger.casts, h.ledger.cost, h.ledger.spent),
        (count, 15, 15 * count)
    );
    assert_eq!(accepted.spells.sequence, count);
    assert_eq!(accepted.vital.as_ref().unwrap().values.mp, 300 - 15 * count);
    let r = h.receipt.as_ref().unwrap();
    let data::Detail::Slow(detail) = &r.detail else {
        panic!("real Slow29 receipt")
    };
    // A paid/accepted Slow command is not guaranteed to land: the server
    // draws 1..=100 while its qualified chance is capped at 90. Key the
    // target-icon assertion to this authoritative receipt, not to the
    // controller's transport-level Slow(None) acknowledgement.
    let applied = slow_landed(detail.probability, detail.roll,
        detail.equipment_haste_items, detail.missing_effect.is_some());
    eprintln!("R396_SLOW_RECEIPT cast={count} target_source={} chance={} roll={} applied={applied}",
        if target_index == 0 { 7 } else { 42 }, detail.probability, detail.roll);
    let crate::modern_lito::battle_payload::haste::StatusTarget::Player(p) = &detail.target else {
        panic!("player target")
    };
    assert_eq!(p.participant.entity, own_actor(&cs[target_index]).entity);
    assert_eq!(
        p.participant.identity.source,
        if target_index == 0 { 7 } else { 42 }
    );
    assert_eq!(p.participant.identity.id, 3333);
    assert_eq!(
        r.impact.usage.before.vitals.mp,
        before.vital.as_ref().unwrap().values.mp
    );
    let m = v
        .canonical
        .observation()
        .unwrap()
        .data()
        .personal
        .materials
        .as_ref()
        .unwrap();
    assert_eq!((m.uses, m.stocks.0[0].remaining), (count, 4 - count));
    wait(
        cs,
        d,
        "Slow29 receipt rendered by both real shared scenes",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.surface.as_ref().is_some_and(|s| {
                        s.observation.data().world.revision >= r.impact.usage.revision
                            && s.observation.data().actors.0.iter().any(|a| {
                                a.entity == accepted.entity
                                    && a.vital.as_ref().unwrap().values.mp == 300 - 15 * count
                                    && a.spells.histories.0.iter().any(|x| {
                                        x.lane == data::Lane::Slow && x.receipt == h.receipt
                                    })
                            })
                    })
            })
        },
    );
    let inventory_after = material_inventory(&cs[0], count);
    let key = crate::modern_lito::battle_inventory::Key::Material(40318);
    assert_eq!(
        inventory_after.retain_selection(Some(&inventory_before), Some(key)),
        Some(key)
    );
    material_inventory(&cs[1], 0);
    assert_eq!(own_actor(&cs[1]).vital.as_ref().unwrap().values.mp, 300);
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::slow(old)))
        .is_err());
    let recovery = crate::modern_lito::battle_motion::paced_nanos(
        &accepted,
        accepted.cast_interval_nanos as u64,
        false,
    ) as i64;
    wait(cs, d, "Slow29 recovery", |cs| {
        cs[0].rendered()
            && cs[0]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .loading_elapsed_nanos
                >= r.loading_elapsed_nanos + recovery
    });
    applied
}

#[test]
#[ignore = "Main + owned SELECT-only SQL + v47 controllers/shared514: INT5e9, gem shop, Slow skillbook, other-source/self casts, move/abort/source reload; no HWND"]
fn java_v47_main_sql_slow_skillbook_materials_cast_move_and_source_return() {
    flow(false, false);
}
#[test]
#[ignore = "R382 real Main/owned SQL/TLS shared514 ordinary player entry with explicit effect XML and server-owned Slow icon; no native HWND claim"]
fn java_v47_main_sql_real_map_owner_status_icon_from_profile() {
    flow(false, true);
}
#[test]
#[ignore = "INTERACTIVE v47 actual Main/owned SQL/shared514: operator casts Slow four times, moves after gem depletion, returns and closes; no automated input"]
fn java_v47_main_sql_window_materials_cast_move_return_and_close() {
    flow(true, false);
}
#[test]
#[ignore = "R383 INTERACTIVE actual HWND with explicit effect XML; operator must capture battle window MP/status before/after and exit; no automated screenshot claim"]
fn java_v47_main_sql_window_shared_hud_status_from_profile() {
    flow(true, true);
}
fn status_icon_pixels(surface: &crate::modern_lito::battle_scene::Surface) -> Vec<u8> {
    (8usize..40)
        .flat_map(|y| {
            let start = (y * 800 + 232) * 4;
            surface.bgra[start..start + 32 * 4].to_vec()
        })
        .collect()
}
// Independent RGB555 oracle matching the documented PakViewer C# converter.
// It reads the raw shared PAK member, never the production HUD decoder.
fn raw_hud(root: &std::path::Path, shard: u8, name: &str, width: usize, height: usize) -> Vec<Option<[u8; 4]>> {
    use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
    let bytes = SharedArchive::open(root, SharedArchiveKind::SpriteShard(shard))
        .unwrap().read_entry(name).unwrap();
    assert_eq!(bytes.len(), 8 + width * height * 2);
    assert_eq!(usize::from(u16::from_le_bytes([bytes[0], bytes[1]])), width);
    assert_eq!(usize::from(u16::from_le_bytes([bytes[2], bytes[3]])), height);
    assert_eq!(&bytes[4..6], &[1, 0]);
    let key = u16::from_le_bytes([bytes[6], bytes[7]]);
    bytes[8..].chunks_exact(2).map(|pair| {
        let pixel = u16::from_le_bytes([pair[0], pair[1]]);
        if pixel == key { None } else {
            let expand = |shift| { let c = ((pixel >> shift) & 0x1fu16) as u8; (c << 3) | (c >> 2) };
            Some([expand(0), expand(5), expand(10), 255])
        }
    }).collect()
}
fn hud_pixel(surface: &[u8], x: usize, y: usize) -> &[u8] {
    &surface[(y * 800 + x) * 4..(y * 800 + x) * 4 + 4]
}
// Separate reference reader for this fixed 32x32 RGB555 TBT. It does not use
// SharedInventoryIcons, TbtImage or vip_status::Picture pixel conversion.
fn raw_status_480(root: &std::path::Path) -> Vec<Option<[u8; 4]>> {
    use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
    let bytes = SharedArchive::open(root, SharedArchiveKind::SpriteShard(4))
        .unwrap().read_entry("480.tbt").unwrap();
    assert_eq!(&bytes[..4], &[0, 0, 32, 32]);
    let mut expected = vec![None; 32 * 32];
    let mut at = 4usize;
    for y in 0..32 {
        let spans = bytes[at] as usize;
        at += 1;
        let mut x = 0usize;
        for _ in 0..spans {
            let skip_bytes = bytes[at] as usize;
            let count = bytes[at + 1] as usize;
            at += 2;
            assert_eq!(skip_bytes % 2, 0);
            x += skip_bytes / 2;
            assert!(x + count <= 32);
            for column in x..x + count {
                let rgb = u16::from_le_bytes([bytes[at], bytes[at + 1]]);
                at += 2;
                assert_eq!(rgb & 0x8000, 0);
                let c = |shift| { let v = ((rgb >> shift) & 31u16) as u8; (v << 3) | (v >> 2) };
                expected[y * 32 + column] = Some([c(0), c(5), c(10), 255]);
            }
            x += count;
        }
    }
    assert_eq!(at, bytes.len());
    expected
}
fn check_real_hud(root: &std::path::Path, before: &[u8], after: &[u8]) {
    let base = raw_hud(root, 13, "2355c.img", 378, 48);
    let hp = raw_hud(root, 14, "2365c.img", 141, 20);
    let mp = raw_hud(root, 0, "2358c.img", 136, 20);
    let mut base_checked = 0;
    for (index, color) in base.iter().enumerate() {
        let x = 268 + index % 378;
        let y = 447 + index / 378;
        if (277..418).contains(&x) && (458..478).contains(&y)
            || (451..587).contains(&x) && (458..478).contains(&y) { continue; }
        if let Some(color) = color {
            assert_eq!(hud_pixel(before, x, y), color, "shared HUD base pixel {index}");
            base_checked += 1;
        }
    }
    assert!(base_checked > 1000);
    for (bar, x0, width) in [(&hp, 277usize, 141usize), (&mp, 451, 136)] {
        let mut checked = 0;
        for (index, color) in bar.iter().enumerate() {
            if let Some(color) = color {
                let x = x0 + index % width;
                let y = 458 + index / width;
                assert_eq!(hud_pixel(before, x, y), color, "full shared gauge pixel {index}");
                checked += 1;
            }
        }
        assert!(checked > 100);
    }
    // One confirmed Slow cast costs 15 MP: 300 -> 285, so 136*285/300=129.
    // The newly hidden right seven columns reveal the base, not stale MP art.
    let mut revealed = 0;
    for (index, color) in mp.iter().enumerate() {
        let x_local = index % 136;
        let y_local = index / 136;
        let x = 451 + x_local;
        let y = 458 + y_local;
        if x_local < 129 {
            if let Some(color) = color { assert_eq!(hud_pixel(after, x, y), color, "remaining MP pixel {index}"); }
        } else if color.is_some() {
            if let Some(base_color) = base[(y - 447) * 378 + (x - 268)] {
                assert_eq!(hud_pixel(after, x, y), base_color, "revealed base pixel {index}");
                revealed += 1;
            }
        }
    }
    assert!(revealed > 0, "MP reduction must expose the source-backed base");
}
fn flow(interactive: bool, with_effects: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(if interactive { 590 } else { 150 }));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernSlowSqlTestPeer",
        if interactive {
            "slow-window"
        } else {
            "slow-sql"
        },
        Some(if interactive { "600000" } else { "150000" }),
    );
    let effect_xml = with_effects.then(|| explicit_path("LITO_EFFECT_CATALOG_PATH"));
    let profile = if let Some(xml) = &effect_xml {
        let mut doc = profile_document(peer.endpoint, &shared, &private_root(&fixtures));
        doc["schema"] = serde_json::json!(2);
        doc["gameplay_protocol"] = serde_json::json!("modern_v47");
        doc["effect_catalog_path"] = serde_json::json!(xml);
        ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap()
    } else {
        configured_modern_profile(peer.endpoint, &shared, &private_root(&fixtures))
    };
    assert_eq!(profile.effect_catalog_path(), effect_xml.as_deref());
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new(profile.clone()),
        runtime: BattleRuntime::with_preferences_and_effects(
            shared.clone(),
            effect_xml.clone(),
            crate::modern_lito::preferences::shared(),
        )
        .unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "Slow29 SQL TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "Slow29 SQL authentication", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        assert!(cs[i].view().canonical.slow_supported());
        gems(&mut cs, i, &d, contexts[i]);
    }
    send(&mut cs, 0, &d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(contexts[0], CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 0,
                        map_id: 514,
                        name: "Main SQL Slow".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "Slow29 SQL room created", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let room = cs[0].view().rooms.joined().unwrap().summary.clone();
    send(&mut cs, 1, &d, || {
        LoginCommand::Rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: room.room_id,
                    expected_revision: room.revision,
                    ..rooms::Common::read(contexts[1], CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "Slow29 SQL room joined", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    room_action(&mut cs, 1, &d, contexts[1], rooms::Action::Team, 1);
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
        cs[i].load = true;
    }
    wait(&mut cs, &d, "Slow29 SQL shared world loaded", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    if with_effects {
        for c in &cs {
            let surface = c.surface.as_ref().unwrap();
            assert_eq!(surface.observation.data().manifest.map, 514);
            assert_eq!(surface.observation.data().personal.player.object, 3333);
            assert_eq!((surface.width, surface.height), (800, 600));
        }
        if !interactive {
            crate::modern_lito::world_window::verify_observed_sword_sprite(
                cs[0].surface.as_ref().unwrap(), &shared);
            crate::modern_lito::world_window::verify_observed_panel_composition(
                cs[0].surface.as_ref().unwrap(), &cs[0].view(), &shared);
        }
    }
    assert_eq!(gear_snapshot(&cs[0]).equipped(1003), Some(true));
    assert_eq!(lane(&cs[0]).ledger.cost, 15);
    assert!(battle_input::skill_available(29, &cs[0].view()));
    assert!(!battle_input::skill_available(29, &cs[1].view()));
    assert!(!battle_input::ready(
        Kind::Slow(target(&cs[1])),
        &cs[1].view()
    ));
    if interactive {
        for c in &cs {
            assert_eq!(
                c.view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .descriptor
                    .active_timeout_millis,
                600000,
                "interactive world must cover the unchanged 540 s operator interval"
            );
        }
        let (result, audit) = window::run(cs, &profile, &d);
        // Even an incomplete manual scenario must finish the peer's actual
        // source/zero-settlement/SQL cleanup checks before its audit assertion.
        peer.expect_successful_exit(&d);
        eprintln!("SLOW_HWND_PEER_CLEANUP verified=true");
        result.unwrap();
        assert!(
            audit.complete_with_status(with_effects),
            "v47 interactive flow incomplete: {audit:?}"
        );
        return;
    }
    assert_eq!(
        cs[0]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .descriptor
            .active_timeout_millis,
        120000,
        "headless world timeout stays unchanged"
    );
    let stale_source = proof(&cs[0]);
    let source_inventory = material_inventory(&cs[0], 0);
    let before_status = with_effects.then(|| status_icon_pixels(cs[1].surface.as_ref().unwrap()));
    let before_hud = with_effects.then(|| cs[0].surface.as_ref().unwrap().bgra.clone());
    for count in 1..=4 {
        let applied = cast(&mut cs, &d, if count == 2 { 0 } else { 1 }, count);
        if count == 1 && with_effects {
            check_real_hud(&shared, before_hud.as_ref().unwrap(), &cs[0].surface.as_ref().unwrap().bgra);
            if !applied {
                // The observation validator itself requires unchanged target
                // buffs on a probabilistic miss. Verify that the painted UI
                // does not manufacture a status icon, then continue the
                // normal four-cast scenario without a hidden retry.
                let surface = cs[1].surface.as_ref().unwrap();
                let owner = surface.observation.data().personal.entity;
                let status = surface.observation.data().actors.0.iter()
                    .find(|actor| actor.entity == owner)
                    .and_then(|actor| actor.buffs.as_ref())
                    .and_then(|buffs| crate::modern_lito::battle_scene::ordinary_status_effect(buffs, true));
                assert_eq!(status, None, "a valid Slow miss cannot grant target status");
                assert_eq!(status_icon_pixels(surface), before_status.as_ref().unwrap().clone(),
                    "a valid Slow miss cannot paint a target status icon");
                continue;
            }
            // The caster's receipt/MP and the target's rendered status arrive
            // on separate client streams. Wait for the target's own shared
            // scene, with the fixture deadline, before asserting its icon.
            let mut last_status_log = Instant::now();
            loop {
                for c in cs.iter_mut() { c.pump(); }
                let (world_revision, actor_found, status) = cs[1].surface.as_ref().map_or(
                    (None, false, None), |surface| {
                        let owner = surface.observation.data().personal.entity;
                        let actor = surface.observation.data().actors.0.iter()
                            .find(|actor| actor.entity == owner);
                        (Some(surface.observation.data().world.revision), actor.is_some(),
                            actor.and_then(|a| a.buffs.as_ref()).and_then(|buffs|
                                crate::modern_lito::battle_scene::ordinary_status_effect(buffs, true)))
                    });
                if status == Some(1) { break; }
                if last_status_log.elapsed() >= Duration::from_secs(10) {
                    eprintln!("R388_TARGET_STATUS rendered={} revision={world_revision:?} actor_found={actor_found} status={status:?} world_pending={} receipt_epoch={:?}",
                        cs[1].rendered(), cs[1].view().world_pending,
                        cs[1].view().canonical.receipt_epoch());
                    last_status_log = Instant::now();
                }
                d.pause("Slow29 target status rendered");
            }
            let surface = cs[1].surface.as_ref().unwrap();
            let owner = surface.observation.data().personal.entity;
            let buffs = surface
                .observation
                .data()
                .actors
                .0
                .iter()
                .find(|actor| actor.entity == owner)
                .unwrap()
                .buffs
                .as_ref()
                .unwrap();
            assert_eq!(
                crate::modern_lito::battle_scene::ordinary_status_effect(buffs, true),
                Some(1)
            );
            assert_ne!(
                status_icon_pixels(surface),
                before_status.as_ref().unwrap().clone()
            );
            let cancel = std::sync::atomic::AtomicBool::new(false);
            let catalog = crate::modern_lito::effect_catalog::EffectCatalog::load(
                effect_xml.as_ref().unwrap(),
                &cancel,
            )
            .unwrap();
            assert_eq!(catalog.get(crate::modern_lito::effect_catalog::EffectId::from_wire(1).unwrap()).unwrap().icon, 480);
            let expected = raw_status_480(&shared);
            let baseline = before_status.as_ref().unwrap();
            let mut opaque = 0;
            let mut clear = 0;
            for (index, expected_pixel) in expected.iter().enumerate() {
                let x = 232 + index % 32;
                let y = 8 + index / 32;
                let byte = (y * 800 + x) * 4;
                match expected_pixel {
                    Some(color) => {
                        opaque += 1;
                        assert_eq!(&surface.bgra[byte..byte + 4], color, "opaque status pixel {index}");
                    }
                    None => {
                        clear += 1;
                        assert_eq!(&surface.bgra[byte..byte + 4], &baseline[index * 4..index * 4 + 4], "transparent status pixel {index}");
                    }
                }
            }
            assert!(opaque > 0 && clear > 0, "entire shared status mask checked");
        }
    }
    if with_effects {
        wait(&mut cs, &d, "Slow29 owner status expires on server", |cs| {
            cs[1].rendered() && cs[1].surface.as_ref().is_some_and(|s| {
                let owner = s.observation.data().personal.entity;
                s.observation.data().actors.0.iter().find(|a| a.entity == owner)
                    .and_then(|a| a.buffs.as_ref())
                    .is_some_and(|b| crate::modern_lito::battle_scene::ordinary_status_effect(b, true).is_none())
            })
        });
        assert_eq!(status_icon_pixels(cs[1].surface.as_ref().unwrap()), before_status.unwrap(),
            "server-removed status must leave the original scene pixels, without icon residue");
    }
    let v = cs[0].view();
    assert!(!battle_input::ready(Kind::Slow(target(&cs[1])), &v));
    let model = crate::modern_lito::battle_skillbook::model(
        cs[0].surface.as_ref().unwrap(),
        &v,
        str::to_owned,
    )
    .unwrap();
    assert!(
        model.choice(29).is_none(),
        "zero gems must disable the skill-page action"
    );
    let a = own_actor(&cs[0]);
    let from = WorldCell {
        x: a.position.x,
        y: a.position.y,
    };
    let s = cs[0].surface.as_ref().unwrap();
    let (heading, to) = DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            (s.navigation.as_ref().unwrap().can_step(from, to)
                && s.observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y))
            .then_some((h as u8, to))
        })
        .unwrap();
    let moved = input(&mut cs, 0, &d, Kind::Local(Local::Step(heading)));
    assert_eq!(
        moved.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    assert_eq!(own_actor(&cs[0]).position, data::Cell { x: to.x, y: to.y });
    assert_eq!(
        own_actor(&cs[0]).last_move_sequence,
        a.last_move_sequence + 1
    );
    wait(&mut cs, &d, "Slow29 SQL moved pixels", |cs| {
        cs[0].rendered()
    });
    let world = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    for i in 0..2 {
        let q = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&world))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(q.clone()));
        wait(&mut cs, &d, "Slow29 SQL battle return", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
        assert!(cs[i].surface.is_none());
        assert!(!battle_input::skill_available(29, &cs[i].view()));
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(&mut cs, &d, "Slow29 SQL no false win", |cs| {
            !cs[i].view().record_pending
        });
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0
            })
        );
        saved(&cs[i], contexts[i]);
    }
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "Slow29 SQL character return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    let moon = select(&mut cs, 0, &d, 42);
    assert_ne!(moon, contexts[0]);
    assert!(!source_inventory
        .selection(crate::modern_lito::battle_inventory::Key::Material(40318))
        .unwrap()
        .current_owner(Stamp {
            context: moon,
            ..source_inventory.stamp
        }));
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::slow(stale_source)))
        .is_err());
    enter(&mut cs, 0, &d, moon);
    assert!(matches!(cs[0].view().skills, SkillsState::Ready(ref s) if s.slow().is_none()));
    assert!(gear_snapshot(&cs[0]).equipped(1003).is_none());
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(
        &mut cs,
        &d,
        "Slow29 SQL controller and resource shutdown",
        |cs| {
            cs.iter()
                .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        },
    );
    peer.expect_successful_exit(&d);
}
