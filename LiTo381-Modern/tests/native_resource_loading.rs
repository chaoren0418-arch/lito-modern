use lito_modern::modern_lito::native_clan_markers::Bank;
use lito_modern::modern_lito::resources::{SharedArchive, SharedArchiveKind};
use sha2::{Digest, Sha256};
#[test]
fn ranged_capture_floor_waits_without_a_frame_rate_assumption() {
    use lito_modern::modern_lito::native_combat_probe::{capture_floor_ready, ranged_capture_ready};
    use std::time::Duration;
    assert!(!ranged_capture_ready(9, Duration::from_millis(2500)).unwrap());
    assert!(!ranged_capture_ready(9, Duration::from_millis(9999)).unwrap());
    assert!(ranged_capture_ready(10, Duration::from_millis(2700)).unwrap());
    let error = ranged_capture_ready(9, Duration::from_secs(10))
        .unwrap_err()
        .to_string();
    assert!(error.contains("9/10") && error.contains("10000ms"));
    assert!(!capture_floor_ready("ground", 7, Duration::from_millis(2500)).unwrap());
    assert!(capture_floor_ready("ground", 10, Duration::from_millis(2750)).unwrap());
    assert!(capture_floor_ready("ground", 7, Duration::from_secs(10))
        .err().unwrap().to_string().contains("ground capture timed out: 7/10"));
}
use std::{path::Path, sync::atomic::AtomicBool};

#[test]
fn observed_native_archives_keep_pak_handles_and_reject_drifted_plans() {
    use lito_modern::modern_lito::native_resources::{
        Limits, NativeAssetPlan, NativeObservedArchives, NativeResourceAdapter, NativeSources,
        ResourceNamespace,
    };
    let root = fixture_root("observed-native-archives");
    for prefix in ["LTtile", "LTsprite"] {
        std::fs::write(root.join(format!("{prefix}.idx")), index(&[("1.til", 0, 1)])).unwrap();
        std::fs::write(root.join(format!("{prefix}.pak")), [1]).unwrap();
    }
    let cancel = AtomicBool::new(false);
    let observed = NativeObservedArchives::open(&root, Limits::default(), &cancel).unwrap();
    let pins = observed.pins();
    assert_eq!(pins.map(|p| p.pin.bytes), [32, 1, 32, 1]);
    #[cfg(windows)]
    for name in ["LTtile.pak", "LTsprite.pak"] {
        assert!(std::fs::OpenOptions::new().write(true).open(root.join(name)).is_err());
    }
    let mut plan = NativeAssetPlan {
        namespace: ResourceNamespace::NativeLitoV1,
        map_id: 564,
        canonical_sha256: [1; 32],
        sectors: vec![],
        archives: pins.to_vec(),
    };
    plan.archives[0].pin.sha256[0] ^= 1;
    let sources = NativeSources { map_root: root.clone(), archive_root: root.clone() };
    assert!(NativeResourceAdapter::pinned_inspection()
        .load_static_observed(&sources, &plan, Limits::default(), observed, &cancel)
        .err().unwrap().to_string().contains("Observed Native archive pins differ"));
    std::fs::write(root.join("LTtile.pak"), [2]).unwrap();
    let next = NativeObservedArchives::open(&root, Limits::default(), &cancel).unwrap();
    assert_ne!(pins[1].pin.sha256, next.pins()[1].pin.sha256);
    drop(next);
    std::fs::write(root.join("LTsprite.idx"), index(&[("1.til", 1, 1)])).unwrap();
    assert!(NativeObservedArchives::open(&root, Limits::default(), &cancel).is_err());
    assert!(NativeObservedArchives::open(&root, Limits::default(), &AtomicBool::new(true)).is_err());
}

