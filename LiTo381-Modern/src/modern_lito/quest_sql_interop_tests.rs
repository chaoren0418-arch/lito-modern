//! Production Main + actual isolated account/catalog/result SQL, two real
//! controllers and shared-map render workers. No HWND or native-balance claim.
use super::super::{
    character_data::CharacterDataState, equipment, login_window, npc_protocol as npc, quest_life,
    shop_protocol as shop,
};
use super::*;
#[path = "lobby_supplies_sql_interop_tests.rs"]
mod lobby_supplies_sql_interop_tests;
#[path = "quest_filtered_chat_sql_interop_tests.rs"]
mod quest_filtered_chat_sql_interop_tests;
#[path = "quest_matchmaking_sql_interop_tests.rs"]
mod quest_matchmaking_sql_interop_tests;
#[path = "quest_npc70_sql_interop_tests.rs"]
mod quest_npc70_sql_interop_tests;
#[path = "quest_npc77_sql_interop_tests.rs"]
mod quest_npc77_sql_interop_tests;
#[path = "siege_sql_interop_tests.rs"]
mod siege_sql_interop_tests;
#[path = "time_attack_sql_interop_tests.rs"]
mod time_attack_sql_interop_tests;

struct Client {
    controller: SessionController,
    runtime: WorldRuntime,
    rendered: Option<Arc<RenderedWorld>>,
    auto_ack: bool,
    acks: BTreeSet<(u64, u64)>,
    light_arrow_sound: bool,
    canonical: Option<crate::modern_lito::battle_runtime::BattleRuntime>,
    canonical_surface: Option<Arc<crate::modern_lito::battle_scene::Surface>>,
}
impl Client {
    fn view(&self) -> LoginSnapshot {
        self.controller.snapshot()
    }
    fn command(&mut self, c: LoginCommand) {
        self.controller.command(c).unwrap();
    }
    fn pump(&mut self, stage: &'static str) {
        self.controller.poll();
        let view = self.view();
        assert_ne!(view.phase, LoginPhase::Failed, "{stage}: {}", view.status);
        assert_canonical_command_binding(&view);
        if let Some(runtime) = &mut self.canonical {
            use crate::modern_lito::{battle_player, battle_runtime::Status, battle_window};
            self.canonical_surface = match battle_player::update(runtime, &view, false).unwrap() {
                Status::Ready(s) if battle_player::admitted(&s, &view) => {
                    battle_window::validate(&s).unwrap();
                    for page in [
                        crate::modern_lito::world_window::DetailsPage::Character,
                        crate::modern_lito::world_window::DetailsPage::Inventory,
                        crate::modern_lito::world_window::DetailsPage::Skills,
                    ] {
                        assert!(crate::modern_lito::battle_details::text(
                            &s.observation,
                            page,
                            str::to_owned
                        )
                        .is_some());
                    }
                    Some(s)
                }
                Status::Failed(_) => panic!("{stage}: canonical shared renderer failed"),
                _ => None,
            };
        }
        if view
            .world
            .frame()
            .is_none_or(|f| f.phase == world::Phase::Aborted)
        {
            self.runtime.clear();
        } else if let Some(frame) = view.npc.as_ref().and_then(|r| r.frame.as_ref()) {
            assert_sql_npc_names(frame);
            if view.world.frame() == Some(&frame.combat.world)
                && view.combat.as_deref() == Some(&frame.combat)
            {
                let key = view.navigation.as_ref().unwrap().context.key;
                let entity = frame
                    .combat
                    .world
                    .actors
                    .iter()
                    .find(|a| a.matches_character(key))
                    .unwrap()
                    .entity_id;
                login_window::update_world_runtime(&mut self.runtime, &view, entity).unwrap();
            }
        } else if let Some(combat) = view.combat.as_ref().filter(|f| f.world.siege.is_some()) {
            if view.world.frame() == Some(&combat.world) {
                let key = view.navigation.as_ref().unwrap().context.key;
                let entity = combat
                    .world
                    .actors
                    .iter()
                    .find(|a| a.matches_character(key))
                    .unwrap()
                    .entity_id;
                login_window::update_world_runtime(&mut self.runtime, &view, entity).unwrap();
            }
        }
        self.rendered = match self.runtime.poll() {
            RuntimeStatus::Ready(r) => {
                if let Some(combat) = &r.combat {
                    assert_eq!(*r.world, combat.world, "the painted roster is immutable");
                }
                if let Some(npc) = r.npc.as_ref().and_then(|n| n.frame.as_ref()) {
                    assert_eq!(*r.world, npc.combat.world);
                    assert_sql_npc_names(npc);
                    for placement in r.npcs.iter() {
                        assert_eq!(npc.name(placement.id).unwrap().name, "龜龍");
                    }
                }
                Some(r)
            }
            RuntimeStatus::Failed(m) => panic!("SQL Quest shared resources: {m}"),
            _ => None,
        };
        // Kent executes the same retirement handoff as the real player.
        // Older NPC cases intentionally retain their separate legacy-render
        // assertions; they are compatibility coverage, not CPU measurements.
        if view.world.frame().is_some_and(|f| f.siege.is_some()) {
            if let Some(surface) = self.canonical_surface.as_ref() {
                crate::modern_lito::battle_player::retire_legacy(&mut self.runtime, surface, &view);
            }
        }
        if self
            .rendered
            .as_ref()
            .and_then(|r| r.audio.as_ref())
            .is_some_and(|a| a.events.iter().any(|e| e.group.candidates().contains(&97)))
        {
            self.light_arrow_sound = true;
        }
        if self.auto_ack {
            if let Some(r) = &self.rendered {
                if let Some(request) = login_window::loaded_request(&view, r) {
                    let canonical_ready = self.canonical.as_ref().is_none_or(|runtime| {
                        self.canonical_surface.as_ref().is_some_and(|surface| {
                            crate::modern_lito::battle_player::load_ready(runtime, surface, &view)
                        })
                    });
                    if canonical_ready && self.acks.insert((request.match_id, request.generation)) {
                        self.command(LoginCommand::World(request));
                    }
                }
            }
        }
    }
}

