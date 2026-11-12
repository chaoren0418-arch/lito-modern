//! T027 batch 1: S39 cloud1804 and fireball171 -> 218 presentation.
//! Coordinates and hit flags come from the received packet; never update HP.
use super::*;
use crate::modern_lito::light_arrow_effect::LightArrowPath;
use std::time::Duration;

pub(super) fn origin(
    attack: &Attack,
    actors: &BTreeMap<u32, super::super::native_world::Actor>,
) -> Option<[i32; 2]> {
    // 3924 (#148): original class1 freezes the first listed target (even a miss)
    // at receipt; an absent first target falls back one heading step, never the second.
    if !matches!(attack.graphics, 1804 | 1805 | 171 | 3924) {
        return None;
    }
    if let Some(actor) = attack.targets.first().and_then(|t| actors.get(&t.id)) {
        return Some([i32::from(actor.x), i32::from(actor.y)]);
    }
    let (dx, dy) = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ][usize::from(attack.heading)];
    Some([
        i32::from(attack.position[0]) + dx,
        i32::from(attack.position[1]) + dy,
    ])
}

pub(super) fn qualify(
    c: &SpriteActionCatalog,
) -> super::super::tower_assets::Result<Vec<(u32, u8, Animation)>> {
    let mut result = Vec::new();
    for gfx in [1804, 171, 218] {
        let (subs, ticks, sound, attrs, directions): (
            Vec<u32>,
            Vec<u32>,
            (usize, u32),
            Vec<(u32, &[u8])>,
            _,
        ) = match gfx {
            1804 => (
                std::iter::once(20).chain(0..20).collect(),
                vec![
                    5, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                ],
                (1, 714),
                vec![(104, b"8"), (109, b"1 -1")],
                Directionality::Undirected,
            ),
            171 => (
                vec![4, 0, 1, 2, 3],
                vec![1; 5],
                (1, 98),
                vec![(104, b"8"), (109, b"5 218")],
                Directionality::EightWay,
            ),
            _ => (
                vec![9, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8],
                vec![1, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                (1, 99),
                vec![(104, b"8")],
                Directionality::Undirected,
            ),
        };
        let row = c
            .sprites
            .get(&gfx)
            .ok_or(HonorAssetError::Unsupported("batch1 area sprite absent"))?;
        let a = c.resolve_action(gfx, ActionKey::new(0))?.animation;
        if row.graphics_id != gfx
            || row.image_count != Some(if gfx == 171 { 8 } else { 1 })
            || row.header_issue.is_some()
            || row.actions.len() != 1
            || row.attributes.len() != attrs.len()
            || attrs.iter().any(|(key, body)| {
                row.attributes
                    .iter()
                    .filter(|attr| {
                        attr.key == ActionKey::new(*key) && attr.body_bytes.trim_ascii() == *body
                    })
                    .count()
                    != 1
            })
            || a.directionality != directions
            || a.frames_per_second != 24
            || a.frames.len() != subs.len()
            || a.frames.iter().enumerate().any(|(i, f)| {
                let mut events = Vec::new();
                if i == sound.0 {
                    events.push(FrameEvent::Sound(sound.1));
                }
                if gfx == 1804 && i == 8 {
                    events.push(FrameEvent::Hit);
                }
                f.image_id != 0
                    || f.subframe != subs[i]
                    || f.duration_ticks != ticks[i]
                    || f.events != events
            })
        {
            return Err(HonorAssetError::Unsupported(
                "batch1 area definition changed",
            ));
        }
        result.push((gfx, 0, a.clone()));
    }
    Ok(result)
}

#[derive(Clone)]
pub(super) struct Flight {
    path: LightArrowPath,
    next_ms: Option<u64>,
    callbacks: u64,
    last_ns: u64,
    pub frame: usize,
    pub position: WorldCell,
    pub arrival: Option<u64>,
    pub recipients: Vec<WorldCell>,
}
impl Flight {
    pub fn launch(c: &Cue, w: &World) -> Result<Self> {
        let saved = c.target_origin.context("area origin absent")?;
        let target = c
            .targets
            .first()
            .and_then(|&(id, _, inc)| w.actors.get(&id).filter(|a| Some(a.incarnation) == inc))
            .map(|a| [i32::from(a.x), i32::from(a.y)])
            .unwrap_or(saved);
        let path = LightArrowPath::new_area_fireball(
            WorldCell {
                x: i32::from(c.attack.position[0]),
                y: i32::from(c.attack.position[1]),
            },
            WorldCell {
                x: target[0],
                y: target[1],
            },
        )?;
        Ok(Self {
            position: path.position(Duration::ZERO),
            path,
            next_ms: Some(0),
            callbacks: 0,
            last_ns: 0,
            frame: 0,
            arrival: None,
            recipients: Vec::new(),
        })
    }
    pub fn heading(&self) -> u8 {
        self.path.heading()
    }
    /// A delayed render dispatch advances one movement callback, not a catch-up loop.
    fn step(&mut self, elapsed: u64) -> Result<(bool, bool)> {
        ensure!(elapsed >= self.last_ns, "fireball clock regressed");
        self.last_ns = elapsed;
        if !self.next_ms.is_some_and(|due| elapsed / 1_000_000 >= due) {
            return Ok((false, false));
        }
        self.position = self
            .path
            .position(Duration::from_millis(self.callbacks * 60));
        self.callbacks += 1;
        self.frame = (self.frame + 1) % 5;
        let arrived = self.position == self.path.target();
        self.next_ms = (!arrived).then_some(elapsed / 1_000_000 + 60);
        Ok((arrived, self.frame == 1))
    }
}

pub(super) fn audio(
    bank: &SharedEffectBank,
    c: &Cue,
    gfx: u16,
    frame: usize,
    sound: u32,
    occurrence: u64,
) -> Result<EnteredEvent> {
    let group = super::super::combat_animation::classify_native_cues(&[FrameEvent::Sound(sound)])?
        .audio
        .context("area audio group")?;
    let payload = super::super::native_audio_delivery::PreparedCue::from_callback_event(
        bank.sound_bank(),
        group,
        0,
        super::super::native_audio_delivery::FiniteEventKey {
            sequence: c.action_sequence,
            graphics: gfx,
            action: 0,
            source_frame: frame,
        },
        occurrence,
    )?;
    Ok(EnteredEvent {
        prepared_audio: Some(payload),
        actor: c.attack.caster,
        sequence: c.action_sequence,
        action: 0,
        frame,
        event: format!("ManySound({sound})"),
        sound: Some(sound),
    })
}

pub(super) fn advance(
    a: &mut Active,
    w: &World,
    bank: &SharedEffectBank,
    now: u64,
    events: &mut Vec<EnteredEvent>,
) -> Result<Vec<Impact>> {
    ensure!(
        w.actors
            .get(&a.cue.attack.caster)
            .is_some_and(|actor| actor.incarnation == a.cue.caster_incarnation),
        "area caster retired"
    );
    let flight = a.flight.as_mut().context("fireball flight absent")?;
    let (arrived, sound) = flight.step(now.saturating_sub(a.start))?;
    let mut impacts = Vec::new();
    if arrived {
        flight.arrival = Some(now);
        for &(id, hit, inc) in &a.cue.targets {
            if let Some(actor) = w
                .actors
                .get(&id)
                .filter(|actor| Some(actor.incarnation) == inc)
            {
                // Each resolved wire recipient, including miss/dead, receives 218.
                flight.recipients.push(WorldCell {
                    x: i32::from(actor.x),
                    y: i32::from(actor.y),
                });
                if hit && !actor.dead && !super::super::native_obstacles::destroyed_guardian(actor)
                {
                    impacts.push(Impact {
                        target: id,
                        sequence: a.cue.action_sequence,
                        projectile: a.cue.attack.sequence,
                    });
                }
            }
        }
    }
    if let Some(arrival) = flight.arrival {
        let sample = a
            .playback
            .advance((now / 1_000_000).saturating_sub(arrival / 1_000_000) * 1_000_000)?;
        a.frame = sample.source_frame;
        a.finished = sample.finished || flight.recipients.is_empty();
        for cue in sample.crossed_cues {
            ensure!(
                cue.source_frame == 1 && cue.event == FrameEvent::Sound(99),
                "fireball impact cue changed"
            );
            for i in 0..flight.recipients.len() {
                events.push(audio(bank, &a.cue, 218, 1, 99, i as u64 + 1)?);
            }
        }
    }
    // Native class5 starts the secondary before dispatching the flight sound.
    if sound {
        events.push(audio(bank, &a.cue, 171, 1, 98, flight.callbacks)?);
    }
    Ok(impacts)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn flight(from: WorldCell, to: WorldCell) -> Flight {
        let path = LightArrowPath::new_area_fireball(from, to).unwrap();
        Flight {
            position: path.position(Duration::ZERO),
            path,
            next_ms: Some(0),
            callbacks: 0,
            last_ns: 0,
            frame: 0,
            arrival: None,
            recipients: Vec::new(),
        }
    }
    #[test]
    fn fireball_late_callback_does_not_skip_cells_and_cycle_sound_reenters() {
        let mut f = flight(WorldCell { x: 100, y: 100 }, WorldCell { x: 110, y: 100 });
        assert_eq!(f.step(0).unwrap(), (false, true));
        assert_eq!(f.position.x, 101);
        assert_eq!(f.step(59_000_000).unwrap(), (false, false));
        assert_eq!(f.callbacks, 1);
        assert_eq!(f.step(9_000_000_000).unwrap(), (false, false));
        assert_eq!(f.position.x, 102);
        assert_eq!(f.next_ms, Some(9060));
        assert_eq!(f.frame, 2);
        for (ms, frame, sound) in [
            (9060, 3, false),
            (9120, 4, false),
            (9180, 0, false),
            (9240, 1, true),
        ] {
            assert_eq!(f.step(ms * 1_000_000).unwrap(), (false, sound));
            assert_eq!(f.frame, frame);
        }
        assert!(f.step(1).is_err());
        let copy = f.clone();
        assert_eq!(copy.next_ms, f.next_ms);
        assert_eq!(copy.callbacks, f.callbacks);
    }
    #[test]
    fn fireball_arrival_is_once_and_fallback_preserves_unsigned_edges() {
        for (from, to) in [
            (WorldCell { x: 0, y: 0 }, WorldCell { x: -1, y: 0 }),
            (
                WorldCell { x: 65535, y: 65535 },
                WorldCell { x: 65536, y: 65535 },
            ),
        ] {
            let mut f = flight(from, to);
            assert_eq!(f.step(0).unwrap(), (true, true));
            assert_eq!(f.position, to);
            assert_eq!(f.step(100_000_000).unwrap(), (false, false));
            assert_eq!(f.callbacks, 1);
        }
        assert!(LightArrowPath::new(WorldCell { x: 0, y: 0 }, WorldCell { x: -1, y: 0 }).is_err());
    }
    #[test]
    fn cloud_origin_freezes_first_wire_target_even_for_miss() {
        let attack = Attack {
            action: 18,
            caster: 1,
            position: [100, 100],
            heading: 7,
            sequence: 1,
            graphics: 1804,
            targets: vec![
                TargetHit { id: 2, hit: false },
                TargetHit { id: 3, hit: true },
            ],
        };
        assert_eq!(origin(&attack, &BTreeMap::new()), Some([99, 99]));
        let mut unsupported = attack;
        unsupported.graphics = 1808;
        assert_eq!(origin(&unsupported, &BTreeMap::new()), None);
    }
}
