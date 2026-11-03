//! Inventory controls retain instance IDs and exact painted-life CAS proofs.
//! Saved tokens and canonical combat facts are never interchangeable.
use super::{
    battle_equipment::Request,
    battle_follow::Observation,
    battle_scene::Surface,
    battle_window::{Intent, Kind, Local, Stamp},
    character_data::ItemKind,
    equipment,
    login_window::LoginSnapshot,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub key: Key,
    pub label: String,
    /// Exact shared `list.ico` graphic when the server/source supplied one.
    /// A missing mapping is not replaced with an arbitrary item icon.
    pub graphic: Option<i64>,
    pub equip: Option<Request>,
    pub unequip: Option<Request>,
    pub local: Option<Intent>,
}
// Source object IDs and match-local catalog IDs inhabit different namespaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Source(u64),
    Supply(i64),
    Ammo(i64),
    Material(i64),
}
/// Read-only focus, not a SET/USE proof. Numeric IDs cannot retain focus across
/// a different character, attachment, world or life, even if the row is reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Selection {
    pub stamp: Stamp,
    pub key: Key,
}
impl Selection {
    pub(crate) fn current_owner(self, stamp: Stamp) -> bool {
        same_owner(self.stamp, stamp)
    }
}
fn same_owner(mut a: Stamp, mut b: Stamp) -> bool {
    a.revision = 0;
    b.revision = 0;
    a == b
}
impl Key {
    pub(crate) fn primary(self) -> &'static str {
        match self {
            Self::Source(_) => "裝備",
            Self::Supply(_) => "使用",
            Self::Ammo(_) => "選用",
            Self::Material(_) => "施法消耗",
        }
    }
    fn kind(self) -> Option<Kind> {
        Some(Kind::Local(match self {
            Self::Supply(id) => Local::UseItem(id),
            Self::Ammo(id) => Local::SelectAmmo(id),
            Self::Source(_) | Self::Material(_) => return None,
        }))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Model {
    pub stamp: Stamp,
    pub rows: Vec<Row>,
}
impl Model {
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        same_owner(self.stamp, other.stamp)
    }
    pub(crate) fn selection(&self, key: Key) -> Option<Selection> {
        self.rows.iter().any(|r| r.key == key).then_some(Selection {
            stamp: self.stamp,
            key,
        })
    }
    pub(crate) fn retain_selection(&self, old: Option<&Self>, key: Option<Key>) -> Option<Key> {
        let key = key?;
        (old.is_some_and(|old| self.same_owner(old)) && self.selection(key).is_some())
            .then_some(key)
    }
    pub(crate) fn request(&self, id: u64, desired: bool) -> Option<Request> {
        let r = self.rows.iter().find(|r| r.key == Key::Source(id))?;
        if desired {
            r.equip
        } else {
            r.unequip
        }
    }
    pub(crate) fn local(&self, key: Key) -> Option<Intent> {
        self.rows.iter().find(|r| r.key == key)?.local
    }
    pub(crate) fn clear_proofs(&mut self) {
        for row in &mut self.rows {
            row.equip = None;
            row.unequip = None;
            row.local = None;
        }
    }
}
pub(crate) fn model(
    surface: &Surface,
    view: &LoginSnapshot,
    label: impl Fn(&str) -> String,
) -> Option<Model> {
    let o = &surface.observation;
    if !super::battle_player::admitted_observation(o, view) {
        return None;
    }
    let source = o.personal().source();
    let gear = match &view.equipment {
        equipment::State::Ready(g) => Some(g),
        _ => None,
    };
    // Validate the complete source/token once, not O(inventory²) on each repaint.
    let seed =
        if !view.world_pending && !view.equipment_pending && view.canonical.equipment_supported() {
            gear.and_then(|g| Request::from_observation(o, g, g.flags.first()?.item_id, false).ok())
                .filter(|q| {
                    view.canonical
                        .observation()
                        .is_some_and(|now| q.current(now))
                })
        } else {
            None
        };
    let mut rows: Vec<Row> = source
        .into_iter()
        .flat_map(|s| &s.items)
        .map(|i| {
            let equipped = o.personal().equipped(i.object_id as i64);
            let request = |desired| {
                let mut q = seed?;
                let g = gear?;
                if !matches!(i.kind, ItemKind::Weapon | ItemKind::Armor)
                    || equipped? == desired
                    || desired && g.lifetime(i.object_id).is_some_and(|l| !l.state.usable())
                {
                    return None;
                }
                q.equipment = equipment::Request::set(g, i.object_id, desired);
                Some(q)
            };
            Row {
                key: Key::Source(i.object_id),
                graphic: Some(i.inventory_graphic),
                label: format!(
                    "來源{} · {} · #{} ×{}",
                    if equipped == Some(true) {
                        "已裝備"
                    } else {
                        "背包"
                    },
                    label(&i.name_token)
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(240)
                        .collect::<String>(),
                    i.object_id,
                    i.quantity
                ),
                equip: request(true),
                unequip: request(false),
                local: None,
            }
        })
        .collect();
    let stamp = Stamp::of(o);
    let ready = super::battle_player::controls_ready(surface, view);
    let mut append = |key: Key, graphic: Option<i64>, name: String, remaining: i64, selected: bool| {
        let kind = key.kind().unwrap();
        let local = (ready
            && remaining > 0
            && !selected
            && view.canonical.observation().is_some_and(|current| {
                stocks_current(kind, &o.data().personal, &current.data().personal)
            })
            && super::battle_input::ready(kind, view))
        .then_some(Intent { stamp, kind });
        rows.push(Row {
            key,
            graphic,
            label: format!(
                "本場{} · {} · 剩餘 {}{}",
                match key {
                    Key::Supply(_) => "補給",
                    _ => "箭矢",
                },
                name.chars()
                    .filter(|c| !c.is_control())
                    .take(240)
                    .collect::<String>(),
                remaining,
                if selected { "／已選用" } else { "" }
            ),
            equip: None,
            unequip: None,
            local,
        });
    };
    if let Some(s) = o.personal().supplies() {
        for stock in &s.stocks.0 {
            let token = match stock.item {
                40010 => "$26".into(),
                40011 => "$235".into(),
                40012 => "$238".into(),
                id => match super::timed_protocol::Kind::from_item(id) {
                    Some(k) => format!("${}", k.description_id()),
                    None => continue,
                },
            };
            append(
                Key::Supply(stock.item),
                super::item_protocol::healing(stock.item)
                    .map(|kind| kind.inventory_graphic)
                    .or_else(|| super::timed_protocol::Kind::from_item(stock.item).map(|kind| kind.icon())),
                label(&token),
                stock.remaining,
                false,
            );
        }
    }
    if let Some(a) = &o.data().personal.ammo {
        for stock in &a.stocks.0 {
            let name = match stock.item {
                40744 => "銀箭",
                40746 => "米索莉箭",
                _ => continue,
            };
            append(
                Key::Ammo(stock.item),
                None,
                name.into(),
                stock.remaining,
                a.selected_item == stock.item,
            );
        }
    }
    // Materials are paid by confirmed spells, never by a separate inventory
    // Use/Equip command. Keep exhausted stock visible and selection stable.
    if let Some(materials) = &o.data().personal.materials {
        for stock in &materials.stocks.0 {
            let name = material_name(stock.item, &label);
            rows.push(Row {
                key: Key::Material(stock.item),
                graphic: None,
                label: format!(
                    "本場材料 · {} · 剩餘 {}／已消耗 {}",
                    name.chars()
                        .filter(|c| !c.is_control())
                        .take(240)
                        .collect::<String>(),
                    stock.remaining,
                    stock.initial - stock.remaining
                ),
                equip: None,
                unequip: None,
                local: None,
            });
        }
    }
    Some(Model { stamp, rows })
}

