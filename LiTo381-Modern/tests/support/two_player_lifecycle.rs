//! Live E149 lifecycle assertions; no injected packets, state, damage or clocks.
use super::two_player_battle::{TwoPlayerBattle, ACCOUNTS, ACTOR_IDS};
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    native_combat::HeldAttack,
    native_login::LoginPhase,
    native_world_scene::{Frame, Model},
};
use std::{
    path::Path,
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

fn wait(
    fixture: &mut TwoPlayerBattle,
    label: &str,
    seconds: u64,
    mut ready: impl FnMut(&mut TwoPlayerBattle) -> Result<bool>,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        fixture.poll().with_context(|| label.to_owned())?;
        if ready(fixture).with_context(|| label.to_owned())? {
            return Ok(());
        }
        ensure!(Instant::now() < deadline, "{label} timeout");
        thread::sleep(Duration::from_millis(20));
    }
}

fn frame(
    model: &mut Model,
    fixture: &TwoPlayerBattle,
    peer: usize,
    clock: Instant,
) -> Result<Frame> {
    model.draw_at(
        &fixture.players[peer].snapshot().world,
        &AtomicBool::new(false),
        clock.elapsed().as_nanos() as u64,
    )
}

// Require read-only Java observations to agree with both real client sessions.
fn verified_vitals(fixture: &mut TwoPlayerBattle, label: &str) -> Result<()> {
    let mut peers = Vec::new();
    wait(fixture, "server/client vitals agreement", 5, |f| {
        let trace = std::fs::read_to_string(f.root.join("server-trace.jsonl"))?;
        let complete = trace.rsplit_once('\n').map_or("", |(rows, _)| rows);
        let rows: Vec<serde_json::Value> = complete
            .lines()
            .map(serde_json::from_str)
            .collect::<serde_json::Result<_>>()?;
        peers.clear();
        for index in 0..2 {
            let snapshot = f.players[index].snapshot();
            let hp = snapshot.world.hp.context("client HP")?[0];
            let mp = snapshot.world.mp.context("client MP")?[0];
            let room = snapshot.joined_room.as_ref().context("client room")?.id;
            if let Some(row) = rows.iter().rev().find(|row| {
                let s = &row["state"];
                s["test_account"] == ACCOUNTS[index]
                    && s["object_id"] == ACTOR_IDS[index]
                    && s["phase"] == "BATTLE"
                    && s["observer"] == false
                    && s["room"] == room
                    && s["hp"] == hp
                    && s["mp"] == mp
            }) {
                peers.push(row.clone());
            }
        }
        Ok(peers.len() == 2)
    })?;
    ensure!(
        peers[0]["state"]["session_identity"].is_number()
            && peers[1]["state"]["session_identity"].is_number()
            && peers[0]["state"]["session_identity"] != peers[1]["state"]["session_identity"],
        "missing/distinct GAME identities"
    );
    fixture.save_snapshots(label)?;
    std::fs::write(
        fixture.root.join(format!("{label}-server.json")),
        serde_json::to_vec_pretty(&peers)?,
    )?;
    Ok(())
}

fn cast_bolt(
    fixture: &mut TwoPlayerBattle,
    model: &mut Model,
    clock: Instant,
    label: &str,
) -> Result<()> {
    wait(fixture, "caster animation", 10, |f| {
        Ok(frame(model, f, 1, clock)?.movement_ready(&f.players[1].snapshot().world))
    })?;
    let hp = fixture.players[0]
        .snapshot()
        .world
        .hp
        .context("target HP")?;
    let mp = fixture.players[1]
        .snapshot()
        .world
        .mp
        .context("caster MP")?;
    let casts = fixture.players[1].snapshot().world.own_cast_packets;
    let presented = frame(model, fixture, 1, clock)?;
    fixture.players[1].cast(4, Some(ACTOR_IDS[0]), &presented)?;
    // Queueing C_USE_SPELL must not spend MP or invent damage locally.
    ensure!(
        fixture.players[0].snapshot().world.hp == Some(hp),
        "predicted target damage"
    );
    ensure!(
        fixture.players[1].snapshot().world.mp == Some(mp),
        "predicted MP consumption"
    );
    wait(fixture, "energy bolt HP/MP receipts", 10, |f| {
        let target = &f.players[0].snapshot().world;
        let caster = &f.players[1].snapshot().world;
        Ok(target.hp.is_some_and(|v| v[0] < hp[0] && v[1] == hp[1])
            && caster.mp.is_some_and(|v| v[0] < mp[0] && v[1] == mp[1])
            && caster.own_cast_packets > casts
            && caster.actors[&ACTOR_IDS[0]].hp_ratio < 100)
    })?;
    verified_vitals(fixture, label)
}