#[test]
fn observed_archive_lifetime_denies_writes_to_all_four_sources() {
    use lito_modern::modern_lito::native_resources::{Limits, NativeObservedArchives};
    let root=fixture_root("observed-native-four-handles-lifetime");
    for stem in ["LTtile","LTsprite"] {
        std::fs::write(root.join(format!("{stem}.idx")),index(&[("1.til",0,1)])).unwrap();
        std::fs::write(root.join(format!("{stem}.pak")),[1]).unwrap();
    }
    let observed=NativeObservedArchives::open(&root,Limits::default(),&AtomicBool::new(false)).unwrap();
    #[cfg(windows)]
    for name in ["LTtile.idx","LTtile.pak","LTsprite.idx","LTsprite.pak"] {
        assert!(std::fs::OpenOptions::new().write(true).open(root.join(name)).is_err(),
            "initially observed source must remain write-denied for its lifetime: {name}");
    }
    drop(observed);
    for name in ["LTtile.idx","LTtile.pak","LTsprite.idx","LTsprite.pak"] {
        assert!(std::fs::OpenOptions::new().write(true).open(root.join(name)).is_ok(),
            "source lease must close with its owner: {name}");
    }
}

#[test]
fn retained_archive_validation_and_failed_windows_keep_the_same_source_owner() {
    use lito_modern::modern_lito::native_resources::{Limits,NativeObservedArchives,
        NativeAssetPlan,NativeResourceAdapter,NativeSources,ResourceNamespace};
    let root=fixture_root("retained-native-source-owner");
    for stem in ["LTtile","LTsprite"] {
        std::fs::write(root.join(format!("{stem}.idx")),index(&[("1.til",0,1)])).unwrap();
        std::fs::write(root.join(format!("{stem}.pak")),[1]).unwrap();
    }
    let cancel=AtomicBool::new(false);
    let mut observed=NativeObservedArchives::open(&root,Limits::default(),&cancel).unwrap();
    let pins=observed.pins();
    for _ in 0..3 {observed.validate_sources(&root,&cancel).unwrap();assert_eq!(observed.pins(),pins);}
    let other=fixture_root("retained-native-different-source-owner");
    assert!(observed.validate_sources(&other,&cancel).err().unwrap().to_string().contains("root differs"));
    assert!(observed.validate_sources(&root,&AtomicBool::new(true)).is_err());
    let plan=NativeAssetPlan {namespace:ResourceNamespace::NativeLitoV1,map_id:564,
        canonical_sha256:[1;32],sectors:vec![],archives:pins.to_vec()};
    let sources=NativeSources {map_root:root.clone(),archive_root:root.clone()};
    let adapter=NativeResourceAdapter::pinned_world();
    assert!(adapter.load_static_retained_window(&sources,&plan,Limits::default(),&mut observed,
        [32768,32768],&cancel).err().unwrap().to_string().contains("sector count invalid"));
    assert!(adapter.load_static_retained_window(&sources,&plan,Limits::default(),&mut observed,
        [32768,32768],&AtomicBool::new(true)).is_err());
    observed.validate_sources(&root,&cancel).unwrap();assert_eq!(observed.pins(),pins);
    #[cfg(windows)]
    for name in ["LTtile.idx","LTtile.pak","LTsprite.idx","LTsprite.pak"] {
        assert!(std::fs::OpenOptions::new().write(true).open(root.join(name)).is_err());
    }
    drop(observed);
    for name in ["LTtile.idx","LTtile.pak","LTsprite.idx","LTsprite.pak"] {
        assert!(std::fs::OpenOptions::new().write(true).open(root.join(name)).is_ok());
    }
}

