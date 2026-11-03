// Included in session::tests; exercises the actual controller mailbox.
fn npc77_view() -> (
    LoginSnapshot,
    world::State,
    super::super::npc_disintegrate_protocol::Request,
) {
    let r = super::super::disintegrate_presentation::npc_tests::reply("normal-before");
    let (mut view, state, _) = encounter_view(r.clone());
    view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
        context: r.echo.context,
        status: spell::Status::Ok,
        skills: vec![spell::Skill {
            skill_id: 77,
            name: "npc77".into(),
            mp_cost: 70,
            reuse_millis: 5000,
            support: spell::Support::Disintegrate,
        }],
    }));
    let f = r.frame.as_ref().unwrap();
    let n = &f.actors[0];
    let q = super::super::npc_disintegrate_protocol::Request {
        context: r.echo.context,
        channel_id: r.echo.channel_id,
        target: npc::Id {
            scope: f.scope,
            serial: n.serial,
        },
        sequence: 1,
        caster_life: 1,
        target_life: n.life,
    };
    assert!(q.ready_in(f));
    (view, state, q)
}
#[test]
fn npc77_controller_requires_exact_scope_lives_sequence_and_current_catalog() {
    let (view, _, request) = npc77_view();
    for mutation in 0..10 {
        let mut stale = request;
        let mut v = view.clone();
        match mutation {
            0 => stale.context.key.source_id += 1,
            1 => stale.context.epoch -= 1,
            2 => stale.channel_id = (stale.channel_id + 1) % 4,
            3 => stale.target.scope.match_id += 1,
            4 => stale.target.scope.generation += 1,
            5 => stale.sequence += 1,
            6 => stale.caster_life += 1,
            7 => stale.target_life += 1,
            8 => v.skills = spell::SkillsState::NotRequested,
            _ => {
                let spell::SkillsState::Ready(s) = &mut v.skills else {
                    unreachable!()
                };
                Arc::make_mut(s).context.epoch -= 1;
            }
        }
        let (mut c, release) = controller_with(v);
        assert!(
            c.command(LoginCommand::CastNpcDisintegrate(stale)).is_err(),
            "case{mutation}"
        );
        assert!(!c.snapshot().spell_pending);
        assert!(c
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .spell
            .is_none());
        assert_eq!(c.snapshot().combat, view.combat);
        release.send(()).unwrap();
    }
}
#[test]
fn npc77_singleflight_waits_for_exact_ack_without_predicting_resources_and_cancels() {
    let (mut view, mut state, request) = npc77_view();
    let (mut c, release) = controller_with(view.clone());
    let inbox = c.active.as_ref().unwrap().inbox.clone();
    inbox.lock().unwrap().auto_world = true;
    assert!(c
        .command(LoginCommand::CastNpcDisintegrate(request))
        .is_err());
    inbox.lock().unwrap().auto_world = false;
    {
        let _held = inbox.lock().unwrap();
        assert!(c
            .command(LoginCommand::CastNpcDisintegrate(request))
            .is_err());
    }
    c.command(LoginCommand::CastNpcDisintegrate(request))
        .unwrap();
    let (queued, epoch) = inbox.lock().unwrap().spell.take().unwrap();
    assert_eq!(queued, Some(SpellRequest::NpcDisintegrate(request)));
    assert_eq!(c.snapshot().combat, view.combat);
    assert!(c.snapshot().spell_pending);
    for command in [
        LoginCommand::CastNpcDisintegrate(request),
        LoginCommand::ReadSkills,
        LoginCommand::CastFireStorm,
        LoginCommand::CastTeleport,
        LoginCommand::UseItem { item_id: 40010 },
        LoginCommand::NpcAttack {
            target: request.target,
            life: request.target_life,
        },
    ] {
        assert!(c.command(command).is_err());
    }
    publish_navigation(
        &c.active.as_ref().unwrap().published,
        view.clone(),
        request.context.epoch,
        epoch - 1,
    );
    c.poll();
    assert!(c.snapshot().spell_pending);
    assert_eq!(c.snapshot().combat, view.combat);
    let ack = crate::modern_lito::disintegrate_presentation::npc_tests::reply("normal-hit");
    receive_npc(&mut view, &mut state, ack).unwrap();
    publish_navigation(
        &c.active.as_ref().unwrap().published,
        view.clone(),
        request.context.epoch,
        epoch,
    );
    c.poll();
    assert!(!c.snapshot().spell_pending);
    assert_eq!(c.snapshot().combat, view.combat);
    let entity = view
        .combat
        .as_ref()
        .unwrap()
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(request.context.key))
        .unwrap()
        .entity_id;
    assert_eq!(
        c.snapshot()
            .combat
            .as_ref()
            .unwrap()
            .npc_disintegrate(entity)
            .unwrap()
            .count,
        1
    );
    let mut next = request;
    next.sequence += 1;
    c.command(LoginCommand::CastNpcDisintegrate(next)).unwrap();
    c.command(LoginCommand::Disconnect).unwrap();
    assert!(inbox.lock().unwrap().spell.is_none());
    assert!(c.snapshot().combat.is_none());
    publish_navigation(
        &c.retiring[0].published,
        view,
        request.context.epoch,
        epoch + 1,
    );
    c.poll();
    assert!(c.snapshot().combat.is_none());
    release.send(()).unwrap();
}
