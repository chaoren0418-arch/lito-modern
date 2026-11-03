//! Owned same-EXE world surface. Painting consumes an already-rendered buffer;
//! this window never reads resources, owns TLS, or predicts an actor position.
use super::compositor::Viewport;
use super::world_map::{PixelPoint, ViewProjection, WorldCell};
use super::{combat_protocol, world_scene::ActorPlacement};
use anyhow::{ensure, Result};
#[path = "world_inventory_controls.rs"]
mod inventory_controls;
#[path = "world_inventory_panel.rs"]
mod inventory_panel;
#[path = "world_hotbar.rs"]
mod hotbar;
#[path = "world_skill_controls.rs"]
mod skill_controls;
#[path = "world_skill_panel.rs"]
mod skill_panel;
#[path = "world_character_panel.rs"]
mod character_panel;
#[path = "world_status_text.rs"]
mod status_text;
use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
        Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, DeleteDC,
            DeleteObject, DrawTextW, EndPaint, FillRect, GetStockObject, GetTextExtentPoint32W,
            InvalidateRect, ScreenToClient, SelectObject, SetBkMode, SetTextColor, StretchDIBits,
            TextOutW, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACK_BRUSH, CLEARTYPE_QUALITY,
            CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DIB_RGB_COLORS, DT_CENTER, DT_END_ELLIPSIS,
            DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC, HFONT, OUT_DEFAULT_PRECIS,
            HGDIOBJ, PAINTSTRUCT, SRCCOPY, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{
                EM_GETFIRSTVISIBLELINE, EM_GETSEL, EM_LINESCROLL, EM_SETLIMITTEXT, EM_SETSEL,
                WM_MOUSELEAVE,
            },
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            Input::KeyboardAndMouse::{
                EnableWindow, IsWindowEnabled, SetFocus, TrackMouseEvent, TME_LEAVE,
                TRACKMOUSEEVENT,
            },
            WindowsAndMessaging::*,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Canonical(super::battle_window::Intent),
    CanonicalRevival(super::battle_revival::Request),
    CanonicalEquipment(super::battle_equipment::Request),
    CanonicalInventory(super::battle_window::Intent),
    CanonicalSkill(super::battle_skillbook::Choice),
    Step(u8),
    Navigate {
        surface: HitSurface,
        goal: WorldCell,
    },
    CancelRoute,
    Teleport,
    FireStorm,
    GreaterHaste,
    UseItem(i64),
    SelectAmmo(i64),
    Attack {
        surface: HitSurface,
        target_entity_id: u64,
    },
    AttackNpc {
        surface: HitSurface,
        target: super::npc_protocol::Id,
        life: u64,
    },
    AttackQuestObjective {
        surface: HitSurface,
        target: super::quest_protocol::Id,
    },
    AttackTimeAttackTower {
        surface: HitSurface,
        target: super::time_attack_protocol::TowerId,
    },
    AttackSiegeTower {
        surface: HitSurface,
        target: super::siege_protocol::Target,
    },
    UseSiegePortal {
        surface: HitSurface,
        scope: super::siege_protocol::Scope,
    },
    ConfirmDeath(super::quest_life::Confirmation),
    CastLightArrow {
        surface: HitSurface,
        target_entity_id: u64,
    },
    CastDisintegrate {
        surface: HitSurface,
        target_entity_id: u64,
        caster_life: u64,
        target_life: u64,
    },
    CastNpcDisintegrate {
        surface: HitSurface,
        target: super::npc_protocol::Id,
        caster_life: u64,
        target_life: u64,
    },
    CastNpcLightArrow {
        surface: HitSurface,
        target: super::npc_protocol::Id,
        life: u64,
    },
    CastBasicHeal {
        surface: HitSurface,
        target_entity_id: u64,
    },
    Leave,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitSurface {
    pub match_id: u64,
    pub generation: u64,
    pub revision: i64,
    pub selected_entity_id: u64,
    pub projection: ViewProjection,
}
#[derive(Clone, Copy)]
struct PaintedSurface {
    surface: HitSurface,
    viewport: Viewport,
}

#[derive(Clone)]
pub struct CombatSurface {
    pub frame: Arc<combat_protocol::Frame>,
    /// Same draw order and pixel rectangles as the supplied immutable image.
    pub actors: Arc<[ActorPlacement]>,
    pub objectives: Arc<[super::world_scene::ObjectivePlacement]>,
    pub quests: Arc<[super::quest_presentation::Placement]>,
    pub time_attack_tower: Option<super::time_attack_presentation::Placement>,
    pub siege_towers: Arc<[super::siege_presentation::Placement]>,
    pub npc: Option<NpcSurface>,
}
#[derive(Clone)]
pub struct NpcSurface {
    pub reply: Arc<super::npc_protocol::Reply>,
    pub placements: Arc<[super::world_scene::NpcPlacement]>,
}

/// Non-combat pixels retain their own roster. This is not a fallback for a
/// combat world whose authority is missing or stale.
#[derive(Clone)]
pub struct PlayerSurface {
    pub frame: Arc<super::world_protocol::Frame>,
    pub actors: Arc<[ActorPlacement]>,
}

pub(crate) fn validate_player_surface(hit: HitSurface, world: &PlayerSurface) -> Result<()> {
    let frame = &world.frame;
    ensure!(
        !frame.combat_enabled(false)
            && (frame.match_id, frame.generation, frame.revision)
                == (hit.match_id, hit.generation, hit.revision),
        "non-combat roster/pixel revision mismatch"
    );
    let selected = frame
        .actor(hit.selected_entity_id)
        .ok_or_else(|| anyhow::anyhow!("selected actor missing from painted roster"))?;
    frame.validate(selected.key)?;
    let mut ids = std::collections::BTreeSet::new();
    ensure!(
        world.actors.len()
            == frame
                .actors
                .iter()
                .filter(|a| frame.actor_occupies(a.entity_id))
                .count()
            && world.actors.iter().all(|p| {
                ids.insert(p.entity_id)
                    && frame.actor_occupies(p.entity_id)
                    && p.width > 0
                    && p.width <= 4096
                    && p.height > 0
                    && p.height <= 4096
                    && frame
                        .actor(p.entity_id)
                        .is_some_and(|a| a.position == p.cell)
            }),
        "invalid non-combat actor geometry"
    );
    Ok(())
}

fn combat_hit_action(
    painted: PaintedSurface,
    combat: Option<&CombatSurface>,
    point: PixelPoint,
) -> Option<Action> {
    combat_hit(painted, combat, point).map(|hit| hit.action)
}

struct CombatHit {
    action: Action,
    hover: Option<HoverTarget>,
}
enum HoverTarget {
    Player(u64),
    Npc(super::npc_protocol::Id),
}

fn combat_hit(
    painted: PaintedSurface,
    combat: Option<&CombatSurface>,
    point: PixelPoint,
) -> Option<CombatHit> {
    let ground = hit_action(painted, point)?;
    let Some(combat) = combat else {
        return Some(CombatHit {
            action: ground,
            hover: None,
        });
    };
    if combat.frame.world.match_id != painted.surface.match_id
        || combat.frame.world.generation != painted.surface.generation
        || combat.frame.world.revision != painted.surface.revision
    {
        return None;
    }
    let view = painted.viewport;
    let x = (i64::from(point.x) - i64::from(view.x)) * i64::from(WIDTH) / i64::from(view.width);
    let y = (i64::from(point.y) - i64::from(view.y)) * i64::from(HEIGHT) / i64::from(view.height);
    let is_npc = combat
        .frame
        .world
        .descriptor
        .is_some_and(|d| d.has(super::world_protocol::MatchDescriptor::NPC));
    if is_npc != combat.npc.is_some() {
        return None;
    }
    let mut placements = Vec::new();
    for actor in combat.actors.iter().chain(combat.objectives.iter()) {
        if actor
            .visibility
            .as_ref()
            .is_some_and(|mask| !mask.contains(x, y))
        {
            continue;
        }
        let action = if actor.entity_id == painted.surface.selected_entity_id
            && super::siege_presentation::portal_ready(&combat.frame, actor.entity_id)
        {
            Action::UseSiegePortal {
                surface: painted.surface,
                scope: super::siege_protocol::Scope::of(&combat.frame.world),
            }
        } else if is_npc
            || actor.entity_id == painted.surface.selected_entity_id
            || super::honor_protocol::target_hp(&combat.frame, actor.entity_id)
                .is_none_or(|hp| hp == 0)
        {
            Action::CancelRoute
        } else {
            Action::Attack {
                surface: painted.surface,
                target_entity_id: actor.entity_id,
            }
        };
        placements.push((
            (actor.anchor.y, actor.anchor.x, actor.entity_id, 0),
            actor.top_left,
            actor.width,
            actor.height,
            action,
            combat
                .frame
                .world
                .actor(actor.entity_id)
                .map(|a| HoverTarget::Player(a.entity_id)),
        ));
    }
    if let Some(npc) = &combat.npc {
        let frame = npc.reply.frame.as_ref()?;
        if npc.reply.validate().is_err()
            || frame.combat != *combat.frame
            || frame.combat.world.match_id != painted.surface.match_id
            || frame.combat.world.generation != painted.surface.generation
            || frame.combat.world.revision != painted.surface.revision
        {
            return None;
        }
        for placement in npc.placements.iter() {
            let actor = frame.actor(placement.id)?;
            if actor.life != placement.life
                || actor.cell != placement.cell
                || actor.graphic != placement.graphics_id
                || actor.alive() != placement.alive
            {
                return None;
            }
            if placement
                .visibility
                .as_ref()
                .is_some_and(|mask| !mask.contains(x, y))
            {
                continue;
            }
            let action = if actor.alive() {
                Action::AttackNpc {
                    surface: painted.surface,
                    target: placement.id,
                    life: placement.life,
                }
            } else {
                Action::CancelRoute
            };
            // Match the compositor's namespace tie-breaker, not two overlaid hit lists.
            placements.push((
                (
                    placement.anchor.y,
                    placement.anchor.x,
                    placement.id.serial,
                    1,
                ),
                placement.top_left,
                placement.width,
                placement.height,
                action,
                Some(HoverTarget::Npc(placement.id)),
            ));
        }
    }
    if combat.quests.len()
        != combat
            .frame
            .world
            .quest
            .as_ref()
            .map_or(0, |q| q.actors.len())
    {
        return None;
    }
    for p in combat.quests.iter() {
        if !super::quest_presentation::valid_placement(&combat.frame.world, p) {
            return None;
        }
        // Ruins remain visible but don't intercept a walk or a living target.
        if !p.alive {
            continue;
        }
        let g = &p.geometry;
        if g.visibility
            .as_ref()
            .is_some_and(|mask| !mask.contains(x, y))
        {
            continue;
        }
        placements.push((
            (g.anchor.y, g.anchor.x, g.entity_id, 0),
            g.top_left,
            g.width,
            g.height,
            Action::AttackQuestObjective {
                surface: painted.surface,
                target: p.id,
            },
            None,
        ));
    }
    if combat.time_attack_tower.is_some() != combat.frame.world.time_attack.is_some() {
        return None;
    }
    if let Some(p) = &combat.time_attack_tower {
        if !super::time_attack_presentation::valid_placement(&combat.frame.world, p) {
            return None;
        }
        if p.alive
            && p.geometry
                .visibility
                .as_ref()
                .is_none_or(|mask| mask.contains(x, y))
        {
            let g = &p.geometry;
            placements.push((
                (g.anchor.y, g.anchor.x, g.entity_id, 0),
                g.top_left,
                g.width,
                g.height,
                Action::AttackTimeAttackTower {
                    surface: painted.surface,
                    target: p.id,
                },
                None,
            ));
        }
    }
    if !super::siege_presentation::valid_placements(&combat.frame.world, &combat.siege_towers) {
        return None;
    }
    for p in combat.siege_towers.iter() {
        let g = &p.geometry;
        if g.visibility
            .as_ref()
            .is_some_and(|mask| !mask.contains(x, y))
        {
            continue;
        }
        let action = if combat.frame.world.siege.as_ref().is_some_and(|d| {
            combat
                .frame
                .world
                .member(painted.surface.selected_entity_id)
                .is_some_and(|m| d.attackable(m.team, p.target.slot))
        }) {
            Action::AttackSiegeTower {
                surface: painted.surface,
                target: p.target,
            }
        } else {
            Action::CancelRoute
        };
        placements.push((
            (g.anchor.y, g.anchor.x, g.entity_id, 0),
            g.top_left,
            g.width,
            g.height,
            action,
            None,
        ));
    }
    placements.sort_by_key(|p| p.0);
    for (_, position, width, height, action, hover) in placements.into_iter().rev() {
        let left = i64::from(position.x);
        let top = i64::from(position.y);
        if x >= left && y >= top && x < left + i64::from(width) && y < top + i64::from(height) {
            return Some(CombatHit { action, hover });
        }
    }
    Some(CombatHit {
        action: ground,
        hover: None,
    })
}

/// Resolve names from the same immutable pixels, scope and frontmost visible
/// target as clicking. In particular, self/dead players can have a name without
/// gaining an attack action; an occluding NPC/tower never exposes the player below.
#[cfg(test)]
fn hovered_player_name(
    painted: PaintedSurface,
    combat: &CombatSurface,
    point: PixelPoint,
) -> Option<(String, PixelPoint)> {
    let HoverTarget::Player(entity) = combat_hit(painted, Some(combat), point)?.hover? else {
        return None;
    };
    let placement = combat.actors.iter().find(|p| p.entity_id == entity)?;
    player_name(&combat.frame.world, placement)
}

fn hovered_combat_name(
    painted: PaintedSurface,
    combat: &CombatSurface,
    point: PixelPoint,
) -> Option<(String, PixelPoint)> {
    let id = match combat_hit(painted, Some(combat), point)?.hover? {
        HoverTarget::Player(id) => {
            return player_name(
                &combat.frame.world,
                combat.actors.iter().find(|p| p.entity_id == id)?,
            )
        }
        HoverTarget::Npc(id) => id,
    };
    let npc = combat.npc.as_ref()?;
    let frame = npc.reply.frame.as_ref()?;
    let name = frame.name(id)?;
    let placement = npc.placements.iter().find(|p| p.id == id)?;
    Some((
        name.name.clone(),
        PixelPoint {
            x: placement.anchor.x,
            y: placement.top_left.y,
        },
    ))
}

fn player_name(
    frame: &super::world_protocol::Frame,
    placement: &ActorPlacement,
) -> Option<(String, PixelPoint)> {
    let actor = frame.actor(placement.entity_id)?;
    let member = frame.member(placement.entity_id)?;
    if actor.identity() != member.identity() || placement.cell != actor.position {
        return None;
    }
    Some((
        format!("{}({})", member.name, member.source_name),
        PixelPoint {
            x: placement.anchor.x,
            y: placement.top_left.y,
        },
    ))
}

fn hovered_noncombat_name(
    painted: PaintedSurface,
    world: &PlayerSurface,
    point: PixelPoint,
) -> Option<(String, PixelPoint)> {
    validate_player_surface(painted.surface, world).ok()?;
    hit_action(painted, point)?;
    let view = painted.viewport;
    let x = (i64::from(point.x) - i64::from(view.x)) * i64::from(WIDTH) / i64::from(view.width);
    let y = (i64::from(point.y) - i64::from(view.y)) * i64::from(HEIGHT) / i64::from(view.height);
    let placement = world
        .actors
        .iter()
        .filter(|p| {
            x >= i64::from(p.top_left.x)
                && x < i64::from(p.top_left.x) + i64::from(p.width)
                && y >= i64::from(p.top_left.y)
                && y < i64::from(p.top_left.y) + i64::from(p.height)
                && p.visibility.as_ref().is_none_or(|mask| mask.contains(x, y))
        })
        .max_by_key(|p| (p.anchor.y, p.anchor.x, p.entity_id))?;
    player_name(&world.frame, placement)
}

/// Client pixels only: desktop origin must never enter this projection. Keep
/// long names inside the world viewport, including letterboxing and high DPI.
fn nameplate_rect(view: Viewport, anchor: PixelPoint, text: SIZE, dpi: u32) -> Option<RECT> {
    if view.width <= 0 || view.height <= 0 || text.cx <= 0 || text.cy <= 0 {
        return None;
    }
    let width = (i64::from(text.cx) + i64::from(scale(8, dpi))).min(i64::from(view.width));
    let height = (i64::from(text.cy) + i64::from(scale(4, dpi))).min(i64::from(view.height));
    let left = (i64::from(view.x) + i64::from(anchor.x) * i64::from(view.width) / i64::from(WIDTH)
        - width / 2)
        .clamp(
            i64::from(view.x),
            i64::from(view.x) + i64::from(view.width) - width,
        );
    let top = (i64::from(view.y)
        + i64::from(anchor.y) * i64::from(view.height) / i64::from(HEIGHT)
        - height
        - i64::from(scale(3, dpi)))
    .clamp(
        i64::from(view.y),
        i64::from(view.y) + i64::from(view.height) - height,
    );
    Some(RECT {
        left: i32::try_from(left).ok()?,
        top: i32::try_from(top).ok()?,
        right: i32::try_from(left + width).ok()?,
        bottom: i32::try_from(top + height).ok()?,
    })
}
/// Exactly the last successfully painted viewport/camera. Client mouse points
/// are already physical pixels under PMv2: never add desktop origin or DPI again.
fn hit_action(painted: PaintedSurface, point: PixelPoint) -> Option<Action> {
    let view = painted.viewport;
    let x = i64::from(point.x) - i64::from(view.x);
    let y = i64::from(point.y) - i64::from(view.y);
    if x < 0 || y < 0 || x >= i64::from(view.width) || y >= i64::from(view.height) {
        return None;
    }
    let pixel = PixelPoint {
        x: i32::try_from(x * i64::from(WIDTH) / i64::from(view.width)).ok()?,
        y: i32::try_from(y * i64::from(HEIGHT) / i64::from(view.height)).ok()?,
    };
    Some(Action::Navigate {
        surface: painted.surface,
        goal: painted.surface.projection.pixel_to_cell(pixel)?,
    })
}
fn mouse_point(value: isize) -> PixelPoint {
    PixelPoint {
        x: (value as u16) as i16 as i32,
        y: ((value as u32 >> 16) as u16) as i16 as i32,
    }
}
/// An armed targeted spell never silently becomes melee or ground movement.
fn selected_hit(action: Option<Action>, light_arrow: bool) -> Option<Action> {
    if !light_arrow {
        return action;
    }
    action.map(|action| match action {
        Action::Attack {
            surface,
            target_entity_id,
        } => Action::CastLightArrow {
            surface,
            target_entity_id,
        },
        Action::AttackNpc {
            surface,
            target,
            life,
        } => Action::CastNpcLightArrow {
            surface,
            target,
            life,
        },
        _ => Action::CancelRoute,
    })
}
fn disintegrate_hit(
    action: Option<Action>,
    selected: bool,
    combat: Option<&CombatSurface>,
) -> Option<Action> {
    if !selected {
        return action;
    }
    action.map(|action| {
        if let Action::AttackNpc {
            surface,
            target,
            life,
        } = action
        {
            let Some(c) = combat else {
                return Action::CancelRoute;
            };
            let f = &c.frame;
            let Some(n) = c.npc.as_ref().and_then(|n| n.reply.frame.as_ref()) else {
                return Action::CancelRoute;
            };
            if &n.combat != f.as_ref()
                || (surface.match_id, surface.generation, surface.revision)
                    != (f.world.match_id, f.world.generation, f.world.revision)
                || f.world.phase != super::world_protocol::Phase::Active
                || !f.world.descriptor.is_some_and(|d| {
                    d.has(super::world_protocol::MatchDescriptor::NPC_DISINTEGRATE)
                })
                || f.vital(surface.selected_entity_id)
                    .is_none_or(|v| v.hp == 0)
                || f.npc_disintegrate(surface.selected_entity_id).is_none()
                || !n.actor(target).is_some_and(|a| a.alive() && a.life == life)
            {
                return Action::CancelRoute;
            }
            return Action::CastNpcDisintegrate {
                surface,
                target,
                target_life: life,
                caster_life: super::quest_life::life(&f.world, surface.selected_entity_id)
                    .map_or(1, |l| l.incarnation()),
            };
        }
        let Action::Attack {
            surface,
            target_entity_id,
        } = action
        else {
            return Action::CancelRoute;
        };
        let Some(c) = combat else {
            return Action::CancelRoute;
        };
        let f = &c.frame;
        if (surface.match_id, surface.generation, surface.revision)
            != (f.world.match_id, f.world.generation, f.world.revision)
            || !f
                .world
                .descriptor
                .is_some_and(|d| d.has(super::world_protocol::MatchDescriptor::DISINTEGRATE))
            || f.world.phase != super::world_protocol::Phase::Active
            || target_entity_id == surface.selected_entity_id
            || f.vital(target_entity_id).is_none_or(|v| v.hp == 0)
            || f.vital(surface.selected_entity_id)
                .is_none_or(|v| v.hp == 0)
        {
            return Action::CancelRoute;
        }
        let Some(caster_life) =
            super::disintegrate_cast_protocol::life(f, surface.selected_entity_id)
        else {
            return Action::CancelRoute;
        };
        let Some(target_life) = super::disintegrate_cast_protocol::life(f, target_entity_id) else {
            return Action::CancelRoute;
        };
        Action::CastDisintegrate {
            surface,
            target_entity_id,
            caster_life,
            target_life,
        }
    })
}
fn healing_hit(action: Option<Action>, selected: bool) -> Option<Action> {
    if !selected {
        return action;
    }
    action.map(|action| match action {
        Action::Attack {
            surface,
            target_entity_id,
        } => Action::CastBasicHeal {
            surface,
            target_entity_id,
        },
        _ => Action::CancelRoute,
    })
}
const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const LEAVE: usize = 100;
const TELEPORT: usize = 101;
const MAGIC: usize = 102;
const USE_FIRST: usize = 103;
const AMMO_FIRST: usize = 110;
const FIRE_STORM: usize = 112;
const CONFIRM_DEATH: usize = 113;
const SETTINGS: usize = 114;
const CHAT: usize = 115;
const DETAILS_FIRST: usize = 116;
const DETAILS_TEXT: usize = 119;
// All children share one HWND parent. Keep nested controls in this namespace.
const INVENTORY_FIRST: usize = 120;
const STATUS_TEXT: usize = 123;
const GREATER_HASTE: usize = 124;
const SKILLS_FIRST: usize = 125;
const CHARACTER_EQUIPPED: usize = 127;
const FOOTER: i32 = 402;
pub(crate) const MAX_DETAILS_UTF16: usize = 1024 * 1024;
fn button_notification(wparam: WPARAM, lparam: LPARAM, id: usize, button: Option<HWND>) -> bool {
    wparam.0 & 0xffff == id
        && wparam.0 >> 16 == BN_CLICKED as usize
        && button.is_some_and(|h| !h.0.is_null() && h.0 as isize == lparam.0)
}
#[test]
fn all_child_control_ids_are_unique_and_text_notifications_cannot_cast() {
    let mut ids = vec![
        LEAVE,
        TELEPORT,
        MAGIC,
        FIRE_STORM,
        CONFIRM_DEATH,
        SETTINGS,
        CHAT,
        DETAILS_TEXT,
        STATUS_TEXT,
        GREATER_HASTE,
    ];
    ids.extend(USE_FIRST..USE_FIRST + 7);
    ids.extend(AMMO_FIRST..AMMO_FIRST + 2);
    ids.extend(DETAILS_FIRST..DETAILS_FIRST + 3);
    ids.extend(INVENTORY_FIRST..INVENTORY_FIRST + 3);
    ids.extend(SKILLS_FIRST..SKILLS_FIRST + 2);
    ids.push(CHARACTER_EQUIPPED);
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count);
    assert!(!inventory_controls::owns(STATUS_TEXT));
    assert!(!inventory_controls::owns(GREATER_HASTE));
    assert!(!inventory_controls::owns(SKILLS_FIRST));
    assert!(!skill_controls::owns(GREATER_HASTE));
    assert!(!skill_controls::owns(STATUS_TEXT));
    let handle = HWND(1234usize as *mut _);
    assert!(button_notification(
        WPARAM(GREATER_HASTE),
        LPARAM(1234),
        GREATER_HASTE,
        Some(handle)
    ));
    for (id, notification, source) in [
        (STATUS_TEXT, 0, 1234),
        (GREATER_HASTE, 0x400, 1234),
        (GREATER_HASTE, 0, 5678),
    ] {
        assert!(!button_notification(
            WPARAM(id | (notification << 16)),
            LPARAM(source),
            GREATER_HASTE,
            Some(handle)
        ));
    }
    assert!(!button_notification(
        WPARAM(GREATER_HASTE),
        LPARAM(1234),
        GREATER_HASTE,
        None
    ));
    assert!(!button_notification(
        WPARAM(GREATER_HASTE),
        LPARAM(0),
        GREATER_HASTE,
        Some(HWND::default())
    ));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DetailsPage {
    Character,
    Inventory,
    Skills,
}
impl DetailsPage {
    const ALL: [Self; 3] = [Self::Character, Self::Inventory, Self::Skills];
    fn from_button(id: usize) -> Option<Self> {
        id.checked_sub(DETAILS_FIRST)
            .and_then(|i| Self::ALL.get(i).copied())
    }
}
fn details_key(key: usize, repeated: bool, modified: bool) -> Option<DetailsPage> {
    if repeated || modified {
        return None;
    }
    match key {
        0x43 => Some(DetailsPage::Character),
        0x49 => Some(DetailsPage::Inventory),
        0x50 => Some(DetailsPage::Skills),
        _ => None,
    }
}
fn details_alt_key(message: u32, key: usize, lparam: isize, blocked: bool) -> Option<DetailsPage> {
    // A system-key message without the Alt context bit may be F10, not an
    // inspection mnemonic. Ctrl+Alt/AltGr and Windows chords stay native.
    if message != WM_SYSKEYDOWN || lparam & (1 << 29) == 0 || blocked {
        return None;
    }
    details_key(key, false, false)
}
pub(crate) fn details_mnemonic(message: &MSG) -> Option<DetailsPage> {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    let blocked = [0x11, 0x5b, 0x5c]
        .into_iter()
        .any(|key| unsafe { GetKeyState(key) < 0 });
    details_alt_key(message.message, message.wParam.0, message.lParam.0, blocked)
}
#[derive(Debug, PartialEq, Eq)]
enum DetailsKeyRoute {
    Close,
    Native,
    Continue,
}
fn details_key_route(open: bool, key: usize) -> DetailsKeyRoute {
    match (open, key) {
        (true, 0x1b) => DetailsKeyRoute::Close,
        (true, 0x09 | 0x0d) | (false, _) => DetailsKeyRoute::Continue,
        (true, _) => DetailsKeyRoute::Native,
    }
}
unsafe fn shortcut_modified() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    // Ctrl+C must remain copy, not open the character page. Shift alone is safe.
    [0x11, 0x12, 0x5b, 0x5c]
        .into_iter()
        .any(|key| GetKeyState(key) < 0)
}
unsafe fn hotbar_bind_modifier() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    GetKeyState(0x11) < 0 && [0x10, 0x12, 0x5b, 0x5c].into_iter().all(|key| GetKeyState(key) >= 0)
}
const STYLE: WINDOW_STYLE = WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 | WS_CLIPCHILDREN.0);
static NEXT_CLASS: AtomicU64 = AtomicU64::new(1);