fn approach(fixture: &mut TwoPlayerBattle, knight: &mut Model, clock: Instant) -> Result<()> {
    let nav = knight.navigation();
    let w = &fixture.players[0].snapshot().world;
    let own = w.own_actor().context("knight")?;
    let target = &w.actors[&ACTOR_IDS[1]];
    let route = nav.route_to_adjacent_avoiding([own.x, own.y], [target.x, target.y], |x, y| {
        w.actors
            .values()
            .any(|a| a.id != own.id && !a.dead && a.x == x && a.y == y)
    })?;
    ensure!(!route.is_empty(), "fixture must exercise approach movement");
    for heading in route {
        wait(fixture, "approach animation", 8, |f| {
            Ok(frame(knight, f, 0, clock)?.movement_ready(&f.players[0].snapshot().world))
        })?;
        let own = fixture.players[0].snapshot().world.own_actor().unwrap();
        let destination =
            lito_modern::modern_lito::native_world::destination(own.x, own.y, heading)?;
        fixture.players[0].move_checked(heading, &nav)?;
        // Local movement is predicted: require the other real session's receipt too.
        wait(fixture, "peer movement receipt", 8, |f| {
            Ok(f.players[1]
                .snapshot()
                .world
                .actors
                .get(&ACTOR_IDS[0])
                .is_some_and(|a| (a.x, a.y) == destination))
        })?;
    }
    Ok(())
}

