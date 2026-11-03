use super::*;

fn framing() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend(super::super::MAGIC.to_be_bytes());
    b.extend(SCHEMA.to_be_bytes());
    b.extend([0; 6]);
    b.extend((SECTION as u16).to_be_bytes());
    b.extend([0; 2]);
    for i in 1..=SECTION {
        b.extend((i as u16).to_be_bytes());
        b.extend([0; 2]);
        b.extend(1u32.to_be_bytes());
        b.push(0);
    }
    let n = b.len() as u32;
    b[8..12].copy_from_slice(&n.to_be_bytes());
    b
}
#[test]
fn appearance_section_has_its_own_bounded_schema_and_exact_scalar_layout() {
    let mut bytes = framing();
    bytes[4..6].copy_from_slice(&APPEARANCE_SCHEMA.to_be_bytes());
    bytes[12..14].copy_from_slice(&(APPEARANCE_SECTION as u16).to_be_bytes());
    let start = bytes.len();
    bytes.extend(19u16.to_be_bytes());
    bytes.extend([0; 2]);
    bytes.extend(4u32.to_be_bytes());
    bytes.extend(0i32.to_be_bytes());
    let n = bytes.len() as u32;
    bytes[8..12].copy_from_slice(&n.to_be_bytes());
    assert_eq!(inspect(&bytes).unwrap().schema(), 11);
    for end in 0..bytes.len() {
        assert!(inspect(&bytes[..end]).is_err());
    }
    for size in [0, 3, 5, APPEARANCE_LIMIT as u32 + 1, u32::MAX] {
        let mut bad = bytes.clone();
        bad[start + 4..start + 8].copy_from_slice(&size.to_be_bytes());
        assert!(inspect(&bad).is_err());
    }
    let a = Appearance {
        entity: 1,
        source: 42,
        object: 3333,
        life: 5,
        runtime: 4_294_967_301,
        lifecycle: 4_294_967_302,
        graphics: 4_294_967_303,
    };
    let mut w = Writer::allocated(52).unwrap();
    w.write(&a).unwrap();
    let raw = w.bytes.unwrap();
    assert_eq!(raw.len(), 52);
    assert_eq!(read::<Appearance>(&raw).unwrap(), a);
    for count in [-1i32, 101, i32::MAX] {
        assert!(read::<List<Appearance, 100>>(&count.to_be_bytes()).is_err());
    }
    assert_eq!(MAX_APPEARANCE_BYTES, MAX_BYTES + 8 + 5204);
}
#[test]
fn framing_is_bounded_and_not_old_or_selected_admission() {
    let b = framing();
    assert_eq!(inspect(&b).unwrap().schema(), 10);
    assert!(decode(&b).is_err());
    assert!(super::super::inspect(&b).is_err());
    for v in 1..=crate::modern_lito::protocol::MAX_VERSION {
        assert!(super::super::decode_versioned(&b, v).is_err());
    }
    for end in 0..b.len() {
        assert!(inspect(&b[..end]).is_err());
    }
    for offset in [0, 4, 6, 8, 12, 14, 16, 18, 20] {
        let mut bad = b.clone();
        bad[offset] ^= 1;
        assert!(inspect(&bad).is_err());
    }
    let mut bad = b.clone();
    bad.push(0);
    let n = bad.len() as u32;
    bad[8..12].copy_from_slice(&n.to_be_bytes());
    assert!(inspect(&bad).is_err());
    for id in 1..=SECTION {
        let mut bad = b.clone();
        let offset = 16 + (id - 1) * 9;
        bad[offset + 4..offset + 8]
            .copy_from_slice(&((section_limit(id).unwrap() + 1) as u32).to_be_bytes());
        assert!(inspect(&bad).is_err());
    }
    assert_eq!(
        LIMIT,
        96 + 4 + 512 * ITEM_LIMIT + 248 + 4 + 512 * 8 + 1 + 8 + 3 + 248
    );
}
#[test]
fn every_signed_ability_slot_round_trips_without_native_narrowing() {
    assert_eq!(FIELD_NAMES.join(","),"STR,DEX,CON,INT,WIS,CHA,AC,MAX_HP,MAX_MP,HPR,MPR,MR,SP,MELEE_DAMAGE,MELEE_HIT,RANGED_DAMAGE,RANGED_HIT,DAMAGE_REDUCTION,MAGIC_REDUCTION,FIRE,WIND,EARTH,WATER,STUN,STONE,SLEEP,FREEZE,SUSTAIN,BLIND,EXP_RATE,ADENA_RATE");
    let a = Abilities(std::array::from_fn(|i| {
        if i % 2 == 0 {
            i64::MAX - i as i64
        } else {
            i64::MIN + i as i64
        }
    }));
    let mut w = Writer::allocated(248).unwrap();
    w.write(&a).unwrap();
    let b = w.bytes.unwrap();
    assert_eq!(read::<Abilities>(&b).unwrap(), a);
    for end in 0..b.len() {
        assert!(read::<Abilities>(&b[..end]).is_err());
    }
    assert_eq!(format!("{a:?}"), "VipAbilities[redacted]");
}
#[test]
fn counts_tags_and_absurd_allocations_are_rejected_before_owner_allocation() {
    for n in [-1i32, 513, i32::MAX] {
        assert!(read::<List<Item, 512>>(&n.to_be_bytes()).is_err());
    }
    assert!(read::<List<Item, 512>>(&512i32.to_be_bytes()).is_err());
    assert!(read::<Option<Shape>>(&[2]).is_err());
    assert!(read::<LifetimeState>(&[7]).is_err());
    assert!(inspect(&vec![0; MAX_BYTES + 1]).is_err());
}
#[test]
fn configured_shape_is_complete_metadata_not_a_resource_load_ack() {
    let shape = Shape {
        graphics: 99999,
        present: true,
        no_death_exp_loss: true,
        retains_on_cancellation: false,
        abilities: Abilities(std::array::from_fn(|i| i as i64 - 15)),
    };
    let mut w = Writer::allocated(Shape::LIMIT).unwrap();
    w.write(&shape).unwrap();
    assert_eq!(read::<Shape>(&w.bytes.unwrap()).unwrap(), shape);
    let mut w = Writer::allocated(1).unwrap();
    w.write(&Option::<Shape>::None).unwrap();
    assert_eq!(read::<Option<Shape>>(&w.bytes.unwrap()).unwrap(), None);
}

