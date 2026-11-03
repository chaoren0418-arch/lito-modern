//! Owner-only presentation facts from one canonical observation. This does not
//! authorize SET, admit the complete world/modes, or prove temporal following.
//! No legacy World/Combat/Equipment reply or invented view token is constructed.
use super::{combat::require, data::*, invalid, ApplicationError};
use crate::modern_lito::{
    ammo_protocol as ammo, application_protocol as app, character_data as source,
    spell_protocol as skill, weight_protocol as weight, world_map::WorldCell,
    world_protocol::MatchDescriptor as Features,
};

/// Independently owned immutable UI input. Mutating the original decoded data
/// cannot change these checked facts or turn them into an equipment SET token.
#[derive(Clone, PartialEq, Eq)]
pub struct PersonalView {
    world: WorldId,
    revision: i64,
    personal: Personal,
    source: Option<source::CharacterData>,
    weight: Option<weight::Snapshot>,
    ammo: Option<ammo::State>,
}
impl std::fmt::Debug for PersonalView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CheckedBattlePersonal[redacted]")
    }
}
impl PersonalView {
    pub fn world(&self) -> &WorldId {
        &self.world
    }
    pub fn revision(&self) -> i64 {
        self.revision
    }
    pub fn player(&self) -> &Player {
        &self.personal.player
    }
    pub fn source(&self) -> Option<&source::CharacterData> {
        self.source.as_ref()
    }
    pub fn effective(&self) -> Option<&Effective> {
        self.personal.equipment.as_ref().map(|e| &e.effective)
    }
    pub fn equipped(&self, object: i64) -> Option<bool> {
        let e = self.personal.equipment.as_ref()?;
        e.equipped
            .0
            .binary_search_by_key(&object, |f| f.object)
            .ok()
            .map(|i| e.equipped.0[i].equipped)
    }
    pub fn lifetime(&self, object: i64) -> Option<&Lifetime> {
        let e = self.personal.equipment.as_ref()?;
        e.lifetimes
            .0
            .binary_search_by_key(&object, |f| f.object)
            .ok()
            .map(|i| &e.lifetimes.0[i])
    }
    pub fn skills(&self) -> &[Skill] {
        &self.personal.skills.0
    }
    pub fn supplies(&self) -> Option<&Supplies> {
        self.personal.supplies.as_ref()
    }
    pub fn weight(&self) -> Option<&weight::Snapshot> {
        self.weight.as_ref()
    }
    pub fn ammunition(&self) -> Option<&ammo::State> {
        self.ammo.as_ref()
    }
}