fn assert_canonical_command_binding(view: &LoginSnapshot) {
    use crate::modern_lito::{
        battle_payload::data::Phase,
        battle_player,
        battle_window::{Intent, Kind, Stamp, Target},
    };
    let Some(o) = view.battle.observation() else {
        return;
    };
    if o.data().world.phase != Phase::Active {
        return;
    }
    let actor = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap();
    if actor.vital.as_ref().is_none_or(|v| v.values.hp == 0) {
        return;
    }
    let Some(npc) = o
        .data()
        .npcs
        .as_ref()
        .and_then(|n| n.actors.0.iter().find(|a| a.hp > 0))
    else {
        return;
    };
    let intent = Intent {
        stamp: Stamp::of(o),
        kind: Kind::Disintegrate(Target::Npc {
            serial: npc.serial,
            life: npc.life,
        }),
    };
    if battle_player::authority_life(view, actor.entity) != Some(intent.stamp.life) {
        assert!(
            battle_player::command(intent, view).is_err(),
            "two streams must agree on caster life"
        );
        return;
    }
    use crate::modern_lito::{battle_window, world_window::Action as A};
    for raw in [
        A::Step(3),
        A::Teleport,
        A::FireStorm,
        A::UseItem(crate::modern_lito::item_protocol::UI_ITEMS[0]),
        A::SelectAmmo(crate::modern_lito::ammo_protocol::UI_ITEMS[0]),
    ] {
        let stamped = battle_window::stamp_local_action(raw, Some(o));
        assert_eq!(
            battle_player::admit_local_action(stamped, view).unwrap(),
            raw
        );
        assert!(battle_player::admit_local_action(raw, view).is_err());
        let A::Canonical(local) = stamped else {
            unreachable!()
        };
        let make_command = || match raw {
            A::Step(h) => {
                let mut q = world::Request::read(local.stamp.context, local.stamp.channel);
                q.action = world::Action::Move;
                q.heading = h;
                LoginCommand::World(q)
            }
            A::Teleport => LoginCommand::CastTeleport,
            A::FireStorm => LoginCommand::CastFireStorm,
            A::UseItem(item_id) => LoginCommand::UseItem { item_id },
            A::SelectAmmo(item_id) => LoginCommand::SelectAmmo { item_id },
            _ => unreachable!(),
        };
        assert!(battle_player::admit_local_command(
            battle_player::bind_local_command(Some(local), make_command()),
            view
        )
        .is_ok());
        // A stamped movement/spell cannot wrap unrelated or nested commands.
        assert!(battle_player::admit_local_command(
            battle_player::bind_local_command(Some(local), LoginCommand::Shutdown),
            view
        )
        .is_err());
        assert!(battle_player::admit_local_command(
            battle_player::bind_local_command(
                Some(local),
                battle_player::bind_local_command(Some(local), make_command())
            ),
            view
        )
        .is_err());
        for field in 0..5 {
            let mut stale = local;
            match field {
                0 => stale.stamp.life -= 1,
                1 => stale.stamp.context.epoch += 1,
                2 => stale.stamp.world[2] += 1,
                3 => stale.stamp.channel ^= 1,
                _ => stale.stamp.revision += 1,
            }
            assert!(battle_player::admit_local_action(A::Canonical(stale), view).is_err());
            assert!(battle_player::admit_local_command(
                battle_player::bind_local_command(Some(stale), make_command()),
                view
            )
            .is_err());
        }
    }
    let LoginCommand::CastNpcDisintegrate(q) = battle_player::command(intent, view).unwrap() else {
        panic!("canonical skill target namespace changed");
    };
    assert_eq!(
        q.caster_life as i64,
        actor.resources.as_ref().unwrap().life + 1
    );
    assert_eq!(q.target_life as i64, npc.life);
    assert_eq!(q.sequence as i64, actor.spells.sequence + 1);
    assert_eq!(q.context, view.navigation.as_ref().unwrap().context);
    let mut stale = intent;
    stale.kind = Kind::Disintegrate(Target::Npc {
        serial: npc.serial,
        life: npc.life + 1,
    });
    assert!(battle_player::command(stale, view).is_err());
    stale = intent;
    stale.stamp.world[2] += 1;
    assert!(battle_player::command(stale, view).is_err());
    stale = intent;
    stale.stamp.life -= 1;
    assert!(
        battle_player::command(stale, view).is_err(),
        "painted intent from the caster's old life"
    );
    for selected in [
        crate::modern_lito::battle_window::Skill::Disintegrate,
        crate::modern_lito::battle_window::Skill::LightArrow,
        crate::modern_lito::battle_window::Skill::Melee,
    ] {
        use crate::modern_lito::{
            battle_payload::data::Lane,
            battle_window::{self, Skill},
        };
        let lane = match selected {
            Skill::Disintegrate => Lane::NpcDisintegrate,
            Skill::LightArrow => Lane::NpcLightArrow,
            _ => Lane::NpcFireStorm,
        };
        if let Some(h) = actor.spells.histories.0.iter().find(|h| h.lane == lane) {
            let text = battle_window::skill_status(o, selected).unwrap();
            assert!(text.contains(&format!("MP {}", h.ledger.cost)));
        }
    }
    let mut old_protocol = view.clone();
    old_protocol.battle_supported = false;
    assert!(battle_player::command(intent, &old_protocol).is_err());
}