pub(crate) fn material_name(item: i64, label: impl Fn(&str) -> String) -> String {
    match item {
        40318 => label("$508"),
        _ => format!("#{item}"),
    }
}

pub(crate) fn command(
    q: Request,
    painted: &Observation,
    view: &LoginSnapshot,
) -> Option<super::battle_control::Command> {
    if view.world_pending
        || !super::battle_player::admitted_observation(painted, view)
        || !q.current(painted)
    {
        return None;
    }
    let equipment::State::Ready(g) = &view.equipment else {
        return None;
    };
    let rebuilt =
        Request::from_observation(painted, g, q.equipment.item_id, q.equipment.desired).ok()?;
    // A newer painted world revision is fine; never replace the original CAS
    // or life proof with newly selected data to rescue a stale click.
    if rebuilt.equipment != q.equipment {
        return None;
    }
    let command = super::battle_control::Command::equipment(q);
    command
        .current(&view.canonical, view.navigation.as_ref())
        .then_some(command)
}

/// A page click cannot silently become a second consumption/selection after a
/// previous receipt or arrow shot changed the stocks shown to the player.
pub(crate) fn local_command(
    intent: Intent,
    surface: &Surface,
    view: &LoginSnapshot,
) -> Option<super::battle_control::Command> {
    let painted = &surface.observation.data().personal;
    let current = &view.canonical.observation()?.data().personal;
    if !stocks_current(intent.kind, painted, current) {
        return None;
    }
    super::battle_input::command(surface, intent, view).ok()
}
fn stocks_current(
    kind: Kind,
    painted: &super::battle_payload::data::Personal,
    current: &super::battle_payload::data::Personal,
) -> bool {
    match kind {
        Kind::Local(Local::UseItem(_))
            if painted.supplies.is_some() && painted.supplies == current.supplies =>
        {
            true
        }
        Kind::Local(Local::SelectAmmo(id))
            if painted.ammo.is_some()
                && painted.ammo == current.ammo
                && painted.ammo.as_ref().is_some_and(|a| a.selected_item != id) =>
        {
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::application_protocol::{CharacterKey, SelectionContext};
    fn model() -> Model {
        let context = SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: 1 << 63,
        };
        let gear = equipment::tests::sample_snapshot(context);
        let stamp = Stamp {
            context,
            channel: 2,
            world: [8, 9, 10],
            entity: 1,
            life: 2,
            revision: 7,
        };
        Model {
            stamp,
            rows: [1, 2]
                .map(|id| Row {
                    key: Key::Source(id),
                    graphic: None,
                    label: format!("#{id}"),
                    equip: Some(Request {
                        stamp,
                        equipment: equipment::Request::set(&gear, id, true),
                    }),
                    unequip: None,
                    local: None,
                })
                .into(),
        }
    }
    #[test]
    fn row_reordering_never_changes_selected_instance_or_rescues_unknown_id() {
        let mut m = model();
        let first = m.request(1, true).unwrap();
        m.rows.reverse();
        assert_eq!(m.request(1, true), Some(first));
        assert!(
            m.request(0, true).is_none()
                && m.request(3, true).is_none()
                && m.request(1, false).is_none()
        );
        m.rows.retain(|r| r.key != Key::Source(1));
        assert!(m.request(1, true).is_none());
    }
    #[test]
    fn selection_survives_only_same_owner_world_and_life_not_reused_numeric_ids() {
        let m = model();
        let mut later = m.clone();
        later.stamp.revision += 100;
        assert!(m.same_owner(&later));
        for field in 0..9 {
            let mut other = m.clone();
            match field {
                0 => other.stamp.context.key.source_id += 1,
                1 => other.stamp.context.key.object_id += 1,
                2 => other.stamp.context.epoch += 1,
                3 => other.stamp.channel ^= 1,
                4..=6 => other.stamp.world[field - 4] += 1,
                7 => other.stamp.entity += 1,
                _ => other.stamp.life += 1,
            }
            assert!(!m.same_owner(&other), "field {field}");
        }
    }
    #[test]
    fn source_objects_and_match_catalogs_do_not_alias_even_with_identical_numbers() {
        let mut m = model();
        for key in [Key::Supply(1), Key::Ammo(1)] {
            m.rows.push(Row {
                key,
                graphic: None,
                label: "本場".into(),
                equip: None,
                unequip: None,
                local: Some(Intent {
                    stamp: m.stamp,
                    kind: key.kind().unwrap(),
                }),
            });
        }
        assert!(m.request(1, true).is_some());
        assert!(m.local(Key::Source(1)).is_none());
        assert_eq!(
            m.local(Key::Supply(1)).unwrap().kind,
            Kind::Local(Local::UseItem(1))
        );
        assert_eq!(
            m.local(Key::Ammo(1)).unwrap().kind,
            Kind::Local(Local::SelectAmmo(1))
        );
        m.rows.push(Row {
            key: Key::Material(1),
            graphic: None,
            label: "本場材料".into(),
            equip: None,
            unequip: None,
            local: None,
        });
        assert!(m.selection(Key::Material(1)).is_some());
        assert!(m.local(Key::Material(1)).is_none());
        m.rows.reverse();
        assert!(m.request(1, true).is_some());
        m.clear_proofs();
        assert!(m
            .rows
            .iter()
            .all(|r| r.equip.is_none() && r.unequip.is_none() && r.local.is_none()));
        assert_eq!(m.rows.len(), 5);
    }
    #[test]
    fn buttons_map_only_the_selected_namespace_and_no_source_item_is_consumed() {
        assert_eq!(Key::Source(40010).primary(), "裝備");
        assert!(Key::Source(40010).kind().is_none());
        assert_eq!(Key::Supply(40010).primary(), "使用");
        assert_eq!(Key::Ammo(40744).primary(), "選用");
        assert_eq!(Key::Material(40318).primary(), "施法消耗");
        assert!(Key::Material(40318).kind().is_none());
        assert_eq!(material_name(40318, str::to_owned), "$508");
        assert_eq!(
            material_name(40010, |_| panic!("unqualified token")),
            "#40010"
        );
        for id in super::super::item_protocol::UI_ITEMS {
            assert_eq!(
                Key::Supply(id).kind(),
                Some(Kind::Local(Local::UseItem(id)))
            );
        }
        for id in super::super::ammo_protocol::UI_ITEMS {
            assert_eq!(
                Key::Ammo(id).kind(),
                Some(Kind::Local(Local::SelectAmmo(id)))
            );
        }
    }
    #[test]
    fn read_only_focus_is_stable_by_namespace_and_cleared_with_owner_or_row() {
        let old = model();
        let key = Key::Source(1);
        let focus = old.selection(key).unwrap();
        let mut next = old.clone();
        next.stamp.revision += 1;
        next.rows.reverse();
        next.rows[1].label = "更新名稱".into();
        next.clear_proofs();
        assert_eq!(next.retain_selection(Some(&old), Some(key)), Some(key));
        assert!(focus.current_owner(next.stamp));
        assert!(
            next.selection(key).is_some(),
            "read-only focus does not authorize SET"
        );
        assert!(next.request(1, true).is_none());
        assert!(next.retain_selection(None, Some(key)).is_none());
        assert!(next.retain_selection(Some(&old), None).is_none());
        assert!(next
            .retain_selection(Some(&old), Some(Key::Supply(1)))
            .is_none());
        assert!(next
            .retain_selection(Some(&old), Some(Key::Ammo(1)))
            .is_none());
        for field in 0..9 {
            let mut changed = next.clone();
            match field {
                0 => changed.stamp.context.key.source_id += 1,
                1 => changed.stamp.context.key.object_id += 1,
                2 => changed.stamp.context.epoch ^= 1,
                3 => changed.stamp.channel ^= 1,
                4..=6 => changed.stamp.world[field - 4] += 1,
                7 => changed.stamp.entity += 1,
                _ => changed.stamp.life += 1,
            }
            assert!(!focus.current_owner(changed.stamp), "field {field}");
            assert!(changed.retain_selection(Some(&old), Some(key)).is_none());
        }
        next.rows.retain(|r| r.key != key);
        assert!(next.retain_selection(Some(&old), Some(key)).is_none());
    }
    #[test]
    fn material_focus_survives_exhaustion_but_not_source_or_life_reuse() {
        let mut old = model();
        let key = Key::Material(40318);
        old.rows.push(Row {
            key,
            graphic: None,
            label: "剩餘 1".into(),
            equip: None,
            unequip: None,
            local: None,
        });
        let focus = old.selection(key).unwrap();
        let mut next = old.clone();
        next.stamp.revision += 1;
        next.rows.last_mut().unwrap().label = "剩餘 0".into();
        assert_eq!(next.retain_selection(Some(&old), Some(key)), Some(key));
        assert!(focus.current_owner(next.stamp));
        assert!(next.local(key).is_none());
        for field in 0..9 {
            let mut other = next.clone();
            match field {
                0 => other.stamp.context.key.source_id += 1,
                1 => other.stamp.context.key.object_id += 1,
                2 => other.stamp.context.epoch ^= 1,
                3 => other.stamp.channel ^= 1,
                4..=6 => other.stamp.world[field - 4] += 1,
                7 => other.stamp.entity += 1,
                _ => other.stamp.life += 1,
            }
            assert!(other.retain_selection(Some(&old), Some(key)).is_none());
            assert!(!focus.current_owner(other.stamp));
        }
        next.rows.retain(|r| r.key != key);
        assert!(next.retain_selection(Some(&old), Some(key)).is_none());
    }
    #[test]
    fn old_stock_proofs_reject_consumption_selection_and_arrow_shots() {
        use crate::modern_lito::battle_payload::{data as d, data_tests::fixture};
        let mut p = fixture().personal;
        let supply = Kind::Local(Local::UseItem(40010));
        let arrow = Kind::Local(Local::SelectAmmo(40746));
        assert!(!stocks_current(supply, &p, &p));
        assert!(!stocks_current(arrow, &p, &p));
        p.supplies = Some(d::Supplies {
            request_sequence: 0,
            request_revision: 1,
            history: d::Items {
                equipment_revision: 0,
                profile_revision: 0,
                fixed_recovery: 0,
                uses: 0,
                restored: 0,
                last: None,
            },
            stocks: d::List(vec![d::Stock {
                item: 40010,
                initial: 3,
                remaining: 3,
            }]),
            effect: 0,
            delay_millis: 0,
        });
        p.ammo = Some(d::Ammo {
            entity: 1,
            request_sequence: 1,
            revision: 1,
            selection_revision: 1,
            selected_item: 40744,
            shot_count: 0,
            shot_revision: 0,
            shot_item: 0,
            attack_sequence: 0,
            last_target_entity: 0,
            from: d::Cell { x: 0, y: 0 },
            to: d::Cell { x: 0, y: 0 },
            projectile_sprite: 0,
            hit: false,
            stocks: d::List(vec![d::AmmoStock {
                item: 40746,
                small: 10,
                large: 9,
                raw_arrow_sprite: -1,
                consume: true,
                material: 1,
                blessing: 0,
                initial: 3,
                remaining: 3,
            }]),
        });
        assert!(stocks_current(supply, &p, &p));
        assert!(stocks_current(arrow, &p, &p));
        assert!(!stocks_current(
            Kind::Local(Local::SelectAmmo(40744)),
            &p,
            &p
        ));
        assert!(!stocks_current(Kind::Local(Local::Step(0)), &p, &p));
        for field in 0..3 {
            let mut next = p.clone();
            let s = next.supplies.as_mut().unwrap();
            match field {
                0 => s.request_sequence += 1,
                1 => s.stocks.0[0].remaining -= 1,
                _ => s.request_revision += 1,
            }
            assert!(!stocks_current(supply, &p, &next));
            assert!(
                stocks_current(arrow, &p, &next),
                "independent item/ammo counters"
            );
        }
        for field in 0..5 {
            let mut next = p.clone();
            let a = next.ammo.as_mut().unwrap();
            match field {
                0 => a.request_sequence += 1,
                1 => a.selected_item = 40746,
                2 => a.shot_count += 1,
                3 => a.stocks.0[0].remaining -= 1,
                _ => a.revision += 1,
            }
            assert!(!stocks_current(arrow, &p, &next));
            assert!(stocks_current(supply, &p, &next));
        }
    }
}