fn scale(value: i32, dpi: u32) -> i32 {
    ((i64::from(value) * i64::from(dpi.clamp(48, 768)) + 48) / 96) as i32
}
fn footer_controls(width: i32, dpi: u32) -> [(i32, i32); 3] {
    let margin = scale(12, dpi).min(width.max(0) / 8);
    let gap = scale(13, dpi).min(width.max(0) / 8);
    let available = (width - 2 * margin - 2 * gap).max(0);
    let first = (i64::from(available) * 180 / 748) as i32;
    let second = (i64::from(available) * 230 / 748) as i32;
    [
        (margin, first),
        (margin + first + gap, second),
        (
            margin + first + second + 2 * gap,
            available - first - second,
        ),
    ]
}
fn supply_control(width: i32, dpi: u32, index: usize, bottom: i32) -> (i32, i32, i32) {
    if index < 3 {
        let (x, w) = footer_controls(width, dpi)[index];
        return (x, w, (bottom - scale(108, dpi)).max(0));
    }
    let margin = scale(12, dpi).min(width.max(0) / 10);
    let gap = scale(8, dpi).min(width.max(0) / 10);
    let available = (width - 2 * margin - 3 * gap).max(0);
    let i = (index - 3) as i32;
    let left = available * i / 4;
    let right = available * (i + 1) / 4;
    (
        margin + left + i * gap,
        right - left,
        (bottom - scale(74, dpi)).max(0),
    )
}
fn utility_controls(width: i32, dpi: u32) -> [(i32, i32); 4] {
    let width = width.max(0);
    let margin = scale(12, dpi).min(width / 10);
    let gap = scale(8, dpi).min(width / 10);
    let available = (width - 2 * margin - 3 * gap).max(0);
    std::array::from_fn(|i| {
        let i = i as i64;
        let left = i64::from(available) * i / 4;
        let right = i64::from(available) * (i + 1) / 4;
        (margin + left as i32 + i as i32 * gap, (right - left) as i32)
    })
}
#[test]
fn utility_hit_rectangles_are_disjoint_at_all_supported_dpi_and_widths() {
    for dpi in [48, 96, 120, 144, 192, 384, 768] {
        for width in [0, 1, 64, 320, 800, 1200, 1920] {
            let mut right = 0;
            for (x, w) in utility_controls(width, dpi) {
                assert!(x >= right && w >= 0 && x + w <= width);
                right = x + w;
            }
        }
    }
    assert_eq!(key_action(0x47, false), Some(Action::GreaterHaste));
    assert_eq!(key_action(0x47, true), None);
    assert!(!combo_keeps_key(0x47));
}
fn ammo_control(width: i32, dpi: u32, index: usize, bottom: i32) -> (i32, i32, i32) {
    let margin = scale(12, dpi).min(width.max(0) / 10);
    let gap = scale(8, dpi).min(width.max(0) / 10);
    let available = (width - 2 * margin - gap).max(0);
    let left = available * index as i32 / 2;
    let right = available * (index as i32 + 1) / 2;
    (
        margin + left + index as i32 * gap,
        right - left,
        (bottom - scale(142, dpi)).max(0),
    )
}
/// Death confirmation belongs inside the game viewport, above the footer.
/// A Quest/NPC70 player can own both this control and the (disabled) learned
/// Fire Storm button; they must not occupy the same native hit rectangle.
fn death_control(width: i32, bottom: i32, dpi: u32) -> (i32, i32, i32, i32) {
    let width = width.max(0);
    let viewport_height = (bottom - scale(FOOTER, dpi)).max(0);
    let margin = scale(12, dpi).min(width / 8);
    let button_width = scale(300, dpi).min((width - 2 * margin).max(0));
    let button_height = scale(28, dpi).min(viewport_height);
    (
        (width - button_width) / 2,
        (viewport_height - scale(12, dpi) - button_height).max(0),
        button_width,
        button_height,
    )
}
/// Directions follow the shared map projection, not screen/global mouse offsets.
fn key_action(key: usize, repeated: bool) -> Option<Action> {
    if repeated {
        return None;
    }
    if key == 0x1b {
        return Some(Action::Leave);
    }
    if key == 0x46 {
        return Some(Action::FireStorm);
    }
    if key == 0x47 {
        return Some(Action::GreaterHaste);
    }
    if key == 0x54 {
        return Some(Action::Teleport);
    }
    if (0x70..=0x76).contains(&key) {
        return Some(Action::UseItem(super::item_protocol::UI_ITEMS[key - 0x70]));
    }
    if (0x77..=0x78).contains(&key) {
        return Some(Action::SelectAmmo(
            super::ammo_protocol::UI_ITEMS[key - 0x77],
        ));
    }
    let heading = match key {
        0x26 | 0x68 => 1,
        0x27 | 0x66 => 3,
        0x28 | 0x62 => 5,
        0x25 | 0x64 => 7,
        0x67 => 0,
        0x69 => 2,
        0x63 => 4,
        0x61 => 6,
        _ => return None,
    };
    Some(Action::Step(heading))
}
unsafe fn bind_hotbar(state: &State, slot: usize) {
    if state.details_page.get() != Some(DetailsPage::Inventory) { return; }
    let selected = state.inventory.selection().map(|s| s.key);
    let valid = {
        let painted = state.painted_inventory.get();
        let model = state.inventory.model.borrow();
        let panel = state.inventory_panel.borrow();
        let battle = state.painted_battle.borrow();
        painted.zip(model.as_ref()).zip(panel.as_ref()).zip(battle.as_ref())
            .is_some_and(|(((painted, model), panel), battle)|
                painted.valid(panel, model, battle.1)
                    && selected.is_none_or(|key| painted.contains(key)))
    };
    if !valid { return; }
    let changed = state.hotbar.borrow_mut().as_mut().is_some_and(|bar| bar.bind(slot, selected));
    if changed {
        state.painted_hotbar.set(None);
        if let Some(hwnd) = state.hwnd.get() { let _ = InvalidateRect(Some(hwnd), None, false); }
    }
}
fn sync_character_combo_selection(state: &State) {
    let Some(combo) = state.character_combo.get() else { return; };
    let index = state.character_selected.get().and_then(|id| state.character_model.borrow()
        .as_ref()?.rows.iter().position(|row| row.id == id));
    unsafe { SendMessageW(combo, CB_SETCURSEL,
        Some(WPARAM(index.unwrap_or(usize::MAX))), None); }
}
unsafe fn bind_skill_hotbar(state: &State, slot: usize) {
    if state.details_page.get() != Some(DetailsPage::Skills) { return; }
    let Some(id) = state.skillbook.selection() else { return; };
    let valid = {
        let model = state.skillbook.model.borrow();
        let battle = state.painted_battle.borrow();
        let panel = state.skill_panel.borrow();
        let painted = state.painted_skill.get();
        model.as_ref().zip(battle.as_ref()).zip(panel.as_ref()).zip(painted)
            .is_some_and(|(((model, battle), panel), painted)|
                painted.valid(panel, model, battle.1) && painted.contains(id)
                    && model.choice(id).is_some_and(|choice| choice.matches(&battle.0.observation)))
    };
    if !valid { return; }
    let changed = state.hotbar.borrow_mut().as_mut().is_some_and(|bar| bar.bind_skill(slot, id));
    if changed {
        state.painted_hotbar.set(None);
        if let Some(hwnd) = state.hwnd.get() { let _ = InvalidateRect(Some(hwnd), None, false); }
    }
}
fn hotbar_action(state: &State, slot: usize) -> Option<hotbar::Action> {
    let proof = state.painted_hotbar.get()?;
    let painted = state.painted_battle.borrow();
    let pending = state.pending_battle.borrow();
    let painted = painted.as_ref()?;
    let pending = pending.as_ref()?;
    let stamp = super::battle_window::Stamp::of(&painted.0.observation);
    if stamp != super::battle_window::Stamp::of(&pending.observation) || stamp != proof.stamp {
        return None;
    }
    proof.action(state.hotbar.borrow().as_ref()?, slot, painted.1)
}
fn skill_choice_eligible(choice: super::battle_skillbook::Choice,
    skills: Option<&super::battle_skillbook::Model>, painted: Option<super::battle_window::Stamp>,
    pending: Option<super::battle_window::Stamp>) -> bool {
    skills.is_some_and(|m| m.stamp == choice.stamp && m.choice(choice.id) == Some(choice))
        && painted == Some(choice.stamp) && pending == Some(choice.stamp)
}
fn activate_hotbar(state: &State, slot: usize) {
    if state.details_page.get().is_none() {
        match hotbar_action(state, slot) {
            Some(hotbar::Action::Supply(intent)) => queue(state, Action::CanonicalInventory(intent)),
            Some(hotbar::Action::Skill(choice)) => queue(state, Action::CanonicalSkill(choice)),
            None => (),
        }
    }
}
fn combo_keeps_key(key: usize) -> bool {
    // This is only the action COMBOBOX, not a chat/edit text-input policy.
    !matches!(key, 0x1b | 0x54 | 0x46 | 0x47 | 0x09 | 0x70..=0x78)
}
fn policy_seconds(nanos: u64) -> String {
    let whole = nanos / 1_000_000_000;
    let remainder = nanos % 1_000_000_000;
    if remainder == 0 {
        whole.to_string()
    } else {
        let fraction = format!("{remainder:09}");
        format!("{whole}.{}", fraction.trim_end_matches('0'))
    }
}

fn match_title(descriptor: Option<&super::world_protocol::MatchDescriptor>) -> String {
    use super::world_protocol::MatchRuleset;
    let Some(d) = descriptor else {
        return "LiTo — 共用資源戰鬥世界（開發驗證）".into();
    };
    let rule = match d.ruleset {
        MatchRuleset::SceneOnly => "純移動場景",
        MatchRuleset::PvpSkirmish => "練習戰（不記勝敗）",
        MatchRuleset::DeathmatchElimination => "死鬥淘汰",
        MatchRuleset::HonorTowers => "榮譽守護塔",
        MatchRuleset::PetRace => "寵物競賽（新版規則：4 圈／首位後 10 秒收尾）",
        MatchRuleset::NpcEncounter
            if d.has(super::world_protocol::MatchDescriptor::QUEST_OBJECTIVES) =>
        {
            "Quest 目標戰（開發驗證）"
        }
        MatchRuleset::NpcEncounter => "NPC 遭遇（Quest 完整流程尚未開放）",
        MatchRuleset::CooperativeTimeAttack => "Time Attack 合作模式（開發驗證）",
        MatchRuleset::SiegeObjectives => "攻城模式（開發驗證）",
    };
    let limit = policy_seconds(u64::from(d.active_timeout_millis) * 1_000_000);
    // This descriptor clock is a pre-tower wait in Time Attack, but the
    // actual match deadline in Siege. The encounter's separate duration is
    // rendered by its validated mode HUD; never invent it from this clock.
    let deadline = match d.ruleset {
        MatchRuleset::CooperativeTimeAttack => {
            format!("啟動塔等待上限 {limit} 秒（擊塔後另依戰鬥時限結算）")
        }
        MatchRuleset::SiegeObjectives => format!("戰鬥時限 {limit} 秒（到期守方勝利）"),
        _ => format!("安全上限 {limit} 秒（到期中止、不判勝）"),
    };
    format!(
        "LiTo — {rule}｜倒數設定 {} 秒｜{deadline}",
        policy_seconds(d.countdown_nanos),
    )
}

fn missing_feature_notice(
    descriptor: Option<&super::world_protocol::MatchDescriptor>,
    feature: u32,
    pending: &'static str,
    disabled: &'static str,
) -> &'static str {
    if descriptor.is_some_and(|d| !d.has(feature)) {
        disabled
    } else {
        pending
    }
}

