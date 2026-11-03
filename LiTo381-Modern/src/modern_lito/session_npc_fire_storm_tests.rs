// Actual controller mailbox; authority is frozen Java v31 full-frame output.
fn npc70_view() -> (
    LoginSnapshot,
    world::State,
    super::super::npc_fire_storm_protocol::Request,
) {
    let r = super::super::npc_fire_storm_protocol::tests::snapshot("normal", "before");
    let (mut view, state, _) = encounter_view(r.clone());
    view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
        context: r.echo.context,
        status: spell::Status::Ok,
        skills: vec![spell::Skill {
            skill_id: 70,
            name: "npc70".into(),
            mp_cost: 48,
            reuse_millis: 5000,
            support: spell::Support::FireStorm,
        }],
    }));
    let (q, _, _) = super::super::npc_fire_storm_protocol::tests::sample("normal", "hit");
    assert!(q.ready_in(r.frame.as_ref().unwrap()));
    (view, state, q)
}
#[test]
fn npc70_controller_requires_exact_source_epoch_scope_life_sequence_and_atomic_state() {
    let (view, _, request) = npc70_view();
    for mutation in 0..13 {
        let mut stale = request;
        let mut v = view.clone();
        match mutation {
            0 => stale.context.key.source_id += 1,
            1 => stale.context.epoch -= 1,
            2 => stale.channel_id = (stale.channel_id + 1) % 4,
            3 => stale.scope.room_id += 1,
            4 => stale.scope.match_id += 1,
            5 => stale.scope.generation += 1,
            6 => stale.sequence += 1,
            7 => stale.caster_life += 1,
            8 => v.skills = spell::SkillsState::NotRequested,
            9 => {
                let spell::SkillsState::Ready(s) = &mut v.skills else {
                    unreachable!()
                };
                Arc::make_mut(s).context.epoch -= 1;
            }
            10 => v.npc = None,
            11 => Arc::make_mut(v.combat.as_mut().unwrap()).world.revision += 1,
            _ => v.world = world::State::NotRequested,
        }
        let (mut c, release) = controller_with(v);
        assert!(
            c.command(LoginCommand::CastNpcFireStorm(stale)).is_err(),
            "case {mutation}"
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
        release.send(()).unwrap();
    }
}
#[test]
fn npc70_command_polls_new_authority_before_reserving_without_predicting_a_cast() {
    let (view, mut world_state, request) = npc70_view();
    let (mut controller, release) = controller_with(view.clone());
    let mut refreshed = (*view.npc.as_ref().unwrap().as_ref()).clone();
    refreshed.frame.as_mut().unwrap().combat.world.revision += 9;
    let mut authoritative = view.clone();
    receive_npc(&mut authoritative, &mut world_state, refreshed).unwrap();
    assert_ne!(authoritative.combat, view.combat);
    publish_navigation(
        &controller.active.as_ref().unwrap().published,
        authoritative.clone(),
        request.context.epoch,
        0,
    );
    // No explicit controller.poll(): command() owns this initial publication.
    controller
        .command(LoginCommand::CastNpcFireStorm(request))
        .unwrap();
    assert!(controller.snapshot().spell_pending);
    assert_eq!(controller.snapshot().combat, authoritative.combat);
    assert_eq!(controller.snapshot().npc, authoritative.npc);
    assert_ne!(controller.snapshot().combat, view.combat);
    assert_eq!(
        controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .spell
            .as_ref()
            .unwrap()
            .0,
        Some(SpellRequest::NpcFireStorm(request))
    );
    release.send(()).unwrap();
}
#[test]
fn npc70_singleflight_ack_and_disconnect_never_predict_or_replay_damage() {
    let (mut view, mut state, request) = npc70_view();
    let (mut c, release) = controller_with(view.clone());
    let inbox = c.active.as_ref().unwrap().inbox.clone();
    inbox.lock().unwrap().auto_world = true;
    assert!(c.command(LoginCommand::CastNpcFireStorm(request)).is_err());
    inbox.lock().unwrap().auto_world = false;
    {
        let _held = inbox.lock().unwrap();
        assert!(c.command(LoginCommand::CastNpcFireStorm(request)).is_err());
    }
    c.command(LoginCommand::CastNpcFireStorm(request)).unwrap();
    let (queued, epoch) = inbox.lock().unwrap().spell.take().unwrap();
    assert_eq!(queued, Some(SpellRequest::NpcFireStorm(request)));
    assert_eq!(c.snapshot().combat, view.combat);
    assert_eq!(c.snapshot().npc, view.npc);
    assert!(c.snapshot().spell_pending);
    for command in [
        LoginCommand::CastNpcFireStorm(request),
        LoginCommand::ReadSkills,
        LoginCommand::CastFireStorm,
        LoginCommand::CastTeleport,
        LoginCommand::UseItem { item_id: 40010 },
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
    let ack = super::super::npc_fire_storm_protocol::tests::snapshot("normal", "hit");
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
    assert_eq!(c.snapshot().npc, view.npc);
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
            .npc_fire(entity)
            .unwrap()
            .count,
        1
    );
    let mut next = request;
    next.sequence += 1;
    c.command(LoginCommand::CastNpcFireStorm(next)).unwrap();
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