fn fixture_root(name: &str) -> std::path::PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/verification/resource-loading")
        .join(name);
    std::fs::create_dir_all(&p).unwrap();
    p
}
#[test]
fn shared_native_sources_release_only_after_the_last_owner_and_respect_limits() {
    use lito_modern::modern_lito::native_resources::{Limits,SharedNativeArchives};
    let root=fixture_root("shared-native-sources-lifetime");
    for stem in ["LTtile","LTsprite"] {
        std::fs::write(root.join(format!("{stem}.idx")),index(&[("1.til",0,2)])).unwrap();
        std::fs::write(root.join(format!("{stem}.pak")),[1,2]).unwrap();
    }
    let cancel=AtomicBool::new(false);
    let a=SharedNativeArchives::open(&root,Limits::default(),&cancel).unwrap();
    let pins=a.lease(&cancel).unwrap().pins();
    let b=SharedNativeArchives::open(&root,Limits::default(),&cancel).unwrap();
    assert!(b.reused_source_owner());assert_eq!(b.lease(&cancel).unwrap().pins(),pins);
    drop(a);
    #[cfg(windows)]
    for name in ["LTtile.idx","LTtile.pak","LTsprite.idx","LTsprite.pak"] {
        assert!(std::fs::OpenOptions::new().write(true).open(root.join(name)).is_err());
    }
    let mut small=Limits::default();small.pak_bytes=1;
    assert!(SharedNativeArchives::open(&root,small,&cancel).is_err(),"cache must not bypass tighter byte limits");
    assert!(SharedNativeArchives::open(&root,Limits::default(),&AtomicBool::new(true)).is_err());
    let other=fixture_root("shared-native-other-root");
    for stem in ["LTtile","LTsprite"] {
        std::fs::write(other.join(format!("{stem}.idx")),index(&[("1.til",0,2)])).unwrap();
        std::fs::write(other.join(format!("{stem}.pak")),[3,4]).unwrap();
    }
    let separate=SharedNativeArchives::open(&other,Limits::default(),&cancel).unwrap();
    assert_ne!(separate.lease(&cancel).unwrap().pins()[1].pin,pins[1].pin);
    assert!(b.lease(&cancel).unwrap().validate_sources(&other,&cancel).is_err());
    drop(separate);drop(b);
    for name in ["LTtile.idx","LTtile.pak","LTsprite.idx","LTsprite.pak"] {
        assert!(std::fs::OpenOptions::new().write(true).open(root.join(name)).is_ok());
    }
    std::fs::write(root.join("LTtile.pak"),[9,8]).unwrap();
    let next=SharedNativeArchives::open(&root,Limits::default(),&cancel).unwrap();
    assert!(!next.reused_source_owner());assert_ne!(next.lease(&cancel).unwrap().pins()[1].pin,pins[1].pin);
}

#[test]
fn cancelled_shared_source_wait_does_not_retire_the_live_owner() {
    use lito_modern::modern_lito::native_resources::{Limits,SharedNativeArchives};
    use std::{sync::{Arc,Barrier,mpsc},time::Duration};
    let root=fixture_root("shared-native-source-cancelled-wait");
    for stem in ["LTtile","LTsprite"] {
        std::fs::write(root.join(format!("{stem}.idx")),index(&[("1.til",0,1)])).unwrap();
        std::fs::write(root.join(format!("{stem}.pak")),[1]).unwrap();
    }
    let source=Arc::new(SharedNativeArchives::open(&root,Limits::default(),&AtomicBool::new(false)).unwrap());
    let stable=AtomicBool::new(false);let lease=source.lease(&stable).unwrap();let pins=lease.pins();
    let cancel=Arc::new(AtomicBool::new(false));let barrier=Arc::new(Barrier::new(2));let(tx,rx)=mpsc::channel();
    let worker={let source=Arc::clone(&source);let cancel=Arc::clone(&cancel);let barrier=Arc::clone(&barrier);
        std::thread::spawn(move||{barrier.wait();let error=source.lease(&cancel).err().unwrap().to_string();tx.send(error).unwrap();})};
    barrier.wait();std::thread::sleep(Duration::from_millis(20));cancel.store(true,std::sync::atomic::Ordering::Release);
    assert!(rx.recv_timeout(Duration::from_secs(3)).unwrap().contains("cancelled"));worker.join().unwrap();
    assert_eq!(lease.pins(),pins);drop(lease);
    assert_eq!(source.lease(&stable).unwrap().pins(),pins);
}

