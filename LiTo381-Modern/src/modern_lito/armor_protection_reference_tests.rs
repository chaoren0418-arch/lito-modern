//! Shared-only inspection for the armor-proc protection presentation.
//! Does not grant a skill, open a game, or read a LiTo-specific archive.
use super::{
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::SharedActionCatalog,
    sprite::{decode_world_spr_with_limits, SpriteLimits},
    sprite_actions::ActionKey,
};
use std::{path::PathBuf, sync::atomic::AtomicBool};

#[test]
#[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; read-only shared action inspection"]
fn actual_shared_armor_protection_action_definitions() {
    let root =
        PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"));
    assert!(root.is_absolute());
    let catalog = SharedActionCatalog::load(&root, &AtomicBool::new(false)).unwrap();
    assert_eq!(catalog.byte_length(), 4_067_417);
    assert_eq!(
        catalog
            .sha256()
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<String>(),
        "523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127"
    );
    println!(
        "ARMOR_ACTION_CATALOG bytes={} sha={:02X?}",
        catalog.byte_length(),
        catalog.sha256()
    );
    for id in [228, 11101] {
        let sprite = catalog
            .catalog()
            .sprites
            .get(&id)
            .expect("required protection sprite");
        assert!(sprite.actions.len() <= 8 && sprite.attributes.len() <= 8);
        if id == 11101 {
            assert_eq!(sprite.graphics_id, id);
            assert_eq!(sprite.image_count, Some(1));
            assert!(
                sprite.actions.is_empty(),
                "11101 is an empty placeholder, not a qualified effect"
            );
            assert_eq!(sprite.attributes.len(), 1);
            assert_eq!(sprite.attributes[0].key, ActionKey::new(102));
            assert_eq!(sprite.attributes[0].body_bytes, b"0");
        }
        println!("ARMOR_SPRITE id={id} definition={sprite:?}");
        match catalog.catalog().resolve_action(id, ActionKey::new(0)) {
            Ok(resolved) => {
                assert_eq!(id, 228, "unreviewed placeholder acquired an action");
                assert!(resolved.animation.frames.len() <= 128);
                println!(
                    "ARMOR_ACTION id={id} resolved={} animation={:?}",
                    resolved.definition_sprite_id, resolved.animation
                );
            }
            Err(error) => {
                assert_eq!(id, 11101, "expected actual228 action");
                println!("ARMOR_ACTION_UNRESOLVED id={id} error={error:?}");
            }
        }
        let name = format!("{id}-0.spr");
        let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
        let mut archive =
            SharedArchive::open(&root, SharedArchiveKind::SpriteShard(shard)).unwrap();
        if id == 11101 {
            assert!(matches!(
                archive.entry(&name),
                Err(ResourceError::MissingEntry(_))
            ));
        }
        if let Err(ResourceError::MissingEntry(_)) = archive.entry(&name) {
            archive = SharedArchive::open(&root, SharedArchiveKind::Sprite).unwrap();
        }
        match archive.entry(&name) {
            Ok(entry) => {
                assert_eq!(id, 228, "unreviewed resource replaced placeholder");
                assert!(entry.size <= 8 * 1024 * 1024);
                let bytes = archive.read_entry(&name).unwrap();
                let decoded =
                    decode_world_spr_with_limits(&bytes, SpriteLimits::default()).unwrap();
                println!(
                    "ARMOR_SPR name={name} bytes={} frames={}",
                    bytes.len(),
                    decoded.frames.len()
                );
            }
            Err(ResourceError::MissingEntry(_)) => {
                assert_eq!(id, 11101, "expected actual228 SPR");
                println!("ARMOR_SPR_MISSING name={name} checked=shared-shard-and-base")
            }
            Err(error) => panic!("resource is not an ordinary miss: {error:?}"),
        }
    }
}
