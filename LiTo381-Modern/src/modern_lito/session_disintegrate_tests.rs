// Included in session::tests to exercise the real private command mailbox.
fn disintegrate_view() -> (LoginSnapshot, world::State, dis::Request) {
    let reply = crate::modern_lito::disintegrate_presentation::tests::reply("normal-active");
    let mut nav = navigation_reply(
        reply.echo.context,
        NavigationAction::EnterChannel,
        NavigationPhase::Lobby,
    );
    nav.snapshot.channel_id = Some(reply.echo.channel_id);
    let mut view = navigation_view(&principal(), nav).unwrap();
    let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
    receive_combat(&mut view, &mut state, &reply).unwrap();
    view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
        context: reply.echo.context,
        status: spell::Status::Ok,
        skills: vec![spell::Skill {
            skill_id: 77,
            name: "dis".into(),
            mp_cost: 70,
            reuse_millis: 5000,
            support: spell::Support::Disintegrate,
        }],
    }));
    let frame = view.combat.as_ref().unwrap();
    let caster = frame
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(reply.echo.context.key))
        .unwrap();
    let target = frame
        .world
        .actors
        .iter()
        .find(|a| a.entity_id != caster.entity_id)
        .unwrap();
    let request = dis::Request {
        context: reply.echo.context,
        channel_id: reply.echo.channel_id,
        match_id: frame.world.match_id,
        generation: frame.world.generation,
        sequence: frame.world.cast(caster.entity_id).unwrap().request_sequence + 1,
        caster_life: 1,
        target_entity_id: target.entity_id,
        target_life: 1,
    };
    assert!(request.ready_in(frame));
    (view, state, request)
}
#[test]
fn disintegrate_controller_requires_exact_scope_lives_sequence_and_current_catalog() {
    let (view, _, request) = disintegrate_view();
    for mutation in 0..10 {
        let mut stale = request;
        let mut v = view.clone();
        match mutation {
            0 => stale.context.key.source_id += 1,
            1 => stale.context.epoch -= 1,
            2 => stale.channel_id = (stale.channel_id + 1) % 4,
            3 => stale.match_id += 1,
            4 => stale.generation += 1,
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
            c.command(LoginCommand::CastDisintegrate(stale)).is_err(),
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
fn disintegrate_command_polls_authority_before_rejection_and_reservation() {
    let (view, mut state, request) = disintegrate_view();
    let (mut controller, release) = controller_with(view.clone());
    let mut refreshed =
        crate::modern_lito::disintegrate_presentation::tests::reply("normal-active");
    let mut authoritative = view.clone();
    let mut stale = request;
    stale.target_life += 1;
    for (command, pending) in [(stale, false), (request, true)] {
        refreshed.frame.as_mut().unwrap().world.revision += 1;
        receive_combat(&mut authoritative, &mut state, &refreshed).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            authoritative.clone(),
            request.context.epoch,
            0,
        );
        // Do not poll manually: even a locally rejected command must retain
        // the newer authority, without predicting that command's HP/MP effect.
        assert_eq!(
            controller
                .command(LoginCommand::CastDisintegrate(command))
                .is_ok(),
            pending
        );
        assert_eq!(controller.snapshot().spell_pending, pending);
        assert_eq!(controller.snapshot().combat, authoritative.combat);
        assert_ne!(controller.snapshot().combat, view.combat);
        let inbox = controller.active.as_ref().unwrap().inbox.lock().unwrap();
        assert_eq!(
            inbox.spell.as_ref().map(|entry| entry.0),
            pending.then_some(Some(SpellRequest::Disintegrate(request)))
        );
    }
    release.send(()).unwrap();
}
#[test]
fn disintegrate_singleflight_waits_for_exact_ack_without_predicting_resources_and_cancels() {
    let (mut view, mut state, request) = disintegrate_view();
    let (mut c, release) = controller_with(view.clone());
    let inbox = c.active.as_ref().unwrap().inbox.clone();
    inbox.lock().unwrap().auto_world = true;
    assert!(c.command(LoginCommand::CastDisintegrate(request)).is_err());
    inbox.lock().unwrap().auto_world = false;
    {
        let _held = inbox.lock().unwrap();
        assert!(c.command(LoginCommand::CastDisintegrate(request)).is_err());
    }
    c.command(LoginCommand::CastDisintegrate(request)).unwrap();
    let (queued, epoch) = inbox.lock().unwrap().spell.take().unwrap();
    assert_eq!(queued, Some(SpellRequest::Disintegrate(request)));
    assert_eq!(c.snapshot().combat, view.combat);
    assert!(c.snapshot().spell_pending);
    for command in [
        LoginCommand::CastDisintegrate(request),
        LoginCommand::ReadSkills,
        LoginCommand::CastFireStorm,
        LoginCommand::CastTeleport,
        LoginCommand::UseItem { item_id: 40010 },
        LoginCommand::CombatAttack {
            target_entity_id: request.target_entity_id,
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
    let ack = crate::modern_lito::disintegrate_presentation::tests::reply("normal-hit");
    receive_combat(&mut view, &mut state, &ack).unwrap();
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
            .disintegrate(entity)
            .unwrap()
            .count,
        1
    );
    let mut next = request;
    next.sequence += 1;
    c.command(LoginCommand::CastDisintegrate(next)).unwrap();
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
