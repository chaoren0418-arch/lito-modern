//! Direct canonical -> shared-resource scene baseline. Never constructs a
//! legacy world/combat frame or changes engine revisions. These pixels are
//! NOT a LOADED permission alone. A sealed resource proof also needs the live
//! runtime attachment, completed pixels and current controller loading state.
//! Loading/painting belongs on an owned worker, never on the UI/network thread.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    compositor::Canvas,
    effect_catalog::EffectCatalog,
    effect_catalog::EffectId,
    npc_assets::{NpcAppearance, NpcAssets},
    pet_animation::{PetAppearance, PetAssets},
    resources::SharedMapSectorKey,
    shared_action_catalog::SharedActionCatalog,
    tower_assets::TowerAssets,
    vip_status,
    world_map::{SectorBinding, WorldCell, WorldMap},
    world_scene::{
        self as scene, ActorState, BaseAction, BaseAppearance, CombatAppearance, SceneLimits,
        SceneSnapshot, WorldSceneAssets,
    },
};
use anyhow::{ensure, Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "battle_scene_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "battle_scene_body_audio_tests.rs"]
pub(crate) mod body_audio_tests;
#[cfg(test)]
#[path = "battle_npc_cadence_tests.rs"]
pub(crate) mod npc_cadence_tests;

#[path = "battle_objective_audio.rs"]
mod objective_audio;

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "canonical scene cancelled");
    Ok(())
}
fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}

pub(crate) fn ordinary_status_effect(buffs: &d::Buffs, speed_supported: bool) -> Option<i64> {
    if speed_supported && buffs.slow_until_nanos > buffs.observed_nanos {
        Some(1)
    } else if buffs.haste_origin == d::HasteOrigin::Haste
        && buffs.haste_until_nanos > buffs.observed_nanos
    {
        Some(0)
    } else {
        None
    }
}

pub(crate) fn owner_hud_vitals(data: &d::UnvalidatedData) -> Result<&d::Vitals> {
    let owner = data.actors.0.iter().find(|a| a.entity == data.personal.entity)
        .context("canonical HUD owner absent")?;
    if data.vip.is_some() {
        Ok(&owner.resources.as_ref()
            .context("VIP HUD owner runtime resources absent")?.vitals)
    } else {
        Ok(&owner.vital.as_ref()
            .context("canonical HUD owner vitals absent")?.values)
    }
}

/// Private painter IDs must never be used as a command target. Keep the real
/// typed canonical target beside each piece of screen geometry instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectiveTarget {
    Honor {
        entity: i64,
        team: i32,
    },
    Quest(d::QuestId),
    TimeAttack(d::WorldId),
    Siege {
        scope: d::WorldId,
        slot: d::SiegeSlot,
    },
}
#[derive(Clone)]
struct Prop {
    target: ObjectiveTarget,
    draw_id: u64,
    cell: WorldCell,
    graphic: u32,
    hp: i64,
    maximum: i64,
}
impl Prop {
    fn action(&self) -> u8 {
        if self.hp == 0 {
            35
        } else if self.hp < self.maximum / 4 {
            34
        } else if self.hp < self.maximum / 2 {
            33
        } else if i128::from(self.hp) < i128::from(self.maximum) * 3 / 4 {
            32
        } else {
            0
        }
    }
}

/// Only visible, typed objectives own clocks. Initial/missed history is held
/// at its terminal pose; an observed state change starts one current sequence.
/// HP changes within the same visual state never restart that sequence.
#[derive(Clone)]
struct ObjectiveClock {
    target: ObjectiveTarget,
    graphic: u32,
    cell: WorldCell,
    action: u8,
    started: Option<Instant>,
}
fn objective_clocks(props: &[Prop], old: &[ObjectiveClock], now: Instant) -> Vec<ObjectiveClock> {
    props
        .iter()
        .map(|p| {
            let action = p.action();
            let previous = old
                .iter()
                .find(|c| c.target == p.target && c.graphic == p.graphic && c.cell == p.cell);
            ObjectiveClock {
                target: p.target.clone(),
                graphic: p.graphic,
                cell: p.cell,
                action,
                started: previous.and_then(|c| {
                    if c.action == action {
                        c.started
                    } else {
                        Some(now)
                    }
                }),
            }
        })
        .collect()
}
impl ObjectiveClock {
    fn elapsed(&self, now: Instant) -> Result<Duration> {
        self.started.map_or(Ok(Duration::MAX), |start| {
            // Windows Instant subtraction may tolerate sub-counter skew;
            // presentation ordering is strict even inside that tolerance.
            ensure!(now >= start, "canonical objective clock reversed");
            now.checked_duration_since(start)
                .context("canonical objective clock reversed")
        })
    }
}