#[test]
fn concurrent_shared_native_archive_windows_match_independent_original_pixels() {
    use lito_modern::modern_lito::{native_resources as nr,scene::CameraRect};
    let root=Path::new(r"D:\lineage開發\0-客戶端").join("LiTo");let cancel=AtomicBool::new(false);
    let sources=nr::NativeSources{map_root:root.join("LTmap"),archive_root:root.clone()};
    let geometry=nr::NativeRetainedGeometry::open(&sources.map_root,500,nr::Limits::default(),&cancel).unwrap();
    let mut independent=nr::NativeObservedArchives::open(&root,nr::Limits::default(),&cancel).unwrap();
    let plan=nr::NativeAssetPlan{namespace:nr::ResourceNamespace::NativeLitoV1,map_id:500,
        canonical_sha256:geometry.geometry().canonical_digest(),sectors:geometry.sector_pins().to_vec(),archives:independent.pins().to_vec()};
    let centers=[[32770,32818],[32777,32819]];
    let render=|assets:&nr::NativeStaticAssets,center:[u16;2]| {
        let key=plan.sectors[0].key;let ox=32704+(i32::from(key.x)-0x7fff)*64;let oy=32704+(i32::from(key.y)-0x7fff)*64;
        let(dx,dy)=(i32::from(center[0])-ox,i32::from(center[1])-oy);
        let camera=CameraRect{x:24+24*(dx+dy)-400,y:768+12*(-dx+dy)-300,width:800,height:600};
        let canvas=assets.render_static(camera,&cancel).unwrap();
        assert!(canvas.rgba().iter().filter(|v|**v!=0).count()>100);
        Sha256::digest(canvas.rgba()).to_vec()
    };
    let expected:Vec<_>=centers.iter().map(|center|{
        let assets=nr::NativeResourceAdapter::pinned_world().load_static_retained_geometry_window(
            &sources,&plan,nr::Limits::default(),&mut independent,&geometry,*center,&cancel).unwrap();render(&assets,*center)
    }).collect();
    let a=nr::SharedNativeArchives::open(&root,nr::Limits::default(),&cancel).unwrap();
    let b=nr::SharedNativeArchives::open(&root,nr::Limits::default(),&cancel).unwrap();assert!(b.reused_source_owner());
    let barrier=std::sync::Barrier::new(2);
    std::thread::scope(|scope|{
        let handles:Vec<_>=[a,b].into_iter().enumerate().map(|(thread,owner)|{
            let expected=&expected;let render=&render;let sources=&sources;let plan=&plan;let geometry=&geometry;let cancel=&cancel;let barrier=&barrier;
            scope.spawn(move||{barrier.wait();for i in 0..4 {
                let slot=(i+thread)%2;let mut lease=owner.lease(cancel).unwrap();
                let assets=nr::NativeResourceAdapter::pinned_world().load_static_retained_geometry_window(
                    sources,plan,nr::Limits::default(),&mut lease,geometry,centers[slot],cancel).unwrap();
                drop(lease);assert_eq!(render(&assets,centers[slot]),expected[slot]);
            }})
        }).collect();
        for handle in handles{handle.join().unwrap();}
    });
}