impl UnvalidatedData {
    /// Checks owner identity, combat accounting, and the complete private item
    /// view. Source/effective/live values remain separate. Mode proofs, cross-
    /// observation ordering and resource rendering are still separate gates.
    pub fn personal_view(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<PersonalView, ApplicationError> {
        if self.vip.is_some() {
            let view = super::vip::attached(self)?;
            view.check_values(player, world)?;
            return vip_view_facts(&view, player, world);
        }
        if self.armor.is_some() {
            super::armor::attached(self)?.check_values(player, world)?;
            return self.personal_facts(player, world);
        }
        if self.healing.is_some() {
            super::healing::attached(self)?.check_values(player, world)?;
            return self.personal_facts(player, world);
        }
        if self.speed.is_some() {
            super::speed::attached(self)?.check_values(player, world)?;
            return self.personal_facts(player, world);
        }
        if self.haste.is_some() {
            super::haste::attached(self)?.check_values(player, world)?;
            return self.personal_facts(player, world);
        }
        self.check_combat_values(player, world)?;
        self.personal_facts(player, world)
    }
    pub(super) fn personal_facts(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<PersonalView, ApplicationError> {
        self.personal_facts_with_vip(player, world, &[])
    }
    fn personal_facts_with_vip(
        &self,
        player: &Player,
        world: &WorldId,
        vip: &[super::vip::Item],
    ) -> Result<PersonalView, ApplicationError> {
        let p = &self.personal;
        let source = p
            .equipment
            .as_ref()
            .map(|e| equipment(e, player, vip))
            .transpose()?;
        self.personal_skills()?;
        let ammo = p.ammo.as_ref().map(ammunition).transpose()?;
        if let Some(a) = &ammo {
            self.personal_ammo(a)?;
        }
        let weight = match (&p.weight, &p.equipment) {
            (Some(w), Some(e)) => Some(carried_weight(
                w,
                e,
                p.supplies.as_ref().ok_or_else(invalid)?,
                ammo.as_ref(),
                p.materials.as_ref(),
            )?),
            (None, _) => None,
            _ => return Err(invalid()),
        };
        if let Some(c) = &p.regen_clock {
            require(
                (1_000_000_000..=1_801_000_000_000).contains(&c.next_second_elapsed_nanos)
                    && c.next_second_elapsed_nanos % 1_000_000_000 == 0
                    && (0..120).contains(&c.hp_counter)
                    && (0..64).contains(&c.mp_counter),
            )?;
        }
        Ok(PersonalView {
            world: world.clone(),
            revision: self.world.revision,
            personal: p.clone(),
            source,
            weight,
            ammo,
        })
    }
    fn personal_skills(&self) -> Result<(), ApplicationError> {
        let p = &self.personal;
        require(
            self.world.descriptor.features as u32 & Features::SPELLS != 0 || p.skills.0.is_empty(),
        )?;
        let skills = skill::SkillsReply {
            context: selection(&p.player)?,
            status: skill::Status::Ok,
            skills: p
                .skills
                .0
                .iter()
                .map(|s| {
                    Ok(skill::Skill {
                        skill_id: s.id.try_into().map_err(|_| invalid())?,
                        name: s.name.0.clone(),
                        mp_cost: s.mp_cost,
                        reuse_millis: s.reuse_millis,
                        support: match s.support {
                            Support::Teleport => skill::Support::Teleport,
                            Support::Unimplemented => skill::Support::Unimplemented,
                            Support::UnsupportedTemplate => skill::Support::UnsupportedTemplate,
                            Support::LightArrow => skill::Support::LightArrow,
                            Support::BasicHeal => skill::Support::BasicHeal,
                            Support::FireStorm => skill::Support::FireStorm,
                            Support::Disintegrate => skill::Support::Disintegrate,
                        },
                    })
                })
                .collect::<Result<Vec<_>, ApplicationError>>()?,
        };
        skills.validate()?;
        let owner = self
            .actors
            .0
            .iter()
            .find(|a| a.entity == p.entity)
            .ok_or_else(invalid)?;
        for s in &owner.spells.histories.0 {
            if s.receipt.is_none() {
                continue;
            }
            let (id, support) = match s.lane {
                Lane::Teleport => (5, Support::Teleport),
                Lane::LightArrow | Lane::NpcLightArrow => (4, Support::LightArrow),
                Lane::Healing => (1, Support::BasicHeal),
                Lane::FireStorm | Lane::NpcFireStorm => (70, Support::FireStorm),
                // These lanes use a separate server qualification catalog.
                // freshSkillsCharacter returns the raw source snapshot; only
                // the standalone SKILLS reply joins it to that catalog. LTB1
                // does not carry that qualified reply. Do not manufacture a
                // learned/support flag from a receipt, or reject valid history
                // because the raw list lacks the independent qualification.
                Lane::Disintegrate | Lane::NpcDisintegrate => continue,
                Lane::GreaterHaste => (54, Support::Unimplemented),
                Lane::Haste => (43, Support::Unimplemented),
                Lane::Slow => (29, Support::Unimplemented),
            };
            require(
                p.skills
                    .0
                    .iter()
                    .any(|s| s.id == id && s.support == support),
            )?;
        }
        Ok(())
    }
    fn personal_ammo(&self, a: &ammo::State) -> Result<(), ApplicationError> {
        let owner = self
            .actors
            .0
            .iter()
            .find(|a| a.entity == self.personal.entity)
            .ok_or_else(invalid)?;
        let vital = owner.vital.as_ref().ok_or_else(invalid)?;
        require(
            a.entity_id == self.personal.entity as u64
                && a.revision <= self.world.revision
                && a.attack_sequence <= vital.attack_sequence as u64
                && (!matches!(self.world.phase, Phase::Loading | Phase::Countdown)
                    || a.revision == 0),
        )?;
        if a.shot_count > 0 {
            let target = a.last_target_entity_id as i64;
            let present = self.actors.0.iter().any(|a| a.entity == target)
                || self.honor.objectives.0.iter().any(|t| t.entity == target);
            require(
                present
                    && a.shot_revision <= vital.attack_revision
                    && (a.shot_revision != vital.attack_revision || target == vital.target)
                    && self.manifest.contains_cell(&Cell {
                        x: a.from.x,
                        y: a.from.y,
                    })
                    && self.manifest.contains_cell(&Cell {
                        x: a.to.x,
                        y: a.to.y,
                    }),
            )?;
            // A retained shot belongs to its event-time loadout. Do not demand
            // the CURRENT weapon is still a bow or rewrite its old coordinates.
        }
        Ok(())
    }
}

fn selection(p: &Player) -> Result<app::SelectionContext, ApplicationError> {
    let c = app::SelectionContext {
        key: app::CharacterKey {
            source_id: p.source.try_into().map_err(|_| invalid())?,
            object_id: p.object.try_into().map_err(|_| invalid())?,
        },
        epoch: p.epoch as u64,
    };
    require(app::valid_context(c))?;
    Ok(c)
}

/// Only the complete, owner-bound VIP envelope can identify an applied ETC
/// item. A feature bit or ordinary equipped flag is not sufficient evidence.
pub(super) fn vip_facts(
    c: &super::vip::Candidate,
    player: &Player,
    world: &WorldId,
) -> Result<PersonalView, ApplicationError> {
    vip_view_facts(&c.view()?, player, world)
}
pub(super) fn vip_view_facts(
    c: &super::vip::View<'_>,
    player: &Player,
    world: &WorldId,
) -> Result<PersonalView, ApplicationError> {
    c.check_binding(player, world)?;
    c.data
        .personal_facts_with_vip(player, world, &c.owner.items.0)
}
fn equipment(
    e: &Equipment,
    p: &Player,
    vip: &[super::vip::Item],
) -> Result<source::CharacterData, ApplicationError> {
    let s = &e.saved;
    let source = source::CharacterData {
        context: selection(p)?,
        character: app::CharacterSummary {
            key: selection(p)?.key,
            name: e.name.0.clone(),
            class: e.class.try_into().map_err(|_| invalid())?,
            sex: e.sex.try_into().map_err(|_| invalid())?,
            level: e.level,
        },
        stats: source::SavedStats {
            max_hp: s.maximum_hp,
            max_mp: s.maximum_mp,
            saved_hp: s.saved_hp,
            saved_mp: s.saved_mp,
            armor_class: s.ac,
            strength: s.strength,
            dexterity: s.dexterity,
            constitution: s.constitution,
            intelligence: s.intelligence,
            wisdom: s.wisdom,
            charisma: s.charisma,
            experience: s.experience,
            lawful: s.lawful,
            original_strength: s.original_strength,
            original_dexterity: s.original_dexterity,
            original_constitution: s.original_constitution,
            original_intelligence: s.original_intelligence,
            original_wisdom: s.original_wisdom,
            food225: s.food225,
        },
        items: e
            .inventory
            .0
            .iter()
            .map(|i| {
                Ok(source::InventoryItem {
                    object_id: i.object.try_into().map_err(|_| invalid())?,
                    catalog_id: i.catalog.try_into().map_err(|_| invalid())?,
                    quantity: i.quantity,
                    enchantment: i.enchantment,
                    durability: i.durability,
                    blessing: i.blessing,
                    charge_count: i.charge_count,
                    inventory_graphic: i.inventory_graphic,
                    ground_graphic: i.ground_graphic,
                    unit_weight: i.unit_weight,
                    kind: match i.kind {
                        ItemKind::Weapon => source::ItemKind::Weapon,
                        ItemKind::Armor => source::ItemKind::Armor,
                        ItemKind::Etc => source::ItemKind::Etc,
                    },
                    equipped: i.equipped,
                    identified: i.identified,
                    sealed: i.sealed,
                    name_token: i.name.0.clone(),
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?,
    };
    source.validate()?;
    let v = &e.effective;
    require(
        e.source == p.source
            && e.object == p.object
            && e.revision >= 0
            && v.maximum_hp >= 1
            && v.maximum_mp >= 0
            && (0..=v.maximum_hp).contains(&v.initial_hp)
            && (0..=v.maximum_mp).contains(&v.initial_mp)
            && [
                v.strength,
                v.dexterity,
                v.constitution,
                v.intelligence,
                v.wisdom,
                v.charisma,
            ]
            .iter()
            .all(|n| *n >= 0)
            && e.evaluated_at_epoch_millis >= 0
            && e.valid_until_epoch_millis >= -1
            && (e.valid_until_epoch_millis < 0
                || e.valid_until_epoch_millis > e.evaluated_at_epoch_millis)
            && e.equipped.0.len() == e.inventory.0.len()
            && e.lifetimes.0.len() == e.inventory.0.len(),
    )?;
    let mut weapons = 0;
    for ((i, f), l) in e.inventory.0.iter().zip(&e.equipped.0).zip(&e.lifetimes.0) {
        require(
            f.object == i.object
                && l.object == i.object
                && l.expires_at_millis >= -1
                && matches!(l.state, LifetimeState::Active | LifetimeState::Expired)
                    == (l.expires_at_millis >= 0)
                && (!f.equipped
                    || usable(l.state)
                        && (i.kind != ItemKind::Etc
                            || vip.iter().any(|v| {
                                v.object == i.object
                                    && v.applied.catalog == i.catalog
                                    && v.lifetime == *l
                            }))),
        )?;
        match l.state {
            LifetimeState::Active => {
                require(l.expires_at_millis > e.evaluated_at_epoch_millis)?;
                if f.equipped {
                    require(
                        e.valid_until_epoch_millis >= 0
                            && e.valid_until_epoch_millis <= l.expires_at_millis,
                    )?;
                }
            }
            LifetimeState::Expired => require(l.expires_at_millis <= e.evaluated_at_epoch_millis)?,
            _ => {}
        }
        if f.equipped && i.kind == ItemKind::Weapon {
            weapons += 1;
        }
    }
    require(weapons <= 1)?;
    Ok(source)
}
fn usable(s: LifetimeState) -> bool {
    matches!(s, LifetimeState::Permanent | LifetimeState::Active)
}
fn carried_weight(
    w: &Weight,
    e: &Equipment,
    supplies: &Supplies,
    ammo: Option<&ammo::State>,
    materials: Option<&super::speed::Materials>,
) -> Result<weight::Snapshot, ApplicationError> {
    let stage = w.stage.try_into().map_err(|_| invalid())?;
    let v = weight::Snapshot {
        source_gear_weight: w.source_gear_weight,
        supplies_weight: w.supplies_weight,
        total: w.total_weight,
        maximum_bits: w.maximum_weight_bits as u64,
        stage,
        hit_penalty: weight::penalty(stage),
        units: w
            .units
            .0
            .iter()
            .map(|u| weight::UnitWeight {
                item_id: u.item,
                unit_weight: u.unit_weight,
            })
            .collect(),
    };
    require(v.applicable())?;
    let mut stocks = supplies
        .stocks
        .0
        .iter()
        .map(|s| (s.item, s.remaining))
        .collect::<Vec<_>>();
    if let Some(a) = ammo {
        stocks.extend(a.stocks.iter().map(|s| (s.facts.item_id, s.remaining)));
    }
    if let Some(m) = materials {
        for stock in &m.stocks.0 {
            require(
                v.units
                    .iter()
                    .any(|u| u.item_id == stock.item && u.unit_weight == stock.unit_weight),
            )?;
            stocks.push((stock.item, stock.remaining));
        }
    }
    stocks.sort_unstable_by_key(|s| s.0);
    require(
        stocks.len() <= if materials.is_some() { 10 } else { 9 }
            && stocks.windows(2).all(|s| s[0].0 < s[1].0),
    )?;
    v.validate_stocks(stocks)?;
    let mut total = 0i64;
    for (i, l) in e.inventory.0.iter().zip(&e.lifetimes.0) {
        if i.kind == ItemKind::Etc || l.state == LifetimeState::Expired {
            continue;
        }
        require(usable(l.state))?;
        total = total
            .checked_add(weight::stack_weight(i.quantity, i.unit_weight)?)
            .ok_or_else(invalid)?;
    }
    require(total == v.source_gear_weight)?;
    Ok(v)
}
fn ammunition(a: &Ammo) -> Result<ammo::State, ApplicationError> {
    let v = ammo::State {
        entity_id: a.entity.try_into().map_err(|_| invalid())?,
        request_sequence: a.request_sequence.try_into().map_err(|_| invalid())?,
        revision: a.revision,
        selection_revision: a.selection_revision,
        selected_item_id: a.selected_item,
        shot_count: a.shot_count.try_into().map_err(|_| invalid())?,
        shot_revision: a.shot_revision,
        shot_item_id: a.shot_item,
        attack_sequence: a.attack_sequence.try_into().map_err(|_| invalid())?,
        last_target_entity_id: a.last_target_entity.try_into().map_err(|_| invalid())?,
        from: WorldCell {
            x: a.from.x,
            y: a.from.y,
        },
        to: WorldCell {
            x: a.to.x,
            y: a.to.y,
        },
        projectile_sprite: a.projectile_sprite,
        hit: a.hit,
        stocks: a
            .stocks
            .0
            .iter()
            .map(|s| {
                Ok(ammo::Stock {
                    facts: ammo::Facts {
                        item_id: s.item,
                        small: s.small,
                        large: s.large,
                        raw_arrow_sprite: s.raw_arrow_sprite,
                        consume: s.consume,
                        material: s.material.try_into().map_err(|_| invalid())?,
                        blessing: s.blessing.try_into().map_err(|_| invalid())?,
                    },
                    initial: s.initial,
                    remaining: s.remaining,
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?,
    };
    v.validate()?;
    Ok(v)
}