struct Input {
    observation: Observation,
    scene: SceneSnapshot,
    bindings: Vec<SectorBinding>,
    props: Vec<Prop>,
    future_cells: Vec<WorldCell>,
    npc_forms: Vec<NpcAppearance>,
    pet_forms: Vec<PetAppearance>,
}
#[cfg(test)]
pub(crate) fn baseline_counts(observation: Observation) -> Result<(usize, usize, usize, usize)> {
    let input = Input::new(observation)?;
    for actor in input.scene.actors() {
        let data = input.observation.data();
        let canonical = data
            .actors
            .0
            .iter()
            .find(|a| a.entity == actor.entity_id as i64)
            .unwrap();
        let member = data.scene_member(canonical).unwrap();
        assert_eq!(actor.source_id, member.identity.source as u32);
        assert_eq!(actor.object_id, member.identity.id as u64);
        assert_eq!(actor.appearance.class, member.class as u8);
        assert_eq!(actor.appearance.sex, member.sex as u8);
    }
    Ok((
        input.scene.actors().len(),
        input.props.len(),
        input.npc_forms.len(),
        input.pet_forms.len(),
    ))
}
impl Input {
    fn new(observation: Observation) -> Result<Self> {
        let data = observation.data();
        // Full VIP READ retains metadata that the old painter cannot qualify.
        // Do not silently omit VIP icons/shape/armor or issue a partial LOAD proof.
        ensure!(
            data.vip.is_none()
                && data.world.descriptor.features & super::battle_payload::vip::FEATURE == 0,
            "full VIP shared-resource presentation is not implemented"
        );
        // Negotiated data is not proof that shared armor effects are loaded.
        // Only schema9 declares all requirements; load_map must qualify them.
        ensure!(
            data.armor.as_ref().is_none_or(|a| a.presentation.is_some()),
            "armor presentation metadata required before resource qualification"
        );
        ensure!(
            data.world.phase != d::Phase::Aborted,
            "aborted canonical scene must retire"
        );
        let actors = data
            .actors
            .0
            .iter()
            .map(|a| {
                let m = data.scene_member(a)?;
                Ok(ActorState {
                    kind: match m.identity.kind {
                        d::MemberKind::Human => super::participant::MemberKind::Human,
                        d::MemberKind::ServerBot => super::participant::MemberKind::ServerBot,
                    },
                    entity_id: a.entity.try_into()?,
                    source_id: m.identity.source.try_into()?,
                    object_id: m.identity.id.try_into()?,
                    appearance: BaseAppearance {
                        class: m.class.try_into()?,
                        sex: m.sex.try_into()?,
                    },
                    weapon_type: a.vital.as_ref().map_or(0, |v| v.weapon_type).try_into()?,
                    cell: cell(&a.position),
                    heading: a.heading.try_into()?,
                    action: if a.vital.as_ref().is_some_and(|v| v.values.hp == 0) {
                        BaseAction::Death
                    } else {
                        BaseAction::Idle
                    },
                    // First attachment never replays an old attack/hurt/death.
                    action_elapsed: if a.vital.as_ref().is_some_and(|v| v.values.hp == 0) {
                        Duration::MAX
                    } else {
                        Duration::ZERO
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let args = (
            data.world.identity.match_id.try_into()?,
            data.world.revision.try_into()?,
            data.manifest.digest,
            data.personal.entity.try_into()?,
        );
        let mut pet_forms = Vec::new();
        let scene = if let Some(race) = &data.race {
            let runners = race
                .runners
                .0
                .iter()
                .zip(&data.actors.0)
                .map(|(r, a)| {
                    let appearance = PetAppearance::new(r.base_graphics.try_into()?)?;
                    pet_forms.push(appearance);
                    Ok(scene::PetRunner {
                        entity_id: r.entity.try_into()?,
                        appearance,
                        sequence: a.last_move_sequence.try_into()?,
                        step_interval_nanos: a.step_interval_nanos.try_into()?,
                        competing: r.placement == 0 && r.dnf == d::Dnf::None,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            SceneSnapshot::new_pet(
                args.0,
                args.1,
                args.2,
                args.3,
                actors,
                scene::PetScene {
                    generation: data.world.identity.generation.try_into()?,
                    course: race.course_sha256,
                    runners,
                },
            )?
        } else {
            let hidden = data
                .time_attack
                .as_ref()
                .map(|t| {
                    t.departed
                        .0
                        .iter()
                        .map(|id| u64::try_from(*id))
                        .collect::<std::result::Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();
            SceneSnapshot::new_with_hidden(args.0, args.1, args.2, args.3, actors, &hidden)?
        };
        pet_forms.sort();
        pet_forms.dedup();
        let bindings = data
            .manifest
            .sectors
            .0
            .iter()
            .map(|s| {
                Ok(SectorBinding {
                    key: SharedMapSectorKey {
                        map_id: data.manifest.map.try_into()?,
                        sector_x: s.x.try_into()?,
                        sector_y: s.y.try_into()?,
                    },
                    byte_length: s.byte_length.try_into()?,
                    sha256: s.sha256,
                })
            })
            .collect::<Result<_>>()?;
        let mut props = Vec::new();
        let mut future_cells = Vec::new();
        for t in &data.honor.objectives.0 {
            props.push(Prop {
                target: ObjectiveTarget::Honor {
                    entity: t.entity,
                    team: t.team,
                },
                draw_id: t.entity.try_into()?,
                cell: cell(&t.cell),
                graphic: t.graphic.try_into()?,
                hp: t.hp,
                maximum: t.maximum_hp,
            });
        }
        if let Some(q) = &data.quest {
            future_cells.extend(q.targets.0.iter().map(|t| cell(&t.cell)));
            future_cells.extend(q.lives.0.iter().map(|l| cell(&l.spawn)));
            for t in &q.objectives.0 {
                props.push(Prop {
                    target: ObjectiveTarget::Quest(t.id.clone()),
                    draw_id: 100 + u64::try_from(t.id.slot)?,
                    cell: cell(&t.target.cell),
                    graphic: t.target.graphic.try_into()?,
                    hp: t.hp,
                    maximum: t.target.maximum_hp,
                });
            }
        }
        if let Some(t) = &data.time_attack {
            props.push(Prop {
                target: ObjectiveTarget::TimeAttack(data.world.identity.clone()),
                draw_id: 200,
                cell: cell(&t.tower.cell),
                graphic: t.tower.graphic.try_into()?,
                hp: t.tower_hp,
                maximum: t.tower.maximum_hp,
            });
        }
        if let Some(s) = &data.siege {
            future_cells.extend(
                s.gate
                    .0
                    .iter()
                    .chain(&s.attack_camp.0)
                    .chain(&s.defense_camp.0)
                    .chain([&s.portal_inside, &s.portal_outside])
                    .map(cell),
            );
            for t in &s.objectives.0 {
                props.push(Prop {
                    target: ObjectiveTarget::Siege {
                        scope: s.scope.clone(),
                        slot: t.tower.slot,
                    },
                    draw_id: 300 + t.tower.slot as u64,
                    cell: cell(&t.tower.cell),
                    graphic: t.tower.graphic.try_into()?,
                    hp: t.hp,
                    maximum: t.tower.maximum_hp,
                });
            }
        }
        let npc_forms = data
            .npcs
            .as_ref()
            .map(|n| {
                n.graphics
                    .0
                    .iter()
                    .map(|g| Ok(NpcAppearance::new((*g).try_into()?)?))
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            observation,
            scene,
            bindings,
            props,
            future_cells,
            npc_forms,
            pet_forms,
        })
    }
    fn check_map(&self, map: &WorldMap, cancel: &AtomicBool) -> Result<()> {
        active(cancel)?;
        let d = self.observation.data();
        ensure!(
            map.map_id() == u32::try_from(d.manifest.map)?
                && map.canonical_digest() == d.manifest.digest,
            "canonical shared geometry identity mismatch"
        );
        for p in self
            .scene
            .actors()
            .iter()
            .map(|a| a.cell)
            .chain(self.props.iter().map(|p| p.cell))
            .chain(self.future_cells.iter().copied())
        {
            active(cancel)?;
            ensure!(
                map.standing(p),
                "canonical actor/objective outside shared terrain"
            );
        }
        if let Some(n) = &d.npcs {
            for actor in &n.actors.0 {
                active(cancel)?;
                ensure!(
                    map.standing(cell(&actor.cell)),
                    "canonical NPC outside shared terrain"
                );
            }
        }
        // Cheap per-observation geometry. Space performs the static siege
        // cutset/course proof once on the resource worker, not each frame.
        super::battle_teleport::check_map(&self.observation, map)?;
        super::battle_spells::check_map(&self.observation, map)?;
        super::battle_healing::check_map(&self.observation, map)?;
        super::battle_armor_effects::check_map(&self.observation, map)?;
        super::battle_haste::check_map(&self.observation, map)?;
        super::battle_target_haste_visual::check_map(&self.observation, map)?;
        super::battle_slow_visual::check_map(&self.observation, map)?;
        super::battle_items::check_map(&self.observation, map)?;
        super::battle_bows::check_map(&self.observation, map)?;
        Ok(())
    }
}

/// Pixels and all overlay identities retain the exact same immutable canonical
/// observation, including the owner's private equipment/stats snapshot.
pub struct Surface {
    pub observation: Observation,
    pub width: u32,
    pub height: u32,
    pub bgra: Arc<[u8]>,
    pub geometry: scene::RenderInfo,
    pub objectives: Vec<(ObjectiveTarget, scene::ObjectivePlacement)>,
    pub limitations: scene::SceneLimitations,
    pub catalog_sha256: [u8; 32],
    /// Exact observation and worker-qualified mode geometry. This is route
    /// input only, not a LOAD/MOVE ACK. Test renderers may omit this proof.
    pub navigation: Option<Arc<super::battle_space::Navigation>>,
    /// Worker-composited locator and its exact immutable input ownership.
    pub(crate) minimap: Option<Arc<super::battle_minimap::Panel>>,
    /// Qualified effect/body cues. The runtime prepares gain and its owned
    /// output deduplicates; integration into the normal player UI is separate.
    pub audio: Option<super::world_audio::AudioFrame>,
    /// Bounded visual queue overflow is explicit, not silently called complete.
    pub effects_skipped: bool,
}
impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CanonicalSceneSurface[redacted]")
    }
}

/// Owned canonical scene and pinned assets. No filesystem I/O during observe/render.
/// Mode geometry and assets are qualified here; the controller must still bind
/// the exact active attachment before this may participate in a LOADED barrier.
pub struct SharedScene {
    effect_catalog_path: Option<PathBuf>,
    input: Input,
    motion: super::battle_motion::Timeline,
    objectives: Vec<ObjectiveClock>,
    objective_audio: objective_audio::Timeline,
    teleports: super::battle_teleport::Timeline,
    teleport_assets: Option<super::teleport_effect::TeleportAssets>,
    spells: super::battle_spells::Timeline,
    spell_assets: super::battle_spells::Assets,
    healing: super::battle_healing::Timeline,
    healing_assets: super::battle_healing::Assets,
    protection: super::battle_protection::Timeline,
    protection_assets: super::battle_protection::Assets,
    ordinary_status_icons: Option<vip_status::Assets>,
    hud: Option<super::battle_hud::Assets>,
    armor_effects: super::battle_armor_effects::Timeline,
    armor_assets: super::battle_armor_effects::Assets,
    haste: super::battle_haste::Timeline,
    haste_assets: super::battle_haste::Assets,
    target_haste: super::battle_target_haste_visual::Timeline,
    target_haste_assets: super::battle_target_haste_visual::Assets,
    slow: super::battle_slow_visual::Timeline,
    slow_assets: super::battle_slow_visual::Assets,
    items: super::battle_items::Timeline,
    item_assets: super::battle_items::Assets,
    bows: super::battle_bows::Timeline,
    bow_assets: super::battle_bows::Assets,
    audio: super::battle_audio::Mixer,
    body_audio: super::battle_body_audio::Timeline,
    body_sound: Arc<super::combat_audio::SoundBank>,
    last_render: Option<Instant>,
    terminal: Option<Observation>,
    map: Arc<WorldMap>,
    space: Arc<super::battle_space::Space>,
    navigation: Arc<super::battle_space::Navigation>,
    minimap: Option<Arc<super::battle_minimap::Panel>>,
    assets: WorldSceneAssets,
    towers: Option<TowerAssets>,
    npcs: Option<NpcAssets>,
    pets: Option<super::world_runtime::PetPresentation>,
    catalog_sha256: [u8; 32],
    roofs: super::world_roofs::Animation,
    canvas: Canvas,
    bgra: Vec<u8>,
}
/// Only the complete shared loader/renderer can mint this certificate. Public
/// Surface fields, a map-only navigation proof or a fake renderer cannot.
pub(crate) struct PreparedResources {
    observation: Observation,
}
impl PreparedResources {
    pub(crate) fn matches(&self, observation: &Observation) -> bool {
        self.observation.data() == observation.data()
    }
    #[cfg(test)]
    pub(crate) fn synthetic_for_runtime_test(observation: Observation) -> Self {
        Self { observation }
    }
}
impl SharedScene {
    pub(crate) fn prepared_resources(&self) -> Option<PreparedResources> {
        (self.terminal.is_none() && self.last_render.is_some()).then(|| PreparedResources {
            observation: self.input.observation.clone(),
        })
    }
    pub fn load(root: &Path, observation: Observation, cancel: &AtomicBool) -> Result<Self> {
        Self::load_with_effects(root, observation, None, cancel)
    }
    pub fn load_with_effects(
        root: &Path,
        observation: Observation,
        effect_catalog_path: Option<&Path>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(
            root.is_absolute(),
            "canonical resource root must be absolute"
        );
        let input = Input::new(observation)?;
        let map = Arc::new(WorldMap::load_shared(root, &input.bindings, cancel)?);
        Self::load_map(root, input, map, effect_catalog_path, cancel)
    }
    fn load_map(
        root: &Path,
        input: Input,
        map: Arc<WorldMap>,
        effect_catalog_path: Option<&Path>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        input.check_map(&map, cancel)?;
        let space =
            super::battle_space::Space::qualify(map.clone(), input.observation.clone(), cancel)?;
        let navigation = space.snapshot(input.observation.clone(), cancel)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        let effect_catalog = effect_catalog_path
            .map(|path| EffectCatalog::load(path, cancel))
            .transpose()?;
        // These are existing Lineage status pictures. Their visibility comes
        // only from the selected owner's validated server buff snapshot.
        let ordinary_status_icons = effect_catalog
            .as_ref()
            .map(|effects| vip_status::Assets::load(root, effects.clone(), &[0, 1], cancel))
            .transpose()?;
        let hud = if input.observation.data().actors.0.iter().any(|a| a.vital.is_some()) {
            #[cfg(test)]
            { super::battle_hud::Assets::load_fixture_optional(root)? }
            #[cfg(not(test))]
            { Some(super::battle_hud::Assets::load(root)?) }
        } else { None };
        let armor_assets =
            super::battle_armor_effects::Assets::load(&catalog, &input.observation, cancel)?;
        let assets = if !input.pet_forms.is_empty() {
            WorldSceneAssets::load_terrain_with_catalog(
                &catalog,
                map.clone(),
                SceneLimits::default(),
                cancel,
            )?
        } else if input
            .observation
            .data()
            .actors
            .0
            .iter()
            .any(|a| a.vital.is_some())
        {
            let forms = input
                .scene
                .actors()
                .iter()
                .map(|a| CombatAppearance {
                    appearance: a.appearance,
                    weapon_type: a.weapon_type,
                })
                .collect::<Vec<_>>();
            WorldSceneAssets::load_combat_armed_with_catalog(
                &catalog,
                map.clone(),
                &forms,
                SceneLimits::default(),
                cancel,
            )?
        } else {
            let forms = input
                .scene
                .actors()
                .iter()
                .map(|a| a.appearance)
                .collect::<Vec<_>>();
            WorldSceneAssets::load_with_catalog(
                &catalog,
                map.clone(),
                &forms,
                SceneLimits::default(),
                cancel,
            )?
        };
        let body_sound = super::world_audio::load_canonical_body_bank(root, &assets, cancel)?;
        let towers = if !input.props.is_empty() || input.observation.data().quest.is_some() {
            let bank = TowerAssets::load_with_catalog(&catalog, cancel)?;
            for p in &input.props {
                bank.state_frame(p.graphic, p.action(), Duration::MAX)?;
            }
            if let Some(q) = &input.observation.data().quest {
                for t in &q.targets.0 {
                    bank.state_frame(t.graphic.try_into()?, 0, Duration::ZERO)?;
                }
            }
            Some(bank)
        } else {
            None
        };
        let npcs = if input.npc_forms.is_empty() {
            None
        } else {
            Some(NpcAssets::load_with_catalog(
                &catalog,
                &input.npc_forms,
                cancel,
            )?)
        };
        let pets = if input.pet_forms.is_empty() {
            None
        } else {
            Some(PetAssets::load_with_catalog(
                &catalog,
                &input.pet_forms,
                cancel,
            )?)
        };
        active(cancel)?;
        let teleport_assets = if super::battle_teleport::enabled(&input.observation)
            || super::battle_healing::enabled(&input.observation)
            || super::battle_haste::enabled(&input.observation)
            || super::battle_target_haste_visual::enabled(&input.observation)
            || super::battle_slow_visual::enabled(&input.observation)
        {
            let forms = input
                .scene
                .actors()
                .iter()
                .map(|a| a.appearance)
                .collect::<Vec<_>>();
            Some(super::teleport_effect::TeleportAssets::load_with_catalog(
                &catalog, &forms, cancel,
            )?)
        } else {
            None
        };
        let forms = input
            .scene
            .actors()
            .iter()
            .map(|a| a.appearance)
            .collect::<Vec<_>>();
        let spell_assets =
            super::battle_spells::Assets::load(root, &catalog, &input.observation, &forms, cancel)?;
        let item_assets = super::battle_items::Assets::load(&catalog, &input.observation, cancel)?;
        let healing_assets =
            super::battle_healing::Assets::load(&catalog, &input.observation, cancel)?;
        let protection_assets = super::battle_protection::Assets::load_with_catalog(
            root,
            &input.observation,
            effect_catalog.as_ref(),
            cancel,
        )?;
        let haste_assets = super::battle_haste::Assets::load(&catalog, &input.observation, cancel)?;
        let slow_assets =
            super::battle_slow_visual::Assets::load(&catalog, &input.observation, cancel)?;
        let target_haste_assets =
            super::battle_target_haste_visual::Assets::load(&catalog, &input.observation, cancel)?;
        let bow_assets =
            super::battle_bows::Assets::load(&catalog, &input.observation, &assets, cancel)?;
        let now = Instant::now();
        let spells = super::battle_spells::Timeline::new(input.observation.clone(), now);
        let armor_effects =
            super::battle_armor_effects::Timeline::new(input.observation.clone(), now);
        let healing = super::battle_healing::Timeline::new(input.observation.clone(), now);
        let protection =
            super::battle_protection::Timeline::new(input.observation.clone(), now, cancel)?;
        let haste = super::battle_haste::Timeline::new(input.observation.clone(), now);
        let target_haste =
            super::battle_target_haste_visual::Timeline::new(input.observation.clone(), now);
        let slow = super::battle_slow_visual::Timeline::new(input.observation.clone(), now);
        let items = super::battle_items::Timeline::new(input.observation.clone(), now);
        let motion = super::battle_motion::Timeline::new(input.observation.clone(), now)?;
        let body_audio = super::battle_body_audio::Timeline::new(input.observation.clone(), now)
            .advance(
                input.observation.clone(),
                &motion,
                &input.scene,
                &assets,
                npcs.as_ref(),
                now,
                cancel,
            )?;
        let bows = super::battle_bows::Timeline::new(input.observation.clone(), now);
        let teleports = super::battle_teleport::Timeline::new(input.observation.clone(), now);
        let objectives = objective_clocks(&input.props, &[], now);
        let pets = pets
            .map(|bank| {
                let mut presentation =
                    super::world_runtime::PetPresentation::from_scene(bank, &input.scene)?;
                // Establish the actual attachment baseline before a first MOVE can
                // arrive, even if no pixels have yet been requested.
                presentation.update(&input.scene, &map, now, cancel)?;
                Ok::<_, anyhow::Error>(presentation)
            })
            .transpose()?;
        Ok(Self {
            effect_catalog_path: effect_catalog_path.map(Path::to_path_buf),
            input,
            motion,
            objectives,
            objective_audio: Default::default(),
            teleports,
            teleport_assets,
            spells,
            spell_assets,
            healing,
            healing_assets,
            protection,
            protection_assets,
            ordinary_status_icons,
            hud,
            armor_effects,
            armor_assets,
            haste,
            haste_assets,
            target_haste,
            target_haste_assets,
            slow,
            slow_assets,
            items,
            item_assets,
            bows,
            bow_assets,
            audio: Default::default(),
            body_audio,
            body_sound,
            last_render: None,
            terminal: None,
            map,
            space,
            navigation,
            minimap: None,
            assets,
            towers,
            npcs,
            pets,
            catalog_sha256: catalog.sha256(),
            roofs: Default::default(),
            canvas: Canvas::new(800, 600)?,
            bgra: vec![0; 800 * 600 * 4],
        })
    }
    pub fn map(&self) -> &WorldMap {
        &self.map
    }
    pub fn effect_catalog_path(&self) -> Option<&Path> {
        self.effect_catalog_path.as_deref()
    }
    #[cfg(test)]
    pub(crate) fn walk_info(&self, entity: u64) -> Result<scene::AnimationInfo> {
        let actor = self
            .input
            .scene
            .actors()
            .iter()
            .find(|a| a.entity_id == entity)
            .context("actor absent")?;
        self.assets
            .animation_info_armed(
                actor.appearance,
                actor.weapon_type,
                BaseAction::Walk,
                actor.heading,
            )
            .cloned()
            .context("walk bank absent")
    }
    pub fn observation(&self) -> &Observation {
        self.terminal.as_ref().unwrap_or(&self.input.observation)
    }
    /// The replacement is already fully loaded. Validate every fallible part
    /// before moving old clocks; a failed/cancelled handoff leaves both owners
    /// intact. Success retires the old painter, not the authoritative world.
    pub(crate) fn carry_from(
        &mut self,
        old: &mut Self,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        active(cancel)?;
        ensure!(
            self.terminal.is_none() && old.terminal.is_none(),
            "retired scene cannot hand off clocks"
        );
        ensure!(
            old.last_render.is_none_or(|last| now >= last),
            "resource handoff clock reversed"
        );
        old.observation().check_successor(self.observation())?;
        ensure!(
            self.input.pet_forms == old.input.pet_forms
                && self.catalog_sha256 == old.catalog_sha256
                && self.map.canonical_digest() == old.map.canonical_digest()
                && self.spell_assets.compatible(&old.spell_assets)
                && self.healing_assets.compatible(&old.healing_assets)
                && self.protection_assets.compatible(&old.protection_assets)
                && self.armor_assets.compatible(&old.armor_assets)
                && self.haste_assets.compatible(&old.haste_assets)
                && self
                    .target_haste_assets
                    .compatible(&old.target_haste_assets)
                && self.slow_assets.compatible(&old.slow_assets)
                && self.item_assets.compatible(&old.item_assets)
                && self.bow_assets.compatible(&old.bow_assets)
                && self.teleport_assets.is_some() == old.teleport_assets.is_some(),
            "resource handoff identity mismatch"
        );
        let motion = old
            .motion
            .advance(self.input.observation.clone(), now, cancel)?
            .with_base_player_returns(&self.input.scene, &self.assets, now, cancel)?;
        let body_audio = old.body_audio.advance(
            self.input.observation.clone(),
            &motion,
            &self.input.scene,
            &self.assets,
            self.npcs.as_ref(),
            now,
            cancel,
        )?;
        let teleports = old
            .teleports
            .advance(self.input.observation.clone(), now, cancel)?;
        let objectives = objective_clocks(&self.input.props, &old.objectives, now);
        let spells = old.spells.advance(
            self.input.observation.clone(),
            now,
            &self.spell_assets,
            cancel,
        )?;
        let items = old.items.advance(
            self.input.observation.clone(),
            now,
            &self.item_assets,
            cancel,
        )?;
        let healing = old.healing.advance(
            self.input.observation.clone(),
            now,
            &self.healing_assets,
            cancel,
        )?;
        let protection = old
            .protection
            .advance(self.input.observation.clone(), now, cancel)?;
        let armor_effects = old.armor_effects.advance(
            self.input.observation.clone(),
            now,
            &self.armor_assets,
            cancel,
        )?;
        let haste = old.haste.advance(
            self.input.observation.clone(),
            now,
            &self.haste_assets,
            cancel,
        )?;
        let target_haste = old.target_haste.advance(
            self.input.observation.clone(),
            now,
            &self.target_haste_assets,
            cancel,
        )?;
        let slow = old.slow.advance(
            self.input.observation.clone(),
            now,
            &self.slow_assets,
            cancel,
        )?;
        let bows = old.bows.advance(
            self.input.observation.clone(),
            now,
            &self.bow_assets,
            &motion,
            cancel,
        )?;
        active(cancel)?;
        self.motion = motion;
        self.body_audio = body_audio;
        self.teleports = teleports;
        self.objectives = objectives;
        self.objective_audio = old.objective_audio.clone();
        self.armor_effects = armor_effects;
        self.spells = spells;
        self.healing = healing;
        self.protection = protection;
        self.haste = haste;
        self.target_haste = target_haste;
        self.slow = slow;
        self.items = items;
        self.audio = old.audio.clone();
        self.bows = bows;
        self.last_render = old.last_render;
        // Pet forms and catalog are unchanged; retain the proven presenter and
        // its queued cursors. The unused newly loaded pet bank drops with old.
        std::mem::swap(&mut self.pets, &mut old.pets);
        std::mem::swap(&mut self.roofs, &mut old.roofs);
        old.terminal = Some(old.input.observation.clone());
        Ok(())
    }
    /// A worker can replace this owned bank after a legitimate equipment
    /// transition. This is a read-only decision, not permission to bypass
    /// chronology/geometry checks or acknowledge all gameplay resources.
    pub(crate) fn requires_reload(&self, next: &Observation) -> Result<bool> {
        self.observation().check_successor(next)?;
        if next.data().world.phase == d::Phase::Aborted {
            return Ok(false);
        }
        ensure!(self.terminal.is_none(), "canonical scene already retired");
        let next = Input::new(next.clone())?;
        Ok(next.npc_forms != self.input.npc_forms
            || next.pet_forms != self.input.pet_forms
            || self.pets.is_none()
                && next.scene.actors().iter().any(|a| {
                    self.assets
                        .animation_info_armed(
                            a.appearance,
                            a.weapon_type,
                            BaseAction::Idle,
                            a.heading,
                        )
                        .is_none()
                }))
    }
    /// Reuse already loaded art without filesystem I/O. A missing new weapon
    /// family requires an owned reload; never substitute an unarmed stance.
    /// Every check precedes the only state replacement. An authoritative abort
    /// retires this scene immediately, so it cannot publish another surface.
    pub fn observe(&mut self, next: Observation, cancel: &AtomicBool) -> Result<()> {
        self.observe_at(next, Instant::now(), cancel)
    }
    pub(crate) fn observe_at(
        &mut self,
        next: Observation,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        active(cancel)?;
        self.observation().check_successor(&next)?;
        if next.data().world.phase == d::Phase::Aborted {
            self.terminal = Some(next);
            return Ok(());
        }
        ensure!(self.terminal.is_none(), "canonical scene already retired");
        let next = Input::new(next)?;
        next.check_map(&self.map, cancel)?;
        let navigation = self.space.snapshot(next.observation.clone(), cancel)?;
        ensure!(
            next.pet_forms == self.input.pet_forms && next.npc_forms == self.input.npc_forms,
            "canonical preloaded form roster changed"
        );
        if self.pets.is_none() {
            for a in next.scene.actors() {
                ensure!(
                    self.assets
                        .animation_info_armed(
                            a.appearance,
                            a.weapon_type,
                            BaseAction::Idle,
                            a.heading
                        )
                        .is_some(),
                    "canonical equipment stance requires resource reload"
                );
            }
        }
        if !next.props.is_empty() {
            let bank = self
                .towers
                .as_ref()
                .context("canonical objective bank absent")?;
            for p in &next.props {
                bank.state_frame(p.graphic, p.action(), Duration::MAX)?;
            }
        }
        active(cancel)?;
        ensure!(
            self.last_render.is_none_or(|last| now >= last),
            "canonical observed clock reversed"
        );
        let motion = self
            .motion
            .advance(next.observation.clone(), now, cancel)?
            .with_base_player_returns(&next.scene, &self.assets, now, cancel)?;
        let body_audio = self.body_audio.advance(
            next.observation.clone(),
            &motion,
            &next.scene,
            &self.assets,
            self.npcs.as_ref(),
            now,
            cancel,
        )?;
        let teleports = self
            .teleports
            .advance(next.observation.clone(), now, cancel)?;
        #[cfg(test)]
        let trace = |stage| super::battle_teleport::scene_timing_diagnostic(stage, &next.observation, now);
        #[cfg(test)]
        trace("observe-teleports-done");
        let objectives = objective_clocks(&next.props, &self.objectives, now);
        let spells =
            self.spells
                .advance(next.observation.clone(), now, &self.spell_assets, cancel)?;
        #[cfg(test)]
        trace("observe-spells-done");
        let items = self
            .items
            .advance(next.observation.clone(), now, &self.item_assets, cancel)?;
        #[cfg(test)]
        trace("observe-items-done");
        let healing =
            self.healing
                .advance(next.observation.clone(), now, &self.healing_assets, cancel)?;
        #[cfg(test)]
        trace("observe-healing-done");
        let protection = self
            .protection
            .advance(next.observation.clone(), now, cancel)?;
        #[cfg(test)]
        trace("observe-protection-done");
        let armor_effects = self.armor_effects.advance(
            next.observation.clone(),
            now,
            &self.armor_assets,
            cancel,
        )?;
        #[cfg(test)]
        trace("observe-armor-done");
        let haste =
            self.haste
                .advance(next.observation.clone(), now, &self.haste_assets, cancel)?;
        #[cfg(test)]
        trace("observe-haste-done");
        let target_haste = self.target_haste.advance(
            next.observation.clone(),
            now,
            &self.target_haste_assets,
            cancel,
        )?;
        #[cfg(test)]
        trace("observe-target-haste-done");
        let slow = self
            .slow
            .advance(next.observation.clone(), now, &self.slow_assets, cancel)?;
        #[cfg(test)]
        trace("observe-slow-done");
        let bows = self.bows.advance(
            next.observation.clone(),
            now,
            &self.bow_assets,
            &motion,
            cancel,
        )?;
        #[cfg(test)]
        trace("observe-bows-done");
        self.input = next;
        self.armor_effects = armor_effects;
        self.navigation = navigation;
        self.motion = motion;
        self.body_audio = body_audio;
        self.teleports = teleports;
        self.objectives = objectives;
        self.spells = spells;
        self.healing = healing;
        self.protection = protection;
        self.haste = haste;
        self.target_haste = target_haste;
        self.slow = slow;
        self.items = items;
        self.bows = bows;
        Ok(())
    }
    pub fn render(&mut self, now: Instant, cancel: &AtomicBool) -> Result<Surface> {
        active(cancel)?;
        ensure!(self.terminal.is_none(), "canonical scene already retired");
        ensure!(
            self.last_render.is_none_or(|last| now >= last),
            "canonical render clock reversed"
        );
        let input = &self.input;
        let data = input.observation.data();
        let snapshot = self.motion.scene(&input.scene, now, |a| {
            let info = self
                .assets
                .animation_info_armed(a.appearance, a.weapon_type, BaseAction::Walk, a.heading)
                .context("canonical walk cycle not qualified")?;
            Ok((info.total_ticks, info.frames_per_second))
        })?;
        let roofs = self.assets.sample_roofs(&mut self.roofs, &snapshot, now)?;
        let walks = self.motion.walks(&snapshot, &self.assets, now, cancel)?;
        let mut overrides = self
            .teleport_assets
            .as_ref()
            .map(|bank| self.teleports.overrides(&snapshot, bank, now))
            .transpose()?
            .unwrap_or_default();
        let mut spells =
            self.spells
                .advance(input.observation.clone(), now, &self.spell_assets, cancel)?;
        let mut items =
            self.items
                .advance(input.observation.clone(), now, &self.item_assets, cancel)?;
        let mut healing =
            self.healing
                .advance(input.observation.clone(), now, &self.healing_assets, cancel)?;
        let protection = self
            .protection
            .advance(input.observation.clone(), now, cancel)?;
        let mut armor_effects = self.armor_effects.advance(
            input.observation.clone(),
            now,
            &self.armor_assets,
            cancel,
        )?;
        let haste =
            self.haste
                .advance(input.observation.clone(), now, &self.haste_assets, cancel)?;
        let target_haste = self.target_haste.advance(
            input.observation.clone(),
            now,
            &self.target_haste_assets,
            cancel,
        )?;
        let slow = self
            .slow
            .advance(input.observation.clone(), now, &self.slow_assets, cancel)?;
        let mut bows = self.bows.advance(
            input.observation.clone(),
            now,
            &self.bow_assets,
            &self.motion,
            cancel,
        )?;
        overrides.extend(spells.overrides(&snapshot, &self.spell_assets, now)?);
        if let Some(bank) = &self.teleport_assets {
            overrides.extend(healing.overrides(&snapshot, bank, now)?);
            overrides.extend(haste.overrides(&snapshot, bank, now)?);
            overrides.extend(target_haste.overrides(&snapshot, bank, now)?);
            overrides.extend(slow.overrides(&snapshot, bank, now)?);
        }
        let disintegrations = spells.disintegrations(now)?;
        let mut props = Vec::new();
        if let Some(bank) = &self.towers {
            for p in &input.props {
                let elapsed = self
                    .objectives
                    .iter()
                    .find(|c| c.target == p.target)
                    .context("canonical objective clock absent")?
                    .elapsed(now)?;
                let body = bank.state_frame(p.graphic, p.action(), elapsed)?;
                let shadow = bank.shadow_frame(p.graphic, p.action(), elapsed)?;
                props.push(scene::ObjectiveDraw {
                    entity_id: p.draw_id,
                    blocking: p.hp > 0,
                    cell: p.cell,
                    frame: body.frame,
                    shadow: Some(shadow.frame),
                    graphics_id: body.graphics_id,
                    source_frame: body.source_frame,
                    sprite_name: body.sprite_name,
                    subframe: body.subframe,
                });
            }
        }
        let mut npc_draws = Vec::new();
        if let (Some(bank), Some(npcs)) = (&self.npcs, &data.npcs) {
            for n in &npcs.actors.0 {
                let form = NpcAppearance::new(n.graphic.try_into()?)?;
                let pose = self.motion.npc_sample(n, bank, now)?;
                let (action, source) = (pose.action, pose.source);
                npc_draws.push(scene::NpcDraw {
                    id: super::npc_protocol::Id {
                        scope: super::npc_protocol::Scope {
                            room_id: data.world.identity.room.try_into()?,
                            match_id: data.world.identity.match_id.try_into()?,
                            generation: data.world.identity.generation.try_into()?,
                        },
                        serial: n.serial.try_into()?,
                    },
                    actor: scene::NpcBody {
                        serial: n.serial.try_into()?,
                        life: n.life.try_into()?,
                        alive: n.hp > 0,
                        cell: cell(&n.cell),
                        graphic: n.graphic.try_into()?,
                    },
                    source_frame: source,
                    walk_from: pose.walk_from,
                    sample: bank.frame(form, action, n.heading.try_into()?, source)?,
                });
            }
        }
        let mut pets = self.pets.clone();
        let mut pet_audio = None;
        let info = if let Some(pet) = &mut pets {
            // Reuse the qualified body/shadow, fixed-walk camera pulses and
            // bounded remote-idle scheduler. Clone only cursors/Arc assets so
            // failed painting cannot consume unpublished poses or audio.
            let (poses, camera, audio) = pet.update(&snapshot, &self.map, now, cancel)?;
            pet_audio = Some(audio);
            let draws = poses
                .iter()
                .map(|pose| {
                    Ok(scene::PetDraw {
                        entity_id: pose.entity_id,
                        anchor_cell: pose.anchor,
                        sample: pet.assets.frame(
                            pose.appearance,
                            pose.action,
                            pose.heading,
                            pose.source,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            self.assets.render_into_pets_roofs(
                &snapshot,
                &mut self.canvas,
                cancel,
                &draws,
                camera,
                Some(&roofs),
            )?
        } else {
            self.assets.render_into_npcs_roofs_effects_walking(
                &snapshot,
                &mut self.canvas,
                cancel,
                &overrides,
                &props,
                &npc_draws,
                Some(&roofs),
                &disintegrations,
                &walks,
            )?
        };
        active(cancel)?;
        if let Some(bank) = &self.teleport_assets {
            self.teleports
                .draw(&mut self.canvas, info.projection, bank, now, cancel)?;
        }
        spells.draw_projected(
            &mut self.canvas,
            info.projection,
            &self.spell_assets,
            now,
            cancel,
        )?;
        items.draw(
            &mut self.canvas,
            info.projection,
            &self.item_assets,
            now,
            cancel,
        )?;
        healing.draw(
            &mut self.canvas,
            info.projection,
            &self.healing_assets,
            now,
            cancel,
        )?;
        armor_effects.draw(
            &mut self.canvas,
            info.projection,
            &self.armor_assets,
            now,
            cancel,
        )?;
        haste.draw(
            &mut self.canvas,
            info.projection,
            &self.haste_assets,
            now,
            cancel,
        )?;
        target_haste.draw(
            &mut self.canvas,
            info.projection,
            &self.target_haste_assets,
            now,
            cancel,
        )?;
        slow.draw(
            &mut self.canvas,
            info.projection,
            &self.slow_assets,
            now,
            cancel,
        )?;
        bows.draw(
            &mut self.canvas,
            info.projection,
            &self.bow_assets,
            now,
            cancel,
        )?;
        let objectives = info
            .objectives
            .iter()
            .map(|p| {
                let prop = input
                    .props
                    .iter()
                    .find(|t| t.draw_id == p.entity_id)
                    .context("canonical objective geometry identity")?;
                Ok((prop.target.clone(), p.clone()))
            })
            .collect::<Result<Vec<_>>>()?;
        let minimap = super::battle_minimap::Panel::prepare(
            &self.navigation,
            self.minimap.as_deref(),
            cancel,
        )?;
        minimap.draw(&mut self.canvas)?;
        if let Some(hud) = &self.hud {
            let v = owner_hud_vitals(data)?;
            hud.draw(&mut self.canvas, v.hp, v.maximum_hp, v.mp, v.maximum_mp)?;
        }
        protection.draw(
            &mut self.canvas,
            &self.protection_assets,
            now,
            cancel,
            (192, 8),
        )?;
        if let Some(icons) = &self.ordinary_status_icons {
            let owner = input
                .observation
                .data()
                .actors
                .0
                .iter()
                .find(|actor| actor.entity == data.personal.entity);
            let selected = owner
                .and_then(|actor| actor.buffs.as_ref())
                .and_then(|buffs| ordinary_status_effect(buffs, data.speed_state().is_some()));
            if let Some(id) = selected {
                let picture = icons
                    .picture(EffectId::from_wire(id)?)
                    .context("qualified ordinary status picture absent")?;
                self.canvas
                    .blit_rgba(picture.rgba(), picture.width, picture.height, 232, 8)?;
            }
        }
        self.canvas.copy_bgra(&mut self.bgra)?;
        let mut teleports = self.teleports.clone();
        let teleport_audio = self
            .teleport_assets
            .as_ref()
            .map(|bank| teleports.audio(bank, now))
            .transpose()?;
        let mut objective_audio = self.objective_audio.clone();
        let tower_audio = self
            .towers
            .as_ref()
            .map(|bank| objective_audio.sample(&self.objectives, bank, now, cancel))
            .transpose()?;
        let mut audio_mixer = self.audio.clone();
        let (body_audio, body_frame) = self.body_audio.sample(
            &self.motion,
            &self.body_sound,
            self.npcs.as_ref(),
            now,
            cancel,
        )?;
        let audio = audio_mixer.merge(
            [
                spells.audio(&self.spell_assets, now),
                items.audio(&self.item_assets, now),
                healing.audio(&self.healing_assets, now),
                teleport_audio,
                tower_audio,
                pet_audio,
                body_frame,
                armor_effects.audio(&self.armor_assets, now),
            ],
            now,
        )?;
        #[cfg(test)]
        super::battle_teleport::frame_diagnostic("scene-mixed-output", &input.observation, Instant::now(), audio.as_ref());
        active(cancel)?;
        self.last_render = Some(now);
        let effects_skipped = spells.skipped
            || items.skipped
            || bows.skipped
            || healing.skipped
            || haste.skipped
            || target_haste.skipped
            || slow.skipped
            || armor_effects.skipped
            || audio_mixer.skipped;
        self.armor_effects = armor_effects;
        self.spells = spells;
        self.healing = healing;
        self.protection = protection;
        self.haste = haste;
        self.target_haste = target_haste;
        self.slow = slow;
        self.items = items;
        self.audio = audio_mixer;
        self.body_audio = body_audio;
        self.teleports = teleports;
        self.objective_audio = objective_audio;
        self.pets = pets;
        self.bows = bows;
        self.minimap = Some(minimap.clone());
        Ok(Surface {
            observation: input.observation.clone(),
            width: 800,
            height: 600,
            bgra: Arc::from(self.bgra.as_slice()),
            geometry: info,
            objectives,
            limitations: self.assets.limitations().clone(),
            catalog_sha256: self.catalog_sha256,
            navigation: Some(self.navigation.clone()),
            minimap: Some(minimap),
            audio,
            effects_skipped,
        })
    }
}