/// Explicit developer-only inspection of the production WorldWindow using an
/// actual isolated Main/SQL/shared-map snapshot. This is not a second player
/// executable or a claim that the whole login HWND flow was exercised.
fn native_canonical_inspection(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    use crate::modern_lito::{
        battle_details,
        battle_payload::data::Phase,
        world_window::{Action, DetailsPage, WorldWindow},
    };
    use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};
    wait(cs, d, "canonical finished window pixels", |cs| {
        cs[i]
            .canonical_surface
            .as_ref()
            .is_some_and(|s| s.observation.data().world.phase == Phase::Finished)
    });
    // A world is an owned player window in production, never a standalone
    // preview. The component test supplies a visible native owner and retires
    // both HWNDs; do not weaken production's valid-owner requirement.
    struct Owner(HWND);
    impl Drop for Owner {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }
    let owner = Owner(unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            windows::core::w!("STATIC"),
            windows::core::w!("LiTo isolated SQL inspection owner"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            900,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    });
    let window = WorldWindow::open(owner.0).unwrap();
    let started = Instant::now();
    let mut pages = 0u8;
    let mut left = false;
    println!("CANONICAL_NATIVE_WINDOW_READY (isolated SQL component inspection)");
    while started.elapsed() < Duration::from_secs(80) {
        for c in cs.iter_mut() {
            c.pump("native canonical window");
        }
        let surface = cs[i].canonical_surface.as_ref().unwrap().clone();
        window
            .update(
                None,
                None,
                "隔離伺服器／共享地圖 · 世界視窗元件檢查 · 非完整成品驗收",
                false,
                None,
                false,
            )
            .unwrap();
        window.update_canonical(surface).unwrap();
        unsafe {
            let mut message = MSG::default();
            while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                if !window.pre_translate(&message) {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
        if let (Some(page), Some(o)) = (window.details_page(), window.painted_observation()) {
            pages |= match page {
                DetailsPage::Character => 1,
                DetailsPage::Inventory => 2,
                DetailsPage::Skills => 4,
            };
            window
                .update_details(&battle_details::text(&o, page, str::to_owned).unwrap())
                .unwrap();
        }
        if matches!(window.take_action(), Some(Action::Leave)) {
            left = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    window.release();
    println!("CANONICAL_NATIVE_WINDOW_END pages={pages} native_leave={left}; not full-player-flow evidence");
}
fn followed(
    cs: &mut [Client; 2],
    d: &Deadline,
    phase: crate::modern_lito::battle_payload::data::Phase,
) {
    wait(cs, d, "normal controller canonical observation", |cs| {
        cs.iter().all(|c| {
            let v = c.view();
            let Some(observation) = v.battle.observation() else {
                return false;
            };
            let request = v.battle.request().unwrap();
            assert_eq!(
                Some(request.clone()),
                crate::modern_lito::battle_session::request_for(&v)
            );
            assert_eq!(observation.personal().player(), &request.player());
            assert_eq!(observation.personal().world(), &request.world);
            assert!(
                observation.data().presentation.is_some(),
                "normal v36 owner must receive schema 2"
            );
            observation.data().world.phase == phase
        })
    });
}
fn assert_sql_npc_names(frame: &npc::Frame) {
    // These literals come from ModernNpcCatalogRealMysqlTest.install(), not
    // another field decoded from the packet. Both player sources share source
    // 7's selected NPC catalog; source 42 must not relabel it as its own.
    let names = frame.names.as_deref().expect("v28 SQL catalog names");
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].source_id, 7);
    assert_eq!(names[0].npc_id, 45262);
    assert_eq!(names[0].graphic, 1649);
    assert_eq!(names[0].name, "龜龍");
    assert_eq!(names[0].name_id, "$1498");
    for actor in &frame.actors {
        assert_eq!(
            frame.name(npc::Id {
                scope: frame.scope,
                serial: actor.serial
            }),
            Some(&names[0])
        );
    }
}
#[track_caller]
fn wait(
    cs: &mut [Client; 2],
    d: &Deadline,
    stage: &'static str,
    condition: impl Fn(&[Client; 2]) -> bool,
) {
    let started = Instant::now();
    let mut diagnosed = false;
    loop {
        for c in cs.iter_mut() {
            c.pump(stage);
            if let world::State::Ready(r) = c.view().world {
                assert_ne!(
                    r.status,
                    world::Status::Unavailable,
                    "{stage}: {:?}",
                    r.reason
                );
            }
        }
        if condition(cs) {
            return;
        }
        if !diagnosed && started.elapsed() > Duration::from_secs(5) {
            diagnosed = true;
            for (i, c) in cs.iter().enumerate() {
                let v = c.view();
                eprintln!("Quest SQL {stage}, client {i}: phase={:?}, status={}, world={:?}, pending={}/{}, render={:?}",v.phase,v.status,v.world.frame().map(|f|(f.phase,f.revision)),v.world_pending,v.combat_pending,c.rendered.as_ref().map(|r|(r.revision,r.npcs.len(),r.quests.len())));
                eprintln!(
                    "canonical: observed={:?} painted={:?} navigation={}",
                    v.battle
                        .observation()
                        .map(|o| (o.data().world.phase, o.data().world.revision)),
                    c.canonical_surface.as_ref().map(|s| (
                        s.observation.data().world.phase,
                        s.observation.data().world.revision
                    )),
                    c.canonical
                        .as_ref()
                        .is_some_and(|r| r.navigation().is_some())
                );
            }
        }
        d.pause(stage);
    }
}
fn room(cs: &mut [Client; 2], i: usize, d: &Deadline, r: rooms::Request) {
    cs[i].command(LoginCommand::Rooms(r));
    wait(cs, d, "room acknowledgment", |cs| {
        !cs[i].view().room_pending
    });
}
fn selected(cs: &mut [Client; 2], i: usize, d: &Deadline, source: u32) -> SelectionContext {
    selected_with_mp(cs, i, d, source, if source == 7 { 30 } else { 40 })
}
fn selected_with_mp(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    source: u32,
    expected_mp: i64,
) -> SelectionContext {
    cs[i].command(LoginCommand::SelectSource(source));
    wait(cs, d, "SQL roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
            && cs[i].view().selected_source == Some(source)
    });
    let v = cs[i].view();
    assert_eq!(v.characters.len(), 1);
    assert_eq!(v.characters[0].object_id, 3333);
    let context = SelectionContext {
        key: CharacterKey {
            source_id: source,
            object_id: 3333,
        },
        epoch: v.roster_epoch,
    };
    cs[i].command(LoginCommand::SelectCharacter(context));
    wait(cs, d, "SQL selected character", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && !v.record_pending
            && !matches!(v.record, record::State::NotRequested)
            && !v.spell_pending
            && !v.equipment_pending
    });
    assert_source_with_mp(&cs[i].view(), context, expected_mp);
    context
}
fn assert_source(v: &LoginSnapshot, context: SelectionContext) {
    assert_source_with_mp(v, context, if context.key.source_id == 7 { 30 } else { 40 });
}
fn assert_source_with_mp(v: &LoginSnapshot, context: SelectionContext, expected_mp: i64) {
    let CharacterDataState::Ready(data) = &v.character_data else {
        panic!("SQL persisted character missing: {:?}", v.character_data)
    };
    let sun = context.key.source_id == 7;
    assert_eq!(data.context, context);
    assert_eq!(data.items.len(), 2);
    assert_eq!(data.items[0].object_id, 1001);
    assert!(data.items[0].equipped);
    assert_eq!(data.items[1].quantity, if sun { 2 } else { 5 });
    assert_eq!(
        data.stats.experience,
        if sun {
            9_007_199_254_740_993
        } else {
            9_007_199_254_740_995
        }
    );
    assert_eq!(data.stats.max_mp, expected_mp);
    assert_eq!(data.stats.food225, if sun { 225 } else { 180 });
    assert_eq!(data.stats.food_percent(), Some(if sun { 100 } else { 80 }));
    let equipment::State::Ready(eq) = &v.equipment else {
        panic!("SQL equipment missing: {:?}", v.equipment)
    };
    assert_eq!(eq.source.context, context);
    assert_eq!(eq.equipped(1001), Some(true));
}
fn clear_waves(cs: &mut [Client; 2], i: usize, d: &Deadline, magic: bool, bots: bool) {
    let mut seen = BTreeSet::new();
    let mut next = Instant::now();
    loop {
        wait(cs, d, "wave pump", |_| true);
        let v = cs[i].view();
        let f = v.npc.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(f.combat.world.phase, world::Phase::Active);
        assert_eq!(f.combat.world.audience_lane, Some(i as u8));
        assert_eq!(f.combat.vitals.len(), if bots { 2 } else { 1 });
        assert!(f.actors.iter().all(|a| a.lane == i as u8));
        if cs[i]
            .rendered
            .as_ref()
            .is_some_and(|r| r.quests.len() == 1 && r.quests[0].alive)
        {
            break;
        }
        if !v.combat_pending && !v.spell_pending && Instant::now() >= next {
            if let Some(proof) = quest_life::confirmation(&f.combat, f.combat.vitals[0].entity_id) {
                cs[i].command(LoginCommand::QuestRevive { proof });
                next = Instant::now() + Duration::from_millis(100);
            } else if let Some(a) = f.actors.iter().find(|a| a.hp > 0) {
                let target = npc::Id {
                    scope: f.scope,
                    serial: a.serial,
                };
                if cs[i].rendered.as_ref().is_some_and(|r| {
                    r.npcs
                        .iter()
                        .any(|n| n.id == target && n.life == a.life && n.alive)
                }) {
                    let command = if magic {
                        LoginCommand::CastNpcLightArrow {
                            target,
                            life: a.life,
                        }
                    } else {
                        LoginCommand::NpcAttack {
                            target,
                            life: a.life,
                        }
                    };
                    // The legacy renderer may finish a new wave before the
                    // canonical stream receives it. Match the real player's
                    // current life/target gate rather than sending from that
                    // old renderer alone. No mutation is queued while waiting.
                    if crate::modern_lito::battle_action::for_command(&v, &command).is_err() {
                        d.pause("canonical NPC action life and wave");
                        continue;
                    }
                    cs[i].command(command);
                    seen.insert((a.stage, a.serial));
                    if magic {
                        // command() first polls already received authoritative frames.
                        // NPC ticks may advance the world; this actor's spell ledger
                        // must not acquire an optimistic cast, receipt, or MP debit.
                        let after = cs[i].view();
                        let entity = f
                            .combat
                            .world
                            .actors
                            .iter()
                            .find(|a| {
                                a.matches_character(v.navigation.as_ref().unwrap().context.key)
                            })
                            .unwrap()
                            .entity_id;
                        assert_eq!(
                            after
                                .combat
                                .as_ref()
                                .unwrap()
                                .npc_spells
                                .iter()
                                .find(|s| s.entity_id == entity),
                            f.combat.npc_spells.iter().find(|s| s.entity_id == entity),
                            "no optimistic own spell, damage receipt or MP debit"
                        );
                        assert!(after.spell_pending);
                    }
                    // Production random misses are retried only after real time,
                    // never by replacing RNG or inventing accepted sequences.
                    next = Instant::now() + Duration::from_millis(if magic { 1250 } else { 750 });
                }
            }
        }
        d.pause("four SQL waves");
    }
    assert_eq!(seen.len(), 4);
    if magic {
        let v = cs[i].view();
        let f = v.combat.as_ref().unwrap();
        assert_eq!(
            f.npc_spells.len(),
            if bots { 2 } else { 1 },
            "force-isolated sparse actor ledger"
        );
        if bots {
            assert_eq!(f.npc_spells[1].mp_cost, 0);
            assert_eq!(f.npc_spells[1].cast_count, 0);
            assert_eq!(f.npc_spells[1].mp_spent, 0);
        }
        assert_eq!(f.npc_spells[0].entity_id, i as u64 + 1);
        assert_eq!(f.npc_spells[0].cast_count, 4);
        // The fixture knight has original INT12 (cost1), the mage cost3.
        // Confirm the independently expected class reduction, not merely a
        // multiple of a server-provided value.
        assert_eq!(f.npc_spells[0].mp_cost, if i == 0 { 1 } else { 3 });
        assert_eq!(f.npc_spells[0].mp_spent, if i == 0 { 4 } else { 12 });
        assert_eq!(f.world.casts[0].mp_spent, 0);
        assert!(f.damages.is_empty());
        assert!(
            cs[i].light_arrow_sound,
            "real shared NPC light-arrow flight must emit qualified sound97"
        );
    }
    assert_eq!(
        seen.iter().map(|(s, _)| *s).collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

#[test]
#[ignore = "requires compiled Java17 Main, owned MySQL5.5 and shared map554/PC/NPC/SPR; no HWND"]
fn java_quest_sql_main_two_controllers_shared_world_saved_records_and_source_reload() {
    sql_flow(false, false, false, false);
}

#[test]
#[ignore = "requires production Main, owned SQL and shared resources; real shop/use/reset; no HWND"]
fn java_quest_sql_supplies_buy_sell_unready_heal_render_and_reset() {
    sql_flow(true, false, false, false);
}

#[test]
#[ignore = "requires production Main, owned SQL and shared resources; v23 teleport/movement/abort; no HWND"]
fn java_quest_sql_teleport_sparse_cast_shared_renderer_move_and_normal_return() {
    sql_flow(false, true, false, false);
}

#[test]
#[ignore = "requires production Main, owned SQL and shared resources; v24 NPC magic/Quest settlement; no HWND"]
fn java_quest_sql_npc_magic_sparse_cast_shared_renderer_saved_records_and_return() {
    sql_flow(false, false, true, false);
}

#[test]
#[ignore = "requires production Main, owned SQL, real GM authorization and shared resources; passive Quest bots; no HWND"]
fn java_quest_sql_gm_bots_capacity_teams_sparse_magic_saved_records_and_return() {
    sql_flow(false, false, true, true);
}

fn sql_flow(supplies: bool, teleport: bool, magic: bool, bots: bool) {
    sql_flow_with_npc77(supplies, teleport, magic, bots, false);
}
fn sql_flow_with_npc77(supplies: bool, teleport: bool, magic: bool, bots: bool, npc77: bool) {
    sql_flow_with_npc_spells(supplies, teleport, magic, bots, npc77, false);
}
fn sql_flow_with_npc_spells(
    supplies: bool,
    teleport: bool,
    magic: bool,
    bots: bool,
    npc77: bool,
    npc70: bool,
) {
    sql_flow_with_chat(supplies, teleport, magic, bots, npc77, npc70, false);
}
fn sql_flow_with_chat(
    supplies: bool,
    teleport: bool,
    magic: bool,
    bots: bool,
    npc77: bool,
    npc70: bool,
    filtered_chat: bool,
) {
    sql_flow_with_matchmaking(
        supplies,
        teleport,
        magic,
        bots,
        npc77,
        npc70,
        filtered_chat,
        false,
    );
}
fn sql_flow_with_matchmaking(
    supplies: bool,
    teleport: bool,
    magic: bool,
    bots: bool,
    npc77: bool,
    npc70: bool,
    filtered_chat: bool,
    matchmaking: bool,
) {
    assert!(!matchmaking || npc70 && filtered_chat);
    assert!(!filtered_chat || npc70);
    assert!(!npc77 || !(supplies || teleport || magic || bots));
    assert!(!npc70 || !(supplies || teleport || magic || bots || npc77));
    let d = Deadline(Instant::now() + Duration::from_secs(160));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernQuestSqlTestPeer",
        if matchmaking {
            "quest-matchmaking-sql"
        } else if npc70 {
            "quest-npc70-sql"
        } else if npc77 {
            "quest-npc77-sql"
        } else if bots {
            "quest-bots-sql"
        } else if magic {
            "quest-magic-sql"
        } else if teleport {
            "quest-teleport-sql"
        } else if supplies {
            "quest-supplies-sql"
        } else {
            "quest-sql"
        },
    );
    let p = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new(p.clone()),
        runtime: WorldRuntime::new(shared.clone()).unwrap(),
        rendered: None,
        auto_ack: false,
        acks: BTreeSet::new(),
        light_arrow_sound: false,
        canonical: Some(
            crate::modern_lito::battle_runtime::BattleRuntime::new(shared.clone()).unwrap(),
        ),
        canonical_surface: None,
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    let mut supply_rounds = [0; 2];
    for i in 0..2 {
        cs[i].command(LoginCommand::Connect);
        wait(&mut cs, &d, "TLS ready", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        cs[i].command(LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "SQL authenticated sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        let v = cs[i].view();
        assert_eq!(v.sources.len(), 2);
        assert_eq!(v.sources[0].label, "太陽神");
        assert_eq!(v.sources[1].label, "月亮");
        contexts[i] = if npc77 || npc70 {
            selected_with_mp(&mut cs, i, &d, if i == 0 { 7 } else { 42 }, 300)
        } else {
            selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 })
        };
        assert!(
            cs[i].view().action_supported && cs[i].view().battle_supported,
            "production Main must explicitly select v37 life-bound actions"
        );
        if npc70 {
            quest_npc70_sql_interop_tests::assert_skill(&cs[i]);
        }
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0
            })
        );
        cs[i].command(LoginCommand::EnterChannel {
            context: contexts[i],
            channel_id: CHANNEL,
        });
        wait(&mut cs, &d, "lobby", |cs| {
            cs[i].view().phase == LoginPhase::InLobby
        });
        if supplies {
            let empty = shop_action(&mut cs, i, &d, shop::Action::Read, 0, 0);
            assert!(empty.stacks.is_empty());
            assert_eq!(empty.balance, 20000);
            assert!(empty.purchasable(40010));
            let bought = shop_action(
                &mut cs,
                i,
                &d,
                shop::Action::Buy,
                40010,
                if i == 0 { 4 } else { 2 },
            );
            assert_eq!(
                bought.stack(40010).unwrap().quantity,
                if i == 0 { 4 } else { 2 }
            );
            assert_eq!(bought.balance, if i == 0 { 19840 } else { 19920 });
            assert_eq!(bought.round_id, empty.round_id);
            supply_rounds[i] = bought.round_id;
        }
    }
    let old_chat = filtered_chat.then(|| quest_filtered_chat_sql_interop_tests::lobby(&mut cs, &d));
    if matchmaking {
        quest_matchmaking_sql_interop_tests::matched(&mut cs, &d, contexts);
    } else {
        let mut create = rooms::Common::read(contexts[0], CHANNEL);
        create.action = rooms::Action::Create;
        room(
            &mut cs,
            0,
            &d,
            rooms::Request::draft(
                create,
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 2,
                        map_id: 554,
                        attributes: 16,
                        name: "Actual SQL Quest".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
    }
    if bots {
        // Before the second human joins, exercise a WHOLE bot-only force
        // through real Main admission, shared rendering and automatic start.
        cs[0].command(LoginCommand::GmBots { count: 20 });
        wait(&mut cs, &d, "single GM fills room", |cs| {
            !cs[0].view().room_pending
        });
        let v = cs[0].view();
        let joined = v.rooms.joined().unwrap();
        assert_eq!(joined.members.len(), 4);
        assert_eq!(
            joined
                .members
                .iter()
                .filter(|m| m.kind == rooms::MemberKind::Human)
                .count(),
            1
        );
        let s = joined.summary.clone();
        room(
            &mut cs,
            0,
            &d,
            rooms::Request::simple(rooms::Common {
                context: contexts[0],
                channel_id: CHANNEL,
                action: rooms::Action::Ready,
                value: 1,
                room_id: s.room_id,
                expected_revision: s.revision,
            })
            .unwrap(),
        );
        cs[0].auto_ack = true;
        wait(&mut cs, &d, "single human and three bots active", |cs| {
            cs[0].rendered.is_some()
                && cs[0]
                    .view()
                    .world
                    .frame()
                    .is_some_and(|f| f.phase == world::Phase::Active)
        });
        let f = cs[0].view().world.frame().unwrap().clone();
        assert_eq!(f.room.members.len(), 4);
        assert_eq!(f.actors.len(), 2, "opposing bot-only force stays private");
        assert!(f.actors.iter().all(|a| a.loaded));
        let rendered = cs[0].rendered.as_ref().unwrap();
        assert_eq!(
            rendered
                .actors
                .iter()
                .map(|a| a.entity_id)
                .collect::<BTreeSet<_>>(),
            f.actors.iter().map(|a| a.entity_id).collect()
        );
        assert!(rendered
            .actors
            .iter()
            .all(|a| a.width > 0 && a.height > 0 && !a.sprite_name.is_empty()));
        assert_eq!(cs[0].acks.len(), 1, "only the human needs a resource ACK");
        assert!(cs[1].acks.is_empty());
        cs[0].auto_ack = false;
        let mut leave = world::Request::read(contexts[0], CHANNEL);
        leave.action = world::Action::Leave;
        leave.match_id = f.match_id;
        leave.generation = f.generation;
        cs[0].command(LoginCommand::World(leave));
        wait(&mut cs, &d, "single GM normal world return", |cs| {
            cs[0].view().world.frame().is_none()
                && !cs[0].view().world_pending
                && cs[0].runtime.is_quiescent()
        });
        room(
            &mut cs,
            0,
            &d,
            rooms::Request::read(contexts[0], CHANNEL).unwrap(),
        );
        cs[0].command(LoginCommand::GmBots { count: 0 });
        wait(&mut cs, &d, "retire first match dummies", |cs| {
            !cs[0].view().room_pending
        });
        assert_eq!(cs[0].view().rooms.joined().unwrap().members.len(), 1);
    }
    if !matchmaking {
        let s = cs[0].view().rooms.joined().unwrap().summary.clone();
        room(
            &mut cs,
            1,
            &d,
            rooms::Request::join(
                rooms::Common {
                    context: contexts[1],
                    channel_id: CHANNEL,
                    action: rooms::Action::Join,
                    value: 0,
                    room_id: s.room_id,
                    expected_revision: s.revision,
                },
                String::new(),
            )
            .unwrap(),
        );
    }
    if bots {
        // The guest source has no GM grant. A real SELECT must reject it
        // without optimistic seats, even though it belongs to the same room.
        let before = cs[1].view().rooms.joined().unwrap().clone();
        cs[1].command(LoginCommand::GmBots { count: 20 });
        wait(&mut cs, &d, "unprivileged GM denial", |cs| {
            !cs[1].view().room_pending
        });
        let rooms::State::Ready(denied) = cs[1].view().rooms else {
            panic!("GM denial reply absent")
        };
        assert_eq!(denied.reason, rooms::Reason::GmNotAuthorized);
        assert_eq!(cs[1].view().rooms.joined(), Some(&before));
        room(
            &mut cs,
            0,
            &d,
            rooms::Request::read(contexts[0], CHANNEL).unwrap(),
        );
        let before = cs[0].view().rooms.clone();
        cs[0].command(LoginCommand::GmBots { count: 20 });
        assert_eq!(cs[0].view().rooms, before, "no optimistic dummy insertion");
        wait(&mut cs, &d, "SQL authorized bounded GM spawn", |cs| {
            !cs[0].view().room_pending
        });
        let view = cs[0].view();
        let joined = view.rooms.joined().unwrap();
        assert_eq!(
            joined.members.len(),
            4,
            "request20 capped by capacity4 minus2 humans"
        );
        for lane in 0..2 {
            assert_eq!(joined.members.iter().filter(|m| m.team == lane).count(), 2);
        }
        for bot in joined
            .members
            .iter()
            .filter(|m| m.kind == rooms::MemberKind::ServerBot)
        {
            assert!(bot.ready);
            assert_eq!(bot.source_name, "太陽神");
            assert!(bot.name.starts_with("BOT"));
            assert!(
                joined.member(bot.key).is_none(),
                "bot ID is not a SQL character ID"
            );
        }
    }
    if supplies {
        room(
            &mut cs,
            0,
            &d,
            rooms::Request::read(contexts[0], CHANNEL).unwrap(),
        );
        let s = cs[0].view().rooms.joined().unwrap().summary.clone();
        room(
            &mut cs,
            0,
            &d,
            rooms::Request::simple(rooms::Common {
                context: contexts[0],
                channel_id: CHANNEL,
                action: rooms::Action::Ready,
                value: 1,
                room_id: s.room_id,
                expected_revision: s.revision,
            })
            .unwrap(),
        );
        assert!(
            cs[0]
                .view()
                .rooms
                .joined()
                .unwrap()
                .members
                .iter()
                .find(|m| m.matches_character(contexts[0].key))
                .unwrap()
                .ready
        );
        let sold = shop_action(&mut cs, 0, &d, shop::Action::Sell, 40010, 1);
        assert_eq!(sold.stack(40010).unwrap().quantity, 3);
        assert_eq!(sold.balance, 19860);
        assert!(
            !cs[0]
                .view()
                .rooms
                .joined()
                .unwrap()
                .members
                .iter()
                .find(|m| m.matches_character(contexts[0].key))
                .unwrap()
                .ready
        );
    }
    for i in 0..2 {
        room(
            &mut cs,
            i,
            &d,
            rooms::Request::read(contexts[i], CHANNEL).unwrap(),
        );
        let s = cs[i].view().rooms.joined().unwrap().summary.clone();
        room(
            &mut cs,
            i,
            &d,
            rooms::Request::simple(rooms::Common {
                context: contexts[i],
                channel_id: CHANNEL,
                action: rooms::Action::Ready,
                value: 1,
                room_id: s.room_id,
                expected_revision: s.revision,
            })
            .unwrap(),
        );
    }
    cs[0].auto_ack = true;
    wait(&mut cs, &d, "both real renderers loaded", |cs| {
        cs.iter().all(|c| {
            c.rendered
                .as_ref()
                .is_some_and(|r| r.ready_to_ack && r.npc.is_some())
        }) && cs[0]
            .view()
            .world
            .frame()
            .is_some_and(|f| f.actors[0].loaded)
    });
    for (i, c) in cs.iter().enumerate() {
        let v = c.view();
        let f = v.world.frame().unwrap();
        assert_eq!(f.phase, world::Phase::Loading);
        assert_eq!(f.audience_lane, Some(i as u8));
        assert_eq!(f.actors.len(), if bots { 2 } else { 1 });
        assert!(f.actors[0].matches_character(contexts[i].key));
        if bots {
            assert!(f.actors[1].loaded);
            assert_eq!(f.actors[1].kind, rooms::MemberKind::ServerBot);
            let r = c.rendered.as_ref().unwrap();
            assert_eq!(
                r.actors
                    .iter()
                    .map(|a| a.entity_id)
                    .collect::<BTreeSet<_>>(),
                f.actors.iter().map(|a| a.entity_id).collect()
            );
            assert!(r
                .actors
                .iter()
                .all(|a| a.width > 0 && a.height > 0 && !a.sprite_name.is_empty()));
            assert_eq!(
                f.room
                    .members
                    .iter()
                    .find(|m| m.identity() == f.actors[1].identity())
                    .unwrap()
                    .team,
                i as u8
            );
        }
    }
    assert_eq!(cs[0].acks.len(), if bots { 2 } else { 1 });
    assert!(cs[1].acks.is_empty());
    if let Some(old) = old_chat {
        quest_filtered_chat_sql_interop_tests::loading(&mut cs, &d, old);
        assert!(
            cs[1].acks.is_empty(),
            "chat must not replace the resource admission barrier"
        );
        assert!(cs
            .iter()
            .all(|c| c.view().world.frame().unwrap().phase == world::Phase::Loading));
    }
    cs[1].auto_ack = true;
    if matchmaking {
        wait(
            &mut cs,
            &d,
            "both observe real countdown after both resource acknowledgments",
            |cs| {
                cs.iter().all(|c| {
                    c.view().world.frame().is_some_and(|f| {
                        f.phase == world::Phase::Countdown && f.actors.iter().all(|a| a.loaded)
                    })
                })
            },
        );
    }
    wait(&mut cs, &d, "actual Main active world", |cs| {
        cs.iter().all(|c| {
            c.view()
                .world
                .frame()
                .is_some_and(|f| f.phase == world::Phase::Active)
        })
    });
    followed(
        &mut cs,
        &d,
        crate::modern_lito::battle_payload::data::Phase::Active,
    );
    if filtered_chat {
        quest_filtered_chat_sql_interop_tests::active(&mut cs, &d);
    }
    if supplies {
        use_supplies(&mut cs, &d);
    }
    if teleport {
        teleport_and_return(&mut cs, &d, &shared, contexts);
        peer.expect_successful_exit(&d);
        return;
    }
    for i in 0..2 {
        if npc70 {
            quest_npc70_sql_interop_tests::clear_waves(&mut cs, i, &d, filtered_chat);
        } else if npc77 {
            quest_npc77_sql_interop_tests::clear_npc77_waves(&mut cs, i, &d);
        } else {
            clear_waves(&mut cs, i, &d, magic, bots);
        }
    }
    let target = cs[0].rendered.as_ref().unwrap().quests[0].id;
    let foreign = cs[1].rendered.as_ref().unwrap().quests[0].id;
    assert!(cs[0]
        .controller
        .command(LoginCommand::QuestAttack { target: foreign })
        .is_err());
    if npc70 {
        quest_npc70_sql_interop_tests::finish_objective(&mut cs, &d);
    }
    let mut next = Instant::now() + Duration::from_millis(750);
    loop {
        wait(&mut cs, &d, "target pump", |_| true);
        let v = cs[0].view();
        let f = v.combat.as_ref().unwrap();
        if f.world.phase == world::Phase::Finished {
            break;
        }
        if !v.combat_pending && Instant::now() >= next {
            if let Some(proof) = quest_life::confirmation(f, f.vitals[0].entity_id) {
                cs[0].command(LoginCommand::QuestRevive { proof });
            } else {
                cs[0].command(LoginCommand::QuestAttack { target });
            }
            next = Instant::now() + Duration::from_millis(750);
        }
        d.pause("target accepted death");
    }
    wait(&mut cs, &d, "actual SQL saved completion", |cs| {
        cs.iter().all(|c| {
            c.view().world.frame().is_some_and(|f| {
                f.phase == world::Phase::Finished
                    && f.completion
                        .as_ref()
                        .is_some_and(|r| r.state == world::SettlementState::Saved)
            })
        })
    });
    let result = cs[0]
        .view()
        .world
        .frame()
        .unwrap()
        .completion
        .clone()
        .unwrap();
    assert_eq!(
        result,
        cs[1]
            .view()
            .world
            .frame()
            .unwrap()
            .completion
            .clone()
            .unwrap()
    );
    assert_eq!(result.event.mode, 2);
    assert_eq!(result.event.participants.len(), if bots { 4 } else { 2 });
    if npc77 {
        quest_npc77_sql_interop_tests::assert_saved(&cs);
    }
    if npc70 {
        quest_npc70_sql_interop_tests::assert_saved(&cs);
    }
    for i in 0..2 {
        assert_eq!(
            cs[i]
                .view()
                .world
                .frame()
                .unwrap()
                .quest
                .as_ref()
                .unwrap()
                .winning_lane,
            Some(0)
        );
        assert_eq!(cs[i].acks.len(), if bots && i == 0 { 2 } else { 1 });
        let f = cs[i].view().world.frame().unwrap().clone();
        let mut leave = world::Request::read(contexts[i], CHANNEL);
        leave.action = world::Action::Leave;
        leave.match_id = f.match_id;
        leave.generation = f.generation;
        cs[i].command(LoginCommand::World(leave));
        wait(&mut cs, &d, "world return", |cs| {
            cs[i].view().world.frame().is_none()
                && !cs[i].view().world_pending
                && cs[i].runtime.is_quiescent()
        });
        if matchmaking {
            quest_matchmaking_sql_interop_tests::still_in_room(&cs[i]);
        }
        cs[i].command(LoginCommand::ReadRecord);
        wait(&mut cs, &d, "record SQL readback", |cs| {
            !cs[i].view().record_pending
        });
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: if i == 0 { 1 } else { 0 },
                losses: if i == 0 { 0 } else { 1 },
                revision: 1
            })
        );
        if supplies {
            let reset = shop_action(&mut cs, i, &d, shop::Action::Read, 0, 0);
            assert!(reset.round_id > supply_rounds[i]);
            assert_eq!(reset.balance, 20000);
            assert!(reset.stacks.is_empty());
            assert_source(&cs[i].view(), contexts[i]);
        }
    }
    if filtered_chat {
        quest_filtered_chat_sql_interop_tests::returned(&mut cs, &d);
    }
    // Release the Moon lease normally before selecting the same source/object
    // on the Sun controller; otherwise a correct presence guard denies it.
    cs[1].command(LoginCommand::Shutdown);
    wait(&mut cs, &d, "guest session retirement", |cs| {
        cs[1].controller.is_quiescent()
    });
    // Local controller quiescence is not a server-side disconnect receipt.
    // The guest can still be in the next room READ, then retire between READ
    // and LEAVE and correctly invalidate its expected_revision. Wait for the
    // authority roster, without auto-resending a mutation or bypassing CONFLICT.
    loop {
        room(
            &mut cs,
            0,
            &d,
            rooms::Request::read(contexts[0], CHANNEL).unwrap(),
        );
        let view = cs[0].view();
        if view
            .rooms
            .joined()
            .unwrap()
            .member(contexts[1].key)
            .is_none()
        {
            break;
        }
        d.pause("guest removed from authoritative room roster");
    }
    let s = cs[0].view().rooms.joined().unwrap().summary.clone();
    room(
        &mut cs,
        0,
        &d,
        rooms::Request::simple(rooms::Common {
            context: contexts[0],
            channel_id: CHANNEL,
            action: rooms::Action::Leave,
            value: 0,
            room_id: s.room_id,
            expected_revision: s.revision,
        })
        .unwrap(),
    );
    let departed = cs[0].view();
    if matchmaking {
        quest_matchmaking_sql_interop_tests::retired(&cs[0]);
    }
    assert!(
        departed.rooms.joined().is_none(),
        "room LEAVE must succeed before lobby navigation: {} / {:?}",
        departed.status,
        departed.rooms
    );
    let previous_supply_request = supplies
        .then(|| lobby_supplies_sql_interop_tests::channels_after_battle(&mut cs, &d, contexts[0]));
    cs[0].command(LoginCommand::LeaveLobby(contexts[0]));
    wait(&mut cs, &d, "back to channels", |cs| {
        cs[0].view().phase == LoginPhase::ChoosingChannel
    });
    cs[0].command(LoginCommand::ReturnCharacters(contexts[0]));
    wait(&mut cs, &d, "back to roster", |cs| {
        cs[0].view().phase == LoginPhase::CharactersReady
    });
    let moon = if npc77 || npc70 {
        selected_with_mp(&mut cs, 0, &d, 42, 300)
    } else {
        selected(&mut cs, 0, &d, 42)
    };
    assert_ne!(moon, contexts[0]);
    if matchmaking {
        quest_matchmaking_sql_interop_tests::retired(&cs[0]);
    }
    if npc70 {
        quest_npc70_sql_interop_tests::assert_skill(&cs[0]);
    }
    assert_eq!(
        cs[0].view().record.totals(),
        Some(record::Totals {
            stored: true,
            wins: 0,
            losses: 1,
            revision: 1
        })
    );
    if let Some(old) = previous_supply_request {
        lobby_supplies_sql_interop_tests::changed_source(&mut cs, &d, moon, old);
    }
    assert!(cs[0].view().npc.is_none());
    assert!(cs[0].runtime.is_quiescent());
    if let Some(old) = old_chat {
        quest_filtered_chat_sql_interop_tests::reselected(&mut cs, &d, old, moon);
    }
    cs[0].command(LoginCommand::Shutdown);
    wait(&mut cs, &d, "both owners retired", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}