pub fn exercise(fixture: &mut TwoPlayerBattle) -> Result<()> {
    fixture.enter_battle()?;
    fixture.save_snapshots("lifecycle-bootstrap")?;
    let epochs = fixture
        .players
        .each_ref()
        .map(|p| p.transport().snapshot().epoch);
    // Scoreboard identity is the selected display name, not the repository's internal name.
    let names = fixture
        .players
        .each_ref()
        .map(|p| p.snapshot().selected_character.clone().unwrap());
    let room = fixture.players[0]
        .snapshot()
        .joined_room
        .as_ref()
        .context("room")?
        .id;
    let cancel = AtomicBool::new(false);
    let assets = Path::new(r"D:\lineage開發\0-客戶端");
    let mut knight = Model::load(assets, &fixture.players[0].snapshot().world, &cancel)?;
    fixture.poll()?;
    let mut mage = Model::load(assets, &fixture.players[1].snapshot().world, &cancel)?;
    let clock = Instant::now();
    approach(fixture, &mut knight, clock)?;
    fixture.save_snapshots("adjacent")?;
    cast_bolt(fixture, &mut mage, clock, "spell-hp-mp")?;

    let hp_before = fixture.players[1].snapshot().world.hp.context("mage HP")?;
    let mp_before = fixture.players[0]
        .snapshot()
        .world
        .mp
        .context("knight MP")?;
    let mut held = HeldAttack::default();
    held.press(
        ACTOR_IDS[1],
        epochs[0],
        &fixture.players[0].snapshot().world,
    );
    let mut attacks = 0;
    let mut hurt = false;
    wait(fixture, "melee damage and authoritative death", 150, |f| {
        let dead = f.players[1]
            .snapshot()
            .world
            .own_actor()
            .is_some_and(|a| a.dead);
        if dead {
            return Ok(f.players[1].snapshot().world.hp.is_some_and(|v| v[0] == 0)
                && f.players[1].snapshot().world.death_confirmation_pending
                && f.players[0].snapshot().world.actors[&ACTOR_IDS[1]].dead);
        }
        if !hurt
            && f.players[1]
                .snapshot()
                .world
                .hp
                .is_some_and(|v| v[0] < hp_before[0])
            && f.players[0].snapshot().world.actors[&ACTOR_IDS[1]].hp_ratio < 100
        {
            f.save_snapshots("melee-hurt")?;
            hurt = true;
        }
        let presented = frame(&mut knight, f, 0, clock)?;
        let before = (
            f.players[0].snapshot().world.hp,
            f.players[0].snapshot().world.mp,
        );
        if f.players[0].continue_attack(&mut held, &presented)? {
            attacks += 1;
            ensure!(attacks <= 150, "attack budget exceeded");
            ensure!(
                before
                    == (
                        f.players[0].snapshot().world.hp,
                        f.players[0].snapshot().world.mp
                    ),
                "predicted melee vitals"
            );
        }
        // Keep the victim's real animation clock progressing before confirming death.
        frame(&mut mage, f, 1, clock)?;
        Ok(false)
    })?;
    ensure!(hurt && attacks > 0, "no live nonlethal melee damage");
    ensure!(
        fixture.players[0].snapshot().world.own_attack_packets > 0,
        "missing melee acknowledgement"
    );
    ensure!(
        fixture.players[0].snapshot().world.mp == Some(mp_before),
        "melee spent MP"
    );
    verified_vitals(fixture, "death")?;
    let tx = fixture.players[1].transport().queued_tx_counter();
    ensure!(
        fixture.players[1].attack(ACTOR_IDS[0]).is_err(),
        "dead player attacked"
    );
    ensure!(
        fixture.players[1].move_step(0).is_err(),
        "dead player moved"
    );
    ensure!(
        fixture.players[1].transport().queued_tx_counter() == tx,
        "dead input sent bytes"
    );
    wait(fixture, "death animation and C_CONFIRM_DEATH", 10, |f| {
        let presented = frame(&mut mage, f, 1, clock)?;
        if !presented.death_ready(&f.players[1].snapshot().world) {
            return Ok(false);
        }
        f.players[1].confirm_death(&presented)?;
        ensure!(
            f.players[1].confirm_death(&presented).is_err(),
            "duplicate death confirmation accepted"
        );
        Ok(true)
    })?;
    wait(fixture, "both settlement scoreboards", 15, |f| {
        Ok(f.players.iter().all(|p| {
            p.snapshot().phase == LoginPhase::BattleResult && p.snapshot().world.scores.len() == 2
        }))
    })?;
    for player in &fixture.players {
        let w = &player.snapshot().world;
        ensure!(w.battle_result == Some(0), "force 0 did not win");
        let winner = w
            .scores
            .iter()
            .find(|s| s.name == names[0])
            .context("winner score")?;
        let loser = w
            .scores
            .iter()
            .find(|s| s.name == names[1])
            .context("loser score")?;
        ensure!(
            (winner.kills, winner.deaths, loser.kills, loser.deaths) == (1, 0, 0, 1),
            "wrong kills/deaths"
        );
    }
    ensure!(
        fixture.players[1].snapshot().world.death_confirmations_sent == 1,
        "death confirmation count"
    );
    ensure!(
        serde_json::to_value(&fixture.players[0].snapshot().world.scores)?
            == serde_json::to_value(&fixture.players[1].snapshot().world.scores)?,
        "different scoreboards"
    );
    fixture.save_snapshots("settlement")?;
    for (index, player) in fixture.players.iter_mut().enumerate() {
        let tx = player.transport().queued_tx_counter();
        ensure!(
            player.attack(ACTOR_IDS[1 - index]).is_err(),
            "attack after settlement"
        );
        ensure!(
            player.transport().queued_tx_counter() == tx,
            "settled attack sent bytes"
        );
        player.return_from_battle()?;
    }
    wait(fixture, "return to lobby", 15, |f| {
        Ok(f.players
            .iter()
            .all(|p| p.snapshot().phase == LoginPhase::LobbyReady))
    })?;
    for player in &fixture.players {
        let s = player.snapshot();
        ensure!(
            s.joined_room.is_none()
                && !s.world.ready()
                && s.world.actors.is_empty()
                && s.world.battle_result.is_none()
                && s.world.scores.is_empty(),
            "stale battle after return"
        );
    }
    fixture.save_snapshots("returned-lobby")?;
    fixture.enter_battle()?;
    for (index, player) in fixture.players.iter().enumerate() {
        let s = player.snapshot();
        let w = &s.world;
        ensure!(
            player.transport().snapshot().epoch == epochs[index],
            "reentry unexpectedly reconnected"
        );
        ensure!(
            s.joined_room.as_ref().context("reentry room")?.id == room,
            "reentered different room"
        );
        ensure!(
            w.hp.is_some_and(|v| v[0] > 0 && v[0] == v[1])
                && w.mp.is_some_and(|v| v[0] > 0 && v[0] == v[1]),
            "reentry vitals not restored"
        );
        ensure!(
            ACTOR_IDS.iter().all(|id| !w.actors[id].dead),
            "stale dead actor"
        );
        ensure!(
            w.battle_result.is_none()
                && w.scores.is_empty()
                && !w.death_confirmation_pending
                && w.death_confirmations_sent == 0
                && w.own_attack_packets == 0
                && w.own_cast_packets == 0,
            "stale lifecycle counters on reentry"
        );
    }
    fixture.save_snapshots("reentered")?;
    // Default spawns are outside bolt range; navigate again using the new world.
    let mut knight = Model::load(assets, &fixture.players[0].snapshot().world, &cancel)?;
    fixture.poll()?;
    let mut mage = Model::load(assets, &fixture.players[1].snapshot().world, &cancel)?;
    let clock = Instant::now();
    approach(fixture, &mut knight, clock)?;
    cast_bolt(fixture, &mut mage, clock, "reentry-spell-hp-mp")?;
    Ok(())
}
