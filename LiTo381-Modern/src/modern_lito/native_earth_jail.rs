//! T027 #30: frozen S39 origin, original body/clothes/shadow and two Hit callbacks.
use super::*;
use crate::modern_lito::native_actor_layers as layers;
use crate::modern_lito::sprite::{DecodedSprite, SpriteFrame};

// Original body and shadow end with two genuine no-block slots. The clothing
// remains visible there, including terminal source0's dwell. No generic blank allowance.
const SPRITES: &[(&str, usize, &str)] = &[
    (
        "1805-0.spr",
        29959,
        "CDDBEC0CA946D10273A3926788FFE376C06C9476C987831734395DF04925E507",
    ),
    (
        "1806-0.spr",
        11361,
        "44831BE42AC75C74FF60F3A338DC26BF3E4250E3CC9EDF3200066BEFD7D10A5A",
    ),
    (
        "1807-0.spr",
        51321,
        "58D9826A4C3734444A29FA330D70C3E341DF7756BA09994F4FC8069671A86906",
    ),
];
pub(crate) fn is_blank(name: &str, index: usize, f: &SpriteFrame) -> bool {
    matches!(name, "1805-0.spr" | "1806-0.spr")
        && matches!(index, 15 | 16)
        && (
            f.x_offset,
            f.y_offset,
            f.width,
            f.height,
            f.frame_type,
            f.unknown1,
            f.unknown2,
        ) == (192, 120, 0, 0, 0, u16::MAX, 0)
        && f.rgba.is_empty()
        && f.fragments
            .as_ref()
            .is_some_and(|v| v.references.is_empty())
}
fn shape_matches(name: &str, sprite: &DecodedSprite) -> bool {
    SPRITES.iter().any(|(n, _, _)| *n == name)
        && sprite.frames.len() == 17
        && sprite.frames.iter().enumerate().all(|(index, f)| {
            if name != "1807-0.spr" && index >= 15 {
                is_blank(name, index, f)
            } else {
                f.width > 0
                    && f.height > 0
                    && f.rgba.len() == f.width as usize * f.height as usize * 4
                    && f.fragments
                        .as_ref()
                        .is_some_and(|v| !v.references.is_empty())
            }
        })
}
pub(crate) fn source_matches(name: &str, raw: &[u8], sprite: &DecodedSprite) -> bool {
    use sha2::{Digest, Sha256};
    name == "1805-0.spr"
        && raw.len() == SPRITES[0].1
        && format!("{:X}", Sha256::digest(raw)) == SPRITES[0].2
        && shape_matches(name, sprite)
}

pub(super) fn qualify(
    c: &SpriteActionCatalog,
) -> super::super::tower_assets::Result<Vec<(u32, u8, Animation)>> {
    let row = c
        .sprites
        .get(&1805)
        .ok_or(HonorAssetError::Unsupported("earth jail absent"))?;
    let animation = c.resolve_action(1805, ActionKey::new(0))?.animation;
    let ticks = [4, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 5, 4];
    let attrs: &[(u32, &[u8])] = &[(101, b"1806"), (105, b"1 1807"), (109, b"1 -1")];
    if row.graphics_id != 1805
        || row.image_count != Some(1)
        || row.header_issue.is_some()
        || row.actions.len() != 1
        || !attributes_match(row, attrs)
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != ticks.len()
        || animation.frames.iter().enumerate().any(|(i, f)| {
            let events = match i {
                2 => vec![FrameEvent::Sound(718)],
                4 | 7 => vec![FrameEvent::Hit],
                _ => vec![],
            };
            f.image_id != 0
                || f.subframe != if i == 0 { 16 } else { i as u32 - 1 }
                || f.duration_ticks != ticks[i]
                || f.events != events
        })
    {
        return Err(HonorAssetError::Unsupported(
            "earth jail definition changed",
        ));
    }
    for (id, attrs) in [
        (1806, &[(102, b"0".as_slice())][..]),
        (1807, &[(102, b"0".as_slice()), (104, b"8".as_slice())][..]),
    ] {
        let row = c
            .sprites
            .get(&id)
            .ok_or(HonorAssetError::Unsupported("earth jail layer absent"))?;
        if row.graphics_id != id
            || row.image_count != Some(1)
            || row.header_issue.is_some()
            || !row.actions.is_empty()
            || !attributes_match(row, attrs)
        {
            return Err(HonorAssetError::Unsupported("earth jail layer changed"));
        }
    }
    Ok(vec![(1805, 0, animation.clone())])
}

fn attributes_match(
    row: &super::super::sprite_actions::SpriteActions,
    attrs: &[(u32, &[u8])],
) -> bool {
    row.attributes.len() == attrs.len()
        && attrs.iter().all(|(key, body)| {
            row.attributes
                .iter()
                .filter(|a| a.key == ActionKey::new(*key) && a.body_bytes.trim_ascii() == *body)
                .count()
                == 1
        })
}