fn shop_action(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    action: shop::Action,
    id: i64,
    quantity: i64,
) -> shop::Snapshot {
    let before = cs[i].view();
    let previous = before.shop.snapshot().cloned();
    let nav = before.navigation.as_ref().unwrap();
    let read = shop::Request::read(
        nav.context,
        nav.snapshot.channel_id.unwrap(),
        before.rooms.joined(),
    );
    let request = if action == shop::Action::Read {
        read
    } else {
        read.mutation(previous.as_ref().unwrap(), action, id, quantity)
    };
    cs[i].command(LoginCommand::Shop(request));
    assert_eq!(cs[i].view().shop.snapshot(), previous.as_ref());
    wait(cs, d, "actual SQL shop acknowledgment", |cs| {
        let v = cs[i].view();
        !v.shop_pending && v.shop.reply().is_some_and(|r| r.echo == request)
    });
    let view = cs[i].view();
    let receipt = view.shop.reply().unwrap();
    assert_eq!(
        (receipt.status, receipt.reason),
        (shop::Status::Ok, shop::Reason::None)
    );
    assert_eq!(
        receipt.room.as_ref().unwrap().joined.as_ref(),
        view.rooms.joined()
    );
    view.shop.snapshot().unwrap().clone()
}

#[track_caller]
fn assert_pending_action_not_predicted(before: &combat::Frame, view: &LoginSnapshot) {
    let current = view.combat.as_ref().unwrap();
    assert_eq!(current.world.match_id, before.world.match_id);
    assert_eq!(current.world.generation, before.world.generation);
    assert!(current.world.revision >= before.world.revision);
    assert_eq!(view.world.frame(), Some(&current.world));
    assert_eq!(
        view.npc.as_ref().unwrap().frame.as_ref().unwrap().combat,
        **current
    );
    // A command first polls confirmed NPC ticks. Those may change HP / recovery,
    // but cannot create a local teleport, item receipt, or stock deduction.
    assert_eq!(current.world.casts, before.world.casts);
    assert_eq!(current.items, before.items);
    assert_eq!(current.npc_spells, before.npc_spells);
    for prior in &before.vitals {
        let next = current.vital(prior.entity_id).unwrap();
        assert_eq!(
            current
                .world
                .actors
                .iter()
                .find(|a| a.entity_id == prior.entity_id)
                .unwrap()
                .position,
            before
                .world
                .actors
                .iter()
                .find(|a| a.entity_id == prior.entity_id)
                .unwrap()
                .position
        );
        let a = before.regen(prior.entity_id).unwrap();
        let b = current.regen(prior.entity_id).unwrap();
        assert_eq!(
            next.mp - prior.mp,
            (b.mp_gain - a.mp_gain) - (b.mp_loss - a.mp_loss)
        );
        assert_eq!(
            next.hp - prior.hp,
            (b.hp_gain - a.hp_gain) - (b.hp_loss - a.hp_loss) + (b.item_gain - a.item_gain)
                - (b.damage_loss - a.damage_loss)
        );
    }
    if current.world.revision == before.world.revision {
        assert_eq!(current.as_ref(), before);
    }
}

