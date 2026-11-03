//! Actual selected Java snapshots and shared1562 pixels. This exercises the
//! protection strip independently while the full armor resource gate is closed.
use super::*;
use crate::modern_lito::{
    battle_healing, battle_motion, battle_protection, battle_window, compositor::Canvas,
    immune_to_harm_icon::ImmuneToHarmIcon, shared_action_catalog::SharedActionCatalog,
};
use std::{
    path::PathBuf,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

#[test]
#[ignore = "actual Java armor engine snapshots and shared1562; status/presentation components only, not full armor LOAD or native UI"]
fn java_armor_shared_status_strip_and_nested_healing_keep_full_observations() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    assert!(root.is_absolute());
    let cancel = AtomicBool::new(false);
    let icon = ImmuneToHarmIcon::load(&root, &cancel).unwrap();
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!("{:X}", Sha256::digest(icon.rgba())),
        "1897E3069B08D57D7D6E48D21D94B075D1836EA23C29011D7D6CDFD68575C9DE"
    );
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let mut peer = Peer::start_named(Some("armor-selected"));
    let mut followers = BTreeMap::new();
    let mut timelines = BTreeMap::<String, battle_protection::Timeline>::new();
    let mut previous = BTreeMap::<String, crate::modern_lito::battle_follow::Observation>::new();
    let mut assets = None;
    let at = Instant::now() + Duration::from_secs(60);
    let (mut count, mut drawn, mut cleared, mut heals) = (0, 0, 0, 0);
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:70");
            break;
        }
        if line.starts_with("SESSION:") {
            continue;
        }
        let fields = line.splitn(5, ':').collect::<Vec<_>>();
        assert_eq!(fields[0], "REPLY");
        let label = fields[1];
        let selection = selected(fields[2].parse().unwrap());
        let q = unhex(fields[3]);
        let b = unhex(fields[4]);
        let data = if label == "heal-cast" {
            let intent = command::decode_candidate_intent(&q).unwrap();
            command::read_reply_selected(&mut Cursor::new(&b), &intent, selection)
                .unwrap()
                .data
        } else {
            decode(&b, &q, selection).unwrap()
        };
        let saved = payload::encode(&data).unwrap();
        let group = if label.starts_with("status") {
            label.split('-').take(2).collect::<Vec<_>>().join("-")
        } else if label.starts_with("dead") || label.starts_with("revived") {
            "life".into()
        } else if label.starts_with("bots")
            || label.starts_with("quest")
            || label.starts_with("heal-")
        {
            label.split('-').next().unwrap().to_owned()
        } else {
            "basic".into()
        };
        let key = format!("{group}-{}", data.personal.entity);
        let f = followers
            .entry(key.clone())
            .or_insert_with(|| Follower::new_selected(request(&data), selection).unwrap());
        f.accept(reply(data.clone()))
            .unwrap_or_else(|e| panic!("{label}: {e:?}"));
        let o = f.observation().unwrap().clone();
        let now = at + Duration::from_nanos(data.world.active_elapsed_nanos.max(0) as u64);
        let a = assets
            .get_or_insert_with(|| battle_protection::Assets::load(&root, &o, &cancel).unwrap());
        let t = if let Some(t) = timelines.get(&key) {
            let t = t
                .advance(o.clone(), now, &cancel)
                .unwrap_or_else(|e| panic!("{label}: {e:?}"));
            assert!(t.advance(o.clone(), now, &AtomicBool::new(true)).is_err());
            t
        } else {
            battle_protection::Timeline::new(o.clone(), now, &cancel).unwrap()
        };
        let owner = data.personal.entity;
        let state = data
            .armor
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .find(|a| a.entity == owner)
            .unwrap()
            .state
            .as_ref()
            .unwrap();
        let expected = if data.world.phase == payload::data::Phase::Active
            && state.until_nanos > data.world.active_elapsed_nanos
        {
            Some(Duration::from_nanos(
                (state.until_nanos - data.world.active_elapsed_nanos) as u64,
            ))
        } else {
            None
        };
        assert_eq!(t.remaining(owner, now).unwrap(), expected, "status {label}");
        let mut canvas = Canvas::new(64, 64).unwrap();
        canvas.clear([17, 29, 43]);
        let mut reference = Canvas::new(64, 64).unwrap();
        reference.clear([17, 29, 43]);
        t.draw(&mut canvas, a, now, &cancel, (8, 8)).unwrap();
        if let Some(duration) = expected {
            icon.draw(&mut reference, 8, 8).unwrap();
            drawn += 1;
            // Exact expiry, a duplicate frame and a delayed local repaint
            // cannot extend the icon. Source status is deliberately unchanged.
            let ended = now + duration;
            let expired = t.advance(o.clone(), ended, &cancel).unwrap();
            assert!(expired.remaining(owner, ended).unwrap().is_none());
            let delayed = expired
                .advance(o.clone(), ended + Duration::from_secs(1), &cancel)
                .unwrap();
            assert!(delayed
                .remaining(owner, ended + Duration::from_secs(1))
                .unwrap()
                .is_none());
            let mut hidden = Canvas::new(64, 64).unwrap();
            let baseline = hidden.rgba().to_vec();
            expired
                .draw(&mut hidden, a, ended, &cancel, (8, 8))
                .unwrap();
            assert_eq!(hidden.rgba(), baseline);
            assert_eq!(payload::encode(o.data()).unwrap(), saved);
        } else {
            cleared += 1;
        }
        assert_eq!(canvas.rgba(), reference.rgba(), "pixels {label}");
        // A different source owner cannot donate its status clocks, even when
        // both observers share the same world/public status and local time.
        if let Some(foreign) = previous.values().find(|p| {
            p.data().world.identity == data.world.identity
                && p.data().personal.player != data.personal.player
        }) {
            assert!(t.advance(foreign.clone(), now, &cancel).is_err());
            assert_eq!(t.remaining(owner, now).unwrap(), expected);
        }
        // Borrow the correct schema8 data, not an empty legacy envelope.
        assert!(data.healing.is_none());
        let nested = data.armor.as_ref().unwrap().healing.as_ref();
        assert_eq!(data.healing_actors(), nested);
        let enabled = nested.is_some()
            || data.actors.0.iter().any(|a| {
                a.spells
                    .histories
                    .0
                    .iter()
                    .any(|h| h.lane == payload::data::Lane::Healing)
            });
        assert_eq!(battle_healing::enabled(&o), enabled);
        let expected_out = nested
            .and_then(|h| h.0.iter().find(|a| a.entity == owner))
            .and_then(|a| a.outgoing.as_ref());
        assert_eq!(battle_window::player_healing_outgoing(&o), expected_out);
        for actor in &data.actors.0 {
            let life = battle_motion::life(actor).unwrap();
            let expected_target = nested.is_some()
                && crate::modern_lito::battle_space::occupies(&data, actor.entity)
                && actor.loaded
                && actor.vital.as_ref().is_some_and(|v| v.values.hp > 0);
            assert_eq!(
                battle_window::can_heal(
                    &o,
                    battle_window::Target::Player {
                        entity: actor.entity,
                        life
                    }
                ),
                expected_target
            );
            if let Some(out) = nested
                .and_then(|h| h.0.iter().find(|h| h.entity == actor.entity))
                .and_then(|a| a.outgoing.as_ref())
            {
                assert!(
                    battle_motion::latest_body_event(
                        &data,
                        actor,
                        data.scene_member(actor).unwrap()
                    ) >= out.ledger.last_cast_revision
                );
            }
        }
        if label == "heal-cast" {
            let before = previous.get(&key).unwrap().clone();
            let healing = battle_healing::Assets::load(&catalog, &o, &cancel).unwrap();
            battle_healing::tests::verify_player_edges(before.clone(), o.clone(), &healing, owner);
            let mut timeline = battle_healing::Timeline::new(before, now)
                .advance(o.clone(), now, &healing, &cancel)
                .unwrap();
            let receipt = expected_out.unwrap().receipt.as_ref().unwrap();
            let target = &receipt.impact.target.cell;
            let projection = crate::modern_lito::world_map::ViewProjection {
                anchor_cell: crate::modern_lito::world_map::WorldCell {
                    x: target.x,
                    y: target.y,
                },
                anchor_pixel: crate::modern_lito::world_map::PixelPoint { x: 128, y: 128 },
            };
            let mut pixels = Canvas::new(256, 256).unwrap();
            let blank = pixels.rgba().to_vec();
            timeline
                .draw(&mut pixels, projection, &healing, now, &cancel)
                .unwrap();
            assert_ne!(
                pixels.rgba(),
                blank,
                "nested healing must produce shared744 pixels"
            );
            let audio = timeline.audio(&healing, now).unwrap();
            assert_eq!(audio.events.len(), 1);
            assert_eq!(audio.events[0].group.candidates(), [109]);
            let mut replay = timeline.advance(o.clone(), now, &healing, &cancel).unwrap();
            assert_eq!(
                replay.audio(&healing, now).unwrap().events[0].sequence,
                audio.events[0].sequence
            );
            heals += 1;
        }
        assert!(crate::modern_lito::battle_scene::baseline_counts(o.clone()).is_err());
        assert_eq!(
            payload::encode(o.data()).unwrap(),
            saved,
            "no projection {label}"
        );
        previous.insert(key.clone(), o);
        timelines.insert(key, t);
        count += 1;
        peer.ack(&b);
    }
    assert_eq!(count, 70);
    assert_eq!((drawn, cleared), (29, 41));
    assert_eq!(heals, 1);
    eprintln!("ARMOR_STATUS_SHARED: {count} complete engine observations; {drawn} visible self status frames/{cleared} cleared; shared1562 pixels, exact local expiry/no extension/replay, death/respawn/abort, equipment and source/lane preservation, nested healing/UI/body/receipt; proc11101 and full armor LOAD remain closed");
}
