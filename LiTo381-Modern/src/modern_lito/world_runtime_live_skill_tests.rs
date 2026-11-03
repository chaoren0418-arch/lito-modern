//! Test-only replay of unmodified, validated live TLS replies. Server authority
//! uses its real clock; receipt delivery/presentation times below are explicit
//! scheduling fixtures (including coalescing), not HWND or network-latency proof.
use super::*;
use crate::modern_lito::npc_protocol as npc;

pub(crate) struct Probe {
    renderer: SharedRenderer,
    timeline: Timeline,
    plan: Arc<super::super::npc_presentation::Plan>,
    cancel: AtomicBool,
    sounds: BTreeMap<u32, std::collections::BTreeSet<u64>>,
    pub(crate) now: Instant,
}
impl Probe {
    pub(crate) fn new(reply: &npc::Reply) -> Self {
        reply.validate().unwrap();
        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let plan = Arc::new(super::super::npc_presentation::Plan::from_reply(reply).unwrap());
        let npc = NpcInput {
            reply: Arc::new(reply.clone()),
            plan: plan.clone(),
        };
        let f = reply.frame.as_ref().unwrap();
        let cancel = AtomicBool::new(false);
        let renderer =
            load_shared_renderer_npcs(&root, &f.combat.world, Some(&f.combat), Some(&npc), &cancel)
                .unwrap();
        let input = Input::with_npc(
            Arc::new(f.combat.world.clone()),
            1,
            Some(Arc::new(f.combat.clone())),
            Some(npc),
        )
        .unwrap();
        let now = input.received;
        let mut probe = Self {
            renderer,
            timeline: Timeline::new(Arc::new(input)),
            plan,
            cancel,
            sounds: BTreeMap::new(),
            now,
        };
        probe.draw(reply, now);
        probe
    }
    fn draw(&mut self, reply: &npc::Reply, at: Instant) -> SceneSnapshot {
        reply.validate().unwrap();
        let npc = NpcInput {
            reply: Arc::new(reply.clone()),
            plan: self.plan.clone(),
        };
        let f = reply.frame.as_ref().unwrap();
        let mut input = Input::with_npc(
            Arc::new(f.combat.world.clone()),
            1,
            Some(Arc::new(f.combat.clone())),
            Some(npc.clone()),
        )
        .unwrap();
        input.received = at;
        self.timeline.update(Arc::new(input)).unwrap();
        let scene = self.timeline.snapshot(at).unwrap();
        let pixels = self
            .renderer
            .render_at_npcs(&scene, Some(&f.combat), Some(&npc), &self.cancel, at)
            .unwrap();
        assert_eq!(pixels.bgra.len(), WIDTH as usize * HEIGHT as usize * 4);
        for e in pixels.audio.iter().flat_map(|a| a.events.iter()) {
            for id in [97, 119, 717] {
                if e.group.candidates() == [id] {
                    self.sounds.entry(id).or_default().insert(e.sequence);
                }
            }
        }
        scene
    }
}

pub(crate) fn prove(skill: u32, before: &npc::Reply, cast: &npc::Reply, melee: &npc::Reply) {
    let frozen = (before.clone(), cast.clone(), melee.clone());
    let c = cast.frame.as_ref().unwrap();
    let m = melee.frame.as_ref().unwrap();
    let revision = match skill {
        4 => c.combat.npc_spell(1).unwrap().revision,
        77 => c.combat.npc_disintegrate(1).unwrap().revision,
        5 => c.combat.world.cast(1).unwrap().cast_revision,
        _ => panic!("unqualified arbitration skill"),
    };
    assert!(
        super::super::npc_presentation::player_attack(m, 1)
            .unwrap()
            .revision
            > revision
    );
    // Cancellation must come from typed melee, not position, hurt, death or a
    // legacy PvP attack field accidentally changed by the fixture.
    assert_eq!(c.combat.world.actor(1), m.combat.world.actor(1));
    let cv = c.combat.vital(1).unwrap();
    let mv = m.combat.vital(1).unwrap();
    assert_eq!(
        (cv.hp, cv.mp, cv.hurt_revision, cv.attack_revision),
        (mv.hp, mv.mp, mv.hurt_revision, mv.attack_revision)
    );
    assert!(cv.hp > 0);
    for phase in 0..if skill == 5 { 2 } else { 3 } {
        let mut p = Probe::new(before);
        let now = p.now;
        let member = c.combat.world.member(1).unwrap();
        let appearance = BaseAppearance {
            class: member.class as u8,
            sex: member.sex,
        };
        let delay = p
            .renderer
            .light_arrow
            .as_ref()
            .unwrap()
            .assets
            .cast_launch_delay(appearance)
            .unwrap();
        assert!(delay > Duration::from_millis(2));
        if phase != 0 {
            p.draw(cast, now);
        }
        if phase == 2 {
            p.draw(cast, now + delay);
        }
        let at = now
            + if skill == 5 {
                Duration::from_millis(1)
            } else if phase == 2 {
                delay + Duration::from_millis(1)
            } else {
                delay / 2
            };
        let scene = p.draw(melee, at);
        assert_eq!(
            scene
                .actors()
                .iter()
                .find(|a| a.entity_id == 1)
                .unwrap()
                .action,
            BaseAction::Attack
        );
        match skill {
            4 => {
                let light = p.renderer.light_arrow.as_ref().unwrap();
                assert!(light.active.is_empty());
                assert_eq!(light.flights.len(), usize::from(phase == 2));
            }
            77 => {
                let dis = p.renderer.disintegrate.as_ref().unwrap();
                assert!(dis
                    .overrides(&p.renderer.light_arrow.as_ref().unwrap().assets, at)
                    .unwrap()
                    .is_empty());
                assert_eq!(dis.draws(at).unwrap().len(), usize::from(phase == 2));
            }
            5 => {
                let tele = p.renderer.teleport.as_ref().unwrap();
                let v = &tele.active[&1];
                assert!(!teleport_body_current(
                    &m.combat,
                    Some(m),
                    &v.event,
                    v.move_sequence
                ));
                assert!(at.duration_since(v.started) < tele.assets.effect_duration());
            }
            _ => unreachable!(),
        }
        // Repeat the same authoritative receipt and advance beyond all owned
        // effects: no mana mutation, late launch, cue replay or retained clock.
        p.draw(melee, at + Duration::from_millis(1));
        p.draw(melee, at + Duration::from_millis(200));
        p.draw(melee, now + Duration::from_secs(11));
        assert!(p.renderer.light_arrow.as_ref().unwrap().active.is_empty());
        assert!(p.renderer.light_arrow.as_ref().unwrap().flights.is_empty());
        assert!(p.renderer.teleport.as_ref().unwrap().active.is_empty());
        assert!(p
            .renderer
            .disintegrate
            .as_ref()
            .unwrap()
            .draws(now + Duration::from_secs(11))
            .unwrap()
            .is_empty());
        let sound = match skill {
            4 => 97,
            77 => 717,
            _ => 119,
        };
        assert_eq!(
            p.sounds.get(&sound).map_or(0, |s| s.len()),
            usize::from(skill == 5 || phase == 2)
        );
    }
    assert_eq!((before, cast, melee), (&frozen.0, &frozen.1, &frozen.2));
}