fn teleport_and_return(
    cs: &mut [Client; 2],
    d: &Deadline,
    _shared: &std::path::Path,
    contexts: [SelectionContext; 2],
) {
    for i in 0..2 {
        let before = cs[i].view();
        assert!(before.skills.teleport().is_some());
        let old = before.combat.as_ref().unwrap();
        assert_eq!(old.world.casts.len(), 1);
        assert_eq!(old.world.casts[0].entity_id, (i + 1) as u64);
        assert_eq!(old.world.casts[0].cast_count, 0);
        cs[i].command(LoginCommand::CastTeleport);
        assert!(cs[i].view().spell_pending);
        assert_pending_action_not_predicted(old, &cs[i].view());
        wait(cs, d, "v23 actual SQL teleport ACK", |cs| {
            !cs[i].view().spell_pending
        });
        let confirmed = cs[i].view();
        let f = confirmed.combat.as_ref().unwrap();
        let cast = &f.world.casts[0];
        assert_eq!(cast.cast_count, 1, "{}", confirmed.status);
        assert_eq!(cast.request_sequence, 1);
        assert_eq!(cast.mp_spent, 10);
        assert_ne!(f.world.actors[0].position, old.world.actors[0].position);
        let target = f.world.actors[0].position;
        assert_eq!(cast.to, target);
        assert_eq!(
            f.vitals[0].mp,
            f.vitals[0].max_mp - 10 + f.regen[0].mp_gain - f.regen[0].mp_loss
        );
        wait(cs, d, "v23 teleport NPC shared renderer", |cs| {
            cs[i].rendered.as_ref().is_some_and(|r| {
                r.npc.is_some()
                    && r.combat.as_ref().is_some_and(|f| {
                        f.world.casts[0].cast_count == 1 && f.world.actors[0].position == target
                    })
            })
        });
        let rendered = cs[i].rendered.as_ref().unwrap();
        assert_eq!(rendered.combat.as_ref().unwrap().world.casts.len(), 1);
        let geometry = rendered.navigation_map().unwrap();
        assert!(geometry.standing(target));
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(1250) {
            wait(cs, d, "teleport cast cadence", |_| true);
            d.pause("teleport cast cadence");
        }
        let view = cs[i].view();
        let world = view.world.frame().unwrap();
        let at = world.actors[0].position;
        let heading = (0..8)
            .find(|&h| {
                let (dx, dy) = DIRECTIONS[h];
                let to = WorldCell {
                    x: at.x + dx,
                    y: at.y + dy,
                };
                geometry.can_step(at, to)
                    && view
                        .npc
                        .as_ref()
                        .unwrap()
                        .frame
                        .as_ref()
                        .unwrap()
                        .actors
                        .iter()
                        .all(|n| n.hp == 0 || n.cell != to)
            })
            .expect("teleported cell must retain an actual move");
        let (dx, dy) = DIRECTIONS[heading];
        let destination = WorldCell {
            x: at.x + dx,
            y: at.y + dy,
        };
        let mut request = world::Request::read(contexts[i], CHANNEL);
        request.action = world::Action::Move;
        request.heading = heading as u8;
        request.match_id = world.match_id;
        request.generation = world.generation;
        request.sequence = world.actors[0].sequence + 1;
        request.from_x = at.x;
        request.from_y = at.y;
        cs[i].command(LoginCommand::World(request));
        wait(
            cs,
            d,
            "movement after teleport has no native cursor lock",
            |cs| !cs[i].view().world_pending,
        );
        assert_eq!(
            cs[i].view().world.frame().unwrap().actors[0].position,
            destination
        );
    }
    for i in 0..2 {
        let f = cs[i].view().world.frame().unwrap().clone();
        let mut leave = world::Request::read(contexts[i], CHANNEL);
        leave.action = world::Action::Leave;
        leave.match_id = f.match_id;
        leave.generation = f.generation;
        cs[i].command(LoginCommand::World(leave));
        wait(cs, d, "teleport world normal return", |cs| {
            cs[i].view().world.frame().is_none()
                && !cs[i].view().world_pending
                && cs[i].runtime.is_quiescent()
        });
        cs[i].command(LoginCommand::ReadRecord);
        wait(cs, d, "aborted teleport world has no false win", |cs| {
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
        assert_source(&cs[i].view(), contexts[i]);
    }
    for c in cs.iter_mut() {
        c.command(LoginCommand::Shutdown);
    }
    wait(
        cs,
        d,
        "teleport controllers and render workers retired",
        |cs| {
            cs.iter()
                .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        },
    );
}

fn use_supplies(cs: &mut [Client; 2], d: &Deadline) {
    for i in 0..2 {
        wait(cs, d, "actual NPC damage before potion", |cs| {
            cs[i]
                .view()
                .combat
                .as_ref()
                .is_some_and(|f| f.vitals[0].hp > 0 && f.vitals[0].hp < f.vitals[0].max_hp)
        });
        let before = cs[i].view();
        let combat = before.combat.as_ref().unwrap();
        let item = &combat.items[0];
        assert_eq!(item.stocks[0].initial, if i == 0 { 3 } else { 2 });
        assert_eq!(item.stocks[0].remaining, item.stocks[0].initial);
        assert_eq!(item.use_count, 0);
        assert_eq!(combat.world.audience_lane, Some(i as u8));
        cs[i].command(LoginCommand::UseItem { item_id: 40010 });
        assert!(cs[i].view().item_pending);
        assert_pending_action_not_predicted(combat, &cs[i].view());
        wait(cs, d, "atomic potion receipt", |cs| {
            !cs[i].view().item_pending
        });
        let after = cs[i].view();
        let f = after.combat.as_ref().unwrap();
        let state = &f.items[0];
        assert_eq!(state.use_count, 1);
        assert_eq!(state.request_sequence, 1);
        assert!(state.restored > 0 && state.restored <= 20);
        assert_eq!(state.hp_after - state.hp_before, state.restored);
        assert_eq!(state.stocks[0].remaining, item.stocks[0].remaining - 1);
        wait(cs, d, "NPC renderer after potion", |cs| {
            cs[i].rendered.as_ref().is_some_and(|r| {
                r.npc.is_some() && r.combat.as_ref().is_some_and(|f| f.items[0].use_count == 1)
            })
        });
        let rendered = cs[i].rendered.as_ref().unwrap();
        assert_eq!(
            rendered.combat.as_ref().unwrap().world.audience_lane,
            Some(i as u8)
        );
        assert_eq!(rendered.combat.as_ref().unwrap().items.len(), 1);
        assert!(!rendered.npcs.is_empty());
    }
}