pub(super) struct Bank {
    pub bank: SharedEffectBank,
    layers: layers::Bank,
    bindings: layers::Bindings,
}
impl Bank {
    pub fn load(
        root: &Path,
        catalog: &SharedActionCatalog,
        cancel: &AtomicBool,
    ) -> Result<(Self, Clip)> {
        let bank = SharedEffectBank::load_reviewed_earth_jail(catalog, cancel, qualify)?;
        let animation = bank.animation(1805, 0)?;
        let clip = Clip::new(Arc::clone(animation))?;
        let bindings = layers::Plan::from_catalog(catalog.catalog(), 1805)?.bind(0)?;
        let mut layers = layers::Bank::open(root)?;
        for frame in &animation.frames {
            let ready = layers.prepare(&bindings, frame.subframe as usize, cancel)?;
            ensure!(
                ready.missing_shadow.is_none() && ready.missing_attachments.is_empty(),
                "earth jail layers incomplete"
            );
        }
        for (name, size, sha) in &SPRITES[1..] {
            let loaded = layers.get(name)?;
            ensure!(
                loaded.source.bytes == *size
                    && loaded.source.sha256 == *sha
                    && shape_matches(name, &loaded.sprite),
                "earth jail original layer/blank slots changed"
            );
        }
        Ok((
            Self {
                bank,
                layers,
                bindings,
            },
            clip,
        ))
    }
    pub fn draw<'s>(
        &'s self,
        a: &Active,
        target: &mut Target<'_, 's>,
        projection: ViewProjection,
    ) -> Result<Value> {
        let body = self.bank.frame(1805, 0, a.frame, a.finished)?;
        let origin = a
            .cue
            .target_origin
            .context("earth jail frozen origin absent")?;
        let position = WorldCell {
            x: origin[0],
            y: origin[1],
        };
        let mut drawn = false;
        let mut visible_layers = Vec::new();
        if !a.finished {
            let mut frames = Vec::new();
            if !body.frame.rgba.is_empty() {
                frames.push((body.frame, Mix::Over));
                visible_layers.push(1805);
            }
            for (name, mix) in &self.bindings.attachments {
                let frame = self.layers.frame(name, body.subframe)?;
                if frame.rgba.is_empty() {
                    continue;
                }
                frames.push((
                    frame,
                    match mix {
                        layers::Mix::Over => Mix::Over,
                        layers::Mix::Screen => Mix::Screen,
                    },
                ));
                visible_layers.push(1807);
            }
            if let Some(name) = &self.bindings.shadow {
                let frame = self.layers.frame(name, body.subframe)?;
                if !frame.rgba.is_empty() {
                    frames.push((frame, Mix::Shadow));
                    visible_layers.push(1806);
                }
            }
            // Preflight all three layers before submitting any fragment.
            target.submit_batch(&frames, position, projection)?;
            drawn = !frames.is_empty();
        }
        Ok(
            json!({"sequence":a.cue.action_sequence,"graphics":1805,"position":origin,
            "source_frame":a.frame,"subframe":body.subframe,"drawn":drawn,"finished":a.finished,"layers":visible_layers,
            "layer_sources":[self.layers.get("1807-0.spr")?.source,self.layers.get("1806-0.spr")?.source]}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::sprite_actions::ActionLimits;
    const DEFINITION: &str = "3 0 0\n#1805 1 earthjail 0.fire(0 17,0.16:4 0.0:3 0.1:3[718 0.2:3 0.3:3! 0.4:3 0.5:3 0.6:4! 0.7:4 0.8:4 0.9:4 0.10:5 0.11:5 0.12:5 0.13:5 0.14:5 0.15:4) 101.shadow(1806) 105.clothes(1 1807) 109.effect(1 -1)\n#1806 1 shadow 102.type(0)\n#1807 1 smoke 102.type(0) 104.attr(8)\n";
    fn catalog(text: &str) -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), ActionLimits::default()).unwrap()
    }
    #[test]
    fn blank_allowance_is_exactly_two_slots_in_original_body_and_shadow() {
        use crate::modern_lito::sprite::SpriteFrameFragments;
        let mut f = SpriteFrame {
            x_offset: 192,
            y_offset: 120,
            width: 0,
            height: 0,
            frame_type: 0,
            unknown1: u16::MAX,
            unknown2: 0,
            rgba: vec![],
            fragments: Some(SpriteFrameFragments {
                references: vec![],
                atlas: Arc::new(vec![]),
            }),
        };
        for name in ["1805-0.spr", "1806-0.spr", "1807-0.spr", "2251-0.spr"] {
            for index in 0..18 {
                assert_eq!(
                    is_blank(name, index, &f),
                    matches!(name, "1805-0.spr" | "1806-0.spr") && matches!(index, 15 | 16)
                );
            }
        }
        f.x_offset = 0;
        assert!(!is_blank("1805-0.spr", 15, &f));
        f.x_offset = 192;
        f.unknown1 = 0;
        assert!(!is_blank("1805-0.spr", 15, &f));
        f.unknown1 = u16::MAX;
        f.rgba.push(0);
        assert!(!is_blank("1805-0.spr", 15, &f));
        f.rgba.clear();
        f.fragments = None;
        assert!(!is_blank("1805-0.spr", 15, &f));
    }
    fn fixture() -> World {
        let mut w = crate::modern_lito::native_login::LoginController::default()
            .snapshot()
            .world
            .clone();
        let data: Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-world-current.json"
        ))
        .unwrap();
        for key in ["world", "start", "status", "actor", "bot"] {
            let hex = data["packets"][key].as_str().unwrap();
            let bytes: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            w.accept(&bytes).unwrap();
        }
        let mut own = w.actors.remove(&0xfedcba98).unwrap();
        own.id = 1;
        w.actors.insert(1, own);
        w.status.as_mut().unwrap().object_id = 1;
        let mut other = w.actors[&2].clone();
        other.id = 3;
        other.incarnation += 1;
        w.actors.insert(3, other);
        let mut packet = vec![
            0x39, 18, 1, 0, 0, 0, 2, 0x80, 0x32, 0x80, 2, 9, 0, 0, 0, 0x0d, 7, 8, 0, 0, 2, 0,
        ];
        packet.extend([2, 0, 0, 0, 0x20, 3, 0, 0, 0, 0x20]);
        w.accept(&packet).unwrap();
        w
    }
    fn effects() -> Effects {
        let mut animation = qualify(&catalog(DEFINITION)).unwrap().remove(0).2;
        // Pure lifecycle test; actual source audio/layers are tested with original resources.
        for f in &mut animation.frames {
            f.events.retain(|e| !matches!(e, FrameEvent::Sound(_)));
        }
        let mut effects = Effects::default();
        effects
            .clips
            .insert(1805, Clip::new(Arc::new(animation)).unwrap());
        effects
    }
    fn event(w: &World) -> EnteredEvent {
        EnteredEvent {
            actor: 1,
            sequence: w.many_attacks[0].action_sequence,
            action: 18,
            frame: 1,
            event: "Hit".into(),
            sound: None,
            prepared_audio: None,
        }
    }
    #[test]
    fn source_contract_rejects_changed_hits_sound_shadow_clothes_and_timing() {
        assert_eq!(qualify(&catalog(DEFINITION)).unwrap()[0].2.frames.len(), 17);
        for text in [
            DEFINITION.replace("[718", "[714"),
            DEFINITION.replace("0.3:3!", "0.3:3"),
            DEFINITION.replace("0.6:4!", "0.6:4"),
            DEFINITION.replace("1806)", "1802)"),
            DEFINITION.replace("1 1807", "1 1808"),
            DEFINITION.replace("attr(8)", "attr(0)"),
            DEFINITION.replace("0.15:4", "0.15:5"),
        ] {
            assert!(qualify(&catalog(&text)).is_err());
        }
    }
    #[test]
    fn history_preserves_frozen_origin_and_callback_progress_without_replay() {
        let mut w = fixture();
        let hit = event(&w);
        let mut fx = effects();
        fx.update(&w, [&hit].into_iter(), 0);
        for step in 1..20 {
            fx.update(&w, std::iter::empty(), step * 10_000_000);
        }
        let h = fx.history();
        let mut restored = effects();
        restored.restore_history(h);
        w.actors.get_mut(&2).unwrap().x += 8;
        for step in 20..200 {
            let expected = fx.update(&w, std::iter::empty(), step * 10_000_000);
            let actual = restored.update(&w, [&hit].into_iter(), step * 10_000_000);
            assert_eq!(
                actual.iter().map(|i| i.target).collect::<Vec<_>>(),
                expected.iter().map(|i| i.target).collect::<Vec<_>>()
            );
            assert_eq!(restored.active[0].frame, fx.active[0].frame);
            assert_eq!(
                restored.active[0].cue.target_origin,
                w.many_attacks[0].target_origin
            );
        }
        assert_eq!(restored.started, 1);
        w.generation += 1;
        w.many_attacks.clear();
        restored.update(&w, std::iter::empty(), 2_000_000_000);
        assert!(restored.active.is_empty());
        assert!(restored.seen.is_empty());
    }
    #[test]
    fn both_hit_callbacks_skip_miss_dead_and_replaced_recipients() {
        for change in 0..3 {
            let mut w = fixture();
            let hit = event(&w);
            match change {
                0 => w.many_attacks[0].targets[1].1 = false,
                1 => w.actors.get_mut(&3).unwrap().dead = true,
                _ => w.actors.get_mut(&3).unwrap().incarnation += 1,
            }
            let before = serde_json::to_value(&w).unwrap();
            let mut fx = effects();
            let mut hits = vec![];
            let mut frames = vec![];
            for step in 0..300 {
                hits.extend(
                    fx.update(&w, [&hit].into_iter(), step * 10_000_000)
                        .into_iter()
                        .map(|i| i.target),
                );
                frames.extend(
                    fx.non_audio
                        .iter()
                        .map(|r| r["source_frame"].as_u64().unwrap()),
                );
            }
            assert_eq!(hits, [2, 2]);
            assert_eq!(frames, [4, 7]);
            assert_eq!(fx.started, 1);
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
    }
}