#[test]
fn overlapping_world_models_reuse_source_leases_without_changing_pixels() {
    use lito_modern::modern_lito::{native_world::World,native_world_scene::Model};
    let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut world=World::default();
    for name in ["world","start","status","actor","hp","mp"] {
        let s=fixture["packets"][name].as_str().unwrap();let bytes:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();world.accept(&bytes).unwrap();
    }
    world.map_id=Some(500);let id=world.own_actor().unwrap().id;
    {let own=world.actors.get_mut(&id).unwrap();own.x=32770;own.y=32818;}
    let cancel=AtomicBool::new(false);let root=Path::new(r"D:\lineage開發\0-客戶端");
    let mut a=Model::load(root,&world,&cancel).unwrap();let mut b=Model::load(root,&world,&cancel).unwrap();
    let before=serde_json::to_value(&world).unwrap();let left=a.draw_at(&world,&cancel,0).unwrap();let right=b.draw_at(&world,&cancel,0).unwrap();
    assert_eq!(right.receipt["load_profile_us"]["shared_archive_sources_reused"],true);
    assert_eq!(left.canvas.rgba(),right.canvas.rgba());assert_eq!(serde_json::to_value(&world).unwrap(),before);
    drop(a);assert!(b.draw_at(&world,&cancel,1_000_000_000).unwrap().movement_ready(&world));
}
#[test]
fn retained_native_geometry_keeps_sector_identity_locks_and_rejects_new_sources() {
    use lito_modern::modern_lito::{native_resources::{Limits,NativeRetainedGeometry},
        world_map::{WorldMap,SectorInput},resources::SharedMapSectorKey};
    let root=fixture_root("retained-native-geometry-owner");let dir=root.join("17");std::fs::create_dir_all(&dir).unwrap();
    let mut bytes=vec![0;32768];bytes.extend(0u16.to_le_bytes());bytes.extend(vec![0;16384]);bytes.extend(0i32.to_le_bytes());
    let file=dir.join("7fff7fff.s32");std::fs::write(&file,&bytes).unwrap();let cancel=AtomicBool::new(false);
    let retained=NativeRetainedGeometry::open(&root,17,Limits::default(),&cancel).unwrap();
    let original=WorldMap::from_native_sector_bytes(vec![SectorInput{key:SharedMapSectorKey{map_id:17,sector_x:0x7fff,sector_y:0x7fff},bytes:bytes.clone()}],&cancel).unwrap();
    assert_eq!(retained.geometry().canonical_digest(),original.canonical_digest());
    assert_eq!(retained.sector_pins().len(),1);assert_eq!(retained.sector_pins()[0].pin.bytes,bytes.len() as u64);
    let pin=retained.sector_pins()[0];
    for _ in 0..3 {retained.validate_sources(&root,17,&cancel).unwrap();assert_eq!(retained.sector_pins(),[pin]);}
    assert!(retained.validate_sources(&root,18,&cancel).is_err());
    assert!(retained.validate_sources(&fixture_root("retained-native-geometry-other-root"),17,&cancel).is_err());
    assert!(retained.validate_sources(&root,17,&AtomicBool::new(true)).is_err());
    #[cfg(windows)] {
        assert!(std::fs::OpenOptions::new().write(true).open(&file).is_err());
        assert!(std::fs::rename(&file,dir.join("moved.s32")).is_err());
    }
    let extra=dir.join("80007fff.s32");std::fs::write(&extra,&bytes).unwrap();
    assert!(retained.validate_sources(&root,17,&cancel).err().unwrap().to_string().contains("sector set changed"));
    assert_eq!(retained.geometry().canonical_digest(),original.canonical_digest());
    std::fs::remove_file(&extra).unwrap();retained.validate_sources(&root,17,&cancel).unwrap();
    drop(retained);assert!(std::fs::OpenOptions::new().write(true).open(&file).is_ok());
    let mut changed=bytes;changed[0]=1;std::fs::write(&file,&changed).unwrap();
    let next=NativeRetainedGeometry::open(&root,17,Limits::default(),&cancel).unwrap();assert_ne!(next.sector_pins()[0].pin.sha256,pin.pin.sha256);
}
fn index(rows: &[(&str, u32, u32)]) -> Vec<u8> {
    let mut b = (rows.len() as u32).to_le_bytes().to_vec();
    for (name, offset, size) in rows {
        b.extend_from_slice(&offset.to_le_bytes());
        let mut field = [0; 20];
        field[..name.len()].copy_from_slice(name.as_bytes());
        b.extend_from_slice(&field);
        b.extend_from_slice(&size.to_le_bytes());
    }
    b
}

