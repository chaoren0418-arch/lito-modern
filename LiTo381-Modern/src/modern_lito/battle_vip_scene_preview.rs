//! Full-VIP shared terrain/player-body preview. This is deliberately not a
//! complete battle renderer: armor, all spell/status and gameplay overlays are
//! not yet qualified together. It never mints PreparedResources or navigation
//! authority, so pixels cannot acknowledge LOAD or enable a command.
use super::{
    battle_follow::Observation,
    battle_hud,
    battle_minimap::Panel,
    battle_payload::data::{self as d, Phase},
    battle_scene::{owner_hud_vitals, Surface},
    battle_space::{Navigation, Space},
    compositor::Canvas,
    effect_catalog::EffectCatalog,
    player_appearance::{Assets, Plan, Playback},
    resources::SharedMapSectorKey,
    shared_action_catalog::SharedActionCatalog,
    vip_status,
    world_map::{SectorBinding, WorldMap},
};
use anyhow::{ensure, Result};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

pub(crate) struct Preview {
    observation: Observation,
    playback: Playback,
    players: Assets,
    hud: Option<battle_hud::Assets>,
    status: Option<vip_status::Assets>,
    navigation: Arc<Navigation>,
    minimap: Option<Arc<Panel>>,
    catalog_sha256: [u8; 32],
    last_render: Option<Instant>,
}

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "VIP preview cancelled");
    Ok(())
}

fn bindings(data: &d::UnvalidatedData) -> Result<Vec<SectorBinding>> {
    data.manifest
        .sectors
        .0
        .iter()
        .map(|sector| {
            Ok(SectorBinding {
                key: SharedMapSectorKey {
                    map_id: data.manifest.map.try_into()?,
                    sector_x: sector.x.try_into()?,
                    sector_y: sector.y.try_into()?,
                },
                byte_length: sector.byte_length.try_into()?,
                sha256: sector.sha256,
            })
        })
        .collect()
}

impl Preview {
    pub(crate) fn load(
        root: &Path,
        observation: Observation,
        effect_catalog_path: Option<&Path>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(
            root.is_absolute(),
            "VIP preview shared root must be absolute"
        );
        let data = observation.data();
        ensure!(
            data.vip.is_some()
                && data.world.descriptor.features & super::battle_payload::vip::FEATURE != 0
                && data.world.phase != Phase::Aborted,
            "VIP preview requires current complete VIP observation"
        );
        let map = Arc::new(WorldMap::load_shared(root, &bindings(data)?, cancel)?);
        Self::load_map(root, observation, map, effect_catalog_path, cancel)
    }

    pub(crate) fn load_map(
        root: &Path,
        observation: Observation,
        map: Arc<WorldMap>,
        effect_catalog_path: Option<&Path>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let data = observation.data();
        ensure!(
            data.vip.is_some() && data.world.phase != Phase::Aborted,
            "retired/non-VIP preview"
        );
        ensure!(
            map.canonical_digest() == data.manifest.digest
                && map.map_id() as i32 == data.manifest.map,
            "VIP preview map identity mismatch"
        );
        let space = Space::qualify(map.clone(), observation.clone(), cancel)?;
        let navigation = space.snapshot(observation.clone(), cancel)?;
        let plan = Plan::from_observation(observation.clone())?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        let players = Assets::load_with_catalog(&catalog, map, &plan, cancel)?;
        let required = vip_status::required(&observation)?;
        // Optional XML can enrich a preview, but missing XML may never turn
        // unpainted VIP effects into a complete LOAD certificate.
        let status = effect_catalog_path
            .map(|path| {
                let effects = EffectCatalog::load(path, cancel)?;
                vip_status::Assets::load(
                    root,
                    effects,
                    &required
                        .iter()
                        .map(|id| i64::from(id.value()))
                        .collect::<Vec<_>>(),
                    cancel,
                )
            })
            .transpose()?;
        #[cfg(test)]
        let hud = battle_hud::Assets::load_fixture_optional(root)?;
        #[cfg(not(test))]
        let hud = Some(battle_hud::Assets::load(root)?);
        let playback = Playback::new(plan, Instant::now())?;
        active(cancel)?;
        Ok(Self {
            observation,
            playback,
            players,
            hud,
            status,
            navigation,
            minimap: None,
            catalog_sha256: catalog.sha256(),
            last_render: None,
        })
    }

    pub(crate) fn requires_reload(&self, next: &Observation) -> Result<bool> {
        self.observation.check_successor(next)?;
        // A preview cannot yet prove replacement spell/armor/status banks.
        // Requalify on every changed authoritative READ rather than reuse art
        // or presentation history across an unknown VIP transition.
        Ok(self.observation.data() != next.data())
    }

    pub(crate) fn observe(&mut self, next: Observation, _: &AtomicBool) -> Result<()> {
        ensure!(
            self.observation.data() == next.data(),
            "changed VIP preview requires owned resource reload"
        );
        self.observation.check_successor(&next)?;
        Ok(())
    }

    pub(crate) fn render(&mut self, now: Instant, cancel: &AtomicBool) -> Result<Surface> {
        active(cancel)?;
        ensure!(
            self.observation.data().world.phase != Phase::Aborted
                && self.last_render.is_none_or(|last| now >= last),
            "retired or reversed VIP preview"
        );
        let mut canvas = Canvas::new(800, 600)?;
        let (playback, geometry, audio) =
            self.players
                .render_playback(&self.playback, now, &mut canvas, cancel)?;
        let minimap = Panel::prepare(&self.navigation, self.minimap.as_deref(), cancel)?;
        minimap.draw(&mut canvas)?;
        if let Some(hud) = &self.hud {
            let v = owner_hud_vitals(self.observation.data())?;
            hud.draw(&mut canvas, v.hp, v.maximum_hp, v.mp, v.maximum_mp)?;
        }
        if let Some(status) = &self.status {
            status.draw(&self.observation, &mut canvas, (224, 8), 16, cancel)?;
        }
        let mut bgra = vec![0; 800 * 600 * 4];
        canvas.copy_bgra(&mut bgra)?;
        active(cancel)?;
        self.playback = playback;
        self.minimap = Some(minimap.clone());
        self.last_render = Some(now);
        Ok(Surface {
            observation: self.observation.clone(),
            width: 800,
            height: 600,
            bgra: Arc::from(bgra),
            geometry,
            objectives: Vec::new(),
            limitations: self.players.limitations().clone(),
            catalog_sha256: self.catalog_sha256,
            // Exact navigation is retained only to validate the minimap's
            // pixels. Runtime route access and all controls remain denied.
            navigation: Some(self.navigation.clone()),
            minimap: Some(minimap),
            audio,
            effects_skipped: true,
        })
    }
}

#[cfg(test)]
#[path = "battle_vip_scene_preview_tests.rs"]
mod tests;
