use super::{
    codec::{Reader, Wire, Writer},
    data::*,
    *,
};

fn primitive<T: Wire>(value: &T) -> Vec<u8> {
    let mut n = Writer::count();
    n.write(value).unwrap();
    let mut w = Writer::allocated(n.pos).unwrap();
    w.write(value).unwrap();
    w.bytes.unwrap()
}
pub(crate) fn fixture() -> UnvalidatedData {
    let member = |source, team| Member {
        identity: Identity {
            kind: MemberKind::Human,
            source,
            id: 3333,
        },
        name: Text("角色".into()),
        source_name: Text("太陽神".into()),
        level: 99,
        class: 1,
        sex: 0,
        team,
        ready: true,
    };
    let actor = |entity, x| Actor {
        entity,
        position: Cell { x, y: 32768 },
        heading: 6,
        loaded: false,
        last_move_sequence: 0,
        step_interval_nanos: 1_000_000,
        vital: None,
        resources: None,
        spells: Spells {
            sequence: 0,
            request_revision: 0,
            initial_lawful: 0,
            lawful: 0,
            histories: List(vec![]),
            structure: None,
        },
        cast_interval_nanos: 0,
        buffs: None,
        regeneration: None,
    };
    UnvalidatedData {
        vip: None,
        armor: None,
        healing: None,
        haste: None,
        speed: None,
        npc_motion: None,
        world: World {
            identity: WorldId {
                room: 101,
                match_id: 102,
                generation: 103,
            },
            revision: 1,
            phase: Phase::Loading,
            abort: Abort::None,
            lane: -1,
            descriptor: Descriptor {
                ruleset: Ruleset::SceneOnly,
                features: 0,
                preparation_timeout_millis: 100,
                load_timeout_millis: 200,
                countdown_nanos: 300,
                active_timeout_millis: 400,
            },
            loading_elapsed_nanos: 0,
            active_elapsed_nanos: -1,
        },
        room: Room {
            id: 101,
            revision: 103,
            name: Text("測試房間".into()),
            capacity: 2,
            mode: 1,
            map: 504,
            attributes: 0,
            users: 2,
            ready: 2,
            satisfied: true,
            owner_source: 7,
            owner_object: 3333,
            members: List(vec![member(7, 0), member(8, 1)]),
        },
        manifest: Manifest {
            map: 504,
            digest: [1; 32],
            sectors: List(vec![Sector {
                x: 0x8000,
                y: 0x8000,
                byte_length: 49158,
                sha256: [2; 32],
            }]),
        },
        actors: List(vec![actor(1, 32768), actor(2, 32770)]),
        honor: Honor {
            objectives: List(vec![]),
            lives: List(vec![]),
        },
        quest: None,
        time_attack: None,
        siege: None,
        race: None,
        npcs: None,
        personal: Personal {
            player: Player {
                source: 7,
                object: 3333,
                epoch: i64::MIN,
                channel: 0,
            },
            entity: 1,
            equipment: None,
            skills: List(vec![]),
            history: None,
            spell_birth_revision: 0,
            supplies: None,
            weight: None,
            ammo: None,
            regen_clock: None,
            materials: None,
        },
        result: None,
        presentation: None,
    }
}
pub(super) fn replacement(bytes: &[u8], id: usize, payload: &[u8]) -> Vec<u8> {
    let mut result = bytes[..HEADER_BYTES].to_vec();
    let mut at = HEADER_BYTES;
    for i in 1..=SECTION_COUNT {
        let n = u32::from_be_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
        let value = if i == id {
            payload
        } else {
            &bytes[at + 8..at + 8 + n]
        };
        result.extend((i as u16).to_be_bytes());
        result.extend([0; 2]);
        result.extend((value.len() as u32).to_be_bytes());
        result.extend(value);
        at += 8 + n;
    }
    let n = result.len() as u32;
    result[8..12].copy_from_slice(&n.to_be_bytes());
    result
}
#[test]
fn complete_typed_scene_round_trip_preserves_opaque_epoch_and_explicit_phase() {
    let d = fixture();
    d.check_context(&d.personal.player, &d.world.identity)
        .unwrap();
    let b = encode(&d).unwrap();
    let parsed = decode(&b).unwrap();
    assert_eq!(parsed, d);
    assert_eq!(encode(&parsed).unwrap(), b);
    assert_eq!(inspect(&b).unwrap().section(Section::World)[32], 0);
    assert_eq!(parsed.world.phase, Phase::Loading);
    assert_eq!(parsed.personal.player.epoch, i64::MIN);
    assert_eq!(format!("{parsed:?}"), "UnvalidatedBattleData[redacted]");
}
#[test]
fn context_rejects_other_selection_world_channel_identity_and_narrowing() {
    let d = fixture();
    let p = &d.personal.player;
    let w = &d.world.identity;
    for change in 0..5 {
        let mut other = p.clone();
        match change {
            0 => other.source += 1,
            1 => other.object += 1,
            2 => other.epoch += 1,
            3 => other.channel = 1,
            _ => other.epoch = 0,
        };
        assert!(d.check_context(&other, w).is_err());
    }
    for change in 0..3 {
        let mut other = w.clone();
        match change {
            0 => other.room += 1,
            1 => other.match_id += 1,
            _ => other.generation += 1,
        };
        assert!(d.check_context(p, &other).is_err());
    }
    for change in 0..11 {
        let mut bad = d.clone();
        match change {
            0 => bad.personal.entity = 2,
            1 => bad.room.members.0[0].identity.kind = MemberKind::ServerBot,
            2 => bad.room.members.0[0].class = 65536,
            3 => bad.manifest.sectors.0[0].x = 65536,
            4 => bad.actors.0[0].entity = i64::MIN,
            5 => bad.world.lane = 0,
            6 => bad.world.revision = 0,
            7 => bad.world.descriptor.features = -1,
            8 => bad.room.members.0.swap(0, 1),
            9 => bad.world.loading_elapsed_nanos = -1,
            _ => bad.world.phase = Phase::Aborted,
        };
        assert!(bad.check_context(p, w).is_err(), "change={change}");
    }
}
#[test]
fn each_inner_section_must_be_complete_even_when_outer_lengths_are_repaired() {
    let b = encode(&fixture()).unwrap();
    let p = inspect(&b).unwrap();
    for (index, id) in [
        Section::World,
        Section::Room,
        Section::Manifest,
        Section::Actors,
        Section::Honor,
        Section::Quest,
        Section::TimeAttack,
        Section::Siege,
        Section::Race,
        Section::Npcs,
        Section::Personal,
        Section::Result,
    ]
    .iter()
    .enumerate()
    {
        let s = p.section(*id);
        let empty = replacement(&b, index + 1, &[]);
        assert!(inspect(&empty).is_ok());
        assert!(decode(&empty).is_err());
        let short = replacement(&b, index + 1, &s[..s.len() - 1]);
        assert!(inspect(&short).is_ok());
        assert!(decode(&short).is_err(), "section {} truncation", index + 1);
        let mut extra = s.to_vec();
        extra.push(0);
        let extra = replacement(&b, index + 1, &extra);
        assert!(inspect(&extra).is_ok());
        assert!(decode(&extra).is_err(), "section {} trailing", index + 1);
    }
    for n in 0..b.len() {
        assert!(decode(&b[..n]).is_err());
    }
}
#[test]
fn all_tags_are_explicit_and_unknown_bool_presence_utf8_counts_are_rejected() {
    macro_rules! enumeration {
        ($ty:ty,$n:expr) => {{
            for tag in 0u8..$n {
                let b = [tag];
                let value = Reader::new(&b).read::<$ty>().unwrap();
                assert_eq!(primitive(&value), b);
            }
            for tag in [$n, 255u8] {
                assert!(Reader::new(&[tag]).read::<$ty>().is_err());
            }
        }};
    }
    enumeration!(Phase, 5);
    enumeration!(Abort, 7);
    enumeration!(Ruleset, 8);
    enumeration!(MemberKind, 2);
    enumeration!(Lane, 8);
    enumeration!(EffectKind, 2);
    enumeration!(QuestKind, 3);
    enumeration!(QuestFlow, 2);
    enumeration!(QuestStatus, 3);
    enumeration!(QuestPhase, 3);
    enumeration!(TimePhase, 5);
    enumeration!(TimeEnd, 5);
    enumeration!(TimeKind, 3);
    enumeration!(SiegeSlot, 3);
    enumeration!(SiegePhase, 5);
    enumeration!(SiegeEnd, 2);
    enumeration!(RacePhase, 5);
    enumeration!(Dnf, 4);
    enumeration!(NpcAction, 5);
    enumeration!(Direction, 2);
    enumeration!(Verdict, 2);
    enumeration!(ItemKind, 3);
    enumeration!(Support, 7);
    enumeration!(LifetimeState, 7);
    enumeration!(Activity, 3);
    assert!(Reader::new(&[5]).read::<Target>().is_err());
    assert!(Reader::new(&[6]).read::<Detail>().is_err());
    for bad in [2, 255] {
        assert!(Reader::new(&[bad]).read::<bool>().is_err());
        assert!(Reader::new(&[bad]).read::<Option<i64>>().is_err());
    }
    for count in [-1, i32::MAX, 513] {
        assert!(Reader::new(&count.to_be_bytes())
            .read::<List<i64, 512>>()
            .is_err());
    }
    assert!(Reader::new(&[0, 0, 0, 1]).read::<List<i64, 512>>().is_err());
    for b in [
        vec![0, 1, 0],
        vec![0, 1, 255],
        vec![0, 3, 0xed, 0xa0, 0x80],
        vec![1, 1],
    ] {
        assert!(Reader::new(&b).read::<Text<256>>().is_err());
    }
    assert!(Writer::count()
        .write(&Text::<256>("界".repeat(86)))
        .is_err());
    let b = primitive(&Text::<256>("界".repeat(85)));
    assert_eq!(b.len(), 257);
    assert!(Reader::new(&b).read::<Text<256>>().is_ok());
}
#[test]
fn signed_extremes_and_utf8_are_lossless_and_not_silently_clipped() {
    for n in [i64::MIN, -1, 0, 1, i64::MAX] {
        assert_eq!(Reader::new(&primitive(&n)).read::<i64>().unwrap(), n);
    }
    for n in [i32::MIN, -1, 0, 1, i32::MAX] {
        assert_eq!(Reader::new(&primitive(&n)).read::<i32>().unwrap(), n);
    }
    let value = Text::<256>("3333(太陽神)".into());
    assert_eq!(
        Reader::new(&primitive(&value)).read::<Text<256>>().unwrap(),
        value
    );
    let v = Vitals {
        hp: i64::MAX,
        maximum_hp: i64::MAX,
        mp: i64::MAX - 1,
        maximum_mp: i64::MAX,
    };
    assert_eq!(Reader::new(&primitive(&v)).read::<Vitals>().unwrap(), v);
}
#[test]
fn count_pass_and_per_actor_limit_reject_oversized_data_before_output() {
    let mut d = fixture();
    d.actors.0.resize(101, d.actors.0[0].clone());
    assert!(encode(&d).is_err());
    let mut d = fixture();
    let anchor = Anchor {
        equipment_revision: 0,
        life: 1,
        vitals: Vitals {
            hp: 100,
            maximum_hp: 100,
            mp: 100,
            maximum_mp: 100,
        },
        cell: Cell { x: 32768, y: 32768 },
    };
    let cast = Cast {
        revision: 2,
        sequence: 1,
        before: anchor,
        cost: 48,
        mp_after: 52,
    };
    let effect = Effect {
        target: Target::Participant {
            entity: 2,
            identity: d.room.members.0[1].identity.clone(),
        },
        kind: EffectKind::Damage,
        life: 1,
        equipment_revision: 0,
        cell: Cell { x: 32770, y: 32768 },
        hp: 100,
        maximum_hp: 100,
        raw: 1,
        applied: 1,
        hp_after: 99,
    };
    let impact = Impact {
        lane: Lane::FireStorm,
        usage: cast.clone(),
        targets: List(vec![effect; 101]),
    };
    let spell = Spell {
        lane: Lane::FireStorm,
        ledger: Ledger {
            equipment_revision: 0,
            profile_revision: 1,
            cost: 48,
            prior_casts: 0,
            prior_spent: 0,
            casts: 1,
            spent: 48,
            last_cast_revision: 2,
            last_cast_sequence: 1,
        },
        last: Some(cast),
        receipt: Some(SpellReceipt {
            impact: impact.clone(),
            casts: 1,
            spent: 48,
            loading_elapsed_nanos: 0,
            detail: Detail::FireStorm {
                damage_loss_after: List(vec![1; 101]),
            },
        }),
    };
    d.actors.0[0].spells.histories = List(vec![spell; 8]);
    assert!(encode(&d).is_err());
    let mut invalid_impact = impact;
    invalid_impact.lane = Lane::Teleport;
    assert!(Writer::count().write(&invalid_impact).is_err());
    invalid_impact.lane = Lane::NpcFireStorm;
    assert!(Writer::count().write(&invalid_impact).is_err());
    assert!(Reader::new(&[0, 0, 0, 1, 0])
        .read::<EquipmentHistory>()
        .is_err());
    let mut w = Writer::allocated(4).unwrap();
    assert!(w.write(&1i64).is_err());
    assert_eq!(w.pos, 0);
    assert!(w.bytes.unwrap().is_empty());
}