#[test]
fn selected_index_preserves_reads_and_checks_every_unselected_record() {
    let root = fixture_root("selected-index");
    let cancel = AtomicBool::new(false);
    let valid = index(&[("WANTED.IMG", 0, 2), ("other.img", 2, 2)]);
    std::fs::write(root.join("Sprite.pak"), [10, 20, 30, 40]).unwrap();
    std::fs::write(root.join("Sprite.idx"), &valid).unwrap();
    {
        let mut full = SharedArchive::open(&root, SharedArchiveKind::Sprite).unwrap();
        let mut small = SharedArchive::open_for_names(
            &root,
            SharedArchiveKind::Sprite,
            &["wanted.img", "absent.img"],
            &cancel,
        )
        .unwrap();
        assert_eq!(small.len(), 1);
        assert_eq!(
            small.read_entry("Wanted.Img").unwrap(),
            full.read_entry("wanted.img").unwrap()
        );
        assert!(matches!(
            small.entry("absent.img"),
            Err(lito_modern::modern_lito::resources::ResourceError::MissingEntry(_))
        ));
        assert!(matches!(
            small.entry("other.img"),
            Err(lito_modern::modern_lito::resources::ResourceError::InvalidLimits)
        ));
        #[cfg(windows)]
        for file in ["Sprite.idx", "Sprite.pak"] {
            assert!(std::fs::OpenOptions::new()
                .write(true)
                .open(root.join(file))
                .is_err());
        }
    }
    let mut padding = valid.clone();
    padding[4 + 28 + 4 + 15] = 1;
    let mut count = valid.clone();
    count[..4].copy_from_slice(&3u32.to_le_bytes());
    let mut trailing = valid.clone();
    trailing.push(0);
    let bad = vec![
        padding,
        count,
        trailing,
        b"_IDX".to_vec(),
        index(&[
            ("wanted.img", 0, 2),
            ("other.img", 2, 1),
            ("OTHER.IMG", 3, 1),
        ]),
        index(&[("wanted.img", 0, 2), ("../bad", 2, 1)]),
        index(&[("wanted.img", 0, 2), ("bad.img", 4, 1)]),
        index(&[("wanted.img", 0, 2), ("large.img", 0, 64 * 1024 * 1024 + 1)]),
    ];
    for bytes in bad {
        std::fs::write(root.join("Sprite.idx"), bytes).unwrap();
        let full = SharedArchive::open(&root, SharedArchiveKind::Sprite)
            .err()
            .unwrap()
            .to_string();
        let selected = SharedArchive::open_for_names(
            &root,
            SharedArchiveKind::Sprite,
            &["wanted.img"],
            &cancel,
        )
        .err()
        .unwrap()
        .to_string();
        assert_eq!(selected, full);
    }
    std::fs::write(root.join("Sprite.idx"), valid).unwrap();
    assert!(SharedArchive::open_for_names(&root, SharedArchiveKind::Sprite, &[], &cancel).is_err());
    assert!(SharedArchive::open_for_names(
        &root,
        SharedArchiveKind::Text,
        &["wanted.img"],
        &cancel
    )
    .is_err());
    assert!(SharedArchive::open_for_names(
        &root,
        SharedArchiveKind::Sprite,
        &["wanted.img"; 65],
        &cancel
    )
    .is_err());
    let cancelled = SharedArchive::open_for_names(
        &root,
        SharedArchiveKind::Sprite,
        &["wanted.img"],
        &AtomicBool::new(true),
    )
    .err()
    .unwrap()
    .to_string();
    assert!(cancelled.contains("cancelled"));
}