#[test]
fn dedicated_descriptor_never_expands_legacy_or_unrelated_modes() {
    use crate::modern_lito::world_protocol::{MatchDescriptor as D, MatchRuleset as R};
    let valid = D {
        ruleset: R::PvpSkirmish,
        features: D::FULL_VIP_CANDIDATE | D::COMBAT,
        preparation_timeout_millis: 2000,
        load_timeout_millis: 2000,
        countdown_nanos: 0,
        active_timeout_millis: 60_000,
    };
    valid.validate_vip_candidate(1).unwrap();
    assert!(valid.validate(1).is_err());
    assert!(valid.validate_canonical(1).is_err());
    assert!(valid.validate_armor_candidate(1).is_err());
    assert_eq!(D::KNOWN_FEATURES, 2097151);
    for mode in [0, 2, 3, 4, 5] {
        assert!(valid.validate_vip_candidate(mode).is_err());
    }
    for feature in [
        D::NPC,
        D::HONOR,
        D::SIEGE,
        D::PET_RACE,
        D::TIME_ATTACK,
        1 << 27,
    ] {
        let mut bad = valid;
        bad.features |= feature;
        assert!(bad.validate_vip_candidate(1).is_err());
    }
    for feature in [D::COMBAT, D::FULL_VIP_CANDIDATE] {
        let mut bad = valid;
        bad.features &= !feature;
        assert!(bad.validate_vip_candidate(1).is_err());
    }
    let mut elimination = valid;
    elimination.ruleset = R::DeathmatchElimination;
    elimination.validate_vip_candidate(1).unwrap();
}