struct State {
    inventory: inventory_controls::Controls,
    inventory_panel: RefCell<Option<inventory_panel::Panel>>,
    painted_inventory: Cell<Option<inventory_panel::Painted>>,
    hotbar: RefCell<Option<hotbar::Hotbar>>,
    painted_hotbar: Cell<Option<hotbar::Painted>>,
    skillbook: skill_controls::Controls,
    skill_panel: RefCell<Option<skill_panel::Panel>>,
    painted_skill: Cell<Option<skill_panel::Painted>>,
    character_panel: RefCell<Option<character_panel::Panel>>,
    character_model: RefCell<Option<character_panel::Model>>,
    character_selected: Cell<Option<u64>>,
    character_combo: Cell<Option<HWND>>,
    painted_character: Cell<Option<character_panel::Painted>>,
    details_font: Cell<HFONT>,
    details_closed_by_escape: Cell<bool>,
    details: Cell<Option<HWND>>,
    details_buttons: Cell<Option<[HWND; 3]>>,
    details_page: Cell<Option<DetailsPage>>,
    details_text: RefCell<String>,
    settings: Cell<Option<HWND>>,
    settings_requested: Cell<bool>,
    chat: Cell<Option<HWND>>,
    chat_requested: Cell<bool>,
    hwnd: Cell<Option<HWND>>,
    leave: Cell<Option<HWND>>,
    teleport: Cell<Option<HWND>>,
    fire_storm: Cell<Option<HWND>>,
    greater_haste: Cell<Option<HWND>>,
    confirm_death: Cell<Option<HWND>>,
    death_confirmation: Cell<Option<super::quest_life::Confirmation>>,
    canonical_revival: Cell<Option<super::battle_revival::Request>>,
    can_fire_storm: Cell<bool>,
    can_greater_haste: Cell<bool>,
    magic: Cell<Option<HWND>>,
    items: Cell<Option<[HWND; 7]>>,
    ammo: Cell<Option<[HWND; 2]>>,
    can_select_ammo: Cell<[bool; 2]>,
    can_use: Cell<[bool; 7]>,
    item_labels: RefCell<[String; 7]>,
    light_arrow_available: Cell<bool>,
    light_arrow_selected: Cell<bool>,
    can_light_arrow: Cell<bool>,
    basic_heal_available: Cell<bool>,
    basic_heal_selected: Cell<bool>,
    can_basic_heal: Cell<bool>,
    haste_available: Cell<bool>,
    haste_selected: Cell<bool>,
    can_haste: Cell<bool>,
    slow_available: Cell<bool>,
    slow_selected: Cell<bool>,
    can_slow: Cell<bool>,
    disintegrate_available: Cell<bool>,
    disintegrate_selected: Cell<bool>,
    can_disintegrate: Cell<bool>,
    image: RefCell<Option<Arc<[u8]>>>,
    pending_hit: Cell<Option<HitSurface>>,
    painted_hit: Cell<Option<PaintedSurface>>,
    pending_combat: RefCell<Option<CombatSurface>>,
    painted_combat: RefCell<Option<CombatSurface>>,
    pending_players: RefCell<Option<PlayerSurface>>,
    painted_players: RefCell<Option<PlayerSurface>>,
    pending_battle: RefCell<Option<Arc<super::battle_scene::Surface>>>,
    painted_battle: RefCell<Option<(Arc<super::battle_scene::Surface>, Viewport)>>,
    match_descriptor: RefCell<Option<super::world_protocol::MatchDescriptor>>,
    status_control: Cell<Option<HWND>>,
    status: RefCell<Vec<u16>>,
    action: Cell<Option<Action>>,
    can_move: Cell<bool>,
    can_attack: Cell<bool>,
    timed_receipt: Cell<Option<(u64, u64, u64, i64, std::time::Instant)>>,
    released: Cell<bool>,
}
struct Class {
    name: Vec<u16>,
    instance: HINSTANCE,
}
impl Drop for Class {
    fn drop(&mut self) {
        unsafe {
            let _ = UnregisterClassW(PCWSTR(self.name.as_ptr()), Some(self.instance));
        }
    }
}
pub struct WorldWindow {
    state: Box<State>,
    class: Class,
    _ui_thread: PhantomData<Rc<()>>,
}
impl WorldWindow {
    pub fn open(parent: HWND) -> Result<Self> {
        Self::open_with_activation(parent, true)
    }
    /// Only the canonical player route supplies the already-selected shared
    /// Lineage root. Asset reads occur in the owned panel worker, never in paint.
    pub(crate) fn open_with_shared_inventory(parent: HWND, activate: bool, root: &std::path::Path) -> Result<Self> {
        let window = Self::open_with_activation(parent, activate)?;
        *window.state.inventory_panel.borrow_mut() = Some(inventory_panel::Panel::start(root.to_path_buf())?);
        *window.state.hotbar.borrow_mut() = Some(hotbar::Hotbar::start(root.to_path_buf())?);
        *window.state.skill_panel.borrow_mut() = Some(skill_panel::Panel::start(root.to_path_buf())?);
        *window.state.character_panel.borrow_mut() = Some(character_panel::Panel::start(root.to_path_buf())?);
        unsafe {
            if let Some(button) = window.state.settings.get() { SetWindowTextW(button, w!("遊戲設定"))?; }
            if let Some(button) = window.state.chat.get() { SetWindowTextW(button, w!("聊天"))?; }
        }
        Ok(window)
    }
    /// Automatic world admission must not steal an active chat EDIT/IME focus.
    pub(crate) fn open_with_activation(parent: HWND, activate: bool) -> Result<Self> {
        let number = NEXT_CLASS
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| anyhow::anyhow!("world window IDs exhausted"))?;
        let name: Vec<u16> = format!("ModernLiToWorld{number}")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        unsafe {
            ensure!(
                IsWindow(Some(parent)).as_bool(),
                "world window owner unavailable"
            );
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            ensure!(
                RegisterClassW(&WNDCLASSW {
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance,
                    lpszClassName: PCWSTR(name.as_ptr()),
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
                    ..Default::default()
                }) != 0,
                "cannot register world window"
            );
            let mut window = Self {
                state: Box::new(State {
                    details_font: Cell::new(HFONT::default()),
                    inventory: inventory_controls::Controls::default(),
                    inventory_panel: RefCell::new(None),
                    painted_inventory: Cell::new(None),
                    hotbar: RefCell::new(None),
                    painted_hotbar: Cell::new(None),
                    skillbook: skill_controls::Controls::default(),
                    skill_panel: RefCell::new(None),
                    painted_skill: Cell::new(None),
                    character_panel: RefCell::new(None),
                    character_model: RefCell::new(None),
                    character_selected: Cell::new(None),
                    character_combo: Cell::new(None),
                    painted_character: Cell::new(None),
                    details_closed_by_escape: Cell::new(false),
                    details: Cell::new(None),
                    details_buttons: Cell::new(None),
                    details_page: Cell::new(None),
                    details_text: RefCell::new(String::new()),
                    settings: Cell::new(None),
                    settings_requested: Cell::new(false),
                    chat: Cell::new(None),
                    chat_requested: Cell::new(false),
                    hwnd: Cell::new(None),
                    leave: Cell::new(None),
                    teleport: Cell::new(None),
                    fire_storm: Cell::new(None),
                    greater_haste: Cell::new(None),
                    confirm_death: Cell::new(None),
                    death_confirmation: Cell::new(None),
                    canonical_revival: Cell::new(None),
                    can_fire_storm: Cell::new(false),
                    can_greater_haste: Cell::new(false),
                    magic: Cell::new(None),
                    items: Cell::new(None),
                    ammo: Cell::new(None),
                    can_select_ammo: Cell::new([false; 2]),
                    can_use: Cell::new([false; 7]),
                    item_labels: RefCell::new(std::array::from_fn(|_| String::new())),
                    light_arrow_available: Cell::new(false),
                    light_arrow_selected: Cell::new(false),
                    can_light_arrow: Cell::new(false),
                    basic_heal_available: Cell::new(false),
                    basic_heal_selected: Cell::new(false),
                    can_basic_heal: Cell::new(false),
                    haste_available: Cell::new(false),
                    haste_selected: Cell::new(false),
                    can_haste: Cell::new(false),
                    slow_available: Cell::new(false),
                    slow_selected: Cell::new(false),
                    can_slow: Cell::new(false),
                    disintegrate_available: Cell::new(false),
                    disintegrate_selected: Cell::new(false),
                    can_disintegrate: Cell::new(false),
                    image: RefCell::new(None),
                    pending_hit: Cell::new(None),
                    painted_hit: Cell::new(None),
                    pending_combat: RefCell::new(None),
                    painted_combat: RefCell::new(None),
                    pending_players: RefCell::new(None),
                    painted_players: RefCell::new(None),
                    pending_battle: RefCell::new(None),
                    painted_battle: RefCell::new(None),
                    match_descriptor: RefCell::new(None),
                    status_control: Cell::new(None),
                    status: RefCell::new("正在準備已確認世界的共用資源…".encode_utf16().collect()),
                    action: Cell::new(None),
                    can_move: Cell::new(false),
                    can_attack: Cell::new(false),
                    timed_receipt: Cell::new(None),
                    released: Cell::new(false),
                }),
                class: Class { name, instance },
                _ui_thread: PhantomData,
            };
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(window.class.name.as_ptr()),
                w!("LiTo — 共用資源戰鬥世界（開發驗證）"),
                STYLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                WIDTH,
                HEIGHT + FOOTER,
                Some(parent),
                None,
                Some(instance),
                Some((&mut *window.state as *mut State).cast()),
            )?;
            window.state.hwnd.set(Some(hwnd));
            let mut detail_buttons = [HWND::default(); 3];
            for (index, label) in [
                w!("人物 (C／Alt+C)"),
                w!("背包 (I／Alt+I)"),
                w!("已學技能 (P／Alt+P)"),
            ]
            .into_iter()
            .enumerate()
            {
                detail_buttons[index] = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    label,
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                    0,
                    0,
                    180,
                    28,
                    Some(hwnd),
                    Some(HMENU((DETAILS_FIRST + index) as *mut _)),
                    Some(instance),
                    None,
                )?;
            }
            window.state.details_buttons.set(Some(detail_buttons));
            let details = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                w!(""),
                WS_CHILD
                    | WS_TABSTOP
                    | WS_VSCROLL
                    | WS_HSCROLL
                    | WINDOW_STYLE(
                        (ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL | ES_AUTOHSCROLL) as u32,
                    ),
                0,
                0,
                WIDTH,
                HEIGHT,
                Some(hwnd),
                Some(HMENU(DETAILS_TEXT as *mut _)),
                Some(instance),
                None,
            )?;
            SendMessageW(
                details,
                EM_SETLIMITTEXT,
                Some(WPARAM(MAX_DETAILS_UTF16)),
                None,
            );
            window.state.details.set(Some(details));
            window
                .state
                .status_control
                .set(Some(status_text::create(hwnd, instance)?));
            window.state.inventory.create(hwnd, instance)?;
            window.state.skillbook.create(hwnd, instance)?;
            window.state.character_combo.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(), w!("COMBOBOX"), w!(""),
                WS_CHILD | WS_TABSTOP | WS_VSCROLL | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
                0, 0, 180, 280, Some(hwnd),
                Some(HMENU(CHARACTER_EQUIPPED as *mut _)), Some(instance), None,
            )?));
            replace_details_font(&window.state)?;
            window.state.chat.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("聊天 (F11)"),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                0,
                0,
                180,
                30,
                Some(hwnd),
                Some(HMENU(CHAT as *mut _)),
                Some(instance),
                None,
            )?));
            window.state.settings.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("遊戲設定 (F10)"),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                0,
                0,
                180,
                30,
                Some(hwnd),
                Some(HMENU(SETTINGS as *mut _)),
                Some(instance),
                None,
            )?));
            window.state.leave.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("返回房間 (Esc)"),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                0,
                0,
                180,
                30,
                Some(hwnd),
                Some(HMENU(LEAVE as *mut _)),
                Some(instance),
                None,
            )?));
            window.state.teleport.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("瞬間移動（T）"),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_DISABLED,
                0,
                0,
                200,
                30,
                Some(hwnd),
                Some(HMENU(TELEPORT as *mut _)),
                Some(instance),
                None,
            )?));
            window.state.fire_storm.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("火風暴（F）"),
                WS_CHILD | WS_TABSTOP | WS_DISABLED,
                0,
                0,
                300,
                28,
                Some(hwnd),
                Some(HMENU(FIRE_STORM as *mut _)),
                Some(instance),
                None,
            )?));
            window.state.greater_haste.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("強力加速術（G）"),
                WS_CHILD | WS_TABSTOP | WS_DISABLED,
                0,
                0,
                200,
                28,
                Some(hwnd),
                Some(HMENU(GREATER_HASTE as *mut _)),
                Some(instance),
                None,
            )?));
            window.state.confirm_death.set(Some(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("確認死亡／回出生點"),
                WS_CHILD | WS_TABSTOP | WS_DISABLED,
                0,
                0,
                300,
                28,
                Some(hwnd),
                Some(HMENU(CONFIRM_DEATH as *mut _)),
                Some(instance),
                None,
            )?));
            let magic = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("COMBOBOX"),
                w!(""),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
                0,
                0,
                300,
                160,
                Some(hwnd),
                Some(HMENU(MAGIC as *mut _)),
                Some(instance),
                None,
            )?;
            window.state.magic.set(Some(magic));
            let mut items = [HWND::default(); 7];
            for (index, item) in items.iter_mut().enumerate() {
                *item = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    w!("本場補給未確認"),
                    WS_CHILD | WS_TABSTOP | WS_DISABLED,
                    0,
                    0,
                    180,
                    28,
                    Some(hwnd),
                    Some(HMENU((USE_FIRST + index) as *mut _)),
                    Some(instance),
                    None,
                )?;
            }
            window.state.items.set(Some(items));
            let mut arrows = [HWND::default(); 2];
            for (index, button) in arrows.iter_mut().enumerate() {
                *button = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    w!("本場箭矢未確認"),
                    WS_CHILD | WS_TABSTOP | WS_DISABLED,
                    0,
                    0,
                    180,
                    28,
                    Some(hwnd),
                    Some(HMENU((AMMO_FIRST + index) as *mut _)),
                    Some(instance),
                    None,
                )?;
            }
            window.state.ammo.set(Some(arrows));
            ensure!(
                SendMessageW(
                    magic,
                    CB_ADDSTRING,
                    None,
                    Some(LPARAM(w!("普通攻擊（左鍵角色）").as_ptr() as isize))
                )
                .0 == 0,
                "cannot populate world action selector"
            );
            ensure!(
                SendMessageW(magic, CB_SETCURSEL, Some(WPARAM(0)), None).0 == 0,
                "cannot select world action"
            );
            let dpi = GetDpiForWindow(hwnd).max(96);
            let mut bounds = RECT {
                left: 0,
                top: 0,
                right: scale(WIDTH, dpi),
                bottom: scale(HEIGHT + FOOTER, dpi),
            };
            AdjustWindowRectExForDpi(&mut bounds, STYLE, false, WINDOW_EX_STYLE::default(), dpi)?;
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            resize(hwnd, &window.state);
            let _ = ShowWindow(hwnd, if activate { SW_SHOW } else { SW_SHOWNOACTIVATE });
            if activate {
                let _ = SetFocus(Some(hwnd));
            }
            Ok(window)
        }
    }
    /// Only immutable BGRA upload; called on the parent's UI thread.
    pub fn update(
        &self,
        image: Option<Arc<[u8]>>,
        hit: Option<HitSurface>,
        status: &str,
        can_move: bool,
        combat: Option<CombatSurface>,
        can_attack: bool,
    ) -> Result<()> {
        self.update_with_spells(image, hit, status, can_move, combat, can_attack, false)
    }
    pub fn update_with_spells(
        &self,
        image: Option<Arc<[u8]>>,
        hit: Option<HitSurface>,
        status: &str,
        can_move: bool,
        combat: Option<CombatSurface>,
        can_attack: bool,
        can_cast: bool,
    ) -> Result<()> {
        self.update_with_magic(
            image, hit, status, can_move, combat, can_attack, can_cast, false, false,
        )
    }
    pub fn update_with_magic(
        &self,
        image: Option<Arc<[u8]>>,
        hit: Option<HitSurface>,
        status: &str,
        can_move: bool,
        combat: Option<CombatSurface>,
        can_attack: bool,
        can_cast: bool,
        light_arrow_available: bool,
        can_light_arrow: bool,
    ) -> Result<()> {
        self.update_with_roster(
            image,
            hit,
            status,
            can_move,
            combat,
            can_attack,
            can_cast,
            light_arrow_available,
            can_light_arrow,
            None,
        )
    }
    pub fn update_with_roster(
        &self,
        image: Option<Arc<[u8]>>,
        hit: Option<HitSurface>,
        status: &str,
        can_move: bool,
        combat: Option<CombatSurface>,
        can_attack: bool,
        can_cast: bool,
        light_arrow_available: bool,
        can_light_arrow: bool,
        players: Option<PlayerSurface>,
    ) -> Result<()> {
        if let Some(players) = &players {
            ensure!(combat.is_none(), "conflicting player name authority");
            validate_player_surface(
                hit.ok_or_else(|| anyhow::anyhow!("player names require matching pixels"))?,
                players,
            )?;
        }
        ensure!(
            image
                .as_ref()
                .is_none_or(|pixels| pixels.len() == WIDTH as usize * HEIGHT as usize * 4),
            "world image buffer has invalid dimensions"
        );
        ensure!(
            hit.is_none() || image.is_some(),
            "world hit surface requires matching pixels"
        );
        ensure!(
            status.len() <= 2048 && !status.chars().any(|c| c == '\0'),
            "world status too large"
        );
        if let Some(combat) = &combat {
            combat.frame.validate()?;
            let hit =
                hit.ok_or_else(|| anyhow::anyhow!("combat needs matched pixels/hit surface"))?;
            ensure!(
                combat.frame.world.match_id == hit.match_id
                    && combat.frame.world.generation == hit.generation
                    && combat.frame.world.revision == hit.revision
                    && combat.actors.len()
                        == combat
                            .frame
                            .world
                            .actors
                            .iter()
                            .filter(|a| combat
                                .frame
                                .world
                                .time_attack
                                .as_ref()
                                .is_none_or(|d| !d.departed(a.entity_id)))
                            .count(),
                "combat hit/pixel revision mismatch"
            );
            let mut ids = std::collections::BTreeSet::new();
            ensure!(
                combat.objectives.len()
                    == combat
                        .frame
                        .honor
                        .as_ref()
                        .map_or(0, |h| h.objectives.len())
                    && combat.objectives.iter().all(|o| combat
                        .frame
                        .honor
                        .as_ref()
                        .and_then(|h| h.objective(o.entity_id))
                        .is_some_and(|a| a.position == o.cell)
                        && ids.insert(o.entity_id)
                        && o.width > 0
                        && o.width <= 4096
                        && o.height > 0
                        && o.height <= 4096),
                "invalid rendered objective rectangles"
            );
            let mut quest_ids = std::collections::HashSet::new();
            ensure!(
                super::siege_presentation::valid_placements(
                    &combat.frame.world,
                    &combat.siege_towers
                ),
                "invalid rendered siege tower rectangles"
            );
            ensure!(
                combat.time_attack_tower.is_some() == combat.frame.world.time_attack.is_some()
                    && combat.time_attack_tower.as_ref().is_none_or(|p| {
                        super::time_attack_presentation::valid_placement(&combat.frame.world, p)
                    }),
                "invalid rendered cooperative tower rectangle"
            );
            ensure!(
                combat.quests.len()
                    == combat
                        .frame
                        .world
                        .quest
                        .as_ref()
                        .map_or(0, |q| q.actors.len())
                    && combat.quests.iter().all(|p| {
                        super::quest_presentation::valid_placement(&combat.frame.world, p)
                            && quest_ids.insert(p.id)
                            && p.geometry.width > 0
                            && p.geometry.width <= 4096
                            && p.geometry.height > 0
                            && p.geometry.height <= 4096
                    }),
                "invalid rendered Quest rectangles"
            );
            ensure!(
                combat.actors.iter().all(|a| ids.insert(a.entity_id)
                    && combat
                        .frame
                        .world
                        .actors
                        .iter()
                        .any(|b| a.entity_id == b.entity_id
                            && combat
                                .frame
                                .world
                                .time_attack
                                .as_ref()
                                .is_none_or(|d| !d.departed(b.entity_id)))
                    && a.width > 0
                    && a.width <= 4096
                    && a.height > 0
                    && a.height <= 4096),
                "invalid rendered combat target rectangles"
            );
        }
        self.state.pending_battle.borrow_mut().take();
        self.state.painted_inventory.set(None);
        self.state.painted_skill.set(None);
        self.state.painted_character.set(None);
        self.state.painted_hotbar.set(None);
        // A worker may be loading the next revision of this same attached
        // world. Keep its last pixels visible, but retire hit/command proofs
        // below until a matching new surface is ready. Scope changes destroy
        // this window, so pixels never cross matches.
        if let Some(image) = image {
            *self.state.image.borrow_mut() = Some(image);
        } else {
            // A retained picture is visual continuity only, never a stale
            // command target while the matching next surface is loading.
            self.state.painted_hit.set(None);
            *self.state.painted_combat.borrow_mut() = None;
            *self.state.painted_players.borrow_mut() = None;
            self.state.painted_battle.borrow_mut().take();
        }
        self.state.pending_hit.set(hit);
        *self.state.pending_combat.borrow_mut() = combat;
        *self.state.pending_players.borrow_mut() = players;
        let next_status: Vec<u16> = status.encode_utf16().collect();
        if *self.state.status.borrow() != next_status {
            if let Some(control) = self.state.status_control.get() {
                unsafe {
                    status_text::replace(control, &next_status)?;
                }
            }
            *self.state.status.borrow_mut() = next_status;
        }
        self.state.can_move.set(can_move);
        self.state.can_attack.set(can_attack);
        self.state.can_light_arrow.set(can_light_arrow);
        if self.state.light_arrow_available.get() != light_arrow_available {
            self.state.light_arrow_available.set(light_arrow_available);
            refresh_magic(&self.state)?;
        }
        if let Some(button) = self.state.teleport.get() {
            unsafe {
                let _ = EnableWindow(button, can_cast);
            }
        }
        if let Some(hwnd) = self.state.hwnd.get() {
            unsafe {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
        }
        Ok(())
    }
    /// Replace only the display/hit authority. Command availability is still
    /// independently supplied by the player controller; this is never LOAD.
    pub fn update_canonical(&self, surface: Arc<super::battle_scene::Surface>) -> Result<()> {
        super::battle_window::validate(&surface)?;
        *self.state.image.borrow_mut() = Some(surface.bgra.clone());
        self.state.pending_hit.set(None);
        self.state.pending_combat.borrow_mut().take();
        self.state.pending_players.borrow_mut().take();
        *self.state.pending_battle.borrow_mut() = Some(surface);
        self.state.painted_inventory.set(None);
        self.state.painted_skill.set(None);
        self.state.painted_character.set(None);
        self.state.painted_hotbar.set(None);
        if let Some(hwnd) = self.state.hwnd.get() {
            unsafe {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
        }
        Ok(())
    }
    pub(crate) fn painted_observation(&self) -> Option<super::battle_follow::Observation> {
        self.state
            .painted_battle
            .borrow()
            .as_ref()
            .map(|(s, _)| s.observation.clone())
    }
    pub(crate) fn painted_surface(&self) -> Option<Arc<super::battle_scene::Surface>> {
        self.state
            .painted_battle
            .borrow()
            .as_ref()
            .map(|(s, _)| s.clone())
    }
    pub fn take_action(&self) -> Option<Action> {
        self.state.action.take()
    }
    pub(crate) fn details_page(&self) -> Option<DetailsPage> {
        self.state.details_page.get()
    }
    pub(crate) fn character_equipped_selection(&self) -> Option<u64> {
        (self.details_page() == Some(DetailsPage::Character))
            .then(|| self.state.character_selected.get()).flatten()
    }
    pub(crate) fn inventory_selection(&self) -> Option<super::battle_inventory::Selection> {
        (self.details_page() == Some(DetailsPage::Inventory))
            .then(|| self.state.inventory.selection())
            .flatten()
    }
    pub(crate) fn update_inventory(
        &self,
        model: Option<super::battle_inventory::Model>,
    ) -> Result<()> {
        unsafe { self.state.inventory.update(&self.state, model)?; }
        let (changed, panel_error) = {
            let mut panel = self.state.inventory_panel.borrow_mut();
            panel.as_mut().map_or((false, None), |panel| {
                let changed = panel.update(self.state.inventory.model.borrow().as_ref());
                (changed, panel.error().map(str::to_owned))
            })
        };
        if self.state.painted_inventory.get().is_some_and(|painted| {
            let model = self.state.inventory.model.borrow();
            let panel = self.state.inventory_panel.borrow();
            !model.as_ref().zip(panel.as_ref()).is_some_and(|(model, panel)|
                painted.valid(panel, model, painted.viewport))
        }) {
            self.state.painted_inventory.set(None);
        }
        if changed {
            if let Some(hwnd) = self.state.hwnd.get() {
                unsafe {
                    resize(hwnd, &self.state);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
            }
        }
        if let Some(error) = panel_error { anyhow::bail!("shared inventory art: {error}"); }
        Ok(())
    }
    /// Updated from the attached canonical snapshot even when the inventory
    /// inspection page is closed. No saved item binding grants a future proof.
    pub(crate) fn update_hotbar(&self, model: Option<super::battle_inventory::Model>,
        skills: Option<super::battle_skillbook::Model>,
        receipt: &super::battle_control::State) -> Result<()> {
        let (changed, error, valid) = {
            let mut holder = self.state.hotbar.borrow_mut();
            let Some(bar) = holder.as_mut() else { return Ok(()); };
            let released = bar.acknowledge(receipt);
            let changed = bar.update(model, skills) || released;
            let error = bar.error().map(str::to_owned);
            let valid = self.state.painted_hotbar.get().is_some_and(|painted|
                painted.valid(bar, painted.viewport));
            (changed, error, valid)
        };
        if !valid { self.state.painted_hotbar.set(None); }
        if changed {
            if let Some(hwnd) = self.state.hwnd.get() {
                unsafe { let _ = InvalidateRect(Some(hwnd), None, false); }
            }
        }
        if let Some(error) = error { anyhow::bail!("shared hotbar art: {error}"); }
        Ok(())
    }
    pub(crate) fn submitted_hotbar_command(&self, stamp: super::battle_window::Stamp,
        command: super::battle_control::Command, prior_receipt: Option<u64>) {
        if let Some(bar) = self.state.hotbar.borrow_mut().as_mut() {
            bar.submitted(stamp, command, prior_receipt);
            self.state.painted_hotbar.set(None);
            if let Some(hwnd) = self.state.hwnd.get() {
                unsafe { let _ = InvalidateRect(Some(hwnd), None, false); }
            }
        }
    }
    pub(crate) fn details_need_text(&self) -> bool {
        self.state.details_text.borrow().is_empty()
    }
    pub(crate) fn update_skillbook(
        &self,
        model: Option<super::battle_skillbook::Model>,
    ) -> Result<()> {
        unsafe { self.state.skillbook.update(&self.state, model)?; }
        let (changed, error) = {
            let mut panel = self.state.skill_panel.borrow_mut();
            panel.as_mut().map_or((false, None), |panel| {
                let changed = panel.update(self.state.skillbook.model.borrow().as_ref());
                (changed, panel.error().map(str::to_owned))
            })
        };
        if self.state.painted_skill.get().is_some_and(|painted| {
            let model = self.state.skillbook.model.borrow();
            let panel = self.state.skill_panel.borrow();
            !model.as_ref().zip(panel.as_ref()).is_some_and(|(model, panel)|
                painted.valid(panel, model, painted.viewport))
        }) {
            self.state.painted_skill.set(None);
        }
        if changed {
            if let Some(hwnd) = self.state.hwnd.get() {
                unsafe {
                    resize(hwnd, &self.state);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
            }
        }
        if let Some(error) = error { anyhow::bail!("shared magic art: {error}"); }
        Ok(())
    }
    pub(crate) fn update_character_panel(
        &self,
        observation: Option<&super::battle_follow::Observation>,
        inventory: Option<&super::battle_inventory::Model>,
    ) -> Result<()> {
        let next = if self.details_page() == Some(DetailsPage::Character) {
            observation.zip(inventory).and_then(|(o, inventory)|
                character_panel::Model::from_observation(o, inventory))
        } else { None };
        let (labels_changed, selection_changed) = {
            let mut current = self.state.character_model.borrow_mut();
            let old = current.take();
            let same_owner = old.as_ref().zip(next.as_ref()).is_some_and(|(a,b)| a.same_owner(b));
            let selected = self.state.character_selected.get().filter(|id|
                same_owner && next.as_ref().is_some_and(|m| m.contains(*id)));
            let selection_changed = self.state.character_selected.replace(selected) != selected;
            let labels_changed = old.as_ref().map(|m| m.rows.iter()
                .map(|r| (r.id, &r.label)).collect::<Vec<_>>())
                != next.as_ref().map(|m| m.rows.iter()
                    .map(|r| (r.id, &r.label)).collect::<Vec<_>>());
            *current = next;
            (labels_changed, selection_changed)
        };
        if selection_changed { self.state.details_text.borrow_mut().clear(); }
        if let Some(combo) = self.state.character_combo.get() {
            if labels_changed {
                unsafe { SendMessageW(combo, CB_RESETCONTENT, None, None); }
                if let Some(model) = self.state.character_model.borrow().as_ref() {
                    for (index, row) in model.rows.iter().enumerate() {
                        let label: Vec<u16> = row.label.encode_utf16().chain(Some(0)).collect();
                        ensure!(!row.label.contains('\0') && label.len() <= 1024,
                            "invalid equipped item label");
                        ensure!(unsafe { SendMessageW(combo, CB_ADDSTRING, None,
                            Some(LPARAM(label.as_ptr() as isize))) }.0 == index as isize,
                            "equipped item combo failed");
                    }
                }
            }
            if labels_changed || selection_changed {
                sync_character_combo_selection(&self.state);
            }
            unsafe { let _ = ShowWindow(combo, if self.details_page() == Some(DetailsPage::Character)
                && self.state.character_model.borrow().is_some() { SW_SHOW } else { SW_HIDE }); }
        }
        let (mut changed, error) = {
            let mut panel = self.state.character_panel.borrow_mut();
            panel.as_mut().map_or((false, None), |panel| {
                let changed = panel.update(self.state.character_model.borrow().as_ref());
                (changed, panel.error().map(str::to_owned))
            })
        };
        if self.state.character_selected.get().is_some_and(|id| {
            let model = self.state.character_model.borrow();
            let panel = self.state.character_panel.borrow();
            !model.as_ref().zip(panel.as_ref()).is_some_and(|(model,panel)| panel.visible(model, id))
        }) {
            self.state.character_selected.set(None);
            self.state.details_text.borrow_mut().clear();
            changed = true;
        }
        if self.state.character_selected.get().is_none() {
            let first = {
                let model = self.state.character_model.borrow();
                let panel = self.state.character_panel.borrow();
                model.as_ref().zip(panel.as_ref()).and_then(|(model,panel)| panel.first_visible(model))
            };
            if let Some(first) = first {
                self.state.character_selected.set(Some(first));
                sync_character_combo_selection(&self.state);
                self.state.details_text.borrow_mut().clear();
                changed = true;
            }
        }
        if self.state.painted_character.get().is_some_and(|painted| {
            let model = self.state.character_model.borrow();
            let panel = self.state.character_panel.borrow();
            !model.as_ref().zip(panel.as_ref()).is_some_and(|(model,panel)|
                painted.valid(panel, model, painted.viewport))
        }) {
            self.state.painted_character.set(None);
        }
        if changed {
            if let Some(hwnd) = self.state.hwnd.get() {
                unsafe {
                    resize(hwnd, &self.state);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
            }
        }
        if let Some(error) = error { anyhow::bail!("shared character art: {error}"); }
        Ok(())
    }
    pub(crate) fn update_target_haste(&self, available: bool, can_cast: bool) -> Result<()> {
        self.state.can_haste.set(available && can_cast);
        if self.state.haste_available.replace(available) != available {
            refresh_magic(&self.state)?;
        }
        Ok(())
    }
    pub(crate) fn update_slow(&self, available: bool, can_cast: bool) -> Result<()> {
        self.state.can_slow.set(available && can_cast);
        if self.state.slow_available.replace(available) != available {
            refresh_magic(&self.state)?;
        }
        Ok(())
    }
    pub(crate) fn selected_skill(&self, skill: super::battle_window::Skill) -> Result<()> {
        use super::battle_window::Skill;
        // Called only after the player adapter revalidates this learned choice.
        // Availability may have arrived since the last native control refresh.
        match skill {
            Skill::LightArrow => self.state.light_arrow_available.set(true),
            Skill::Heal => self.state.basic_heal_available.set(true),
            Skill::Disintegrate => self.state.disintegrate_available.set(true),
            Skill::Haste => self.state.haste_available.set(true),
            Skill::Slow => self.state.slow_available.set(true),
            Skill::Melee => {}
        }
        self.state
            .light_arrow_selected
            .set(skill == Skill::LightArrow);
        self.state.basic_heal_selected.set(skill == Skill::Heal);
        self.state.haste_selected.set(skill == Skill::Haste);
        self.state.slow_selected.set(skill == Skill::Slow);
        self.state
            .disintegrate_selected
            .set(skill == Skill::Disintegrate);
        refresh_magic(&self.state)?;
        unsafe { show_details(&self.state, None) };
        Ok(())
    }
    pub(crate) fn close_details(&self) {
        unsafe { show_details(&self.state, None) };
    }
    pub(crate) fn update_details(&self, text: &str) -> Result<()> {
        ensure!(
            !text.contains('\0') && text.encode_utf16().count() <= MAX_DETAILS_UTF16,
            "invalid world details text"
        );
        if self.state.details_page.get().is_none() || *self.state.details_text.borrow() == text {
            return Ok(());
        }
        if let Some(control) = self.state.details.get() {
            let encoded: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
            unsafe {
                // Keep selection and scroll stable while confirmed vitals change.
                let (mut start, mut end) = (0u32, 0u32);
                SendMessageW(
                    control,
                    EM_GETSEL,
                    Some(WPARAM((&mut start as *mut u32) as usize)),
                    Some(LPARAM((&mut end as *mut u32) as isize)),
                );
                let first = SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None);
                SetWindowTextW(control, PCWSTR(encoded.as_ptr()))?;
                SendMessageW(
                    control,
                    EM_SETSEL,
                    Some(WPARAM(start as usize)),
                    Some(LPARAM(end as isize)),
                );
                let current = SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None);
                SendMessageW(
                    control,
                    EM_LINESCROLL,
                    None,
                    Some(LPARAM(first.0 - current.0)),
                );
            }
            *self.state.details_text.borrow_mut() = text.into();
        }
        Ok(())
    }
    pub fn take_settings_request(&self) -> bool {
        self.state.settings_requested.replace(false)
    }
    pub fn take_chat_request(&self) -> bool {
        self.state.chat_requested.replace(false)
    }
    pub fn set_chat_available(&self, available: bool) {
        if let Some(button) = self.state.chat.get() {
            unsafe {
                let _ = EnableWindow(button, available);
            }
        }
        if !available {
            self.state.chat_requested.set(false);
        }
    }
    /// The immutable server policy, not an inferred remaining-time countdown.
    pub fn update_match_descriptor(
        &self,
        descriptor: Option<&super::world_protocol::MatchDescriptor>,
    ) -> Result<()> {
        if self.state.match_descriptor.borrow().as_ref() == descriptor {
            return Ok(());
        }
        if let Some(hwnd) = self.state.hwnd.get() {
            let title: Vec<u16> = match_title(descriptor)
                .encode_utf16()
                .chain(Some(0))
                .collect();
            unsafe {
                SetWindowTextW(hwnd, PCWSTR(title.as_ptr()))?;
            }
        }
        *self.state.match_descriptor.borrow_mut() = descriptor.cloned();
        Ok(())
    }
    pub fn update_fire_storm(&self, available: bool, can_cast: bool) {
        self.state.can_fire_storm.set(available && can_cast);
        if let Some(button) = self.state.fire_storm.get() {
            unsafe {
                let _ = ShowWindow(button, if available { SW_SHOW } else { SW_HIDE });
                let _ = EnableWindow(button, available && can_cast);
            }
        }
    }
    pub fn update_greater_haste(&self, available: bool, can_cast: bool) {
        self.state.can_greater_haste.set(available && can_cast);
        if let Some(button) = self.state.greater_haste.get() {
            unsafe {
                let _ = ShowWindow(button, if available { SW_SHOWNA } else { SW_HIDE });
                let _ = EnableWindow(button, available && can_cast);
            }
        }
    }
    pub fn update_quest_confirmation(
        &self,
        proof: Option<super::quest_life::Confirmation>,
        enabled: bool,
    ) {
        self.state.canonical_revival.set(None);
        self.state.death_confirmation.set(proof.filter(|_| enabled));
        if let Some(button) = self.state.confirm_death.get() {
            unsafe {
                let _ = ShowWindow(button, if proof.is_some() { SW_SHOWNA } else { SW_HIDE });
                let _ = EnableWindow(button, proof.is_some() && enabled);
            }
        }
    }
    pub(crate) fn update_canonical_revival(
        &self,
        proof: Option<super::battle_revival::Request>,
        enabled: bool,
    ) {
        self.state.death_confirmation.set(None);
        let proof = proof.filter(|q| {
            self.painted_observation()
                .as_ref()
                .and_then(super::battle_revival::Request::from_observation)
                == Some(*q)
        });
        self.state.canonical_revival.set(proof.filter(|_| enabled));
        if let Some(button) = self.state.confirm_death.get() {
            unsafe {
                let _ = ShowWindow(button, if proof.is_some() { SW_SHOWNA } else { SW_HIDE });
                let _ = EnableWindow(button, proof.is_some() && enabled);
            }
        }
    }
    pub fn update_honor(&self, available: bool, can_cast: bool) -> Result<()> {
        self.state.can_basic_heal.set(can_cast);
        if self.state.basic_heal_available.replace(available) != available {
            refresh_magic(&self.state)?;
        }
        Ok(())
    }
    pub fn update_disintegrate(&self, available: bool, can_cast: bool) -> Result<()> {
        self.state.can_disintegrate.set(available && can_cast);
        if self.state.disintegrate_available.replace(available) != available {
            refresh_magic(&self.state)?;
        }
        Ok(())
    }
    pub fn update_items(
        &self,
        labels: [String; 3],
        available: bool,
        can_use: [bool; 3],
    ) -> Result<()> {
        self.update_supplies(
            std::array::from_fn(|i| {
                if i < 3 {
                    labels[i].clone()
                } else {
                    String::new()
                }
            }),
            available,
            false,
            std::array::from_fn(|i| i < 3 && can_use[i]),
        )
    }
    pub fn update_supplies(
        &self,
        labels: [String; 7],
        available: bool,
        timed_available: bool,
        can_use: [bool; 7],
    ) -> Result<()> {
        self.state.can_use.set(std::array::from_fn(|i| {
            available && (i < 3 || timed_available) && can_use[i]
        }));
        if let Some(buttons) = self.state.items.get() {
            let previous = self.state.item_labels.borrow().clone();
            for (i, button) in buttons.iter().enumerate() {
                unsafe {
                    let visible = available && (i < 3 || timed_available);
                    let _ = ShowWindow(*button, if visible { SW_SHOWNA } else { SW_HIDE });
                    let _ = EnableWindow(*button, visible && can_use[i]);
                    if previous[i] != labels[i] {
                        let wide: Vec<u16> = labels[i].encode_utf16().chain(Some(0)).collect();
                        ensure!(
                            wide.len() <= 1025 && !labels[i].contains('\0'),
                            "supply label bound"
                        );
                        SetWindowTextW(*button, PCWSTR(wide.as_ptr()))?;
                        let mut actual = vec![0; wide.len()];
                        ensure!(
                            GetWindowTextLengthW(*button) as usize == wide.len() - 1
                                && GetWindowTextW(*button, &mut actual) as usize == wide.len() - 1
                                && actual == wide,
                            "supply label truncated"
                        );
                    }
                }
            }
            *self.state.item_labels.borrow_mut() = labels;
        }
        Ok(())
    }
    pub fn light_arrow_available(&self) -> bool {
        self.state.light_arrow_available.get()
    }
    pub fn update_ammo(
        &self,
        labels: [String; 2],
        available: bool,
        enabled: [bool; 2],
    ) -> Result<()> {
        self.state
            .can_select_ammo
            .set(std::array::from_fn(|i| available && enabled[i]));
        if let Some(buttons) = self.state.ammo.get() {
            for (i, button) in buttons.iter().enumerate() {
                unsafe {
                    let _ = ShowWindow(*button, if available { SW_SHOWNA } else { SW_HIDE });
                    let _ = EnableWindow(*button, available && enabled[i]);
                    let wide: Vec<u16> = labels[i].encode_utf16().chain(Some(0)).collect();
                    ensure!(
                        wide.len() <= 1025 && !labels[i].contains('\0'),
                        "arrow label bound"
                    );
                    SetWindowTextW(*button, PCWSTR(wide.as_ptr()))?;
                    let mut actual = vec![0; wide.len()];
                    ensure!(
                        GetWindowTextLengthW(*button) as usize == wide.len() - 1
                            && GetWindowTextW(*button, &mut actual) as usize == wide.len() - 1
                            && actual == wide,
                        "arrow label truncated"
                    );
                }
            }
        }
        Ok(())
    }
    pub fn invalidate_input(&self) {
        invalidate_input(&self.state);
    }
    pub fn window_handle(&self) -> Option<HWND> {
        self.state.hwnd.get()
    }
    /// Route the owner's inspection buttons into this same live world.
    /// The caller releases all owner RefCell borrows before native re-entry.
    pub(crate) fn show_details_window(hwnd: HWND, page: DetailsPage) -> Result<()> {
        unsafe {
            ensure!(IsWindow(Some(hwnd)).as_bool(), "world window retired");
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
            ensure!(!pointer.is_null(), "world window state unavailable");
            let state = &*pointer;
            ensure!(
                state.hwnd.get() == Some(hwnd) && !state.released.get(),
                "world window retired"
            );
            let _ = ShowWindow(
                hwnd,
                if IsIconic(hwnd).as_bool() {
                    SW_RESTORE
                } else {
                    SW_SHOW
                },
            );
            let _ = SetForegroundWindow(hwnd);
            show_details(state, Some(page));
        }
        Ok(())
    }
    /// The lobby owner may regain focus while a world is still attached.
    /// Esc closes inspection first, otherwise requests the same confirmed Leave
    /// as the world button. Preserve F10/F11 before the owner's system menu.
    /// The caller has already
    /// checked that the message belongs to the lobby or one of its children.
    pub(crate) fn pre_translate_owner_shortcut(hwnd: HWND, message: &MSG) -> bool {
        unsafe {
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
            if pointer.is_null() {
                return false;
            }
            let state = &*pointer;
            state.hwnd.get() == Some(hwnd)
                && handle_owner_shortcut(
                    state,
                    message.message,
                    message.wParam.0,
                    message.lParam.0 & (1 << 30) != 0,
                )
        }
    }
    /// Consumes world/child keyboard messages before the parent's dialog and global Esc routes.
    pub fn pre_translate(&self, message: &MSG) -> bool {
        let Some(hwnd) = self.window_handle() else {
            return false;
        };
        Self::pre_translate_window(hwnd, message)
    }
    pub fn pre_translate_window(hwnd: HWND, message: &MSG) -> bool {
        unsafe {
            if message.hwnd != hwnd && !IsChild(hwnd, message.hwnd).as_bool() {
                return false;
            }
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
            if pointer.is_null() {
                return false;
            }
            if let Some(page) = details_mnemonic(message) {
                // Alt+C/I/P work from either the lobby owner or this world,
                // including an open details EDIT/COMBOBOX. Ordinary C remains
                // copy/type-ahead inside controls; holding Alt+I cannot keep
                // resetting selection and equipment proofs on every repeat.
                if message.lParam.0 & (1 << 30) == 0 {
                    show_details(&*pointer, Some(page));
                }
                return true;
            }
            if state_has_legacy_shortcuts(&*pointer)
                && matches!(message.message, WM_KEYDOWN | WM_SYSKEYDOWN) && message.wParam.0 == 0x79 {
                if message.lParam.0 & (1 << 30) == 0 {
                    request_settings(&*pointer);
                }
                return true;
            }
            if state_has_legacy_shortcuts(&*pointer)
                && matches!(message.message, WM_KEYDOWN | WM_SYSKEYDOWN) && message.wParam.0 == 0x7a {
                if message.lParam.0 & (1 << 30) == 0 {
                    request_chat(&*pointer);
                }
                return true;
            }
            if matches!(message.message, WM_SYSKEYDOWN | WM_SYSKEYUP)
                && hotbar::key_slot(message.wParam.0).is_some()
                && !state_has_legacy_shortcuts(&*pointer) {
                if message.message == WM_SYSKEYDOWN && message.hwnd == hwnd
                    && (*pointer).details_page.get().is_none()
                    && !shortcut_modified() && message.lParam.0 & (1 << 30) == 0 {
                    activate_hotbar(&*pointer, hotbar::key_slot(message.wParam.0).unwrap());
                }
                return true;
            }
            if message.message == WM_KEYDOWN {
                let state = &*pointer;
                // A read-only inspection page owns its editing/navigation keys.
                // Escape closes this page, never the match; copy/select/scroll
                // cannot leak into movement, casting or the parent's logout.
                if handle_details_escape(state, message.wParam.0, message.lParam.0 & (1 << 30) != 0)
                {
                    return true;
                }
                if Some(message.hwnd) == state.status_control.get() {
                    match status_text::key_route(message.wParam.0) {
                        status_text::KeyRoute::FocusWorld => {
                            let _ = SetFocus(Some(hwnd));
                            return true;
                        }
                        status_text::KeyRoute::Native => {
                            let _ = TranslateMessage(message);
                            DispatchMessageW(message);
                            return true;
                        }
                        status_text::KeyRoute::Tab => {}
                    }
                }
                if message.wParam.0 == 0x0d {
                    let focus = windows::Win32::UI::Input::KeyboardAndMouse::GetFocus();
                    if button_handles(state).contains(&focus) && IsWindowEnabled(focus).as_bool() {
                        if message.lParam.0 & (1 << 30) == 0 {
                            SendMessageW(focus, BM_CLICK, None, None);
                        }
                        return true;
                    }
                }
                if let Some(slot) = hotbar::key_slot(message.wParam.0)
                    .filter(|_| state.hotbar.borrow().is_some()) {
                    if hotbar_bind_modifier() && matches!(state.details_page.get(), Some(DetailsPage::Inventory | DetailsPage::Skills)) {
                        if message.lParam.0 & (1 << 30) == 0 {
                            if state.details_page.get() == Some(DetailsPage::Skills) { bind_skill_hotbar(state, slot); }
                            else { bind_hotbar(state, slot); }
                        }
                        return true;
                    }
                    if state.details_page.get().is_some() || message.hwnd != hwnd {
                        let _ = TranslateMessage(message);
                        DispatchMessageW(message);
                        return true;
                    }
                    if !shortcut_modified() && message.lParam.0 & (1 << 30) == 0 {
                        activate_hotbar(state, slot);
                    }
                    return true;
                }
                if details_key_route(state.details_page.get().is_some(), message.wParam.0)
                    == DetailsKeyRoute::Native
                    || message.wParam.0 == 0x0d && state.details_page.get().is_some()
                {
                    let _ = TranslateMessage(message);
                    DispatchMessageW(message);
                    return true;
                }
                if let Some(page) = details_key(
                    message.wParam.0,
                    message.lParam.0 & (1 << 30) != 0,
                    shortcut_modified(),
                ) {
                    show_details(state, Some(page));
                    return true;
                }
                // Standard combo arrows/Return operate its selection, never the actor.
                if message.hwnd == (*pointer).magic.get().unwrap_or_default()
                    && combo_keeps_key(message.wParam.0)
                {
                    let _ = TranslateMessage(message);
                    DispatchMessageW(message);
                    return true;
                }
                if message.wParam.0 == 0x09 {
                    let previous =
                        windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(0x10) < 0;
                    if let Ok(next) = GetNextDlgTabItem(hwnd, Some(message.hwnd), previous) {
                        let _ = SetFocus(Some(next));
                    }
                    return true;
                }
                if let Some(action) =
                    key_action(message.wParam.0, message.lParam.0 & (1 << 30) != 0)
                {
                    queue(&*pointer, action);
                    return true;
                }
            }
            // Dispatch here so the parent's dialog handling never captures these inputs.
            let _ = TranslateMessage(message);
            DispatchMessageW(message);
            true
        }
    }
    /// Destruction is a parent-confirmed release, never an optimistic WM_CLOSE.
    pub fn release(&self) {
        self.state.released.set(true);
        if let Some(hwnd) = self.state.hwnd.get() {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }
    }
}
impl Drop for WorldWindow {
    fn drop(&mut self) {
        self.release();
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as _);
    }
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
    if pointer.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let state = &*pointer;
    match message {
        WM_COMMAND if skill_controls::owns(wparam.0 & 0xffff) => {
            state.skillbook.command(state, wparam, lparam);
            LRESULT(0)
        }
        WM_COMMAND if inventory_controls::owns(wparam.0 & 0xffff) => {
            state.inventory.command(state, wparam, lparam);
            if wparam.0 & 0xffff == INVENTORY_FIRST && wparam.0 >> 16 == CBN_SELCHANGE as usize {
                if let Some(selection) = state.inventory.selection() {
                    let changed = {
                        let model = state.inventory.model.borrow();
                        let mut panel = state.inventory_panel.borrow_mut();
                        model.as_ref().zip(panel.as_mut()).is_some_and(|(m,p)| p.focus(m, selection.key))
                    };
                    if changed { resize(hwnd, state); }
                    if changed { state.painted_inventory.set(None); }
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
            }
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == CHARACTER_EQUIPPED => {
            if wparam.0 >> 16 == CBN_SELCHANGE as usize
                && state.character_combo.get().is_some_and(|h| h.0 as isize == lparam.0)
                && state.details_page.get() == Some(DetailsPage::Character)
                && !state.released.get() {
                let combo = state.character_combo.get().unwrap();
                let index = SendMessageW(combo, CB_GETCURSEL, None, None).0;
                let id = usize::try_from(index).ok().and_then(|index|
                    state.character_model.borrow().as_ref()?.rows.get(index).map(|row| row.id));
                let valid = id.is_some_and(|id| {
                    let battle = state.painted_battle.borrow();
                    let pending = state.pending_battle.borrow();
                    let model = state.character_model.borrow();
                    let panel = state.character_panel.borrow();
                    battle.as_ref().zip(pending.as_ref()).zip(model.as_ref())
                        .zip(panel.as_ref()).zip(state.painted_character.get())
                        .is_some_and(|((((battle, pending), model), panel), painted)|
                            painted.text_choice(panel, model, battle.1,
                                super::battle_window::Stamp::of(&battle.0.observation),
                                super::battle_window::Stamp::of(&pending.observation), id))
                });
                if valid {
                    let id = id.unwrap();
                    state.character_selected.set(Some(id));
                    state.details_text.borrow_mut().clear();
                    let changed = {
                        let model = state.character_model.borrow();
                        let mut panel = state.character_panel.borrow_mut();
                        model.as_ref().zip(panel.as_mut()).is_some_and(|(model,panel)| panel.focus(model, id))
                    };
                    if changed { state.painted_character.set(None); }
                    let _ = InvalidateRect(Some(hwnd), None, false);
                } else {
                    sync_character_combo_selection(state);
                }
            }
            LRESULT(0)
        }
        WM_COMMAND if DetailsPage::from_button(wparam.0 & 0xffff).is_some() => {
            let id = wparam.0 & 0xffff;
            if wparam.0 >> 16 == BN_CLICKED as usize
                && state
                    .details_buttons
                    .get()
                    .is_some_and(|buttons| buttons[id - DETAILS_FIRST].0 as isize == lparam.0)
            {
                let page = DetailsPage::from_button(id);
                show_details(
                    state,
                    if state.details_page.get() == page {
                        None
                    } else {
                        page
                    },
                );
            }
            LRESULT(0)
        }
        WM_KEYDOWN | WM_SYSKEYDOWN if wparam.0 == 0x7a => {
            if lparam.0 & (1 << 30) == 0 {
                request_chat(state);
            }
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == CHAT => {
            if (wparam.0 >> 16) == BN_CLICKED as usize
                && state
                    .chat
                    .get()
                    .is_some_and(|button| button.0 as isize == lparam.0)
            {
                request_chat(state);
            }
            LRESULT(0)
        }
        WM_KEYDOWN | WM_SYSKEYDOWN if wparam.0 == 0x79 => {
            if lparam.0 & (1 << 30) == 0 {
                request_settings(state);
            }
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == SETTINGS => {
            if (wparam.0 >> 16) == BN_CLICKED as usize
                && state
                    .settings
                    .get()
                    .is_some_and(|button| button.0 as isize == lparam.0)
            {
                request_settings(state);
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if handle_details_escape(state, wparam.0, lparam.0 & (1 << 30) != 0) {
                return LRESULT(0);
            }
            if let Some(slot) = hotbar::key_slot(wparam.0).filter(|_| state.hotbar.borrow().is_some()) {
                if hotbar_bind_modifier()
                    && matches!(state.details_page.get(), Some(DetailsPage::Inventory | DetailsPage::Skills))
                    && lparam.0 & (1 << 30) == 0 {
                    if state.details_page.get() == Some(DetailsPage::Skills) { bind_skill_hotbar(state, slot); }
                    else { bind_hotbar(state, slot); }
                } else if state.details_page.get().is_none() && !shortcut_modified()
                    && lparam.0 & (1 << 30) == 0 {
                    activate_hotbar(state, slot);
                }
            } else if state.details_page.get().is_some() {
                // Text/navigation keys belong to the inspection control.
            } else if let Some(page) =
                details_key(wparam.0, lparam.0 & (1 << 30) != 0, shortcut_modified())
            {
                show_details(state, Some(page));
            } else if let Some(action) = key_action(wparam.0, lparam.0 & (1 << 30) != 0) {
                queue(state, action);
            }
            LRESULT(0)
        }
        WM_SYSKEYDOWN | WM_SYSKEYUP if !state_has_legacy_shortcuts(state)
            && hotbar::key_slot(wparam.0).is_some() => {
            if message == WM_SYSKEYDOWN && state.details_page.get().is_none()
                && !shortcut_modified() && lparam.0 & (1 << 30) == 0 {
                activate_hotbar(state, hotbar::key_slot(wparam.0).unwrap());
            }
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == LEAVE => {
            queue(state, Action::Leave);
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == CONFIRM_DEATH => {
            if let Some(proof) = state.canonical_revival.get() {
                queue(state, Action::CanonicalRevival(proof));
            } else if let Some(proof) = state.death_confirmation.get() {
                queue(state, Action::ConfirmDeath(proof));
            }
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == FIRE_STORM => {
            queue(state, Action::FireStorm);
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == GREATER_HASTE => {
            if button_notification(wparam, lparam, GREATER_HASTE, state.greater_haste.get()) {
                queue(state, Action::GreaterHaste);
            }
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == TELEPORT => {
            queue(state, Action::Teleport);
            LRESULT(0)
        }
        WM_COMMAND if (AMMO_FIRST..AMMO_FIRST + 2).contains(&(wparam.0 & 0xffff)) => {
            queue(
                state,
                Action::SelectAmmo(
                    super::ammo_protocol::UI_ITEMS[(wparam.0 & 0xffff) - AMMO_FIRST],
                ),
            );
            LRESULT(0)
        }
        WM_COMMAND if (USE_FIRST..USE_FIRST + 7).contains(&(wparam.0 & 0xffff)) => {
            let index = (wparam.0 & 0xffff) - USE_FIRST;
            queue(
                state,
                Action::UseItem(super::item_protocol::UI_ITEMS[index]),
            );
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == MAGIC && (wparam.0 >> 16) as u32 == CBN_SELCHANGE => {
            if state.magic.get() == Some(HWND(lparam.0 as *mut _)) {
                let selected = SendMessageW(state.magic.get().unwrap(), CB_GETCURSEL, None, None).0;
                select_magic(state, selected);
                queue(state, Action::CancelRoute);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            queue(state, Action::Leave);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let has_hotbar = state.hotbar.borrow().is_some();
            if has_hotbar {
                let hit = state.painted_battle.borrow().as_ref()
                    .and_then(|(_, view)| hotbar::hit(*view, mouse_point(lparam.0)));
                if let Some(slot) = hit {
                    if state.details_page.get().is_none() {
                        if let Some(slot) = slot { activate_hotbar(state, slot); }
                    }
                    return LRESULT(0);
                }
            }
            if state.details_page.get() == Some(DetailsPage::Inventory) {
                let painted = {
                    let painted = state.painted_battle.borrow();
                    let pending = state.pending_battle.borrow();
                    let model = state.inventory.model.borrow();
                    let panel = state.inventory_panel.borrow();
                    let inventory = state.painted_inventory.get();
                    painted.as_ref().zip(pending.as_ref()).zip(model.as_ref())
                        .zip(panel.as_ref()).zip(inventory)
                        .filter(|((((painted, pending), model), panel), inventory)| {
                            model.stamp == super::battle_window::Stamp::of(&painted.0.observation)
                                && model.stamp == super::battle_window::Stamp::of(&pending.observation)
                                && inventory.valid(panel, model, painted.1)
                        })
                        .map(|(_, inventory)| inventory)
                };
                if let Some(painted) = painted {
                    let point = mouse_point(lparam.0);
                    let hit = inventory_panel::hit(painted.viewport, point);
                    if let Some(hit) = hit {
                        match hit {
                            inventory_panel::Hit::Slot(slot) => {
                                if let Some(key) = painted.key(slot) {
                                    state.inventory.select_key(state, key);
                                } else {
                                    state.inventory.clear_selection(state);
                                }
                            }
                            inventory_panel::Hit::Previous | inventory_panel::Hit::Next => {
                                let changed = {
                                    let model = state.inventory.model.borrow();
                                    let mut panel = state.inventory_panel.borrow_mut();
                                    model.as_ref().zip(panel.as_mut()).is_some_and(|(m,p)|
                                        p.turn(m, if hit == inventory_panel::Hit::Next { 1 } else { -1 }))
                                };
                                if changed {
                                    state.painted_inventory.set(None);
                                    state.inventory.clear_selection(state);
                                    resize(hwnd, state);
                                }
                            }
                            inventory_panel::Hit::Close => show_details(state, None),
                            inventory_panel::Hit::Body => {}
                        }
                        let _ = InvalidateRect(Some(hwnd), None, false);
                    }
                }
                return LRESULT(0);
            }
            if state.details_page.get() == Some(DetailsPage::Character) {
                let painted = {
                    let battle = state.painted_battle.borrow();
                    let pending = state.pending_battle.borrow();
                    let model = state.character_model.borrow();
                    let panel = state.character_panel.borrow();
                    battle.as_ref().zip(pending.as_ref()).zip(model.as_ref())
                        .zip(panel.as_ref()).zip(state.painted_character.get())
                        .filter(|((((battle, pending), model), panel), painted)| {
                            model.stamp == super::battle_window::Stamp::of(&battle.0.observation)
                                && model.stamp == super::battle_window::Stamp::of(&pending.observation)
                                && painted.valid(panel, model, battle.1)
                        })
                        .map(|(_, painted)| painted)
                };
                if let Some(painted) = painted {
                    if let Some(hit) = character_panel::hit(painted.viewport, mouse_point(lparam.0)) {
                        match hit {
                            character_panel::Hit::Slot(slot) => {
                                state.character_selected.set(painted.id(slot));
                                sync_character_combo_selection(state);
                                state.details_text.borrow_mut().clear();
                            }
                            character_panel::Hit::Previous | character_panel::Hit::Next => {
                                let (changed, first) = {
                                    let model = state.character_model.borrow();
                                    let mut panel = state.character_panel.borrow_mut();
                                    model.as_ref().zip(panel.as_mut()).map_or((false, None), |(model,panel)| {
                                        let changed = panel.turn(model,
                                            if hit == character_panel::Hit::Next { 1 } else { -1 });
                                        (changed, panel.first_visible(model))
                                    })
                                };
                                if changed {
                                    state.character_selected.set(first);
                                    sync_character_combo_selection(state);
                                    state.painted_character.set(None);
                                    state.details_text.borrow_mut().clear();
                                }
                            }
                            character_panel::Hit::Close => show_details(state, None),
                            character_panel::Hit::Body => {},
                        }
                        let _ = InvalidateRect(Some(hwnd), None, false);
                    }
                }
                return LRESULT(0);
            }
            if state.details_page.get() == Some(DetailsPage::Skills) {
                let painted = {
                    let battle = state.painted_battle.borrow();
                    let pending = state.pending_battle.borrow();
                    let model = state.skillbook.model.borrow();
                    let panel = state.skill_panel.borrow();
                    battle.as_ref().zip(pending.as_ref()).zip(model.as_ref())
                        .zip(panel.as_ref()).zip(state.painted_skill.get())
                        .filter(|((((battle, pending), model), panel), painted)| {
                            model.stamp == super::battle_window::Stamp::of(&battle.0.observation)
                                && model.stamp == super::battle_window::Stamp::of(&pending.observation)
                                && painted.valid(panel, model, battle.1)
                        })
                        .map(|(_, painted)| painted)
                };
                if let Some(painted) = painted {
                    if let Some(hit) = skill_panel::hit(painted.viewport, mouse_point(lparam.0)) {
                        match hit {
                            skill_panel::Hit::Slot(slot) => {
                                if let Some(id) = painted.id(slot) {
                                    state.skillbook.select_id(state, id);
                                    let _ = InvalidateRect(Some(hwnd), None, false);
                                }
                            }
                            skill_panel::Hit::Close => show_details(state, None),
                            skill_panel::Hit::Body => {},
                        }
                    }
                }
                return LRESULT(0);
            }
            if state.details_page.get().is_some() {
                return LRESULT(0);
            }
            let _ = SetFocus(Some(hwnd));
            if let Some((surface, viewport)) = state.painted_battle.borrow().as_ref() {
                use super::battle_window::Skill;
                let skill = if state.slow_selected.get() {
                    Skill::Slow
                } else if state.haste_selected.get() {
                    Skill::Haste
                } else if state.disintegrate_selected.get() {
                    Skill::Disintegrate
                } else if state.basic_heal_selected.get() {
                    Skill::Heal
                } else if state.light_arrow_selected.get() {
                    Skill::LightArrow
                } else {
                    Skill::Melee
                };
                if let Some(hit) =
                    super::battle_window::hit(surface, *viewport, mouse_point(lparam.0), skill)
                {
                    queue(state, Action::Canonical(hit.intent));
                }
                return LRESULT(0);
            }
            if let Some(action) = disintegrate_hit(
                healing_hit(
                    selected_hit(
                        state.painted_hit.get().and_then(|hit| {
                            combat_hit_action(
                                hit,
                                state.painted_combat.borrow().as_ref(),
                                mouse_point(lparam.0),
                            )
                        }),
                        state.light_arrow_selected.get(),
                    ),
                    state.basic_heal_selected.get(),
                ),
                state.disintegrate_selected.get(),
                state.painted_combat.borrow().as_ref(),
            ) {
                queue(state, action);
            }
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            if state.details_page.get().is_some() {
                return LRESULT(0);
            }
            state.light_arrow_selected.set(false);
            state.basic_heal_selected.set(false);
            state.disintegrate_selected.set(false);
            state.haste_selected.set(false);
            state.slow_selected.set(false);
            if let Some(combo) = state.magic.get() {
                let _ = SendMessageW(combo, CB_SETCURSEL, Some(WPARAM(0)), None);
            }
            queue(state, Action::CancelRoute);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            // No network or gameplay command on hover. WM_PAINT consumes the
            // latest already-rendered frame and re-reads the actual cursor.
            let _ = TrackMouseEvent(&mut TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            });
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_MOUSELEAVE | WM_MOVE | WM_ACTIVATE => {
            let _ = InvalidateRect(Some(hwnd), None, false);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        WM_SIZE => {
            state.painted_inventory.set(None);
            state.painted_skill.set(None);
            state.painted_character.set(None);
            state.painted_hotbar.set(None);
            resize(hwnd, state);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            state.painted_inventory.set(None);
            state.painted_skill.set(None);
            state.painted_character.set(None);
            state.painted_hotbar.set(None);
            let _ = replace_details_font(state);
            if lparam.0 != 0 {
                let bounds = &*(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    bounds.left,
                    bounds.top,
                    bounds.right - bounds.left,
                    bounds.bottom - bounds.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            resize(hwnd, state);
            LRESULT(0)
        }
        WM_PAINT => {
            paint(hwnd, state);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_DESTROY => {
            state.hwnd.set(None);
            if !state.released.get() {
                state.action.set(Some(Action::Leave));
            }
            LRESULT(0)
        }
        WM_NCDESTROY => {
            let font = state.details_font.replace(HFONT::default());
            if !font.0.is_null() {
                let _ = DeleteObject(font.into());
            }
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}
unsafe fn replace_details_font(state: &State) -> Result<()> {
    let Some(hwnd) = state.hwnd.get() else {
        return Ok(());
    };
    let font = CreateFontW(
        -scale(16, GetDpiForWindow(hwnd)),
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        0,
        w!("Microsoft JhengHei UI"),
    );
    ensure!(!font.0.is_null(), "world details font unavailable");
    for control in state
        .details
        .get()
        .into_iter()
        .chain(state.details_buttons.get().into_iter().flatten())
        .chain(state.inventory.handles.get().into_iter().flatten())
        .chain(state.skillbook.handles.get().into_iter().flatten())
        .chain(state.character_combo.get())
        .chain(state.status_control.get())
    {
        SendMessageW(
            control,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
    let old = state.details_font.replace(font);
    if !old.0.is_null() {
        let _ = DeleteObject(old.into());
    }
    Ok(())
}
fn button_handles(state: &State) -> Vec<HWND> {
    [
        state.leave.get(),
        state.teleport.get(),
        state.fire_storm.get(),
        state.greater_haste.get(),
        state.confirm_death.get(),
        state.settings.get(),
        state.chat.get(),
    ]
    .into_iter()
    .flatten()
    .chain(state.items.get().into_iter().flatten())
    .chain(state.ammo.get().into_iter().flatten())
    .chain(state.details_buttons.get().into_iter().flatten())
    .chain(state.inventory.handles.get().into_iter().flatten().skip(1))
    .chain(state.skillbook.handles.get().into_iter().flatten().skip(1))
    .collect()
}
unsafe fn handle_owner_shortcut(state: &State, message: u32, key: usize, repeated: bool) -> bool {
    if state.released.get() {
        return false;
    }
    if state.hotbar.borrow().is_some() && hotbar::key_slot(key).is_some()
        && matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP) {
        // The lobby owner has focus: consume, but never cast or open settings/chat.
        return true;
    }
    if matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN) && matches!(key, 0x79 | 0x7a) {
        if !repeated {
            if key == 0x79 {
                request_settings(state);
            } else {
                request_chat(state);
            }
        }
        return true;
    }
    if message == WM_KEYDOWN && key == 0x1b {
        if !handle_details_escape(state, key, repeated) {
            if let Some(action) = key_action(key, repeated) {
                queue(state, action);
            }
        }
        // Even a repeat without an open page belongs to this world. It must
        // never fall through to the owner's whole-client shutdown route.
        return true;
    }
    false
}
fn state_has_legacy_shortcuts(state: &State) -> bool {
    state.hotbar.borrow().is_none()
}
unsafe fn handle_details_escape(state: &State, key: usize, repeated: bool) -> bool {
    if key != 0x1b {
        return false;
    }
    if !repeated {
        state.details_closed_by_escape.set(false);
    }
    if state.details_closed_by_escape.get() {
        return true;
    }
    if details_key_route(state.details_page.get().is_some(), key) == DetailsKeyRoute::Close {
        state.details_closed_by_escape.set(true);
        show_details(state, None);
        return true;
    }
    false
}
unsafe fn show_details(state: &State, page: Option<DetailsPage>) {
    state.details_page.set(page);
    state.painted_inventory.set(None);
    state.painted_skill.set(None);
    state.painted_character.set(None);
    let _ = state.inventory.update(state, None);
    let _ = state.skillbook.update(state, None);
    if let Some(combo) = state.character_combo.get() {
        let _ = ShowWindow(combo, SW_HIDE);
    }
    state.details_text.borrow_mut().clear();
    if let Some(edit) = state.details.get() {
        let _ = SetWindowTextW(edit, w!("正在讀取目前場次的確認資料…（Esc 關閉檢視）"));
        let _ = ShowWindow(edit, if page.is_some() { SW_SHOW } else { SW_HIDE });
        let _ = SetFocus(if page.is_some() {
            Some(edit)
        } else {
            state.hwnd.get()
        });
    }
    if let Some(hwnd) = state.hwnd.get() {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}
fn invalidate_input(state: &State) {
    // Clear queued equipment proofs with invalidated pixels as well.
    state.inventory.clear_proofs();
    state.skillbook.clear_proofs();
    // A queued mutation cannot survive invalidated pixels (DPI, teleport,
    // respawn, resize). Explicit leaving/cancellation must remain available.
    if !matches!(
        state.action.get(),
        Some(
            Action::Leave
                | Action::CancelRoute
                | Action::Canonical(super::battle_window::Intent {
                    kind: super::battle_window::Kind::Cancel,
                    ..
                })
        )
    ) {
        state.action.set(None);
    }
    state.pending_battle.borrow_mut().take();
    state.painted_inventory.set(None);
    state.painted_skill.set(None);
    state.painted_character.set(None);
    state.painted_hotbar.set(None);
    state.painted_battle.borrow_mut().take();
    state.painted_hit.set(None);
    *state.painted_combat.borrow_mut() = None;
    state.pending_hit.set(None);
    *state.pending_combat.borrow_mut() = None;
    *state.pending_players.borrow_mut() = None;
    *state.painted_players.borrow_mut() = None;
    state.can_move.set(false);
    state.can_attack.set(false);
    state.can_light_arrow.set(false);
    state.can_basic_heal.set(false);
    state.can_disintegrate.set(false);
    state.can_haste.set(false);
    state.can_slow.set(false);
    state.can_fire_storm.set(false);
    state.can_greater_haste.set(false);
    state.death_confirmation.set(None);
    state.canonical_revival.set(None);
    state.can_use.set([false; 7]);
    state.can_select_ammo.set([false; 2]);
}
fn queue(state: &State, action: Action) {
    if action == Action::Leave {
        state.settings_requested.set(false);
        state.chat_requested.set(false);
    }
    if action == Action::Leave
        || matches!(
            action,
            Action::CancelRoute
                | Action::Canonical(super::battle_window::Intent {
                    kind: super::battle_window::Kind::Cancel,
                    ..
                })
        ) && state.action.get() != Some(Action::Leave)
        || matches!(action, Action::Teleport | Action::FireStorm)
            && state.action.get() != Some(Action::Leave)
        || (if let Action::UseItem(id) = action {
            super::item_protocol::UI_ITEMS
                .iter()
                .position(|k| *k == id)
                .is_some_and(|i| state.can_use.get()[i])
        } else if let Action::SelectAmmo(id) = action {
            super::ammo_protocol::UI_ITEMS
                .iter()
                .position(|k| *k == id)
                .is_some_and(|i| state.can_select_ammo.get()[i])
        } else if let Action::CanonicalInventory(intent) = action {
            state.hotbar.borrow().as_ref().is_some_and(|bar|
                bar.model.as_ref().is_some_and(|m| m.stamp == intent.stamp
                    && m.rows.iter().any(|r| r.local == Some(intent))))
                && state.painted_battle.borrow().as_ref().is_some_and(|(surface, _)|
                    super::battle_window::Stamp::of(&surface.observation) == intent.stamp)
        } else if let Action::CanonicalSkill(choice) = action {
            let bar = state.hotbar.borrow();
            let painted = state.painted_battle.borrow();
            let pending = state.pending_battle.borrow();
            skill_choice_eligible(choice, bar.as_ref().and_then(|b| b.skills.as_ref()),
                painted.as_ref().map(|(s, _)| super::battle_window::Stamp::of(&s.observation)),
                pending.as_ref().map(|s| super::battle_window::Stamp::of(&s.observation)))
        } else if let Action::Canonical(intent) = action {
            match intent.kind {
                super::battle_window::Kind::Cancel => true,
                super::battle_window::Kind::Navigate(_) | super::battle_window::Kind::Portal => {
                    state.can_move.get()
                }
                super::battle_window::Kind::Attack(_) => state.can_attack.get(),
                super::battle_window::Kind::LightArrow(_) => state.can_light_arrow.get(),
                super::battle_window::Kind::Disintegrate(_) => state.can_disintegrate.get(),
                super::battle_window::Kind::Heal(_) => state.can_basic_heal.get(),
                super::battle_window::Kind::Haste(_) => state.can_haste.get(),
                super::battle_window::Kind::Slow(_) => state.can_slow.get(),
                // Local intents are stamped only after ordinary admission
                // below, never injected back through the point-target lane.
                super::battle_window::Kind::Local(_) => false,
            }
        } else if matches!(
            action,
            Action::CastLightArrow { .. } | Action::CastNpcLightArrow { .. }
        ) {
            state.can_light_arrow.get()
        } else if action == Action::FireStorm {
            state.can_fire_storm.get()
        } else if action == Action::GreaterHaste {
            state.can_greater_haste.get() && state.painted_battle.borrow().is_some()
        } else if matches!(action, Action::CastBasicHeal { .. }) {
            state.can_basic_heal.get()
        } else if matches!(
            action,
            Action::CastDisintegrate { .. } | Action::CastNpcDisintegrate { .. }
        ) {
            state.can_disintegrate.get()
        } else if let Action::ConfirmDeath(proof) = action {
            state.death_confirmation.get() == Some(proof)
        } else if let Action::CanonicalRevival(proof) = action {
            state.canonical_revival.get() == Some(proof)
        } else if matches!(
            action,
            Action::Attack { .. }
                | Action::AttackNpc { .. }
                | Action::AttackQuestObjective { .. }
                | Action::AttackTimeAttackTower { .. }
                | Action::AttackSiegeTower { .. }
        ) {
            state.can_attack.get()
        } else {
            state.can_move.get()
        }) && state.action.get().is_none()
    {
        let painted = state.painted_battle.borrow();
        let action = super::battle_window::stamp_local_action(
            action,
            painted.as_ref().map(|(s, _)| &s.observation),
        );
        state.action.set(Some(action));
        if matches!(action, Action::CanonicalInventory(_) | Action::CanonicalSkill(_)) {
            if let Some(bar) = state.hotbar.borrow_mut().as_mut() { bar.clear_proofs(); }
            state.inventory.clear_proofs();
            state.skillbook.clear_proofs();
        }
        state.can_move.set(false);
        state.can_attack.set(false);
        state.can_light_arrow.set(false);
        state.can_basic_heal.set(false);
        state.can_disintegrate.set(false);
        state.can_haste.set(false);
        state.can_slow.set(false);
        state.can_fire_storm.set(false);
        state.can_greater_haste.set(false);
        state.death_confirmation.set(None);
        state.canonical_revival.set(None);
        state.can_use.set([false; 7]);
        state.can_select_ammo.set([false; 2]);
    }
}
fn request_chat(state: &State) {
    if !state.released.get() && state.action.get() != Some(Action::Leave) {
        state.chat_requested.set(true);
    }
}
fn request_settings(state: &State) {
    if !state.released.get() && state.action.get() != Some(Action::Leave) {
        state.settings_requested.set(true);
    }
}
fn magic_choices(light: bool, heal: bool, dis: bool) -> Vec<(u32, &'static str)> {
    let mut rows = vec![(0, "普攻（左鍵目標）")];
    if light {
        rows.push((4, "已學光箭（左鍵目標／右鍵取消）"));
    }
    if heal {
        rows.push((1, "初級治癒術（左鍵合格目標／右鍵取消）"));
    }
    if dis {
        rows.push((77, "究極光裂術（左鍵角色／右鍵取消）"));
    }
    rows
}
fn magic_rows(state: &State) -> Vec<(u32, &'static str)> {
    let mut rows = magic_choices(
        state.light_arrow_available.get(),
        state.basic_heal_available.get(),
        state.disintegrate_available.get(),
    );
    if state.haste_available.get() {
        rows.push((43, "加速術（左鍵自己／角色／NPC，右鍵取消）"));
    }
    if state.slow_available.get() {
        rows.push((29, "緩速術（需魔法寶石；左鍵目標／右鍵取消）"));
    }
    rows
}
fn select_magic(state: &State, index: isize) {
    let rows = magic_rows(state);
    let skill = usize::try_from(index)
        .ok()
        .and_then(|i| rows.get(i))
        .map_or(0, |r| r.0);
    state.light_arrow_selected.set(skill == 4);
    state.basic_heal_selected.set(skill == 1);
    state.disintegrate_selected.set(skill == 77);
    state.haste_selected.set(skill == 43);
    state.slow_selected.set(skill == 29);
}
fn refresh_magic(state: &State) -> Result<()> {
    let Some(combo) = state.magic.get() else {
        return Ok(());
    };
    let rows = magic_rows(state);
    let skill = if state.slow_selected.get() {
        29
    } else if state.haste_selected.get() {
        43
    } else if state.disintegrate_selected.get() {
        77
    } else if state.basic_heal_selected.get() {
        1
    } else if state.light_arrow_selected.get() {
        4
    } else {
        0
    };
    let selected = rows.iter().position(|r| r.0 == skill).unwrap_or(0);
    select_magic(state, selected as isize);
    unsafe {
        SendMessageW(combo, CB_RESETCONTENT, None, None);
        for (index, (_, row)) in rows.iter().enumerate() {
            let text: Vec<u16> = row.encode_utf16().chain(Some(0)).collect();
            ensure!(
                SendMessageW(
                    combo,
                    CB_ADDSTRING,
                    None,
                    Some(LPARAM(text.as_ptr() as isize))
                )
                .0 == index as isize,
                "cannot populate confirmed skill"
            );
        }
        ensure!(
            SendMessageW(combo, CB_SETCURSEL, Some(WPARAM(selected)), None).0 == selected as isize,
            "cannot select confirmed skill"
        );
    }
    Ok(())
}
unsafe fn resize(hwnd: HWND, state: &State) {
    state.painted_hit.set(None); // New viewport is not hittable until actually painted.
    *state.painted_combat.borrow_mut() = None;
    *state.painted_players.borrow_mut() = None;
    let mut bounds = RECT::default();
    if GetClientRect(hwnd, &mut bounds).is_err() {
        return;
    }
    let dpi = GetDpiForWindow(hwnd).max(96);
    let controls = footer_controls(bounds.right, dpi);
    if let Some(control) = state.status_control.get() {
        let r = status_text::bounds(bounds.right, bounds.bottom, dpi);
        let _ = SetWindowPos(
            control,
            None,
            r.left,
            r.top,
            r.right - r.left,
            r.bottom - r.top,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(buttons) = state.details_buttons.get() {
        for (button, (x, width)) in buttons.into_iter().zip(controls) {
            let _ = SetWindowPos(
                button,
                None,
                x,
                (bounds.bottom - scale(302, dpi)).max(0),
                width,
                scale(28, dpi),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
    if let Some(edit) = state.details.get() {
        let panel_visible = state.details_page.get() == Some(DetailsPage::Inventory)
            && state.inventory.model.borrow().is_some()
            && state.inventory_panel.borrow().as_ref().is_some_and(|panel| panel.ready());
        let skill_panel_visible = state.details_page.get() == Some(DetailsPage::Skills)
            && state.skillbook.model.borrow().is_some()
            && state.skill_panel.borrow().as_ref().is_some_and(|panel| panel.ready());
        let character_panel_visible = state.details_page.get() == Some(DetailsPage::Character)
            && state.character_model.borrow().is_some()
            && state.character_panel.borrow().is_some();
        let inventory_height = if (!panel_visible && state.details_page.get() == Some(DetailsPage::Inventory)
            && state.inventory.model.borrow().is_some())
            || (!skill_panel_visible && state.details_page.get() == Some(DetailsPage::Skills)
                && state.skillbook.model.borrow().is_some())
            || character_panel_visible
        {
            scale(38, dpi)
        } else {
            0
        };
        let (edit_left, edit_width) = if character_panel_visible {
            let left = Viewport::fit(bounds.right, (bounds.bottom - scale(FOOTER, dpi)).max(0),
                WIDTH as u32, HEIGHT as u32)
                .map(|view| view.x + view.width * character_panel::RIGHT / WIDTH)
                .unwrap_or(0);
            (left, bounds.right - left)
        } else if panel_visible || skill_panel_visible {
            Viewport::fit(bounds.right, (bounds.bottom - scale(FOOTER, dpi)).max(0),
                WIDTH as u32, HEIGHT as u32)
                .map(|view| view.x + view.width * if skill_panel_visible {
                    skill_panel::LEFT
                } else { inventory_panel::LEFT } / WIDTH)
                .map(|width| (0, width)).unwrap_or((0, bounds.right))
        } else { (0, bounds.right) };
        let _ = SetWindowPos(
            edit,
            None,
            edit_left,
            inventory_height,
            edit_width.max(0),
            (bounds.bottom - scale(FOOTER, dpi) - inventory_height).max(0),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(combo) = state.character_combo.get() {
        let view = Viewport::fit(bounds.right, (bounds.bottom - scale(FOOTER, dpi)).max(0),
            WIDTH as u32, HEIGHT as u32);
        let left = view.map(|view| view.x + view.width * character_panel::RIGHT / WIDTH)
            .unwrap_or(0);
        let _ = SetWindowPos(combo, None, left + scale(4, dpi), scale(4, dpi),
            (bounds.right - left - scale(8, dpi)).max(0), scale(280, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE);
    }
    state.inventory.layout(bounds.right, (bounds.bottom - scale(FOOTER, dpi) + scale(4, dpi)).max(0), dpi);
    state.skillbook.layout(bounds.right, dpi);
    let utilities = utility_controls(bounds.right, dpi);
    if let Some(button) = state.chat.get() {
        let _ = SetWindowPos(
            button,
            None,
            utilities[1].0,
            (bounds.bottom - scale(268, dpi)).max(0),
            utilities[1].1,
            scale(28, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(button) = state.settings.get() {
        let _ = SetWindowPos(
            button,
            None,
            utilities[0].0,
            (bounds.bottom - scale(268, dpi)).max(0),
            utilities[0].1,
            scale(28, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(button) = state.fire_storm.get() {
        let _ = SetWindowPos(
            button,
            None,
            utilities[2].0,
            (bounds.bottom - scale(268, dpi)).max(0),
            utilities[2].1,
            scale(28, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(button) = state.confirm_death.get() {
        // This control retains its separate in-world death-confirmation bounds.
        let (x, y, width, height) = death_control(bounds.right, bounds.bottom, dpi);
        let _ = SetWindowPos(
            button,
            None,
            x,
            y,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(button) = state.greater_haste.get() {
        let _ = SetWindowPos(
            button,
            None,
            utilities[3].0,
            (bounds.bottom - scale(268, dpi)).max(0),
            utilities[3].1,
            scale(28, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(buttons) = state.ammo.get() {
        for (i, button) in buttons.iter().enumerate() {
            let (x, width, y) = ammo_control(bounds.right, dpi, i, bounds.bottom);
            let _ = SetWindowPos(
                *button,
                None,
                x,
                y,
                width,
                scale(28, dpi),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
    if let Some(items) = state.items.get() {
        for (i, button) in items.iter().enumerate() {
            let (x, width, y) = supply_control(bounds.right, dpi, i, bounds.bottom);
            let _ = SetWindowPos(
                *button,
                None,
                x,
                y,
                width,
                scale(28, dpi),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
    if let Some(button) = state.leave.get() {
        let _ = SetWindowPos(
            button,
            None,
            controls[0].0,
            (bounds.bottom - scale(40, dpi)).max(0),
            controls[0].1,
            scale(28, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(button) = state.teleport.get() {
        let _ = SetWindowPos(
            button,
            None,
            controls[1].0,
            (bounds.bottom - scale(40, dpi)).max(0),
            controls[1].1,
            scale(28, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    if let Some(combo) = state.magic.get() {
        let _ = SetWindowPos(
            combo,
            None,
            controls[2].0,
            (bounds.bottom - scale(40, dpi)).max(0),
            controls[2].1,
            scale(160, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    let _ = InvalidateRect(Some(hwnd), None, false);
}
fn canonical_hud_line(view: Viewport, minimap: bool, index: usize, dpi: u32) -> Option<RECT> {
    if view.width <= 0 || view.height <= 0 {
        return None;
    }
    let margin = i64::from(scale(12, dpi));
    let left = i64::from(view.x)
        + if minimap {
            let image_x =
                i64::from(super::battle_minimap::X) + i64::from(super::battle_minimap::WIDTH) + 8;
            (image_x * i64::from(view.width) + 799) / 800
        } else {
            margin
        };
    let right = i64::from(view.x) + i64::from(view.width) - margin;
    let height = i64::from(scale(22, dpi));
    let top = i64::from(view.y) + margin + i64::try_from(index).ok()?.checked_mul(height)?;
    let bottom = top.checked_add(height)?;
    if left >= right
        || top < i64::from(view.y)
        || bottom > i64::from(view.y) + i64::from(view.height)
    {
        return None;
    }
    Some(RECT {
        left: left.try_into().ok()?,
        right: right.try_into().ok()?,
        top: top.try_into().ok()?,
        bottom: bottom.try_into().ok()?,
    })
}
fn canonical_hud_bar_rect(view: Viewport, bar: (i32, i32, i32, i32)) -> Option<RECT> {
    if view.width <= 0 || view.height <= 0 || bar.2 <= 0 || bar.3 <= 0
        || bar.0 < 0 || bar.1 < 0 || bar.0.checked_add(bar.2)? > 800
        || bar.1.checked_add(bar.3)? > 600
    {
        return None;
    }
    let x = |position: i32| i64::from(view.x) + i64::from(position) * i64::from(view.width) / 800;
    let y = |position: i32| i64::from(view.y) + i64::from(position) * i64::from(view.height) / 600;
    let left = x(bar.0);
    let right = x(bar.0 + bar.2);
    let top = y(bar.1);
    let bottom = y(bar.1 + bar.3);
    (left < right && top < bottom).then_some(RECT {
        left: left.try_into().ok()?,
        right: right.try_into().ok()?,
        top: top.try_into().ok()?,
        bottom: bottom.try_into().ok()?,
    })
}
struct HudFont { dc: HDC, font: HFONT, previous: HGDIOBJ }
impl HudFont {
    unsafe fn select(dc: HDC, bar_height: i32) -> Option<Self> {
        let usable = bar_height.checked_sub(2)?;
        if usable < 6 { return None; }
        let desired = (bar_height.saturating_mul(4) / 5).max(6);
        for pixels in (6..=desired).rev() {
            let font = CreateFontW(-pixels, 0, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 0,
                w!("Microsoft JhengHei UI"));
            if font.0.is_null() { return None; }
            let previous = SelectObject(dc, font.into());
            if previous.0.is_null() {
                let _ = DeleteObject(font.into());
                return None;
            }
            let selected = Self { dc, font, previous };
            if hud_text_extent(dc, "800/1000").is_some_and(|size| size.cy <= usable) {
                return Some(selected);
            }
            drop(selected);
        }
        None
    }
}
impl Drop for HudFont {
    fn drop(&mut self) {
        unsafe {
            let _ = SelectObject(self.dc, self.previous);
            let _ = DeleteObject(self.font.into());
        }
    }
}
unsafe fn hud_text_extent(dc: HDC, label: &str) -> Option<SIZE> {
    let text = label.encode_utf16().collect::<Vec<_>>();
    let mut size = SIZE::default();
    GetTextExtentPoint32W(dc, &text, &mut size).as_bool().then_some(size)
}
/// Only the native bar string may be shortened. The footer always retains
/// the complete signed 64-bit authority values from this same observation.
unsafe fn fitted_hud_text(dc: HDC, label: &str, width: i32) -> Option<Vec<u16>> {
    debug_assert!(label.is_ascii());
    let available = width.checked_sub(4)?;
    if hud_text_extent(dc, label)?.cx <= available {
        return Some(label.encode_utf16().collect());
    }
    if hud_text_extent(dc, "…")?.cx > available { return None; }
    let (mut low, mut high) = (0usize, label.len());
    while low < high {
        let mid = low + (high - low + 1) / 2;
        let candidate = format!("{}…", &label[..mid]);
        if hud_text_extent(dc, &candidate)?.cx <= available { low = mid; }
        else { high = mid - 1; }
    }
    Some(format!("{}…", &label[..low]).encode_utf16().collect())
}
unsafe fn draw_canonical_hud_numbers(dc: HDC, view: Viewport, labels: &[String; 2]) {
    let Some(hp) = canonical_hud_bar_rect(view, super::battle_hud::HP_BAR) else { return };
    let Some(mp) = canonical_hud_bar_rect(view, super::battle_hud::MP_BAR) else { return };
    let Some(_font) = HudFont::select(dc, (hp.bottom - hp.top).min(mp.bottom - mp.top)) else { return };
    let prior = SetTextColor(dc, COLORREF(0));
    for (mut rect, label) in [hp, mp].into_iter().zip(labels) {
        let Some(mut text) = fitted_hud_text(dc, label, rect.right - rect.left) else { continue };
        let mut shadow = rect;
        shadow.left = shadow.left.saturating_add(1);
        shadow.top = shadow.top.saturating_add(1);
        let flags = DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX;
        DrawTextW(dc, &mut text, &mut shadow, flags);
        SetTextColor(dc, COLORREF(0x00ffffff));
        DrawTextW(dc, &mut text, &mut rect, flags);
        SetTextColor(dc, COLORREF(0));
    }
    SetTextColor(dc, prior);
}
unsafe fn draw_inventory_page_label(dc: HDC, view: Viewport, page: usize, pages: usize) {
    let Some(mut rect) = canonical_hud_bar_rect(view, (inventory_panel::LEFT + 47, 305, 134, 24)) else { return };
    let Some(_font) = HudFont::select(dc, rect.bottom - rect.top) else { return };
    let label = format!("◀ {} / {} ▶", page + 1, pages);
    let mut text: Vec<u16> = label.encode_utf16().collect();
    let prior = SetTextColor(dc, COLORREF(0x00ffffff));
    DrawTextW(dc, &mut text, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    SetTextColor(dc, prior);
}
/// Shared 2460.img has a source-native AC box x101..160/y254..268.
/// Pad inward after the same 5/4 asset projection so GDI never straddles its
/// border; the full unabridged server value remains in Character details.
fn character_ac_source_rect() -> (i32, i32, i32, i32) {
    (128, 319, 70, 15)
}
unsafe fn draw_character_ledger_labels(dc: HDC, view: Viewport, page: usize,
    pages: usize, ac: Option<i64>) {
    {
        let Some(_font) = HudFont::select(dc, (view.height * 21 / HEIGHT).max(12)) else { return };
        let prior = SetTextColor(dc, COLORREF(0x00ffffff));
        for (source, label) in [
            ((8, 412, character_panel::RIGHT - 16, 26), "已裝備（槽位未提供）".to_string()),
            ((8, 548, character_panel::RIGHT - 16, 30),
                format!("◀ {} / {} ▶", page + 1, pages)),
        ] {
            if let Some(mut rect) = canonical_hud_bar_rect(view, source) {
                let mut text: Vec<u16> = label.encode_utf16().collect();
                DrawTextW(dc, &mut text, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
            }
        }
        SetTextColor(dc, prior);
    }
    if let Some(ac) = ac {
        if let Some(mut rect) = canonical_hud_bar_rect(view, character_ac_source_rect()) {
            if let Some(_font) = HudFont::select(dc, rect.bottom - rect.top) {
                if let Some(mut text) = fitted_hud_text(dc, &ac.to_string(), rect.right - rect.left) {
                    let prior = SetTextColor(dc, COLORREF(0x00ffffff));
                    DrawTextW(dc, &mut text, &mut rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                    SetTextColor(dc, prior);
                }
            }
        }
    }
}
unsafe fn draw_hover_name(dc: HDC, view: Viewport, dpi: u32, name: String, anchor: PixelPoint) {
    let mut text: Vec<u16> = name.encode_utf16().collect();
    let mut size = SIZE::default();
    if GetTextExtentPoint32W(dc, &text, &mut size).as_bool() {
        if let Some(mut rect) = nameplate_rect(view, anchor, size, dpi) {
            FillRect(dc, &rect, HBRUSH(GetStockObject(BLACK_BRUSH).0));
            DrawTextW(
                dc,
                &mut text,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            );
        }
    }
}

unsafe fn paint(hwnd: HWND, state: &State) {
    let mut paint = PAINTSTRUCT::default();
    let paint_dc = BeginPaint(hwnd, &mut paint);
    let mut bounds = RECT::default();
    if GetClientRect(hwnd, &mut bounds).is_ok() {
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        if width <= 0 || height <= 0 {
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        let dc = CreateCompatibleDC(Some(paint_dc));
        if dc.0.is_null() {
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        let bitmap = CreateCompatibleBitmap(paint_dc, width, height);
        if bitmap.0.is_null() {
            let _ = DeleteDC(dc);
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        let previous = SelectObject(dc, bitmap.into());
        if previous.0.is_null() {
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        state.painted_hit.set(None);
        state.painted_inventory.set(None);
        state.painted_skill.set(None);
        state.painted_character.set(None);
        state.painted_hotbar.set(None);
        *state.painted_combat.borrow_mut() = None;
        *state.painted_players.borrow_mut() = None;
        state.painted_battle.borrow_mut().take();
        FillRect(dc, &bounds, HBRUSH(GetStockObject(BLACK_BRUSH).0));
        let dpi = GetDpiForWindow(hwnd).max(96);
        let footer = scale(FOOTER, dpi);
        let mut inventory_commit = None;
        let mut skill_commit = None;
        let mut character_commit = None;
        let mut hotbar_commit = None;
        if let Some(image) = state.image.borrow().as_ref() {
            if let Some(view) = Viewport::fit(
                bounds.right,
                (bounds.bottom - footer).max(0),
                WIDTH as u32,
                HEIGHT as u32,
            ) {
                let bitmap = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: WIDTH,
                        biHeight: -HEIGHT,
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let inventory_overlay = if state.details_page.get() == Some(DetailsPage::Inventory) {
                    let pending = state.pending_battle.borrow();
                    let model = state.inventory.model.borrow();
                    let panel = state.inventory_panel.borrow();
                    match (pending.as_ref(), model.as_ref(), panel.as_ref()) {
                        (Some(surface), Some(model), Some(panel))
                            if model.stamp == super::battle_window::Stamp::of(&surface.observation) => {
                            panel.render(image, model, state.inventory.selection().map(|s| s.key))
                        }
                        _ => None,
                    }
                } else { None };
                let inventory_painted = if inventory_overlay.is_some() {
                    let model = state.inventory.model.borrow();
                    let panel = state.inventory_panel.borrow();
                    model.as_ref().zip(panel.as_ref()).and_then(|(model, panel)| panel.painted(model, view))
                } else { None };
                let base_pixels = inventory_overlay.as_deref().unwrap_or(image.as_ref());
                let skill_overlay = if state.details_page.get() == Some(DetailsPage::Skills) {
                    let pending = state.pending_battle.borrow();
                    let model = state.skillbook.model.borrow();
                    let panel = state.skill_panel.borrow();
                    match (pending.as_ref(), model.as_ref(), panel.as_ref()) {
                        (Some(surface), Some(model), Some(panel))
                            if model.stamp == super::battle_window::Stamp::of(&surface.observation) => {
                            panel.render(base_pixels, model, state.skillbook.selection())
                        }
                        _ => None,
                    }
                } else { None };
                let skill_painted = if skill_overlay.is_some() {
                    let model = state.skillbook.model.borrow();
                    let panel = state.skill_panel.borrow();
                    model.as_ref().zip(panel.as_ref()).and_then(|(model, panel)| panel.painted(model, view))
                } else { None };
                let base_pixels = skill_overlay.as_deref().unwrap_or(base_pixels);
                let character_overlay = if state.details_page.get() == Some(DetailsPage::Character) {
                    let pending = state.pending_battle.borrow();
                    let model = state.character_model.borrow();
                    let panel = state.character_panel.borrow();
                    match (pending.as_ref(), model.as_ref(), panel.as_ref()) {
                        (Some(surface), Some(model), Some(panel))
                            if model.stamp == super::battle_window::Stamp::of(&surface.observation) => {
                            panel.render(base_pixels, model, state.character_selected.get())
                        }
                        _ => None,
                    }
                } else { None };
                let character_painted = if character_overlay.is_some() {
                    let model = state.character_model.borrow();
                    let panel = state.character_panel.borrow();
                    model.as_ref().zip(panel.as_ref()).and_then(|(model, panel)| panel.painted(model, view))
                } else { None };
                let base_pixels = character_overlay.as_deref().unwrap_or(base_pixels);
                let hotbar_overlay = {
                    let pending = state.pending_battle.borrow();
                    let bar = state.hotbar.borrow();
                    pending.as_ref().zip(bar.as_ref()).and_then(|(surface, bar)| {
                        (bar.model.as_ref().is_some_and(|m|
                            m.stamp == super::battle_window::Stamp::of(&surface.observation)))
                            .then(|| bar.render(base_pixels)).flatten()
                    })
                };
                let hotbar_painted = if hotbar_overlay.is_some() {
                    state.hotbar.borrow().as_ref().and_then(|bar| bar.painted(view))
                } else { None };
                let pixels = hotbar_overlay.as_deref().unwrap_or(base_pixels);
                let copied = StretchDIBits(
                    dc,
                    view.x,
                    view.y,
                    view.width,
                    view.height,
                    0,
                    0,
                    WIDTH,
                    HEIGHT,
                    Some(pixels.as_ptr().cast()),
                    &bitmap,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
                if copied > 0 {
                    inventory_commit = inventory_painted;
                    skill_commit = skill_painted;
                    character_commit = character_painted;
                    hotbar_commit = hotbar_painted;
                    *state.painted_battle.borrow_mut() = state
                        .pending_battle
                        .borrow()
                        .as_ref()
                        .map(|s| (s.clone(), view));
                    *state.painted_combat.borrow_mut() = state.pending_combat.borrow().clone();
                    *state.painted_players.borrow_mut() = state.pending_players.borrow().clone();
                    state
                        .painted_hit
                        .set(state.pending_hit.get().map(|surface| PaintedSurface {
                            surface,
                            viewport: view,
                        }));
                }
            }
        }
        SetTextColor(dc, COLORREF(0x00dddddd));
        SetBkMode(dc, TRANSPARENT);
        if let Some((surface, viewport)) = state.painted_battle.borrow().as_ref() {
            if state.details_page.get() == Some(DetailsPage::Inventory) {
                let model = state.inventory.model.borrow();
                let panel = state.inventory_panel.borrow();
                if let (Some(model), Some(panel)) = (model.as_ref(), panel.as_ref()) {
                    if panel.ready() && model.stamp == super::battle_window::Stamp::of(&surface.observation) {
                        draw_inventory_page_label(dc, *viewport, panel.page(), panel.pages(model));
                    }
                }
            }
            if state.details_page.get() == Some(DetailsPage::Character) {
                let model = state.character_model.borrow();
                let panel = state.character_panel.borrow();
                if let (Some(model), Some(panel)) = (model.as_ref(), panel.as_ref()) {
                    if panel.ready() && model.stamp == super::battle_window::Stamp::of(&surface.observation) {
                        let ac = surface.observation.data().personal.equipment.as_ref()
                            .map(|equipment| equipment.effective.ac);
                        draw_character_ledger_labels(dc, *viewport, panel.page(), panel.pages(model), ac);
                    }
                }
            }
            // The dynamic numbers come from the same immutable owner snapshot
            // that cropped the shared HP/MP art. The footer below retains exact
            // values when a narrow viewport ellipsizes a long 64-bit label.
            if let Ok(vitals) = super::battle_scene::owner_hud_vitals(surface.observation.data()) {
                if let Ok([hp, mp]) = super::battle_hud::numbers(
                    vitals.hp, vitals.maximum_hp, vitals.mp, vitals.maximum_mp,
                ) {
                    draw_canonical_hud_numbers(dc, *viewport, &[hp, mp]);
                }
            }
            for (i, line) in super::battle_window::hud(&surface.observation)
                .iter()
                .enumerate()
            {
                if let Some(mut rect) =
                    canonical_hud_line(*viewport, surface.minimap.is_some(), i, dpi)
                {
                    let mut text = line.encode_utf16().collect::<Vec<_>>();
                    DrawTextW(
                        dc,
                        &mut text,
                        &mut rect,
                        DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX | DT_VCENTER,
                    );
                }
            }
            let mut cursor = POINT::default();
            if GetCursorPos(&mut cursor).is_ok()
                && WindowFromPoint(cursor) == hwnd
                && ScreenToClient(hwnd, &mut cursor).as_bool()
            {
                if let Some((name, anchor)) = super::battle_window::hit(
                    surface,
                    *viewport,
                    PixelPoint {
                        x: cursor.x,
                        y: cursor.y,
                    },
                    super::battle_window::Skill::Melee,
                )
                .and_then(|h| h.name)
                {
                    draw_hover_name(dc, *viewport, dpi, name, anchor);
                }
            }
        }
        if let (Some(painted), Some(combat)) = (
            state.painted_hit.get(),
            state.painted_combat.borrow().as_ref(),
        ) {
            for (i, line) in super::time_attack_presentation::hud(
                &combat.frame.world,
                painted.surface.selected_entity_id,
            )
            .into_iter()
            .chain(super::siege_presentation::hud(
                &combat.frame.world,
                painted.surface.selected_entity_id,
            ))
            .enumerate()
            {
                let _ = TextOutW(
                    dc,
                    scale(12, dpi),
                    scale(12 + i as i32 * 22, dpi),
                    &line.encode_utf16().collect::<Vec<_>>(),
                );
            }
            let mut cursor = POINT::default();
            if GetCursorPos(&mut cursor).is_ok()
                && WindowFromPoint(cursor) == hwnd
                && ScreenToClient(hwnd, &mut cursor).as_bool()
            {
                let point = PixelPoint {
                    x: cursor.x,
                    y: cursor.y,
                };
                if let Some((name, anchor)) = hovered_combat_name(painted, combat, point) {
                    draw_hover_name(dc, painted.viewport, dpi, name, anchor);
                }
                if matches!(
                    combat_hit_action(
                        painted,
                        Some(combat),
                        PixelPoint {
                            x: cursor.x,
                            y: cursor.y
                        }
                    ),
                    Some(Action::AttackTimeAttackTower { .. })
                ) {
                    if let Some(tower) = &combat.frame.world.time_attack {
                        let text = format!(
                            "啟動塔 · HP {}/{} · 普通攻擊",
                            tower.tower_hp, tower.tower_max_hp
                        );
                        let _ = TextOutW(
                            dc,
                            scale(12, dpi),
                            scale(80, dpi),
                            &text.encode_utf16().collect::<Vec<_>>(),
                        );
                    }
                }
            }
        }
        if let (Some(painted), Some(players)) = (
            state.painted_hit.get(),
            state.painted_players.borrow().as_ref(),
        ) {
            let mut cursor = POINT::default();
            if state.painted_combat.borrow().is_none()
                && GetCursorPos(&mut cursor).is_ok()
                && WindowFromPoint(cursor) == hwnd
                && ScreenToClient(hwnd, &mut cursor).as_bool()
            {
                if let Some((name, anchor)) = hovered_noncombat_name(
                    painted,
                    players,
                    PixelPoint {
                        x: cursor.x,
                        y: cursor.y,
                    },
                ) {
                    draw_hover_name(dc, painted.viewport, dpi, name, anchor);
                }
            }
        }
        // The native read-only status child owns multiline layout, scrolling
        // and copy. TextOutW here would flatten/clip the mode HUD and overlap
        // the inspection buttons; WS_CLIPCHILDREN protects the child rectangle.
        let descriptor = state.match_descriptor.borrow();
        let help: Vec<u16> = if state.painted_combat.borrow().is_some()
            || state
                .painted_battle
                .borrow()
                .as_ref()
                .is_some_and(|(s, _)| {
                    s.observation
                        .data()
                        .actors
                        .0
                        .iter()
                        .any(|a| a.vital.is_some())
                }) {
            "選單：普攻／已開放魔法；左鍵目標；地面尋路；右鍵取消；T瞬移；F火風暴；Esc返回。"
        } else {
            missing_feature_notice(
                descriptor.as_ref(),
                super::world_protocol::MatchDescriptor::COMBAT,
                "左鍵地面：尋路；右鍵：停止；方向鍵：單步；Esc：返回。戰鬥資料未確認。",
                "本場純移動：左鍵地面尋路；右鍵停止；方向鍵單步；Esc返回。",
            )
        }
        .encode_utf16()
        .collect();
        let _ = TextOutW(
            dc,
            scale(12, dpi),
            (bounds.bottom - scale(234, dpi)).max(0),
            &help,
        );
        let health = if let Some((surface, _)) = state.painted_battle.borrow().as_ref() {
            super::battle_window::health(&surface.observation)
        } else if let (Some(painted), Some(combat)) = (
            state.painted_hit.get(),
            state.painted_combat.borrow().as_ref(),
        ) {
            if let Some(vital) = combat
                .frame
                .vitals
                .iter()
                .find(|v| v.entity_id == painted.surface.selected_entity_id)
            {
                format!(
                    "HP {}/{}  MP {}/{}{}",
                    vital.hp,
                    vital.max_hp,
                    vital.mp,
                    vital.max_mp,
                    if combat.frame.regen(vital.entity_id).is_some() {
                        " · 自然恢復由伺服器確認（可為負值）"
                    } else {
                        ""
                    },
                )
            } else {
                "HP 未確認".into()
            }
        } else {
            missing_feature_notice(
                descriptor.as_ref(),
                super::world_protocol::MatchDescriptor::COMBAT,
                "HP／戰鬥資料未確認，不顯示推定數值。",
                "本場未啟用戰鬥，不提供 HP／MP 戰鬥數值。",
            )
            .into()
        };
        let canonical_skill = state
            .painted_battle
            .borrow()
            .as_ref()
            .and_then(|(surface, _)| {
                let selected = if state.slow_selected.get() {
                    super::battle_window::Skill::Slow
                } else if state.haste_selected.get() {
                    super::battle_window::Skill::Haste
                } else if state.disintegrate_selected.get() {
                    super::battle_window::Skill::Disintegrate
                } else if state.basic_heal_selected.get() {
                    super::battle_window::Skill::Heal
                } else if state.light_arrow_selected.get() {
                    super::battle_window::Skill::LightArrow
                } else {
                    super::battle_window::Skill::Melee
                };
                super::battle_window::skill_status(&surface.observation, selected)
            });
        if let Some(text) = canonical_skill {
            let _ = TextOutW(
                dc,
                scale(12, dpi),
                (bounds.bottom - scale(260, dpi)).max(0),
                &text.encode_utf16().collect::<Vec<_>>(),
            );
        }
        if let (Some(painted), Some(combat)) = (
            state.painted_hit.get(),
            state.painted_combat.borrow().as_ref(),
        ) {
            let skill_status = if state.disintegrate_selected.get() {
                combat
                    .frame
                    .disintegrate(painted.surface.selected_entity_id)
                    .map(|d| (d.mp_cost, d.lawful))
                    .or_else(|| {
                        combat
                            .frame
                            .npc_disintegrate(painted.surface.selected_entity_id)
                            .map(|d| (d.mp_cost, d.lawful))
                    })
                    .map(|(cost, lawful)| {
                        format!(
                            "究極光裂術：MP{} · 正義值{}（需500）· 重用5秒",
                            cost, lawful
                        )
                    })
            } else {
                combat
                    .frame
                    .fire(painted.surface.selected_entity_id)
                    .map(|fire| {
                        format!(
                            "火風暴：MP{} · 最近{}目標 · 重用2秒",
                            fire.mp_cost,
                            fire.targets.len()
                        )
                    })
            };
            if let Some(text) = skill_status {
                let _ = TextOutW(
                    dc,
                    scale(12, dpi),
                    (bounds.bottom - scale(260, dpi)).max(0),
                    &text.encode_utf16().collect::<Vec<_>>(),
                );
            }
        }
        let _ = TextOutW(
            dc,
            scale(12, dpi),
            (bounds.bottom - scale(214, dpi)).max(0),
            &health.encode_utf16().collect::<Vec<_>>(),
        );
        let weight = if let Some((surface, _)) = state.painted_battle.borrow().as_ref() {
            super::battle_window::weight(&surface.observation)
        } else if let (Some(painted), Some(combat)) = (
            state.painted_hit.get(),
            state.painted_combat.borrow().as_ref(),
        ) {
            combat
                .frame
                .weight(painted.surface.selected_entity_id)
                .map(|w| {
                    format!(
                        "比賽負重 {}／{:.3} · {:.1}% · 階段 {}/240 · {}",
                        w.total,
                        w.maximum(),
                        f64::from(w.stage) * 100.0 / 240.0,
                        w.stage,
                        if w.can_attack() {
                            "可攻擊／施法"
                        } else {
                            "過重：可移動／喝藥，禁攻擊／施法"
                        }
                    )
                })
                .unwrap_or_else(|| "比賽負重未提供；不推定為零。".into())
        } else {
            missing_feature_notice(
                descriptor.as_ref(),
                super::world_protocol::MatchDescriptor::WEIGHT,
                "比賽負重待同版戰鬥資料確認。",
                "本場未啟用比賽負重。",
            )
            .into()
        };
        let _ = TextOutW(
            dc,
            scale(12, dpi),
            (bounds.bottom - scale(194, dpi)).max(0),
            &weight.encode_utf16().collect::<Vec<_>>(),
        );
        let timed = if let Some((surface, _)) = state.painted_battle.borrow().as_ref() {
            super::battle_window::buffs(&surface.observation)
        } else if let (Some(painted), Some(combat)) = (
            state.painted_hit.get(),
            state.painted_combat.borrow().as_ref(),
        ) {
            combat
                .frame
                .timed(painted.surface.selected_entity_id)
                .map(|s| {
                    let key = (
                        painted.surface.match_id,
                        painted.surface.generation,
                        s.entity_id,
                        s.revision,
                    );
                    let now = std::time::Instant::now();
                    let seen = state
                        .timed_receipt
                        .get()
                        .filter(|r| (r.0, r.1, r.2, r.3) == key)
                        .map_or(now, |r| r.4);
                    state
                        .timed_receipt
                        .set(Some((key.0, key.1, key.2, key.3, seen)));
                    s.notice_estimate(now.saturating_duration_since(seen))
                })
                .unwrap_or_else(|| "此世界未開放限時用品。".into())
        } else {
            missing_feature_notice(
                descriptor.as_ref(),
                super::world_protocol::MatchDescriptor::TIMED,
                "限時效果等待同版權威資料。",
                "本場未啟用限時用品。",
            )
            .into()
        };
        let _ = TextOutW(
            dc,
            scale(12, dpi),
            (bounds.bottom - scale(172, dpi)).max(0),
            &timed.encode_utf16().collect::<Vec<_>>(),
        );
        // Publish the composed world and HUD together. A failed commit must
        // not install hit proofs for pixels the player never saw.
        if BitBlt(paint_dc, 0, 0, width, height, Some(dc), 0, 0, SRCCOPY).is_err() {
            state.painted_hit.set(None);
            *state.painted_combat.borrow_mut() = None;
            *state.painted_players.borrow_mut() = None;
            state.painted_battle.borrow_mut().take();
            state.painted_inventory.set(None);
            state.painted_skill.set(None);
            state.painted_character.set(None);
            state.painted_hotbar.set(None);
        } else {
            state.painted_inventory.set(inventory_commit);
            state.painted_skill.set(skill_commit);
            state.painted_character.set(character_commit);
            state.painted_hotbar.set(hotbar_commit);
        }
        SelectObject(dc, previous);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
    }
    let _ = EndPaint(hwnd, &paint);
}

#[cfg(test)]
pub(crate) fn verify_observed_panel_composition(
    surface: &super::battle_scene::Surface,
    view: &super::login_window::LoginSnapshot,
    root: &std::path::Path,
) {
    use super::battle_inventory::{self, Key};
    use super::battle_skillbook;
    assert_eq!((surface.width, surface.height), (800, 600));
    let inventory = battle_inventory::model(surface, view, str::to_owned)
        .expect("current official adapter inventory");
    let character = character_panel::Model::from_observation(&surface.observation, &inventory)
        .expect("current official adapter character/equipped model");
    let skills = battle_skillbook::model(surface, view, str::to_owned)
        .expect("current official adapter learned skill model");
    assert!(!character.rows.is_empty(), "source-confirmed equipped rows, never fabricated");
    assert!(character.rows.iter().all(|row| inventory.rows.iter().any(|item|
        item.key == Key::Source(row.id) && item.graphic == row.graphic)));
    assert!(skills.choice(29).is_some(), "source 7 SQL learned/affordable Slow29");
    let source_key = inventory.rows.iter().find_map(|row|
        matches!(row.key, Key::Source(_)).then_some(row.key)).unwrap();
    let source_id = character.rows[0].id;
    let mut inventory_art = inventory_panel::Panel::start(root.to_path_buf()).unwrap();
    let mut character_art = character_panel::Panel::start(root.to_path_buf()).unwrap();
    let mut skill_art = skill_panel::Panel::start(root.to_path_buf()).unwrap();
    let mut hotbar_art = hotbar::Hotbar::start(root.to_path_buf()).unwrap();
    assert!(inventory_art.update(Some(&inventory)));
    assert!(character_art.update(Some(&character)));
    assert!(skill_art.update(Some(&skills)));
    assert!(hotbar_art.update(Some(inventory.clone()), Some(skills.clone())));
    assert!(hotbar_art.bind_skill(0, 29));
    let viewport = Viewport { x: 0, y: 0, width: 800, height: 600 };
    for _ in 0..1000 {
        inventory_art.update(Some(&inventory));
        character_art.update(Some(&character));
        skill_art.update(Some(&skills));
        hotbar_art.update(Some(inventory.clone()), Some(skills.clone()));
        if inventory_art.ready() && character_art.ready() && skill_art.ready()
            && hotbar_art.painted(viewport).is_some() { break; }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(inventory_art.ready(), "observed inventory art: {:?}", inventory_art.error());
    assert!(character_art.ready(), "observed character art: {:?}", character_art.error());
    assert!(skill_art.ready(), "observed skill art: {:?}", skill_art.error());
    assert!(hotbar_art.painted(viewport).is_some(), "observed hotbar art: {:?}", hotbar_art.error());
    let base = &surface.bgra;
    let inventory_pixels = hotbar_art.render(&inventory_art.render(base, &inventory,
        Some(source_key)).unwrap()).unwrap();
    let character_pixels = hotbar_art.render(&character_art.render(base, &character,
        Some(source_id)).unwrap()).unwrap();
    let skill_pixels = hotbar_art.render(&skill_art.render(base, &skills, Some(29)).unwrap()).unwrap();
    assert_ne!(inventory_pixels.as_slice(), base.as_ref());
    assert_ne!(character_pixels.as_slice(), base.as_ref());
    assert_ne!(skill_pixels.as_slice(), base.as_ref());
    assert_ne!(inventory_pixels, character_pixels);
    assert_ne!(character_pixels, skill_pixels);
    {
        use sha2::Digest;
        eprintln!(
            "R393_OBSERVED source={} object={} life={} inventory_rows={} equipped_ids={:?} skill_ids={:?} inventory_sha256={:x} character_sha256={:x} skill_sha256={:x}",
            inventory.stamp.context.key.source_id,
            inventory.stamp.context.key.object_id,
            inventory.stamp.life,
            inventory.rows.len(),
            character.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            skills.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            sha2::Sha256::digest(&inventory_pixels),
            sha2::Sha256::digest(&character_pixels),
            sha2::Sha256::digest(&skill_pixels),
        );
    }
    let inventory_proof = inventory_art.painted(&inventory, viewport).unwrap();
    let character_proof = character_art.painted(&character, viewport).unwrap();
    let skill_proof = skill_art.painted(&skills, viewport).unwrap();
    let hotbar_proof = hotbar_art.painted(viewport).unwrap();
    assert!(inventory_proof.valid(&inventory_art, &inventory, viewport));
    assert!(character_proof.valid(&character_art, &character, viewport));
    assert!(skill_proof.valid(&skill_art, &skills, viewport));
    assert!(hotbar_proof.valid(&hotbar_art, viewport));
    let mut pending = view.clone();
    pending.world_pending = true;
    let pending_inventory = battle_inventory::model(surface, &pending, str::to_owned);
    let pending_skills = battle_skillbook::model(surface, &pending, str::to_owned);
    if let Some(model) = &pending_inventory {
        assert!(model.rows.iter().all(|row|
            row.equip.is_none() && row.unequip.is_none() && row.local.is_none()));
    }
    if let Some(model) = &pending_skills {
        assert!(model.rows.iter().all(|row| row.choice.is_none()));
    }
    inventory_art.update(pending_inventory.as_ref());
    skill_art.update(pending_skills.as_ref());
    hotbar_art.update(pending_inventory, pending_skills);
    assert!(!hotbar_proof.valid(&hotbar_art, viewport),
        "pending real adapter must revoke a previously painted skill command");
    let mut detached = view.clone();
    detached.navigation = None;
    assert!(battle_inventory::model(surface, &detached, str::to_owned).is_none());
    assert!(battle_skillbook::model(surface, &detached, str::to_owned).is_none());
    inventory_art.update(None);
    character_art.update(None);
    skill_art.update(None);
    hotbar_art.update(None, None);
    assert!(!inventory_proof.valid(&inventory_art, &inventory, viewport));
    assert!(!character_proof.valid(&character_art, &character, viewport));
    assert!(!skill_proof.valid(&skill_art, &skills, viewport));
    assert!(!hotbar_proof.valid(&hotbar_art, viewport));
}

/// Cross-check the first real SQL owner against the shared action table. The
/// fixture's equipped item 1 is a sword; a bare-body fallback would use a
/// different action key even though it shares the same graphics bank.
#[cfg(test)]
pub(crate) fn verify_observed_sword_sprite(
    surface: &super::battle_scene::Surface,
    root: &std::path::Path,
) {
    use super::{shared_action_catalog::SharedActionCatalog, sprite_actions::ActionKey};
    use std::sync::atomic::AtomicBool;
    let data = surface.observation.data();
    assert_eq!(data.personal.player.source, 7);
    assert_eq!(data.personal.player.object, 3333);
    let owner = data.actors.0.iter().find(|a| a.entity == data.personal.entity)
        .expect("observed owner actor");
    assert_eq!(owner.vital.as_ref().expect("combat vital").weapon_type, 1,
        "the equipped fixture sword must determine the server stance");
    let placement = surface.geometry.actors.iter()
        .find(|a| a.entity_id == owner.entity as u64)
        .expect("owner painted in official world surface");
    let cancel = AtomicBool::new(false);
    let shared = SharedActionCatalog::load(root, &cancel).expect("shared list.spr");
    // Native class/sex 1/0 maps to logical sprite 61; sword idle is action 7.
    let resolved = shared.catalog().resolve_action(61, ActionKey::new(7))
        .expect("qualified knight sword idle action");
    let graphics = shared.catalog().sprites[&61].graphics_id;
    let expected = resolved.animation.frame_asset(
        graphics, placement.source_frame, owner.heading as u8,
    ).expect("qualified knight sword idle frame");
    let bare = shared.catalog().resolve_action(61, ActionKey::new(3))
        .expect("qualified knight bare-hand idle action");
    assert!(bare.animation.frames.iter().enumerate().all(|(index, _)| {
        let frame = bare.animation.frame_asset(graphics, index, owner.heading as u8)
            .expect("qualified knight bare-hand idle frame");
        (frame.file_name(), frame.subframe) != (expected.file_name(), expected.subframe)
    }), "equipped sword frame must not alias a bare-hand idle frame");
    assert_eq!(placement.graphics_id, graphics);
    assert_eq!(placement.sprite_name, expected.file_name(),
        "official world painter must use the equipped sword frame, not bare-body art");
    assert_eq!(placement.subframe, expected.subframe as usize);
    assert!(placement.width > 0 && placement.height > 0);
    let mask = placement.visibility.as_ref().expect("formal raster visibility proof");
    let mut visible = 0usize;
    for y in i64::from(placement.top_left.y).max(0)..
        (i64::from(placement.top_left.y) + i64::from(placement.height)).min(600) {
        for x in i64::from(placement.top_left.x).max(0)..
            (i64::from(placement.top_left.x) + i64::from(placement.width)).min(800) {
            if mask.contains(x, y) {
                let offset = ((y as usize * 800) + x as usize) * 4;
                assert_eq!(surface.bgra[offset + 3], 255, "visible actor has an opaque DIB pixel");
                visible += 1;
            }
        }
    }
    assert!(visible > 0, "equipped sword must reach visible raster pixels");
    eprintln!("R395_OBSERVED source=7 object=3333 weapon_type=1 action_key=7 graphics={} sprite={} subframe={} frame={} rect={}x{} visible_pixels={}",
        graphics, placement.sprite_name, placement.subframe, placement.source_frame,
        placement.width, placement.height, visible);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; four real shared-art workers and CPU-composed page lifecycle"]
    fn real_shared_pages_close_reopen_resize_and_new_life_keep_only_current_paint_proofs() {
        use crate::modern_lito::{application_protocol::{CharacterKey, SelectionContext},
            battle_inventory::{Key, Model as InventoryModel, Row as InventoryRow},
            battle_skillbook::{Choice, Model as SkillModel, Row as SkillRow}};
        let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
            .expect("shared resource root"));
        let stamp = super::super::battle_window::Stamp { context: SelectionContext {
            key: CharacterKey { source_id: 7, object_id: 3333 }, epoch: 1 },
            channel: 0, world: [1, 2, 3], entity: 1, life: 1, revision: 1 };
        let inventory = InventoryModel { stamp, rows: vec![
            InventoryRow { key: Key::Source(100), graphic: Some(80),
                label: "equipped #100".into(), equip: None, unequip: None, local: None },
            InventoryRow { key: Key::Supply(40010), graphic: Some(80),
                label: "supply #40010".into(), equip: None, unequip: None, local: None },
        ] };
        let character = character_panel::Model { stamp, sex: 0, rows: vec![
            character_panel::Row { id: 100, graphic: Some(80), label: "equipped #100".into() },
        ] };
        let skills = SkillModel { stamp, rows: vec![SkillRow { id: 4,
            label: "Light Arrow #4".into(), button: "選擇目標",
            choice: Some(Choice { stamp, id: 4 }) }] };
        let mut inventory_art = inventory_panel::Panel::start(root.clone()).unwrap();
        let mut character_art = character_panel::Panel::start(root.clone()).unwrap();
        let mut skill_art = skill_panel::Panel::start(root.clone()).unwrap();
        let mut hotbar_art = hotbar::Hotbar::start(root).unwrap();
        assert!(inventory_art.update(Some(&inventory)));
        assert!(character_art.update(Some(&character)));
        assert!(skill_art.update(Some(&skills)));
        assert!(hotbar_art.update(Some(inventory.clone()), Some(skills.clone())));
        assert!(hotbar_art.bind(0, Some(Key::Supply(40010))));
        assert!(hotbar_art.bind_skill(1, 4));
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        for _ in 0..1000 {
            inventory_art.update(Some(&inventory));
            character_art.update(Some(&character));
            skill_art.update(Some(&skills));
            hotbar_art.update(Some(inventory.clone()), Some(skills.clone()));
            if inventory_art.ready() && character_art.ready() && skill_art.ready()
                && hotbar_art.painted(view).is_some() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(inventory_art.ready(), "inventory: {:?}", inventory_art.error());
        assert!(character_art.ready(), "character: {:?}", character_art.error());
        assert!(skill_art.ready(), "skill: {:?}", skill_art.error());
        assert!(hotbar_art.painted(view).is_some(), "hotbar: {:?}", hotbar_art.error());
        let base = vec![0u8; 800 * 600 * 4];
        let inventory_pixels = hotbar_art.render(&inventory_art.render(&base,
            &inventory, Some(Key::Source(100))).unwrap()).unwrap();
        let character_pixels = hotbar_art.render(&character_art.render(&base,
            &character, Some(100)).unwrap()).unwrap();
        let skill_pixels = hotbar_art.render(&skill_art.render(&base,
            &skills, Some(4)).unwrap()).unwrap();
        assert_ne!(inventory_pixels, character_pixels);
        assert_ne!(character_pixels, skill_pixels);
        assert_ne!(skill_pixels, inventory_pixels);
        let old_inventory = inventory_art.painted(&inventory, view).unwrap();
        let old_character = character_art.painted(&character, view).unwrap();
        let old_skill = skill_art.painted(&skills, view).unwrap();
        let old_hotbar = hotbar_art.painted(view).unwrap();
        assert_eq!(old_inventory.key(0), Some(Key::Source(100)));
        assert_eq!(old_character.id(0), Some(100));
        assert!(old_skill.contains(4));
        assert!(old_hotbar.valid(&hotbar_art, view));
        assert!(!inventory_art.update(Some(&inventory)));
        assert!(!character_art.update(Some(&character)));
        assert!(!skill_art.update(Some(&skills)));
        assert!(!hotbar_art.update(Some(inventory.clone()), Some(skills.clone())));
        let resized = Viewport { x: 48, y: 0, width: 1200, height: 900 };
        assert!(!old_inventory.valid(&inventory_art, &inventory, resized));
        assert!(!old_character.valid(&character_art, &character, resized));
        assert!(!old_skill.valid(&skill_art, &skills, resized));
        assert!(!old_hotbar.valid(&hotbar_art, resized));
        assert!(inventory_art.update(None));
        assert!(character_art.update(None));
        assert!(skill_art.update(None));
        assert!(!old_inventory.valid(&inventory_art, &inventory, view));
        assert!(!old_character.valid(&character_art, &character, view));
        assert!(!old_skill.valid(&skill_art, &skills, view));
        assert!(!inventory_art.update(None));
        assert!(!character_art.update(None));
        assert!(!skill_art.update(None));
        assert_eq!(hotbar_art.render(&base).unwrap().len(), base.len(),
            "closed details leave the global hotbar composed");
        assert!(inventory_art.update(Some(&inventory)));
        for _ in 0..1000 {
            inventory_art.update(Some(&inventory));
            if inventory_art.ready() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(inventory_art.ready(), "reopened inventory: {:?}", inventory_art.error());
        assert!(!old_inventory.valid(&inventory_art, &inventory, view));
        let fresh_inventory = inventory_art.painted(&inventory, resized).unwrap();
        assert!(fresh_inventory.valid(&inventory_art, &inventory, resized));
        let mut next_inventory = inventory.clone();
        let mut next_character = character.clone();
        let mut next_skills = skills.clone();
        next_inventory.stamp.life += 1;
        next_character.stamp.life += 1;
        next_skills.stamp.life += 1;
        next_skills.rows[0].choice = Some(Choice { stamp: next_skills.stamp, id: 4 });
        assert!(!fresh_inventory.valid(&inventory_art, &next_inventory, resized));
        hotbar_art.update(Some(next_inventory.clone()), Some(next_skills.clone()));
        assert!(!old_hotbar.valid(&hotbar_art, view));
        assert!(!old_hotbar.valid(&hotbar_art, resized));
        assert!(inventory_art.render(&base, &next_inventory, Some(Key::Source(100))).is_some());
        assert!(character_art.update(Some(&next_character)));
        assert!(skill_art.update(Some(&next_skills)));
        for _ in 0..1000 {
            inventory_art.update(Some(&next_inventory));
            character_art.update(Some(&next_character));
            skill_art.update(Some(&next_skills));
            hotbar_art.update(Some(next_inventory.clone()), Some(next_skills.clone()));
            if inventory_art.ready() && character_art.ready() && skill_art.ready()
                && hotbar_art.painted(resized).is_some() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(character_art.ready(), "new-life character: {:?}", character_art.error());
        assert!(skill_art.ready(), "new-life skill: {:?}", skill_art.error());
        assert!(hotbar_art.painted(resized).is_some(), "new-life hotbar: {:?}", hotbar_art.error());
        assert!(!old_character.valid(&character_art, &next_character, view));
        assert!(!old_skill.valid(&skill_art, &next_skills, view));
        assert!(character_art.painted(&next_character, resized).unwrap()
            .valid(&character_art, &next_character, resized));
        assert!(skill_art.painted(&next_skills, resized).unwrap()
            .valid(&skill_art, &next_skills, resized));
    }
    #[test]
    fn character_ac_text_rect_is_inside_real_2460_frame_and_gdi_fits() {
        assert_eq!(character_ac_source_rect(), (128, 319, 70, 15));
        // Source-native frame border x101..160/y254..268, projected 5/4.
        let (x, y, w, h) = character_ac_source_rect();
        assert!(x > 101 * 5 / 4 && x + w < 160 * 5 / 4);
        assert!(y > 254 * 5 / 4 && y + h <= 268 * 5 / 4);
        unsafe {
            let dc = CreateCompatibleDC(None);
            assert!(!dc.0.is_null());
            for view in [Viewport { x: 0, y: 0, width: 800, height: 600 },
                Viewport { x: 48, y: 0, width: 1200, height: 900 }] {
                let rect = canonical_hud_bar_rect(view, character_ac_source_rect()).unwrap();
                let height = rect.bottom - rect.top;
                let _font = HudFont::select(dc, height).expect("real AC box has fitting GDI font");
                let fitted = fitted_hud_text(dc, "9223372036854775807", rect.right - rect.left).unwrap();
                let display = String::from_utf16(&fitted).unwrap();
                let size = hud_text_extent(dc, &display).unwrap();
                assert!(size.cx <= rect.right - rect.left - 4);
                assert!(size.cy <= height - 2);
            }
            assert!(DeleteDC(dc).as_bool());
        }
    }
    #[test]
    fn real_gdi_hud_font_scales_and_fits_both_bars_with_explicit_ellipsis() {
        unsafe {
            let dc = CreateCompatibleDC(None);
            assert!(!dc.0.is_null(), "GDI metrics DC");
            let default = hud_text_extent(dc, "800/1000").unwrap();
            let mut heights = Vec::new();
            for view in [
                Viewport { x: 100, y: 190, width: 400, height: 300 },
                Viewport { x: 0, y: 0, width: 800, height: 600 },
                Viewport { x: 48, y: 0, width: 1200, height: 900 },
            ] {
                let hp = canonical_hud_bar_rect(view, super::super::battle_hud::HP_BAR).unwrap();
                let mp = canonical_hud_bar_rect(view, super::super::battle_hud::MP_BAR).unwrap();
                let bar_height = hp.bottom - hp.top;
                assert_eq!(bar_height, mp.bottom - mp.top);
                let font = HudFont::select(dc, bar_height).expect("source-sized GDI HUD font");
                let normal = hud_text_extent(dc, "800/1000").unwrap();
                assert!(normal.cy <= bar_height - 2, "glyph must fit, not clip");
                heights.push(normal.cy);
                for (rect, label) in [(hp, "800/1000"), (mp, "0/0")] {
                    let fitted = fitted_hud_text(dc, label, rect.right - rect.left).unwrap();
                    assert_eq!(String::from_utf16(&fitted).unwrap(), label);
                    let measured = hud_text_extent(dc, label).unwrap();
                    assert!(measured.cx <= rect.right - rect.left - 4);
                    assert!(measured.cy <= rect.bottom - rect.top - 2);
                }
                let long = "9223372036854775807/9223372036854775807";
                let fitted = fitted_hud_text(dc, long, hp.right - hp.left).unwrap();
                let display = String::from_utf16(&fitted).unwrap();
                assert!(display.ends_with('…') && display.len() < long.len());
                assert!(hud_text_extent(dc, &display).unwrap().cx <= hp.right - hp.left - 4);
                drop(font);
                assert_eq!(hud_text_extent(dc, "800/1000").unwrap().cy, default.cy,
                    "HUD font must restore the original DC font");
            }
            assert!(heights[0] < heights[1] && heights[1] < heights[2],
                "font metrics must grow with the same viewport as the shared art");
            assert!(DeleteDC(dc).as_bool());
        }
    }
    #[test]
    fn canonical_numbers_share_source_bar_bounds_after_scale_and_translation() {
        for view in [
            Viewport { x: 0, y: 0, width: 800, height: 600 },
            Viewport { x: 100, y: 190, width: 400, height: 300 },
            Viewport { x: 48, y: 0, width: 1200, height: 900 },
        ] {
            let hp = canonical_hud_bar_rect(view, super::super::battle_hud::HP_BAR).unwrap();
            let mp = canonical_hud_bar_rect(view, super::super::battle_hud::MP_BAR).unwrap();
            assert!(hp.left >= view.x && hp.right < mp.left && mp.right <= view.x + view.width);
            assert!(hp.top >= view.y && hp.bottom <= view.y + view.height);
            assert_eq!((hp.top, hp.bottom), (mp.top, mp.bottom));
        }
        let original = canonical_hud_bar_rect(Viewport { x: 0, y: 0, width: 800, height: 600 }, super::super::battle_hud::HP_BAR).unwrap();
        assert_eq!((original.left, original.top, original.right, original.bottom), (277, 458, 418, 478));
        assert!(canonical_hud_bar_rect(Viewport { x: 0, y: 0, width: 0, height: 600 }, super::super::battle_hud::HP_BAR).is_none());
        assert!(canonical_hud_bar_rect(Viewport { x: 0, y: 0, width: 800, height: 600 }, (790, 458, 20, 20)).is_none());
    }
    #[test]
    fn canonical_hud_stays_beside_minimap_inside_scaled_and_translated_viewport() {
        for dpi in [96, 144, 192] {
            for view in [
                Viewport {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                },
                Viewport {
                    x: 100,
                    y: 190,
                    width: 400,
                    height: 300,
                },
                Viewport {
                    x: 48,
                    y: 0,
                    width: 1200,
                    height: 900,
                },
            ] {
                let mut prior = None;
                for i in 0..4 {
                    let r = canonical_hud_line(view, true, i, dpi).unwrap();
                    let edge = view.x
                        + ((super::super::battle_minimap::X
                            + super::super::battle_minimap::WIDTH as i32)
                            * view.width
                            + 799)
                            / 800;
                    assert!(
                        r.left > edge
                            && r.right <= view.x + view.width
                            && r.top >= view.y
                            && r.bottom <= view.y + view.height
                    );
                    if let Some(y) = prior {
                        assert!(r.top >= y);
                    }
                    prior = Some(r.bottom);
                }
                assert!(canonical_hud_line(view, true, usize::MAX, dpi).is_none());
            }
        }
        assert!(canonical_hud_line(
            Viewport {
                x: 0,
                y: 0,
                width: 0,
                height: 0
            },
            true,
            0,
            96
        )
        .is_none());
    }
    fn npc_surface() -> (
        CombatSurface,
        PaintedSurface,
        super::super::npc_protocol::Id,
    ) {
        use super::super::npc_protocol as npc;
        let mut reply = npc::tests::read_reply();
        let frame = reply.frame.as_mut().unwrap();
        frame.resources = Some(vec![1649, 2158]);
        let actor = &frame.actors[0];
        let id = npc::Id {
            scope: frame.scope,
            serial: actor.serial,
        };
        let placement = super::super::world_scene::NpcPlacement {
            visibility: None,
            id,
            life: actor.life,
            alive: actor.alive(),
            cell: actor.cell,
            anchor: PixelPoint { x: 400, y: 300 },
            top_left: PixelPoint { x: 376, y: 260 },
            width: 48,
            height: 60,
            graphics_id: actor.graphic,
            source_frame: 0,
            sprite_name: "synthetic hit fixture".into(),
            subframe: 0,
        };
        let player = ActorPlacement {
            visibility: None,
            entity_id: 1,
            cell: frame.combat.world.actors[0].position,
            anchor: placement.anchor,
            top_left: placement.top_left,
            width: 48,
            height: 60,
            graphics_id: 3213,
            source_frame: 0,
            sprite_name: "synthetic player hit fixture".into(),
            subframe: 0,
        };
        let painted = PaintedSurface {
            surface: HitSurface {
                match_id: frame.scope.match_id,
                generation: frame.scope.generation,
                revision: frame.combat.world.revision,
                selected_entity_id: 1,
                projection: ViewProjection {
                    anchor_cell: player.cell,
                    anchor_pixel: PixelPoint { x: 400, y: 300 },
                },
            },
            viewport: Viewport {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
        };
        let surface = CombatSurface {
            frame: Arc::new(frame.combat.clone()),
            actors: Arc::from([player]),
            objectives: Arc::from([]),
            quests: Arc::from([]),
            time_attack_tower: None,
            siege_towers: Arc::from([]),
            npc: Some(NpcSurface {
                reply: Arc::new(reply),
                placements: Arc::from([placement]),
            }),
        };
        (surface, painted, id)
    }
    #[test]
    fn npc_hit_uses_compositor_namespace_depth_and_last_painted_dpi_viewport() {
        let (surface, mut painted, target) = npc_surface();
        for (width, height) in [(800, 600), (1000, 750), (1200, 900), (1600, 1200)] {
            painted.viewport = Viewport {
                x: 35,
                y: 47,
                width,
                height,
            };
            let point = PixelPoint {
                x: 35 + 400 * width / 800,
                y: 47 + 280 * height / 600,
            };
            let hit = combat_hit_action(painted, Some(&surface), point);
            assert_eq!(
                hit,
                Some(Action::AttackNpc {
                    surface: painted.surface,
                    target,
                    life: 1
                })
            );
            assert_eq!(
                selected_hit(hit, true),
                Some(Action::CastNpcLightArrow {
                    surface: painted.surface,
                    target,
                    life: 1,
                })
            );
            assert_eq!(healing_hit(hit, true), Some(Action::CancelRoute));
            assert!(
                combat_hit_action(painted, Some(&surface), PixelPoint { x: 0, y: 0 }).is_none()
            );
        }
        let mut behind = surface.clone();
        Arc::make_mut(&mut behind.npc.as_mut().unwrap().placements)[0]
            .anchor
            .y -= 1;
        painted.viewport = Viewport {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        assert_eq!(
            combat_hit_action(painted, Some(&behind), PixelPoint { x: 400, y: 280 }),
            Some(Action::CancelRoute)
        );
        // Numeric serial 1 is not the selected player's entity 1.
        assert_eq!(target.serial, painted.surface.selected_entity_id);
    }
    #[test]
    fn npc_hit_rejects_stale_life_foreign_scope_and_missing_complete_authority() {
        let (surface, painted, _) = npc_surface();
        for case in 0..4 {
            let mut bad = surface.clone();
            match case {
                0 => Arc::make_mut(&mut bad.npc.as_mut().unwrap().placements)[0].life += 1,
                1 => {
                    Arc::make_mut(&mut bad.npc.as_mut().unwrap().placements)[0]
                        .id
                        .scope
                        .generation += 1
                }
                2 => Arc::make_mut(&mut bad.npc.as_mut().unwrap().placements)[0].alive = false,
                _ => bad.npc = None,
            }
            assert!(
                combat_hit_action(painted, Some(&bad), PixelPoint { x: 400, y: 280 }).is_none()
            );
        }
    }
    #[test]
    fn time_attack_title_distinguishes_tower_wait_from_encounter_deadline() {
        let mut descriptor = super::super::world_runtime::tests::match_descriptor(false);
        descriptor.ruleset = super::super::world_protocol::MatchRuleset::CooperativeTimeAttack;
        descriptor.active_timeout_millis = 60_000;
        let title = match_title(Some(&descriptor));
        assert!(title.contains("啟動塔等待上限 60 秒"), "{title}");
        assert!(title.contains("擊塔後另依戰鬥時限結算"), "{title}");
        assert!(!title.contains("到期中止、不判勝"));
        assert!(
            !title.contains("420"),
            "descriptor alone has no encounter duration"
        );
    }
    #[test]
    fn siege_title_states_defender_timeout_win_not_safety_abort() {
        let mut descriptor = super::super::world_runtime::tests::match_descriptor(false);
        descriptor.ruleset = super::super::world_protocol::MatchRuleset::SiegeObjectives;
        descriptor.active_timeout_millis = 1_200_000;
        let title = match_title(Some(&descriptor));
        assert!(title.contains("戰鬥時限 1200 秒"), "{title}");
        assert!(title.contains("到期守方勝利"), "{title}");
        assert!(!title.contains("到期中止、不判勝"));
        descriptor.active_timeout_millis = 50;
        assert!(match_title(Some(&descriptor)).contains("戰鬥時限 0.05 秒"));
    }
    #[test]
    fn clock_policy_title_retains_nanoseconds_and_full_supported_limits() {
        let mut descriptor = super::super::world_runtime::tests::match_descriptor(false);
        descriptor.countdown_nanos = 30_000_000_000;
        descriptor.active_timeout_millis = 1_800_000;
        assert!(match_title(Some(&descriptor)).contains("倒數設定 30 秒"));
        assert!(match_title(Some(&descriptor)).contains("安全上限 1800 秒"));
        descriptor.countdown_nanos = 1;
        descriptor.active_timeout_millis = 50;
        assert!(match_title(Some(&descriptor)).contains("倒數設定 0.000000001 秒"));
        assert!(match_title(Some(&descriptor)).contains("安全上限 0.05 秒"));
        assert!(!match_title(Some(&descriptor)).contains("剩餘"));
    }
    #[test]
    fn match_title_shows_exact_policy_not_nominal_mode_or_remaining_countdown() {
        let mut descriptor = super::super::world_runtime::tests::match_descriptor(false);
        let text = match_title(Some(&descriptor));
        assert!(text.contains("純移動場景"));
        assert!(text.contains("倒數設定 3.000000001 秒"));
        assert!(text.contains("安全上限 150.123 秒"));
        assert!(text.contains("到期中止、不判勝"));
        assert!(!text.contains("剩餘"));
        descriptor.countdown_nanos = 0;
        descriptor.active_timeout_millis = 300_000;
        assert!(match_title(Some(&descriptor)).contains("倒數設定 0 秒"));
        assert!(match_title(Some(&descriptor)).contains("安全上限 300 秒"));
        assert_eq!(policy_seconds(1), "0.000000001");
        assert_eq!(policy_seconds(1_500_000_000), "1.5");
        assert!(
            !match_title(None).contains("安全上限"),
            "legacy authority did not send a policy"
        );
    }

    #[test]
    fn explicitly_disabled_features_do_not_display_as_waiting_for_missing_data() {
        use super::super::world_protocol::MatchDescriptor;
        let scene = super::super::world_runtime::tests::match_descriptor(false);
        for feature in [
            MatchDescriptor::COMBAT,
            MatchDescriptor::WEIGHT,
            MatchDescriptor::TIMED,
        ] {
            assert_eq!(
                missing_feature_notice(Some(&scene), feature, "等待", "未啟用"),
                "未啟用"
            );
            assert_eq!(
                missing_feature_notice(None, feature, "等待", "未啟用"),
                "等待"
            );
        }
        let combat = super::super::world_runtime::tests::match_descriptor(true);
        assert_eq!(
            missing_feature_notice(Some(&combat), MatchDescriptor::COMBAT, "等待", "未啟用"),
            "等待"
        );
        assert_eq!(
            missing_feature_notice(Some(&combat), MatchDescriptor::WEIGHT, "等待", "未啟用"),
            "未啟用"
        );
    }
    #[test]
    fn fire_key_routes_from_action_combo_without_stealing_selection_keys() {
        for key in [0x46, 0x54, 0x1b, 0x09, 0x70, 0x78] {
            assert!(!combo_keeps_key(key));
        }
        for key in [0x26, 0x28, 0x0d, 0x20, 0x41] {
            assert!(combo_keeps_key(key));
        }
        assert_eq!(key_action(0x46, false), Some(Action::FireStorm));
        assert_eq!(key_action(0x46, true), None);
        for dpi in [96, 144, 192, 384] {
            let top = scale(268, dpi);
            let height = scale(28, dpi);
            assert!(top - height > scale(234, dpi)); // above help, HP, weight and buffs
            assert!(top < scale(292, dpi) - scale(20, dpi)); // below status
        }
    }
    #[test]
    fn npc70_fire_and_death_confirmation_have_disjoint_native_rectangles_at_every_dpi() {
        for dpi in [48, 96, 120, 144, 192, 288, 384, 768] {
            for width in [0, 37, scale(WIDTH, dpi), scale(WIDTH * 2, dpi)] {
                for bottom in [
                    0,
                    100,
                    scale(HEIGHT + FOOTER, dpi),
                    scale(1200 + FOOTER, dpi),
                ] {
                    let (x, y, w, h) = death_control(width, bottom, dpi);
                    let game_bottom = (bottom - scale(FOOTER, dpi)).max(0);
                    let footer_top = (bottom - scale(302, dpi)).max(0);
                    let fire_top = (bottom - scale(268, dpi)).max(0);
                    assert!(x >= 0 && y >= 0 && w >= 0 && h >= 0);
                    assert!(x + w <= width && y + h <= game_bottom);
                    assert!(y + h <= footer_top && y + h <= fire_top);
                    assert_eq!(
                        x,
                        (width - w) / 2,
                        "centering depends on client size, never screen origin"
                    );
                    if width >= scale(WIDTH, dpi) && bottom >= scale(HEIGHT + FOOTER, dpi) {
                        assert_eq!((w, h), (scale(300, dpi), scale(28, dpi)));
                        assert!(y + h < game_bottom, "visible margin above the footer");
                    }
                }
            }
        }
    }
    #[test]
    fn seven_supply_controls_share_dpi_layout_without_viewport_or_return_overlap() {
        for dpi in [96, 144, 192, 384] {
            for width in [0, 37, scale(800, dpi)] {
                for i in 0..7 {
                    let (x, w, y) = supply_control(width, dpi, i, scale(824, dpi));
                    assert!(x >= 0 && w >= 0 && x + w <= width.max(0));
                    assert!(y >= scale(600, dpi));
                    if i != 2 && i != 6 {
                        let (next, _, ny) = supply_control(width, dpi, i + 1, scale(824, dpi));
                        assert_eq!(ny, y);
                        assert!(next >= x + w);
                    }
                }
            }
        }
    }
    #[test]
    fn arrow_controls_have_separate_dpi_row_and_do_not_steal_item_or_return_keys() {
        for dpi in [96, 144, 192, 384] {
            for width in [0, 37, scale(800, dpi)] {
                let bottom = scale(HEIGHT + FOOTER, dpi);
                let a = ammo_control(width, dpi, 0, bottom);
                let b = ammo_control(width, dpi, 1, bottom);
                assert!(a.0 >= 0 && a.1 >= 0 && a.0 + a.1 <= b.0 && b.0 + b.1 <= width.max(0));
                assert!(a.2 >= scale(HEIGHT, dpi));
                assert_eq!(a.2, b.2);
                assert!(a.2 + scale(28, dpi) <= supply_control(width, dpi, 0, bottom).2);
            }
        }
        for (key, id) in [(0x77, 40744), (0x78, 40746)] {
            assert_eq!(key_action(key, false), Some(Action::SelectAmmo(id)));
            assert_eq!(key_action(key, true), None);
        }
    }
    #[test]
    fn potion_hotkeys_are_exact_catalog_ids_not_arbitrary_source_instances_and_never_repeat() {
        for (i, k) in super::super::item_protocol::UI_ITEMS.iter().enumerate() {
            assert_eq!(key_action(0x70 + i, false), Some(Action::UseItem(*k)));
            assert_eq!(key_action(0x70 + i, true), None);
        }
        assert_eq!(key_action(0x77, false), Some(Action::SelectAmmo(40744)));
        assert_eq!(key_action(0x78, false), Some(Action::SelectAmmo(40746)));
        assert_eq!(key_action(0x77, true), None);
        assert_eq!(key_action(0x78, true), None);
        assert_eq!(key_action(0x79, false), None);
        assert_eq!(key_action(0x54, false), Some(Action::Teleport));
        assert_eq!(key_action(0x1b, false), Some(Action::Leave));
    }
    #[test]
    fn action_combo_and_buttons_remain_client_relative_under_dpi_and_resize() {
        for dpi in [96, 120, 144, 192, 288] {
            for width in [0, 200, 640, 800, 1200, 2400] {
                let columns = footer_controls(width, dpi);
                assert!(columns
                    .iter()
                    .all(|(x, w)| *x >= 0 && *w >= 0 && x + w <= width));
                assert!(columns.windows(2).all(|p| p[0].0 + p[0].1 <= p[1].0));
            }
        }
    }

    #[test]
    fn details_shortcuts_are_unmodified_nonrepeating_and_do_not_alias_game_actions() {
        for (id, key, page) in [
            (116, 0x43, DetailsPage::Character),
            (117, 0x49, DetailsPage::Inventory),
            (118, 0x50, DetailsPage::Skills),
        ] {
            assert_eq!(DetailsPage::from_button(id), Some(page));
            assert_eq!(details_key(key, false, false), Some(page));
            assert_eq!(details_key(key, true, false), None);
            assert_eq!(details_key(key, false, true), None);
            assert_eq!(key_action(key, false), None);
        }
        for id in [0, LEAVE, TELEPORT, CHAT, SETTINGS, DETAILS_TEXT, usize::MAX] {
            assert_eq!(DetailsPage::from_button(id), None);
        }
    }

    #[test]
    fn inspection_alt_mnemonics_are_shared_explicit_and_never_altgr_or_game_commands() {
        let alt = 1 << 29;
        for (key, page) in [
            (0x43, DetailsPage::Character),
            (0x49, DetailsPage::Inventory),
            (0x50, DetailsPage::Skills),
        ] {
            assert_eq!(details_alt_key(WM_SYSKEYDOWN, key, alt, false), Some(page));
            // Recognize (and consume) autorepeat without activating the page again.
            assert_eq!(
                details_alt_key(WM_SYSKEYDOWN, key, alt | (1 << 30), false),
                Some(page)
            );
            assert_eq!(details_alt_key(WM_SYSKEYDOWN, key, alt, true), None);
            assert_eq!(details_alt_key(WM_SYSKEYDOWN, key, 0, false), None);
            for message in [WM_KEYDOWN, WM_CHAR, WM_SYSCHAR, WM_KEYUP, WM_SYSKEYUP] {
                assert_eq!(details_alt_key(message, key, alt, false), None);
            }
        }
        // Native system controls, arrows, Esc, F10/F11 and mutations do not alias inspection.
        for key in [0x73, 0x79, 0x7a, 0x26, 0x28, 0x1b, 0x0d, 0x46, 0x54] {
            assert_eq!(details_alt_key(WM_SYSKEYDOWN, key, alt, false), None);
        }
    }

    #[test]
    fn details_owns_escape_scrolling_copy_and_game_keys_but_preserves_tab_and_enter() {
        assert_eq!(details_key_route(true, 0x1b), DetailsKeyRoute::Close);
        assert_eq!(details_key_route(false, 0x1b), DetailsKeyRoute::Continue);
        for key in [
            0x25, 0x26, 0x27, 0x28, 0x21, 0x22, 0x23, 0x24, 0x43, 0x41, 0x49, 0x50, 0x54, 0x46,
            0x70, 0x78,
        ] {
            assert_eq!(details_key_route(true, key), DetailsKeyRoute::Native);
            assert_eq!(details_key_route(false, key), DetailsKeyRoute::Continue);
        }
        for key in [0x09, 0x0d] {
            assert_eq!(details_key_route(true, key), DetailsKeyRoute::Continue);
        }
    }

    #[test]
    fn details_row_and_viewport_do_not_overlap_existing_footer_at_all_supported_dpi() {
        for dpi in [48, 96, 120, 144, 192, 288, 768] {
            let bottom = scale(HEIGHT + FOOTER, dpi);
            assert!(bottom - scale(FOOTER, dpi) <= bottom - scale(302, dpi));
            assert!(bottom - scale(302, dpi) + scale(28, dpi) <= bottom - scale(268, dpi));
            assert!(scale(16, dpi) <= scale(28, dpi));
            for width in [0, 1, 8, 200, 640, 800, 1200, 2400] {
                let columns = footer_controls(width, dpi);
                assert!(columns
                    .iter()
                    .all(|(x, w)| *x >= 0 && *w >= 0 && x + w <= width));
                assert!(columns.windows(2).all(|p| p[0].0 + p[0].1 <= p[1].0));
            }
        }
    }

    fn combat_surface() -> CombatSurface {
        let frame = super::super::world_runtime::tests::combat_frame();
        let actors = vec![(1, 370), (2, 420)]
            .into_iter()
            .map(|(entity_id, x)| ActorPlacement {
                visibility: None,
                entity_id,
                cell: frame.world.actors[(entity_id - 1) as usize].position,
                anchor: PixelPoint { x: x + 24, y: 300 },
                top_left: PixelPoint { x, y: 275 },
                width: 48,
                height: 60,
                graphics_id: 3213,
                source_frame: 1,
                sprite_name: "fixture".into(),
                subframe: 0,
            })
            .collect::<Vec<_>>();
        CombatSurface {
            frame: Arc::new(frame),
            actors: actors.into(),
            objectives: Arc::from([]),
            quests: Arc::from([]),
            time_attack_tower: None,
            siege_towers: Arc::from([]),
            npc: None,
        }
    }

    fn name_test_surface(combat: &CombatSurface) -> PaintedSurface {
        PaintedSurface {
            surface: HitSurface {
                match_id: combat.frame.world.match_id,
                generation: combat.frame.world.generation,
                revision: combat.frame.world.revision,
                selected_entity_id: 1,
                projection: ViewProjection {
                    anchor_cell: combat.frame.world.actors[0].position,
                    anchor_pixel: PixelPoint { x: 400, y: 300 },
                },
            },
            viewport: Viewport {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
        }
    }

    #[test]
    fn hover_uses_authoritative_player_and_source_name_including_self_without_attacking() {
        let mut combat = combat_surface();
        let frame = Arc::make_mut(&mut combat.frame);
        frame.world.room.members[0].name = "3333".into();
        frame.world.room.members[0].source_name = "太陽神".into();
        frame.world.room.members[1].name = "3333".into();
        frame.world.room.members[1].source_name = "月亮神".into();
        let painted = name_test_surface(&combat);
        let self_point = PixelPoint { x: 390, y: 290 };
        assert_eq!(
            hovered_player_name(painted, &combat, self_point).unwrap().0,
            "3333(太陽神)"
        );
        assert_eq!(
            combat_hit_action(painted, Some(&combat), self_point),
            Some(Action::CancelRoute)
        );
        let other = PixelPoint { x: 440, y: 290 };
        assert_eq!(
            hovered_player_name(painted, &combat, other).unwrap().0,
            "3333(月亮神)"
        );
        assert!(matches!(
            combat_hit_action(painted, Some(&combat), other),
            Some(Action::Attack {
                target_entity_id: 2,
                ..
            })
        ));
        // No persistent selected-target name; moving to ground or leaving the
        // world viewport immediately produces no label.
        for point in [
            PixelPoint { x: 300, y: 290 },
            PixelPoint { x: 800, y: 290 },
            PixelPoint { x: 0, y: 600 },
        ] {
            assert!(hovered_player_name(painted, &combat, point).is_none());
        }
        let frame = Arc::make_mut(&mut combat.frame);
        frame.world.room.members[1].kind = super::super::rooms::MemberKind::ServerBot;
        frame.world.actors[1].kind = super::super::rooms::MemberKind::ServerBot;
        assert_eq!(
            hovered_player_name(painted, &combat, other).unwrap().0,
            "3333(月亮神)"
        );
    }

    fn noncombat_name_fixture(
        frame: super::super::world_protocol::Frame,
    ) -> (PlayerSurface, PaintedSurface) {
        let combat = combat_surface();
        let mut painted = name_test_surface(&combat);
        painted.surface.match_id = frame.match_id;
        painted.surface.generation = frame.generation;
        painted.surface.revision = frame.revision;
        let mut actors = combat.actors.to_vec();
        actors.retain(|p| frame.actor_occupies(p.entity_id));
        for p in &mut actors {
            p.cell = frame.actor(p.entity_id).unwrap().position;
        }
        (
            PlayerSurface {
                frame: Arc::new(frame),
                actors: actors.into(),
            },
            painted,
        )
    }

    #[test]
    fn noncombat_and_race_hover_use_their_own_painted_roster_at_each_viewport_scale() {
        for frame in [
            super::super::world_runtime::tests::frame(1),
            super::super::race_protocol::tests::waiting(),
            super::super::race_protocol::tests::running(),
            super::super::race_protocol::tests::complete(),
        ] {
            let (world, mut painted) = noncombat_name_fixture(frame);
            validate_player_surface(painted.surface, &world).unwrap();
            for factor in [1, 2, 3] {
                painted.viewport = Viewport {
                    x: 35,
                    y: 47,
                    width: 800 * factor,
                    height: 600 * factor,
                };
                for (id, x) in [(1, 390), (2, 440)] {
                    let point = PixelPoint {
                        x: 35 + x * factor,
                        y: 47 + 290 * factor,
                    };
                    let hover = hovered_noncombat_name(painted, &world, point);
                    if world.frame.actor_occupies(id) {
                        let member = world.frame.member(id).unwrap();
                        assert_eq!(
                            hover.unwrap().0,
                            format!("{}({})", member.name, member.source_name)
                        );
                    } else {
                        assert!(hover.is_none(), "finished runners have no painted name");
                    }
                    assert!(matches!(
                        hit_action(painted, point),
                        Some(Action::Navigate { .. })
                    ));
                }
                assert!(
                    hovered_noncombat_name(painted, &world, PixelPoint { x: 0, y: 0 }).is_none()
                );
            }
        }
    }

    #[test]
    fn race_departure_keeps_the_roster_without_ghost_geometry_or_names() {
        use super::super::race_protocol::DnfReason;
        for reason in [DnfReason::Left, DnfReason::Disconnected] {
            let before = super::super::race_protocol::tests::running();
            let (previous, _) = noncombat_name_fixture(before.clone());
            let mut after = before;
            after.revision += 1;
            let runner = &mut after.race.as_mut().unwrap().runners[1];
            runner.revision = after.revision;
            runner.dnf = reason;
            let (world, painted) = noncombat_name_fixture(after);
            assert_eq!(world.frame.actors.len(), 2, "result identity is retained");
            assert_eq!(world.actors.len(), 1);
            validate_player_surface(painted.surface, &world).unwrap();
            assert!(
                hovered_noncombat_name(painted, &world, PixelPoint { x: 390, y: 290 }).is_some()
            );
            assert!(
                hovered_noncombat_name(painted, &world, PixelPoint { x: 440, y: 290 }).is_none()
            );
            let mut ghost = world.clone();
            ghost.actors = previous.actors.clone();
            assert!(validate_player_surface(painted.surface, &ghost).is_err());
            // An equally sized set containing only the retired runner is also invalid.
            ghost.actors = Arc::from([previous.actors[1].clone()]);
            assert!(validate_player_surface(painted.surface, &ghost).is_err());
        }
    }

    #[test]
    fn noncombat_name_surface_rejects_stale_missing_duplicate_geometry_and_missing_combat_authority(
    ) {
        let (world, painted) = noncombat_name_fixture(super::super::world_runtime::tests::frame(1));
        let point = PixelPoint { x: 440, y: 290 };
        for fault in 0..7 {
            let mut bad = world.clone();
            match fault {
                0 => Arc::make_mut(&mut bad.frame).generation += 1,
                1 => Arc::make_mut(&mut bad.frame).revision += 1,
                2 => Arc::make_mut(&mut bad.actors)[1].cell.x += 1,
                3 => Arc::make_mut(&mut bad.actors)[1].entity_id = 1,
                4 => Arc::make_mut(&mut bad.actors)[1].width = 0,
                5 => bad.actors = Arc::from([]),
                _ => {
                    Arc::make_mut(&mut bad.frame).descriptor =
                        Some(super::super::world_runtime::tests::match_descriptor(true))
                }
            }
            assert!(
                validate_player_surface(painted.surface, &bad).is_err(),
                "fault {fault}"
            );
            assert!(
                hovered_noncombat_name(painted, &bad, point).is_none(),
                "fault {fault}"
            );
        }
    }

    #[test]
    fn hover_does_not_reuse_old_world_revision_position_or_source_identity() {
        let combat = combat_surface();
        let painted = name_test_surface(&combat);
        let point = PixelPoint { x: 440, y: 290 };
        for fault in 0..5 {
            let mut bad = combat.clone();
            let frame = Arc::make_mut(&mut bad.frame);
            match fault {
                0 => frame.world.match_id += 1,
                1 => frame.world.generation += 1,
                2 => frame.world.revision += 1,
                3 => frame.world.room.members[1].key.source_id += 1,
                _ => frame.world.actors[1].position.x += 1,
            }
            assert!(
                hovered_player_name(painted, &bad, point).is_none(),
                "fault {fault}"
            );
            if fault < 3 {
                assert!(combat_hit_action(painted, Some(&bad), point).is_none());
            }
        }
    }

    #[test]
    fn hover_respects_npc_namespace_and_frontmost_depth_instead_of_showing_player_below() {
        let (mut combat, painted, npc) = npc_surface();
        let point = PixelPoint { x: 400, y: 280 };
        assert_eq!(npc.serial, painted.surface.selected_entity_id);
        assert!(hovered_player_name(painted, &combat, point).is_none());
        Arc::make_mut(&mut combat.npc.as_mut().unwrap().placements)[0]
            .anchor
            .y -= 1;
        let member = combat.frame.world.member(1).unwrap();
        assert_eq!(
            hovered_player_name(painted, &combat, point).unwrap().0,
            format!("{}({})", member.name, member.source_name)
        );
    }

    #[test]
    fn npc_hover_names_use_qualified_catalog_scope_life_and_frontmost_pixels() {
        use super::super::npc_protocol::Name;
        let (mut combat, painted, id) = npc_surface();
        let point = PixelPoint { x: 400, y: 280 };
        assert!(
            hovered_combat_name(painted, &combat, point).is_none(),
            "old protocol has no invented name"
        );
        let frame = Arc::make_mut(&mut combat.npc.as_mut().unwrap().reply)
            .frame
            .as_mut()
            .unwrap();
        let actor = frame.actors[0].clone();
        frame.names = Some(vec![
            Name {
                source_id: actor.catalog_source_id,
                npc_id: actor.npc_id,
                graphic: actor.graphic,
                name: "骷髏&(來源88)".into(),
                name_id: "$41".into(),
            },
            Name {
                source_id: actor.catalog_source_id,
                npc_id: actor.npc_id + 1,
                graphic: 2158,
                name: "未出生的狼".into(),
                name_id: String::new(),
            },
        ]);
        frame.validate().unwrap();
        assert_eq!(
            hovered_combat_name(painted, &combat, point).unwrap().0,
            "骷髏&(來源88)"
        );
        assert!(hovered_player_name(painted, &combat, point).is_none());
        assert!(
            matches!(combat_hit_action(painted, Some(&combat), point), Some(Action::AttackNpc { target, .. }) if target == id)
        );
        let mut stale = combat.clone();
        Arc::make_mut(&mut stale.npc.as_mut().unwrap().placements)[0].life += 1;
        assert!(hovered_combat_name(painted, &stale, point).is_none());
        let mut wrong_source = combat.clone();
        Arc::make_mut(&mut wrong_source.npc.as_mut().unwrap().reply)
            .frame
            .as_mut()
            .unwrap()
            .names
            .as_mut()
            .unwrap()[0]
            .source_id = 7;
        assert!(hovered_combat_name(painted, &wrong_source, point).is_none());
        Arc::make_mut(&mut combat.npc.as_mut().unwrap().placements)[0]
            .anchor
            .y -= 1;
        assert_eq!(
            hovered_combat_name(painted, &combat, point),
            hovered_player_name(painted, &combat, point)
        );
    }

    #[test]
    fn nameplate_is_client_relative_measured_and_clamped_for_dpi_letterbox_and_extreme_edges() {
        for dpi in [96, 120, 144, 192, 288] {
            for (width, height) in [(1, 1), (800, 600), (1600, 1200)] {
                let view = Viewport {
                    x: 35,
                    y: 47,
                    width,
                    height,
                };
                for text in [
                    SIZE { cx: 80, cy: 16 },
                    SIZE {
                        cx: i32::MAX,
                        cy: 32,
                    },
                ] {
                    for anchor in [
                        PixelPoint { x: 400, y: 275 },
                        PixelPoint {
                            x: i32::MIN,
                            y: i32::MIN,
                        },
                        PixelPoint {
                            x: i32::MAX,
                            y: i32::MAX,
                        },
                    ] {
                        let rect = nameplate_rect(view, anchor, text, dpi).unwrap();
                        assert!(rect.left >= view.x && rect.top >= view.y);
                        assert!(rect.right <= view.x + width && rect.bottom <= view.y + height);
                        assert!(rect.right > rect.left && rect.bottom > rect.top);
                    }
                }
            }
        }
        let view = name_test_surface(&combat_surface()).viewport;
        assert!(nameplate_rect(view, PixelPoint { x: 400, y: 275 }, SIZE::default(), 96).is_none());
    }

    #[test]
    fn npc77_hit_keeps_distinct_id_lives_and_rejects_stale_painted_frames() {
        let r = super::super::disintegrate_presentation::npc_tests::reply("normal-before");
        let f = r.frame.as_ref().unwrap();
        let mut c = combat_surface();
        c.frame = Arc::new(f.combat.clone());
        c.npc = Some(NpcSurface {
            reply: Arc::new(r.clone()),
            placements: Arc::from([]),
        });
        let surface = HitSurface {
            match_id: f.scope.match_id,
            generation: f.scope.generation,
            revision: f.combat.world.revision,
            selected_entity_id: 1,
            projection: ViewProjection {
                anchor_cell: f.combat.world.actors[0].position,
                anchor_pixel: PixelPoint { x: 400, y: 300 },
            },
        };
        let target = super::super::npc_protocol::Id {
            scope: f.scope,
            serial: f.actors[0].serial,
        };
        let attack = Some(Action::AttackNpc {
            surface,
            target,
            life: f.actors[0].life,
        });
        let expected = Some(Action::CastNpcDisintegrate {
            surface,
            target,
            caster_life: 1,
            target_life: 1,
        });
        assert_eq!(disintegrate_hit(attack, true, Some(&c)), expected);
        assert_eq!(disintegrate_hit(attack, false, Some(&c)), attack);
        assert_eq!(
            disintegrate_hit(
                Some(Action::Attack {
                    surface,
                    target_entity_id: target.serial
                }),
                true,
                Some(&c)
            ),
            Some(Action::CancelRoute)
        );
        for bad in [
            Action::AttackNpc {
                surface: HitSurface {
                    revision: surface.revision + 1,
                    ..surface
                },
                target,
                life: 1,
            },
            Action::AttackNpc {
                surface,
                target,
                life: 2,
            },
            Action::AttackNpc {
                surface,
                target: super::super::npc_protocol::Id {
                    serial: 99,
                    ..target
                },
                life: 1,
            },
            Action::AttackNpc {
                surface: HitSurface {
                    selected_entity_id: 99,
                    ..surface
                },
                target,
                life: 1,
            },
        ] {
            assert_eq!(
                disintegrate_hit(Some(bad), true, Some(&c)),
                Some(Action::CancelRoute)
            );
        }
        c.npc = None;
        assert_eq!(
            disintegrate_hit(attack, true, Some(&c)),
            Some(Action::CancelRoute)
        );
    }
    #[test]
    fn disintegrate_hit_captures_painted_lives_and_rejects_non_actor_or_stale_surface() {
        for name in ["normal-active", "honor-respawn"] {
            let mut c = combat_surface();
            c.frame = Arc::new(super::super::disintegrate_presentation::tests::frame(name));
            let f = &c.frame;
            let surface = HitSurface {
                match_id: f.world.match_id,
                generation: f.world.generation,
                revision: f.world.revision,
                selected_entity_id: f.world.actors[0].entity_id,
                projection: ViewProjection {
                    anchor_cell: f.world.actors[0].position,
                    anchor_pixel: PixelPoint { x: 400, y: 300 },
                },
            };
            let target = f.world.actors[1].entity_id;
            let attack = Some(Action::Attack {
                surface,
                target_entity_id: target,
            });
            assert_eq!(disintegrate_hit(attack, false, Some(&c)), attack);
            assert_eq!(
                disintegrate_hit(attack, true, Some(&c)),
                Some(Action::CastDisintegrate {
                    surface,
                    target_entity_id: target,
                    caster_life: super::super::disintegrate_cast_protocol::life(
                        f,
                        surface.selected_entity_id
                    )
                    .unwrap(),
                    target_life: super::super::disintegrate_cast_protocol::life(f, target).unwrap(),
                })
            );
            for invalid in [
                Action::Attack {
                    surface,
                    target_entity_id: surface.selected_entity_id,
                },
                Action::Attack {
                    surface: HitSurface {
                        revision: surface.revision + 1,
                        ..surface
                    },
                    target_entity_id: target,
                },
                Action::Attack {
                    surface: HitSurface {
                        generation: surface.generation + 1,
                        ..surface
                    },
                    target_entity_id: target,
                },
                Action::Attack {
                    surface,
                    target_entity_id: u64::MAX,
                },
                Action::Navigate {
                    surface,
                    goal: surface.projection.anchor_cell,
                },
                Action::CastLightArrow {
                    surface,
                    target_entity_id: target,
                },
            ] {
                assert_eq!(
                    disintegrate_hit(Some(invalid), true, Some(&c)),
                    Some(Action::CancelRoute)
                );
            }
            if let Some(h) = &f.honor {
                for o in &h.objectives {
                    assert_eq!(
                        disintegrate_hit(
                            Some(Action::Attack {
                                surface,
                                target_entity_id: o.entity_id
                            }),
                            true,
                            Some(&c)
                        ),
                        Some(Action::CancelRoute)
                    );
                }
            }
            assert_eq!(
                disintegrate_hit(attack, true, None),
                Some(Action::CancelRoute)
            );
            assert_eq!(disintegrate_hit(None, true, Some(&c)), None);
        }
    }

    #[test]
    fn disintegrate_magic_choice_never_relabels_existing_arrow_or_heal_indices() {
        for light in [false, true] {
            for heal in [false, true] {
                for dis in [false, true] {
                    let rows = magic_choices(light, heal, dis);
                    let expected = [(0, true), (4, light), (1, heal), (77, dis)]
                        .into_iter()
                        .filter_map(|(id, on)| on.then_some(id))
                        .collect::<Vec<_>>();
                    assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), expected);
                    if heal {
                        let label = rows.iter().find(|r| r.0 == 1).unwrap().1;
                        assert!(label.contains("合格目標") && label.contains("右鍵取消"));
                        assert!(!label.contains("同隊活塔"));
                    }
                }
            }
        }
    }

    #[test]
    fn combat_clicks_use_painted_fragment_coverage_at_each_viewport_scale() {
        use super::super::{
            compositor::Canvas,
            sprite::{CompactSpriteBlock, SpriteBlockReference, SpriteFrame, SpriteFrameFragments},
            tile::{ClassicTileType, DecodedTile},
            world_depth::NativeDrawCell,
            world_raster::{Blend, Raster},
        };
        let mut combat = combat_surface();
        let cell = WorldCell { x: 32768, y: 32768 };
        let mut canvas = Canvas::new(800, 600).unwrap();
        let mut rgba = [255, 0, 0, 255].repeat(576);
        rgba[..4].fill(0);
        let body = SpriteFrame {
            fragments: Some(SpriteFrameFragments {
                references: vec![SpriteBlockReference {
                    half_x: 0,
                    y: 0,
                    depth: 10,
                    block_id: 0,
                }],
                atlas: Arc::new(vec![CompactSpriteBlock::test_rgba(&rgba)]),
            }),
            x_offset: 0,
            y_offset: 0,
            width: 24,
            height: 24,
            frame_type: 10,
            unknown1: 0,
            unknown2: 0,
            rgba,
        };
        let mut wall = DecodedTile {
            tile_type: ClassicTileType::RleEven,
            source_offset: 0,
            x_offset: 0,
            y_offset: 0,
            content_width: 24,
            content_height: 24,
            width: 24,
            height: 24,
            rgba: vec![0; 2304],
        };
        for y in 0..24 {
            for x in 12..24 {
                wall.rgba[(y * 24 + x) * 4..(y * 24 + x) * 4 + 4]
                    .copy_from_slice(&[0, 0, 255, 255]);
            }
        }
        let mut raster = Raster::new(
            ViewProjection {
                anchor_cell: cell,
                anchor_pixel: PixelPoint { x: 444, y: 287 },
            },
            &canvas,
        );
        let origin = NativeDrawCell::from_world(cell).unwrap();
        raster.sprite(origin, &body, 1, Blend::Over).unwrap();
        let wall = super::super::world_raster::PreparedTile::new(wall).unwrap();
        raster.tile(origin, Some(20), &wall).unwrap();
        let mask = raster
            .paint(&mut canvas, &std::sync::atomic::AtomicBool::new(false), 1)
            .unwrap()
            .remove(0);
        let mut actors = combat.actors.to_vec();
        actors[1].visibility = Some(mask);
        combat.actors = actors.into();
        for factor in [1, 2, 3] {
            let painted = PaintedSurface {
                surface: HitSurface {
                    match_id: combat.frame.world.match_id,
                    generation: combat.frame.world.generation,
                    revision: combat.frame.world.revision,
                    selected_entity_id: 1,
                    projection: ViewProjection {
                        anchor_cell: combat.frame.world.actors[0].position,
                        anchor_pixel: PixelPoint { x: 400, y: 300 },
                    },
                },
                viewport: Viewport {
                    x: 100,
                    y: 50,
                    width: 800 * factor,
                    height: 600 * factor,
                },
            };
            let point = |x, y| PixelPoint {
                x: 100 + x * factor,
                y: 50 + y * factor,
            };
            assert_eq!(
                combat_hit_action(painted, Some(&combat), point(421, 275)),
                Some(Action::Attack {
                    surface: painted.surface,
                    target_entity_id: 2
                })
            );
            assert!(hovered_player_name(painted, &combat, point(421, 275)).is_some());
            for (x, y) in [(420, 275), (433, 275), (430, 310)] {
                assert!(hovered_player_name(painted, &combat, point(x, y)).is_none());
                assert!(
                    matches!(
                        combat_hit_action(painted, Some(&combat), point(x, y)),
                        Some(Action::Navigate { .. })
                    ),
                    "transparent/covered/outside sprite at {x},{y}"
                );
            }
        }
    }

    #[test]
    fn siege_hit_targets_are_three_typed_slots_not_players_and_portal_is_role_gated() {
        use super::super::{siege_presentation as tower, siege_protocol as siege};
        for name in ["active", "left", "right", "inner"] {
            let frame = siege::tests::sample(name).frame.unwrap();
            let d = frame.world.siege.as_ref().unwrap();
            let geometry = |id, cell, graphic, x| ActorPlacement {
                visibility: None,
                entity_id: id,
                cell,
                anchor: PixelPoint { x, y: 300 },
                top_left: PixelPoint { x: x - 20, y: 230 },
                width: 40,
                height: 90,
                graphics_id: graphic,
                source_frame: 0,
                sprite_name: "shared siege test geometry".into(),
                subframe: 0,
            };
            let slots: Vec<_> = siege::Slot::ALL
                .into_iter()
                .map(|slot| {
                    let t = &d.objectives[slot as usize];
                    tower::Placement {
                        target: siege::Target {
                            scope: siege::Scope::of(&frame.world),
                            slot,
                        },
                        alive: t.hp > 0,
                        geometry: geometry(
                            tower::draw_id(slot),
                            t.cell,
                            t.graphic,
                            200 + 100 * slot as i32,
                        ),
                    }
                })
                .collect();
            let mut surface = CombatSurface {
                frame: Arc::new(frame.clone()),
                actors: Arc::from([]),
                objectives: Arc::from([]),
                quests: Arc::from([]),
                time_attack_tower: None,
                siege_towers: slots.into(),
                npc: None,
            };
            for scale in [1, 2] {
                let painted = PaintedSurface {
                    surface: HitSurface {
                        match_id: frame.world.match_id,
                        generation: frame.world.generation,
                        revision: frame.world.revision,
                        selected_entity_id: 1,
                        projection: ViewProjection {
                            anchor_cell: frame.world.actors[0].position,
                            anchor_pixel: PixelPoint { x: 400, y: 300 },
                        },
                    },
                    viewport: Viewport {
                        x: 17,
                        y: 23,
                        width: 800 * scale,
                        height: 600 * scale,
                    },
                };
                for slot in siege::Slot::ALL {
                    let point = PixelPoint {
                        x: 17 + (200 + 100 * slot as i32) * scale,
                        y: 23 + 260 * scale,
                    };
                    let got = combat_hit_action(painted, Some(&surface), point);
                    let target = surface.siege_towers[slot as usize].target;
                    assert_eq!(
                        got,
                        Some(if d.attackable(frame.world.member(1).unwrap().team, slot) {
                            Action::AttackSiegeTower {
                                surface: painted.surface,
                                target,
                            }
                        } else {
                            Action::CancelRoute
                        })
                    );
                    assert_eq!(selected_hit(got, true), Some(Action::CancelRoute));
                    assert_eq!(healing_hit(got, true), Some(Action::CancelRoute));
                }
                let mut bad = surface.clone();
                let mut slots = bad.siege_towers.to_vec();
                slots[0].target.scope.generation += 1;
                bad.siege_towers = slots.into();
                assert_eq!(
                    combat_hit_action(
                        painted,
                        Some(&bad),
                        PixelPoint {
                            x: 17 + 200 * scale,
                            y: 23 + 260 * scale
                        }
                    ),
                    None
                );
            }
            // Defense portal never becomes a synthetic tower/player attack.
            let mut f = frame.clone();
            let entity = 2;
            f.world.actors[1].position = d.portal_inside;
            surface.frame = Arc::new(f);
            surface.actors = Arc::from([geometry(entity, d.portal_inside, 0, 500)]);
            let painted = PaintedSurface {
                surface: HitSurface {
                    match_id: frame.world.match_id,
                    generation: frame.world.generation,
                    revision: frame.world.revision,
                    selected_entity_id: entity,
                    projection: ViewProjection {
                        anchor_cell: d.portal_inside,
                        anchor_pixel: PixelPoint { x: 400, y: 300 },
                    },
                },
                viewport: Viewport {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                },
            };
            let action = combat_hit_action(painted, Some(&surface), PixelPoint { x: 500, y: 260 });
            assert_eq!(
                action,
                Some(
                    if frame.world.phase == super::super::world_protocol::Phase::Active {
                        Action::UseSiegePortal {
                            surface: painted.surface,
                            scope: siege::Scope::of(&frame.world),
                        }
                    } else {
                        Action::CancelRoute
                    }
                )
            );
        }
    }
    #[test]
    fn time_attack_tower_hit_is_typed_scaled_and_rejects_stale_or_missing_geometry() {
        use super::super::{time_attack_presentation as tower, time_attack_protocol as ta};
        for name in ["waiting", "tower"] {
            let reply = ta::tests::sample(name);
            let f = &reply.frame.as_ref().unwrap().combat;
            let d = f.world.time_attack.as_ref().unwrap();
            let id = ta::TowerId {
                scope: reply.echo.world,
            };
            let p = tower::Placement {
                id,
                alive: d.tower_hp > 0,
                geometry: ActorPlacement {
                    visibility: None,
                    entity_id: tower::DRAW_ID,
                    cell: d.tower_cell,
                    anchor: PixelPoint { x: 440, y: 300 },
                    top_left: PixelPoint { x: 410, y: 230 },
                    width: 60,
                    height: 90,
                    graphics_id: d.tower_graphic,
                    source_frame: 0,
                    sprite_name: "shared tower geometry test".into(),
                    subframe: 0,
                },
            };
            let surface = CombatSurface {
                frame: Arc::new(f.clone()),
                actors: Arc::from([]),
                objectives: Arc::from([]),
                quests: Arc::from([]),
                time_attack_tower: Some(p),
                siege_towers: Arc::from([]),
                npc: Some(NpcSurface {
                    reply: Arc::new(reply.clone()),
                    placements: Arc::from([]),
                }),
            };
            for scale in [1, 2] {
                let painted = PaintedSurface {
                    surface: HitSurface {
                        match_id: f.world.match_id,
                        generation: f.world.generation,
                        revision: f.world.revision,
                        selected_entity_id: 1,
                        projection: ViewProjection {
                            anchor_cell: f.world.actors[0].position,
                            anchor_pixel: PixelPoint { x: 400, y: 300 },
                        },
                    },
                    viewport: Viewport {
                        x: 17,
                        y: 23,
                        width: 800 * scale,
                        height: 600 * scale,
                    },
                };
                let point = PixelPoint {
                    x: 17 + 440 * scale as i32,
                    y: 23 + 260 * scale as i32,
                };
                let got = combat_hit_action(painted, Some(&surface), point);
                if name == "waiting" {
                    assert_eq!(
                        got,
                        Some(Action::AttackTimeAttackTower {
                            surface: painted.surface,
                            target: id
                        })
                    );
                    assert_eq!(selected_hit(got, true), Some(Action::CancelRoute));
                    assert_eq!(healing_hit(got, true), Some(Action::CancelRoute));
                } else {
                    assert_eq!(got, hit_action(painted, point));
                }
                for fault in 0..7 {
                    let mut bad = surface.clone();
                    let p = bad.time_attack_tower.as_mut().unwrap();
                    match fault {
                        0 => p.id.scope.generation += 1,
                        1 => p.geometry.entity_id = 1,
                        2 => p.geometry.cell.x += 1,
                        3 => p.geometry.graphics_id += 2,
                        4 => p.alive = !p.alive,
                        5 => p.geometry.width = 0,
                        _ => bad.time_attack_tower = None,
                    }
                    assert_eq!(combat_hit_action(painted, Some(&bad), point), None);
                }
            }
        }
    }
    #[test]
    fn quest_click_uses_typed_slot_painted_viewport_and_ruins_do_not_intercept_ground() {
        use super::super::{quest_presentation, quest_protocol as quest};
        for sample in ["before", "win"] {
            let reply = quest::tests::sample(sample);
            let frame = &reply.frame.as_ref().unwrap().combat;
            let q = frame.world.quest.as_ref().unwrap();
            let target = &q.targets[0];
            let id = quest::Id {
                scope: quest::scope(&frame.world),
                slot: target.slot,
            };
            let placement = quest_presentation::Placement {
                id,
                alive: q.actors[0].hp > 0,
                geometry: ActorPlacement {
                    visibility: None,
                    entity_id: quest_presentation::draw_id(id.slot),
                    cell: target.cell,
                    anchor: PixelPoint { x: 440, y: 300 },
                    top_left: PixelPoint { x: 410, y: 230 },
                    width: 60,
                    height: 90,
                    graphics_id: target.graphic,
                    source_frame: 0,
                    sprite_name: "qualified Quest geometry".into(),
                    subframe: 0,
                },
            };
            let surface = CombatSurface {
                frame: Arc::new(frame.clone()),
                actors: Arc::from([]),
                objectives: Arc::from([]),
                quests: Arc::from([placement]),
                time_attack_tower: None,
                siege_towers: Arc::from([]),
                npc: Some(NpcSurface {
                    reply: Arc::new(reply.clone()),
                    placements: Arc::from([]),
                }),
            };
            for scale in [1, 2] {
                let painted = PaintedSurface {
                    surface: HitSurface {
                        match_id: frame.world.match_id,
                        generation: frame.world.generation,
                        revision: frame.world.revision,
                        selected_entity_id: 1,
                        projection: ViewProjection {
                            anchor_cell: frame.world.actors[0].position,
                            anchor_pixel: PixelPoint { x: 400, y: 300 },
                        },
                    },
                    viewport: Viewport {
                        x: 17,
                        y: 23,
                        width: 800 * scale,
                        height: 600 * scale,
                    },
                };
                let point = PixelPoint {
                    x: 17 + 440 * scale as i32,
                    y: 23 + 260 * scale as i32,
                };
                let got = combat_hit_action(painted, Some(&surface), point);
                if sample == "before" {
                    assert_eq!(
                        got,
                        Some(Action::AttackQuestObjective {
                            surface: painted.surface,
                            target: id
                        })
                    );
                } else {
                    assert_eq!(got, hit_action(painted, point));
                }
                for fault in 0..5 {
                    let mut bad = surface.clone();
                    let p = &mut Arc::make_mut(&mut bad.quests)[0];
                    match fault {
                        0 => p.id.scope.generation += 1,
                        1 => p.id.slot = 8,
                        2 => p.geometry.cell.x += 1,
                        3 => p.geometry.entity_id = 1,
                        _ => p.alive = !p.alive,
                    }
                    assert!(combat_hit_action(painted, Some(&bad), point).is_none());
                }
            }
        }
    }

    #[test]
    fn objective_targets_use_the_same_painted_depth_order_and_healing_never_becomes_ground_move() {
        let frame = super::super::honor_protocol::tests::initial()
            .frame
            .unwrap();
        let tower = &frame.honor.as_ref().unwrap().objectives[0];
        let placement = ActorPlacement {
            visibility: None,
            entity_id: tower.entity_id,
            cell: tower.position,
            anchor: PixelPoint { x: 440, y: 300 },
            top_left: PixelPoint { x: 410, y: 230 },
            width: 60,
            height: 90,
            graphics_id: tower.graphics_id as u32,
            source_frame: 0,
            sprite_name: "qualified tower fixture".into(),
            subframe: 0,
        };
        let combat = CombatSurface {
            frame: Arc::new(frame),
            actors: Arc::from([]),
            objectives: Arc::from([placement]),
            quests: Arc::from([]),
            time_attack_tower: None,
            siege_towers: Arc::from([]),
            npc: None,
        };
        let painted = PaintedSurface {
            surface: HitSurface {
                match_id: combat.frame.world.match_id,
                generation: combat.frame.world.generation,
                revision: combat.frame.world.revision,
                selected_entity_id: 1,
                projection: ViewProjection {
                    anchor_cell: combat.frame.world.actors[0].position,
                    anchor_pixel: PixelPoint { x: 400, y: 300 },
                },
            },
            viewport: Viewport {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
        };
        let point = PixelPoint { x: 440, y: 270 };
        assert_eq!(
            combat_hit_action(painted, Some(&combat), point),
            Some(Action::Attack {
                surface: painted.surface,
                target_entity_id: 3
            })
        );
        assert_eq!(
            healing_hit(combat_hit_action(painted, Some(&combat), point), true),
            Some(Action::CastBasicHeal {
                surface: painted.surface,
                target_entity_id: 3
            })
        );
        assert_eq!(
            healing_hit(
                combat_hit_action(painted, Some(&combat), PixelPoint { x: 100, y: 500 }),
                true
            ),
            Some(Action::CancelRoute)
        );
        assert_eq!(healing_hit(None, true), None);
        let mut dead = combat.clone();
        let mut frame = (*dead.frame).clone();
        frame.honor.as_mut().unwrap().objectives[0].hp = 0;
        dead.frame = Arc::new(frame);
        assert_eq!(
            combat_hit_action(painted, Some(&dead), point),
            Some(Action::CancelRoute)
        );
        assert_eq!(
            combat.frame.world.actors.len(),
            2,
            "tower never enters source-character roster"
        );
    }

    #[test]
    fn combat_targets_use_last_painted_rectangles_self_dead_and_ground_are_distinct() {
        let combat = combat_surface();
        let painted = PaintedSurface {
            surface: HitSurface {
                match_id: combat.frame.world.match_id,
                generation: combat.frame.world.generation,
                revision: combat.frame.world.revision,
                selected_entity_id: 1,
                projection: ViewProjection {
                    anchor_cell: combat.frame.world.actors[0].position,
                    anchor_pixel: PixelPoint { x: 400, y: 300 },
                },
            },
            viewport: Viewport {
                x: 100,
                y: 50,
                width: 1600,
                height: 1200,
            },
        };
        let target = PixelPoint {
            x: 100 + 430 * 2,
            y: 50 + 300 * 2,
        };
        assert_eq!(
            combat_hit_action(painted, Some(&combat), target),
            Some(Action::Attack {
                surface: painted.surface,
                target_entity_id: 2
            })
        );
        assert_eq!(
            selected_hit(combat_hit_action(painted, Some(&combat), target), true),
            Some(Action::CastLightArrow {
                surface: painted.surface,
                target_entity_id: 2
            })
        );
        assert_eq!(
            selected_hit(
                Some(Action::Navigate {
                    surface: painted.surface,
                    goal: WorldCell { x: 1, y: 2 }
                }),
                true
            ),
            Some(Action::CancelRoute)
        );
        assert_eq!(selected_hit(None, true), None);
        let npc = super::super::npc_protocol::Id {
            scope: super::super::npc_protocol::Scope {
                room_id: 1,
                match_id: painted.surface.match_id,
                generation: painted.surface.generation,
            },
            serial: 2,
        };
        assert_eq!(
            selected_hit(
                Some(Action::AttackNpc {
                    surface: painted.surface,
                    target: npc,
                    life: 7
                }),
                true
            ),
            Some(Action::CastNpcLightArrow {
                surface: painted.surface,
                target: npc,
                life: 7
            })
        );
        assert_eq!(
            selected_hit(Some(Action::CancelRoute), true),
            Some(Action::CancelRoute)
        );
        assert_eq!(
            combat_hit_action(
                painted,
                Some(&combat),
                PixelPoint {
                    x: 100 + 390 * 2,
                    y: 50 + 300 * 2
                }
            ),
            Some(Action::CancelRoute)
        );
        assert!(matches!(
            combat_hit_action(painted, Some(&combat), PixelPoint { x: 1300, y: 850 }),
            Some(Action::Navigate { .. })
        ));
        assert_eq!(
            combat_hit_action(painted, Some(&combat), PixelPoint { x: 99, y: 100 }),
            None
        );
        let mut dead = combat.clone();
        let mut frame = (*dead.frame).clone();
        frame.vitals[1].hp = 0;
        frame.vitals[1].death_revision = 1;
        frame.vitals[1].hurt_revision = 1;
        dead.frame = Arc::new(frame);
        assert_eq!(
            combat_hit_action(painted, Some(&dead), target),
            Some(Action::CancelRoute)
        );
        // A future world/camera object exists but is not the one that was drawn.
        // Using the retained surface continues to select exactly the old image.
        let mut future = painted;
        future.surface.revision = 2;
        future.surface.projection.anchor_cell.x += 3;
        assert_ne!(future.surface, painted.surface);
        assert_eq!(
            combat_hit_action(painted, Some(&combat), target),
            Some(Action::Attack {
                surface: painted.surface,
                target_entity_id: 2
            })
        );
    }
    #[test]
    fn screen_directions_match_isometric_world_and_repeat_does_not_duplicate() {
        for (key, heading) in [
            (0x67, 0),
            (0x26, 1),
            (0x69, 2),
            (0x27, 3),
            (0x63, 4),
            (0x28, 5),
            (0x61, 6),
            (0x25, 7),
        ] {
            assert_eq!(key_action(key, false), Some(Action::Step(heading)));
            assert_eq!(key_action(key, true), None);
        }
        assert_eq!(key_action(0x1b, false), Some(Action::Leave));
        assert_eq!(key_action(0x1b, true), None);
        assert_eq!(key_action(0x20, false), None);
        assert_eq!(key_action(0x54, false), Some(Action::Teleport));
        assert_eq!(key_action(0x54, true), None);
        assert_eq!(key_action(0x46, false), Some(Action::FireStorm));
        assert_eq!(key_action(0x46, true), None);
    }
    #[test]
    fn leave_has_priority_over_single_bounded_move_intent() {
        let state = State {
            details_font: Cell::new(HFONT::default()),
            inventory: inventory_controls::Controls::default(),
            inventory_panel: RefCell::new(None),
            painted_inventory: Cell::new(None),
            hotbar: RefCell::new(None),
            painted_hotbar: Cell::new(None),
            skillbook: skill_controls::Controls::default(),
            skill_panel: RefCell::new(None),
            painted_skill: Cell::new(None),
            character_panel: RefCell::new(None),
            character_model: RefCell::new(None),
            character_selected: Cell::new(None),
            character_combo: Cell::new(None),
            painted_character: Cell::new(None),
            details_closed_by_escape: Cell::new(false),
            details: Cell::new(None),
            details_buttons: Cell::new(None),
            details_page: Cell::new(None),
            details_text: RefCell::new(String::new()),
            settings: Cell::new(None),
            settings_requested: Cell::new(false),
            chat: Cell::new(None),
            chat_requested: Cell::new(false),
            hwnd: Cell::new(None),
            leave: Cell::new(None),
            teleport: Cell::new(None),
            fire_storm: Cell::new(None),
            greater_haste: Cell::new(None),
            confirm_death: Cell::new(None),
            death_confirmation: Cell::new(None),
            canonical_revival: Cell::new(None),
            can_fire_storm: Cell::new(false),
            can_greater_haste: Cell::new(false),
            magic: Cell::new(None),
            items: Cell::new(None),
            ammo: Cell::new(None),
            can_select_ammo: Cell::new([false; 2]),
            can_use: Cell::new([false; 7]),
            item_labels: RefCell::new(std::array::from_fn(|_| String::new())),
            light_arrow_available: Cell::new(false),
            light_arrow_selected: Cell::new(false),
            can_light_arrow: Cell::new(true),
            basic_heal_available: Cell::new(false),
            basic_heal_selected: Cell::new(false),
            can_basic_heal: Cell::new(false),
            haste_available: Cell::new(false),
            haste_selected: Cell::new(false),
            can_haste: Cell::new(false),
            slow_available: Cell::new(false),
            slow_selected: Cell::new(false),
            can_slow: Cell::new(false),
            disintegrate_available: Cell::new(false),
            disintegrate_selected: Cell::new(false),
            can_disintegrate: Cell::new(false),
            image: RefCell::new(None),
            pending_hit: Cell::new(None),
            painted_hit: Cell::new(None),
            pending_combat: RefCell::new(None),
            painted_combat: RefCell::new(None),
            pending_players: RefCell::new(None),
            painted_players: RefCell::new(None),
            match_descriptor: RefCell::new(None),
            status_control: Cell::new(None),
            status: RefCell::new(vec![]),
            action: Cell::new(None),
            pending_battle: RefCell::new(None),
            painted_battle: RefCell::new(None),
            can_move: Cell::new(true),
            can_attack: Cell::new(true),
            timed_receipt: Cell::new(None),
            released: Cell::new(false),
        };
        queue(&state, Action::Step(1));
        queue(&state, Action::Step(3));
        assert_eq!(state.action.get(), Some(Action::Step(1)));
        for page in DetailsPage::ALL {
            *state.details_text.borrow_mut() = "old match / character data".into();
            unsafe {
                show_details(&state, Some(page));
            }
            assert_eq!(state.details_page.get(), Some(page));
            assert!(state.details_text.borrow().is_empty());
            assert_eq!(
                state.action.get(),
                Some(Action::Step(1)),
                "inspection does not replace an admitted game intent"
            );
        }
        unsafe {
            assert!(handle_details_escape(&state, 0x1b, false));
            for _ in 0..10 {
                assert!(
                    handle_details_escape(&state, 0x1b, true),
                    "holding Esc cannot leave after closing inspection"
                );
            }
            assert!(!handle_details_escape(&state, 0x54, false));
            assert!(
                !handle_details_escape(&state, 0x1b, false),
                "a new physical Esc press may request Leave"
            );
        }
        assert_eq!(state.details_page.get(), None);
        assert!(state.details_text.borrow().is_empty());
        assert_eq!(
            state.action.get(),
            Some(Action::Step(1)),
            "closing inspection is not Leave"
        );
        // A keyboard message for the lobby owner must close each world page
        // before global Esc can shut down its controller. Unrelated keys and
        // system-key/up messages never steal the page or mutate game state.
        for page in DetailsPage::ALL {
            unsafe { show_details(&state, Some(page)) };
            for message in [WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP, WM_CHAR] {
                assert!(!unsafe { handle_owner_shortcut(&state, message, 0x1b, false) });
                assert_eq!(state.details_page.get(), Some(page));
            }
            assert!(!unsafe { handle_owner_shortcut(&state, WM_KEYDOWN, 0x54, false) });
            state.released.set(true);
            assert!(!unsafe { handle_owner_shortcut(&state, WM_KEYDOWN, 0x1b, false) });
            assert_eq!(state.details_page.get(), Some(page));
            state.released.set(false);
            assert!(unsafe { handle_owner_shortcut(&state, WM_KEYDOWN, 0x1b, false) });
            assert_eq!(state.details_page.get(), None);
            for _ in 0..10 {
                assert!(unsafe { handle_owner_shortcut(&state, WM_KEYDOWN, 0x1b, true) });
            }
            assert_eq!(state.action.get(), Some(Action::Step(1)));
            assert!(state.details_text.borrow().is_empty());
            assert!(unsafe { handle_owner_shortcut(&state, WM_KEYDOWN, 0x1b, false) });
            assert_eq!(state.action.take(), Some(Action::Leave));
            // The first request may already have been consumed by the async
            // owner. Auto-repeat still cannot enqueue another Leave or close.
            for _ in 0..10 {
                assert!(unsafe { handle_owner_shortcut(&state, WM_KEYDOWN, 0x1b, true) });
                assert_eq!(state.action.get(), None);
            }
            state.action.set(Some(Action::Step(1)));
        }
        // The lobby owner must not swallow the world window's advertised
        // settings/chat shortcuts. They are local UI, not combat commands;
        // key-up, a retired world, and held-key repeats cannot reopen them.
        for page in DetailsPage::ALL {
            unsafe { show_details(&state, Some(page)) };
            for (key, flag) in [
                (0x79, &state.settings_requested),
                (0x7a, &state.chat_requested),
            ] {
                for message in [WM_KEYDOWN, WM_SYSKEYDOWN] {
                    flag.set(false);
                    state.released.set(true);
                    assert!(!unsafe { handle_owner_shortcut(&state, message, key, false) });
                    assert!(!flag.get());
                    state.released.set(false);
                    for ignored in [WM_KEYUP, WM_SYSKEYUP, WM_CHAR] {
                        assert!(!unsafe { handle_owner_shortcut(&state, ignored, key, false) });
                    }
                    assert!(unsafe { handle_owner_shortcut(&state, message, key, false) });
                    assert!(flag.replace(false));
                    for _ in 0..10 {
                        assert!(unsafe { handle_owner_shortcut(&state, message, key, true) });
                    }
                    assert!(!flag.get());
                    assert_eq!(state.details_page.get(), Some(page));
                    assert_eq!(state.action.get(), Some(Action::Step(1)));
                }
            }
        }
        unsafe { show_details(&state, None) };
        request_settings(&state);
        assert!(state.settings_requested.get());
        request_chat(&state);
        assert!(state.chat_requested.get());
        assert_eq!(
            state.action.get(),
            Some(Action::Step(1)),
            "local settings never replaces a game intent"
        );
        queue(&state, Action::Leave);
        assert_eq!(state.action.get(), Some(Action::Leave));
        assert!(!state.settings_requested.get());
        assert!(!state.chat_requested.get());
        request_chat(&state);
        assert!(!state.chat_requested.get());
        request_settings(&state);
        assert!(
            !state.settings_requested.get(),
            "leave wins over a later settings request"
        );
        queue(&state, Action::CancelRoute);
        assert_eq!(state.action.get(), Some(Action::Leave));
        queue(&state, Action::Teleport);
        assert_eq!(state.action.get(), Some(Action::Leave));
        state.can_fire_storm.set(true);
        queue(&state, Action::FireStorm);
        assert_eq!(state.action.get(), Some(Action::Leave));
        state.action.set(None);
        queue(&state, Action::FireStorm);
        assert_eq!(state.action.get(), Some(Action::FireStorm));
        assert!(!state.can_fire_storm.get());
        queue(&state, Action::FireStorm);
        assert_eq!(state.action.get(), Some(Action::FireStorm));
        queue(&state, Action::Leave);
        let proof = super::super::quest_life::Confirmation {
            scope: super::super::npc_protocol::Scope {
                room_id: 3,
                match_id: 4,
                generation: 5,
            },
            entity_id: 1,
            incarnation: 2,
            death_revision: 10,
        };
        state.death_confirmation.set(Some(proof));
        invalidate_input(&state);
        assert_eq!(
            state.action.get(),
            Some(Action::Leave),
            "invalidation preserves queued leave"
        );
        assert!(state.death_confirmation.get().is_none());
        state.action.set(None);
        queue(&state, Action::ConfirmDeath(proof));
        assert!(
            state.action.get().is_none(),
            "invalidated viewport cannot queue a retained death proof"
        );
        state.death_confirmation.set(Some(proof));
        queue(&state, Action::Leave);
        queue(&state, Action::ConfirmDeath(proof));
        assert_eq!(state.action.get(), Some(Action::Leave));
        assert!(
            state.death_confirmation.get().is_none(),
            "leave consumes the old proof too"
        );
        state.action.set(None);
        // Independent fresh-view case; clearing the intent alone must never
        // re-arm the proof consumed by Leave.
        state.death_confirmation.set(Some(proof));
        let mut stale = proof;
        stale.death_revision -= 1;
        queue(&state, Action::ConfirmDeath(stale));
        assert_eq!(state.action.get(), None);
        queue(&state, Action::ConfirmDeath(proof));
        assert_eq!(state.action.get(), Some(Action::ConfirmDeath(proof)));
        assert!(state.death_confirmation.get().is_none());
        state.action.set(None);
        queue(&state, Action::ConfirmDeath(proof));
        assert_eq!(
            state.action.get(),
            None,
            "a consumed proof cannot be queued twice"
        );
        let canonical = super::super::battle_revival::Request {
            stamp: super::super::battle_window::Stamp {
                context: super::super::application_protocol::SelectionContext {
                    key: super::super::application_protocol::CharacterKey {
                        source_id: 7,
                        object_id: 3333,
                    },
                    epoch: 9,
                },
                channel: 0,
                world: [3, 4, 5],
                entity: 1,
                life: 2,
                revision: 12,
            },
            death_revision: 10,
        };
        state.canonical_revival.set(Some(canonical));
        let mut stale = canonical;
        stale.stamp.life += 1;
        queue(&state, Action::CanonicalRevival(stale));
        assert_eq!(state.action.get(), None);
        queue(&state, Action::CanonicalRevival(canonical));
        assert_eq!(
            state.action.get(),
            Some(Action::CanonicalRevival(canonical))
        );
        assert!(state.canonical_revival.get().is_none());
        state.action.set(None);
        queue(&state, Action::CanonicalRevival(canonical));
        assert_eq!(state.action.get(), None);
        state.canonical_revival.set(Some(canonical));
        invalidate_input(&state);
        assert!(state.canonical_revival.get().is_none());
        queue(&state, Action::Leave);
        let surface = HitSurface {
            match_id: 1,
            generation: 1,
            revision: 1,
            selected_entity_id: 1,
            projection: ViewProjection {
                anchor_cell: WorldCell { x: 1, y: 1 },
                anchor_pixel: PixelPoint { x: 0, y: 0 },
            },
        };
        queue(
            &state,
            Action::CastLightArrow {
                surface,
                target_entity_id: 2,
            },
        );
        assert_eq!(state.action.get(), Some(Action::Leave));
        state.action.set(None);
        let dis = Action::CastDisintegrate {
            surface,
            target_entity_id: 2,
            caster_life: 1,
            target_life: 2,
        };
        queue(&state, dis);
        assert!(state.action.get().is_none());
        state.can_disintegrate.set(true);
        queue(&state, dis);
        assert_eq!(state.action.get(), Some(dis));
        assert!(!state.can_disintegrate.get());
        queue(&state, Action::Leave);
        assert_eq!(state.action.get(), Some(Action::Leave));
        invalidate_input(&state);
        assert!(!state.can_disintegrate.get());
        state.light_arrow_available.set(true);
        state.basic_heal_available.set(true);
        state.disintegrate_available.set(true);
        for (index, expected) in [
            (1, (true, false, false)),
            (2, (false, true, false)),
            (3, (false, false, true)),
            (99, (false, false, false)),
        ] {
            select_magic(&state, index);
            assert_eq!(
                (
                    state.light_arrow_selected.get(),
                    state.basic_heal_selected.get(),
                    state.disintegrate_selected.get()
                ),
                expected
            );
        }
        state.haste_available.set(true);
        state.slow_available.set(true);
        let rows = magic_rows(&state);
        for id in [29, 43, 1] {
            let index = rows.iter().position(|(skill, _)| *skill == id).unwrap();
            select_magic(&state, index as isize);
            assert_eq!(state.slow_selected.get(), id == 29);
            assert_eq!(state.haste_selected.get(), id == 43);
            assert_eq!(state.basic_heal_selected.get(), id == 1);
        }
        state.can_slow.set(true);
        invalidate_input(&state);
        assert!(
            !state.can_slow.get(),
            "invalidated pixels cannot dispatch a targeted Slow"
        );
        // Queue capture uses successfully painted canonical pixels, never a
        // pending frame or the next selection's current network observation.
        for action in [
            Action::Step(3),
            Action::Teleport,
            Action::FireStorm,
            Action::GreaterHaste,
            Action::UseItem(super::super::item_protocol::UI_ITEMS[0]),
            Action::SelectAmmo(super::super::ammo_protocol::UI_ITEMS[0]),
        ] {
            let surface = Arc::new(super::super::battle_window::tests::scene());
            let stamp = super::super::battle_window::Stamp::of(&surface.observation);
            *state.painted_battle.borrow_mut() = Some((
                surface,
                Viewport {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                },
            ));
            state.pending_battle.borrow_mut().take();
            state.action.set(None);
            state.can_move.set(true);
            state.can_greater_haste.set(true);
            state.can_use.set([true; 7]);
            state.can_select_ammo.set([true; 2]);
            queue(&state, action);
            let Some(Action::Canonical(intent)) = state.action.get() else {
                panic!("unstamped shortcut");
            };
            assert_eq!(intent.stamp, stamp);
            assert_eq!(
                intent.kind,
                super::super::battle_window::Kind::Local(
                    super::super::battle_window::Local::from_action(action).unwrap()
                )
            );
            invalidate_input(&state);
            assert!(
                state.action.get().is_none(),
                "invalidated mutation must be discarded"
            );
        }
        for action in [Action::Leave, Action::CancelRoute] {
            state.action.set(Some(action));
            invalidate_input(&state);
            assert_eq!(state.action.get(), Some(action));
        }
        state.action.set(None);
        state.can_greater_haste.set(true);
        queue(&state, Action::GreaterHaste);
        assert!(
            state.action.get().is_none(),
            "no painted canonical picture grants no G command"
        );
        invalidate_input(&state);
        assert!(!state.can_greater_haste.get());
    }
    #[test]
    fn physical_mouse_uses_last_painted_camera_and_letterbox_not_desktop_origin() {
        let surface = HitSurface {
            match_id: 7,
            generation: 9,
            revision: 10,
            selected_entity_id: 1,
            projection: ViewProjection {
                anchor_cell: WorldCell { x: 32724, y: 32784 },
                anchor_pixel: PixelPoint { x: 400, y: 300 },
            },
        };
        for (width, height) in [
            (800, 600),
            (1200, 900),
            (1000, 600),
            (1000, 1000),
            (1600, 1200),
        ] {
            let view = Viewport::fit(width, height, 800, 600).unwrap();
            let painted = PaintedSurface {
                surface,
                viewport: view,
            };
            let center = PixelPoint {
                x: view.x + view.width / 2,
                y: view.y + view.height / 2,
            };
            assert_eq!(
                hit_action(painted, center),
                Some(Action::Navigate {
                    surface,
                    goal: surface.projection.anchor_cell
                })
            );
            for point in [
                PixelPoint {
                    x: view.x - 1,
                    y: center.y,
                },
                PixelPoint {
                    x: view.x + view.width,
                    y: center.y,
                },
                PixelPoint {
                    x: center.x,
                    y: view.y - 1,
                },
                PixelPoint {
                    x: center.x,
                    y: view.y + view.height,
                },
            ] {
                assert_eq!(hit_action(painted, point), None);
            }
            let east = PixelPoint {
                x: view.x + 424 * view.width / 800,
                y: view.y + 288 * view.height / 600,
            };
            assert_eq!(
                hit_action(painted, east),
                Some(Action::Navigate {
                    surface,
                    goal: WorldCell { x: 32725, y: 32784 }
                })
            );
        }
        assert_eq!(
            mouse_point(0xfffe_ffffu32 as isize),
            PixelPoint { x: -1, y: -2 }
        );
        assert_eq!(
            mouse_point((900 << 16) | 1200),
            PixelPoint { x: 1200, y: 900 }
        );
    }
    #[test]
    fn right_click_cancels_even_when_movement_is_waiting_for_ack() {
        let state = State {
            details_font: Cell::new(HFONT::default()),
            inventory: inventory_controls::Controls::default(),
            inventory_panel: RefCell::new(None),
            painted_inventory: Cell::new(None),
            hotbar: RefCell::new(None),
            painted_hotbar: Cell::new(None),
            skillbook: skill_controls::Controls::default(),
            skill_panel: RefCell::new(None),
            painted_skill: Cell::new(None),
            character_panel: RefCell::new(None),
            character_model: RefCell::new(None),
            character_selected: Cell::new(None),
            character_combo: Cell::new(None),
            painted_character: Cell::new(None),
            details_closed_by_escape: Cell::new(false),
            details: Cell::new(None),
            details_buttons: Cell::new(None),
            details_page: Cell::new(None),
            details_text: RefCell::new(String::new()),
            settings: Cell::new(None),
            settings_requested: Cell::new(false),
            chat: Cell::new(None),
            chat_requested: Cell::new(false),
            hwnd: Cell::new(None),
            leave: Cell::new(None),
            teleport: Cell::new(None),
            fire_storm: Cell::new(None),
            greater_haste: Cell::new(None),
            confirm_death: Cell::new(None),
            death_confirmation: Cell::new(None),
            canonical_revival: Cell::new(None),
            can_fire_storm: Cell::new(false),
            can_greater_haste: Cell::new(false),
            magic: Cell::new(None),
            items: Cell::new(None),
            ammo: Cell::new(None),
            can_select_ammo: Cell::new([false; 2]),
            can_use: Cell::new([false; 7]),
            item_labels: RefCell::new(std::array::from_fn(|_| String::new())),
            light_arrow_available: Cell::new(false),
            light_arrow_selected: Cell::new(false),
            can_light_arrow: Cell::new(false),
            basic_heal_available: Cell::new(false),
            basic_heal_selected: Cell::new(false),
            can_basic_heal: Cell::new(false),
            haste_available: Cell::new(false),
            haste_selected: Cell::new(false),
            can_haste: Cell::new(false),
            slow_available: Cell::new(false),
            slow_selected: Cell::new(false),
            can_slow: Cell::new(false),
            disintegrate_available: Cell::new(false),
            disintegrate_selected: Cell::new(false),
            can_disintegrate: Cell::new(false),
            image: RefCell::new(None),
            pending_hit: Cell::new(None),
            painted_hit: Cell::new(None),
            pending_combat: RefCell::new(None),
            painted_combat: RefCell::new(None),
            pending_players: RefCell::new(None),
            painted_players: RefCell::new(None),
            match_descriptor: RefCell::new(None),
            status_control: Cell::new(None),
            status: RefCell::new(vec![]),
            action: Cell::new(Some(Action::Step(1))),
            pending_battle: RefCell::new(None),
            painted_battle: RefCell::new(None),
            can_move: Cell::new(false),
            can_attack: Cell::new(false),
            timed_receipt: Cell::new(None),
            released: Cell::new(false),
        };
        queue(&state, Action::CancelRoute);
        assert_eq!(state.action.get(), Some(Action::CancelRoute));
        queue(&state, Action::Step(2));
        assert_eq!(state.action.get(), Some(Action::CancelRoute));
        state.action.set(None);
        state.can_select_ammo.set([true, true]);
        queue(&state, Action::SelectAmmo(40744));
        assert_eq!(state.action.get(), Some(Action::SelectAmmo(40744)));
        assert_eq!(state.can_select_ammo.get(), [false, false]);
        queue(&state, Action::SelectAmmo(40746));
        assert_eq!(state.action.get(), Some(Action::SelectAmmo(40744)));
        queue(&state, Action::Leave);
        assert_eq!(state.action.get(), Some(Action::Leave));
        state.can_select_ammo.set([true, true]);
        queue(&state, Action::SelectAmmo(40746));
        assert_eq!(state.action.get(), Some(Action::Leave));
    }
}
