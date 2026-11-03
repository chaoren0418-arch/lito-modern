use lito_modern::modern_lito::{native_environment::State,
    native_render_environment::{Bound, Key}, native_world::World};

fn current() -> (World, State) {
    let mut world = World::default(); world.map_id = Some(500); world.generation = 7;
    let mut environment = State::default(); environment.begin(2).unwrap();
    environment.map_changed(2, 500, Some(0)).unwrap();
    environment.accept(2, &[0x21, 0, 0, 0, 0]).unwrap();
    (world, environment)
}

#[test]
fn water_phase_is_a_frame_animation_sample_and_does_not_starve_environment_publication() {
    let (world,mut environment)=current();environment.map_changed(2,500,Some(1)).unwrap();
    let captured=Bound::capture(2,&world,&environment).unwrap();
    let due=environment.water_clock.core.events().iter().find(|e|e.callback==lito_modern::modern_lito::native_water_clock::Callback::Water).unwrap().due;
    environment.water_clock.tick(2,||due).unwrap();
    assert_eq!(environment.water_clock.core.phase(),1);
    assert_eq!(captured.state().water_clock.core.phase(),0);
    assert!(captured.is_current(2,&world,&environment));
    let mut stale=environment.clone();stale.water_clock.retire();
    assert!(Key::capture(2,&world,&stale).is_err());
}

#[test]
fn retired_owners_and_different_maps_cannot_bind_or_reuse_a_render() {
    let (world, mut environment) = current();
    let bound = Bound::capture(2, &world, &environment).unwrap();
    assert!(bound.is_current(2, &world, &environment));
    assert!(Bound::capture(0, &world, &environment).is_err());
    assert!(Bound::capture(4, &world, &environment).is_err());
    let mut changed = World::default(); changed.map_id = Some(501);
    assert!(bound.validate_world(&changed).is_err());
    assert!(Bound::capture(2, &changed, &environment).is_err());
    environment.retire(); assert!(!bound.is_current(2, &world, &environment));
    environment.begin(4).unwrap(); environment.map_changed(4, 500, Some(0)).unwrap();
    assert!(!bound.is_current(4, &world, &environment));
    assert!(Bound::capture(4, &world, &environment).is_ok());
}

#[test]
fn repeated_time_receipts_keep_pixels_current_while_blind_weather_changes_do_not() {
    let (world, mut environment) = current();
    let bound = Bound::capture(2, &world, &environment).unwrap();
    let revision = environment.revision;
    for _ in 0..20 {environment.accept(2, &[0x21, 0, 0, 0, 0]).unwrap();}
    assert!(environment.revision > revision);
    assert!(bound.is_current(2, &world, &environment));
    assert_eq!(bound.state().revision, revision); // Original diagnostic snapshot.
    environment.accept(2, &[0x0a, 2]).unwrap();
    assert!(!bound.is_current(2, &world, &environment));
    let blind = Bound::capture(2, &world, &environment).unwrap();
    let weather = environment.weather;
    environment.accept(2, &[0x33, 0xf1]).unwrap(); // Unknown channel still reduces ambient.
    assert_eq!(environment.weather, weather);
    assert!(!blind.is_current(2, &world, &environment));
}

#[test]
fn portal_water_retention_and_invalid_visual_domains_are_explicit() {
    let (mut world, mut environment) = current();
    environment.map_changed(2, 500, Some(7)).unwrap();
    let water = Bound::capture(2, &world, &environment).unwrap();
    world.map_id = Some(501); environment.map_changed(2, 501, None).unwrap();
    let next = Bound::capture(2, &world, &environment).unwrap();
    assert!(next.key().underwater);
    assert!(!water.is_current(2, &world, &environment));
    for invalid in 0..4 {
        let mut bad = environment.clone();
        match invalid {0 => bad.blind = 3, 1 => bad.weather[2] = 4,
            2 => bad.map_mask_argument = Some(16), _ => bad.map_mask_argument = None}
        assert!(Key::capture(2, &world, &bad).is_err());
    }
}