#[test]
fn observed_surface_pins_and_override_rejections_survive_sparse_lookup() {
    let root = fixture_root("surface");
    let lt = root.join("LiTo");
    std::fs::create_dir_all(&lt).unwrap();
    for stem in std::iter::once("Sprite".to_owned()).chain((0..16).map(|i| format!("Sprite{i:02}")))
    {
        std::fs::write(root.join(format!("{stem}.idx")), index(&[])).unwrap();
        std::fs::write(root.join(format!("{stem}.pak")), [0]).unwrap();
    }
    let names: Vec<_> = (7550..=7556).map(|id| format!("{id}.img")).collect();
    let mut img = vec![0; 872];
    img[..8].copy_from_slice(&[24, 0, 18, 0, 1, 0, 0, 0]);
    img[8..10].copy_from_slice(&0x7fffu16.to_le_bytes());
    let mut pak = img.repeat(7);
    let rows: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i as u32 * 872, 872))
        .collect();
    let idx = index(&rows);
    std::fs::write(lt.join("LTsurf.idx"), &idx).unwrap();
    std::fs::write(lt.join("LTsurf.pak"), &pak).unwrap();
    let cancel = AtomicBool::new(false);
    let first = Bank::load(&root, &cancel).unwrap();
    assert_eq!(
        first.sources["index"]["sha256"],
        format!("{:X}", Sha256::digest(&idx))
    );
    assert_eq!(
        first.sources["pak"]["sha256"],
        format!("{:X}", Sha256::digest(&pak))
    );
    pak[8] = 0x1f;
    pak[9] = 0;
    std::fs::write(lt.join("LTsurf.pak"), &pak).unwrap();
    let second = Bank::load(&root, &cancel).unwrap();
    assert_ne!(first.sources["pak"], second.sources["pak"]);
    assert_ne!(first.images[0].rgba, second.images[0].rgba);
    let candidate = "7550c.img";
    let shard = candidate.bytes().map(u32::from).sum::<u32>() % 16;
    let shard_idx = root.join(format!("Sprite{shard:02}.idx"));
    std::fs::write(&shard_idx, index(&[(candidate, 0, 1)])).unwrap();
    assert!(Bank::load(&root, &cancel)
        .err()
        .unwrap()
        .to_string()
        .contains("shard override"));
    std::fs::write(&shard_idx, index(&[])).unwrap();
    std::fs::write(root.join("Sprite.idx"), index(&[(candidate, 0, 1)])).unwrap();
    assert!(Bank::load(&root, &cancel)
        .err()
        .unwrap()
        .to_string()
        .contains("localized clan marker"));
    std::fs::write(root.join("Sprite.idx"), index(&[])).unwrap();
    let mut localized = rows.clone();
    localized.push((candidate, 0, 872));
    std::fs::write(lt.join("LTsurf.idx"), index(&localized)).unwrap();
    assert!(Bank::load(&root, &cancel)
        .err()
        .unwrap()
        .to_string()
        .contains("localized clan marker"));
    let mut bad = rows.clone();
    bad[6].1 = pak.len() as u32;
    std::fs::write(lt.join("LTsurf.idx"), index(&bad)).unwrap();
    assert!(Bank::load(&root, &cancel).is_err());
    std::fs::write(lt.join("LTsurf.idx"), &idx).unwrap();
    assert!(Bank::load(&root, &AtomicBool::new(true)).is_err());
}

#[test]
fn marker_load_profile_keeps_identical_sources_across_fresh_banks() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let mut profiles = Vec::new();
    let mut sources = None;
    for _ in 0..3 {
        let bank = Bank::load(root, &AtomicBool::new(false)).unwrap();
        assert_eq!(bank.images.len(), 7);
        if let Some(previous) = &sources {
            assert_eq!(&bank.sources, previous);
        } else {
            sources = Some(bank.sources.clone());
        }
        profiles.push(bank.load_profile_us);
    }
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(
        output.join("native-marker-load-profile.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"profiles_us":profiles,"sources":sources}))
            .unwrap(),
    )
    .unwrap();
}
