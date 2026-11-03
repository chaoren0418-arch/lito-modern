//! Independent PakViewer .NET10 memory-only references, 2026-09-15.
//! No native executable, LT pack, image file, GUI or audio device is needed.
use super::{
    combat_animation::{classify_native_cues, CombatAction, QualifiedCombatAnimation},
    combat_audio::{BankLimits, SoundBank},
    inventory_icons::SharedInventoryIcons,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    sprite::{decode_classic_spr_with_limits, DecodedSprite, SpriteLimits},
    sprite_actions::{ActionLimits, FrameEvent, SpriteActionCatalog},
    world_scene::BaseAppearance,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::Duration,
};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}
struct Reader {
    root: PathBuf,
    base: SharedArchive,
    shards: BTreeMap<u8, SharedArchive>,
}
impl Reader {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
            base: SharedArchive::open(root, SharedArchiveKind::Sprite).unwrap(),
            shards: BTreeMap::new(),
        }
    }
    fn read(&mut self, name: &str) -> (SharedArchiveKind, Vec<u8>) {
        let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
        let archive = self.shards.entry(shard).or_insert_with(|| {
            SharedArchive::open(&self.root, SharedArchiveKind::SpriteShard(shard)).unwrap()
        });
        match archive.entry(name) {
            Ok(_) => (
                SharedArchiveKind::SpriteShard(shard),
                archive.read_entry(name).unwrap(),
            ),
            Err(ResourceError::MissingEntry(_)) => (
                SharedArchiveKind::Sprite,
                self.base.read_entry(name).unwrap(),
            ),
            Err(error) => panic!("corrupt shared shard is not a member miss: {error}"),
        }
    }
}
#[test]
#[ignore = "LITO_SHARED_RESOURCE_ROOT; 12 bow bodies, 192 SPRs and 920 frame samples; no GUI"]
fn actual_shared_bow_bodies_match_independent_pixels_and_launch_cues() {
    let root = root();
    let cancel = AtomicBool::new(false);
    let mut text = SharedArchive::open(&root, SharedArchiveKind::Text).unwrap();
    let list = text.read_entry("list.spr").unwrap();
    assert_eq!(
        hex(&Sha256::digest(&list)),
        "523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127"
    );
    let catalog =
        SpriteActionCatalog::parse_for_inspection(&list, ActionLimits::default()).unwrap();
    let mut reader = Reader::new(&root);
    let mut all_groups = Vec::new();
    let mut samples = 0usize;
    for (class, sex, logical, gfx, count, rgba_bytes, sha) in [
        (
            0,
            0,
            0,
            3225,
            10,
            1490688,
            "39BF6D585DD38C56E523FF808390FAD0D1BDC2D2A28B302C287AED03A6DA33E7",
        ),
        (
            0,
            1,
            1,
            3227,
            10,
            1363968,
            "5740B38373AFFBAD47B7D1118D52BA034C4C6C026A5288AD8DBB6B8A81FE85BD",
        ),
        (
            1,
            0,
            61,
            3213,
            11,
            1881216,
            "AB53AE4065847C932A7BF2D3BE98B1280601621E34AB416F4F0FD75F92320B50",
        ),
        (
            1,
            1,
            48,
            3215,
            11,
            1698048,
            "3CFC37684C476CD4D4F0651CD4BDBE10E3314AF416E6C579FB1ED8D00FCE1CDF",
        ),
        (
            2,
            0,
            138,
            3217,
            9,
            1435392,
            "DA4E847B3A56D3DB64128072BFE6988ECF0E9223C4C6CC01370663B0CA134B0B",
        ),
        (
            2,
            1,
            37,
            3474,
            9,
            1370880,
            "FBE77AC1691CB0508AC8B6D1BAB396AC0EFDAE800EC78865FA676F45A4B3C22C",
        ),
        (
            3,
            0,
            734,
            3476,
            9,
            1689984,
            "8217DEDCC0A6145334034390A9DF2C7D65BEF0EAE9B7B4A66373F3969CF24C28",
        ),
        (
            3,
            1,
            1186,
            3221,
            12,
            1820160,
            "68D973F9FA5D73ED053AA45376BFD31E8E34EEFF0FE873777350F37D1304994C",
        ),
        (
            4,
            0,
            2786,
            2786,
            8,
            1231488,
            "4AD8A5280F6047A7494CECED8D58C2B682076FA05D8BCA6E7FE1CE9E1EB6F80A",
        ),
        (
            4,
            1,
            2796,
            2796,
            8,
            1243008,
            "3D3A64B816581435274D5477E087EAE4201E2BD03DAFCA81AB45B288DC04AAE4",
        ),
        (
            6,
            0,
            6671,
            6671,
            10,
            1830528,
            "A1F1FB10C87F9EF303B596142F1A4CC60CA2792D76B4D417C01A6281413377E8",
        ),
        (
            6,
            1,
            6650,
            6650,
            8,
            1078272,
            "A3F3359322C7E0904B092AEC5ECD2C601EFBD13D49E0741E052B79C274381212",
        ),
    ] {
        let appearance = BaseAppearance { class, sex };
        let action = QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            appearance,
            4,
            CombatAction::Attack,
        )
        .unwrap();
        let single = QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            appearance,
            13,
            CombatAction::Attack,
        )
        .unwrap();
        assert_eq!(action.animation(), single.animation());
        let p = action.provenance();
        assert_eq!(
            (p.logical_sprite_id, p.graphics_id, p.action_key.id),
            (logical, gfx, 21)
        );
        let animation = action.animation();
        assert_eq!(animation.frames.len(), count);
        let mut digest = Sha256::new();
        let mut cache = BTreeMap::<String, DecodedSprite>::new();
        let mut total = 0usize;
        for heading in 0..8 {
            for index in 0..count {
                let asset = animation.frame_asset(gfx, index, heading).unwrap();
                let name = asset.file_name();
                if !cache.contains_key(&name) {
                    let (_, bytes) = reader.read(&name);
                    cache.insert(
                        name.clone(),
                        decode_classic_spr_with_limits(&bytes, SpriteLimits::default()).unwrap(),
                    );
                }
                let frame = &cache[&name].frames[asset.subframe as usize];
                digest.update(frame.x_offset.to_le_bytes());
                digest.update(frame.y_offset.to_le_bytes());
                digest.update(frame.width.to_le_bytes());
                digest.update(frame.height.to_le_bytes());
                digest.update(&frame.rgba);
                total += frame.rgba.len();
                samples += 1;
            }
        }
        assert_eq!(cache.len(), 16);
        assert_eq!(total, rgba_bytes);
        assert_eq!(hex(&digest.finalize()), sha, "logical {logical}");
        let mut playback = action.start();
        let crossed = playback.advance(action.duration().unwrap(), 0).unwrap();
        assert!(crossed.finished);
        assert_eq!(crossed.source_frame, 0);
        assert_eq!(
            crossed
                .crossed_cues
                .iter()
                .filter(|c| c.event == FrameEvent::Hit)
                .count(),
            1
        );
        assert_eq!(
            crossed
                .crossed_cues
                .iter()
                .filter(|c| matches!(c.event, FrameEvent::Sound(_) | FrameEvent::Effect(_)))
                .count(),
            1
        );
        let sound = if class == 2 {
            1555
        } else if class == 4 {
            461
        } else {
            13
        };
        let audio = crossed
            .crossed_cues
            .iter()
            .find(|c| matches!(c.event, FrameEvent::Sound(_) | FrameEvent::Effect(_)))
            .unwrap();
        assert_eq!(
            audio.event,
            if class == 2 || class == 6 || class == 4 && sex == 0 {
                FrameEvent::Effect(sound)
            } else {
                FrameEvent::Sound(sound)
            }
        );
        let hit = crossed
            .crossed_cues
            .iter()
            .find(|c| c.event == FrameEvent::Hit)
            .unwrap();
        assert!(
            audio.tick < hit.tick,
            "draw sound must precede server-shot presentation marker"
        );
        for frame in &animation.frames {
            all_groups.extend(classify_native_cues(&frame.events).unwrap().audio);
        }
        assert!(playback
            .advance(Duration::MAX, 0)
            .unwrap()
            .crossed_cues
            .is_empty());
    }
    assert_eq!(samples, 920);
    for sex in 0..2 {
        assert!(QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            BaseAppearance { class: 5, sex },
            4,
            CombatAction::Attack
        )
        .is_err());
    }
    let bank = SoundBank::load_shared(&root, &all_groups, BankLimits::default(), &cancel).unwrap();
    assert_eq!(bank.ids().collect::<Vec<_>>(), [13, 461, 1555]);
    for (id, bytes, pcm_bytes, rate, bits, sha) in [
        (
            13,
            25172,
            25128,
            22000,
            8,
            "FC39283467871F856292659781CF029083C2F2201C7C8B2DFB8A8C025D69C897",
        ),
        (
            461,
            36276,
            36160,
            22050,
            16,
            "02B33FB030415C0E07EAFDA9CE1889D557C2F430D54C91F8F7F9992C595774D8",
        ),
        (
            1555,
            10172,
            10048,
            22000,
            8,
            "423A8E998AB456EEDD7EAF1E4CC5A69F233E30659E276A68F433406DCC463D39",
        ),
    ] {
        let sound = bank.get(id).unwrap();
        assert_eq!(sound.file_bytes(), bytes);
        assert_eq!(hex(&sound.sha256()), sha);
        assert_eq!(
            (
                sound.format().channels,
                sound.format().sample_rate,
                sound.format().bits_per_sample
            ),
            (1, rate, bits)
        );
        assert_eq!(
            sound.pcm_bytes().len(),
            pcm_bytes,
            "RIFF may include non-PCM chunks after data"
        );
    }
}
#[test]
#[ignore = "LITO_SHARED_RESOURCE_ROOT; 24 independent arrow frames and shared inventory96; no GUI"]
fn actual_shared_arrow_eight_banks_match_independent_reference() {
    let root = root();
    let cancel = AtomicBool::new(false);
    let mut reader = Reader::new(&root);
    let mut sprites = BTreeMap::new();
    let assets = super::ranged_effect::RangedAssets::load(&root, &cancel).unwrap();
    assert_eq!(assets.encoded_bytes(), 8666);
    assert_eq!(assets.sprite_sources().len(), 8);
    assert_eq!(
        hex(&assets.catalog_sha256()),
        "523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127"
    );
    for (heading, length, sha) in [
        (
            0,
            1056,
            "4795B0D2150D7D075D753EFC378799B8D471E52D32CDCE47EEC77FA502BB758C",
        ),
        (
            1,
            1038,
            "90B4C738925D38AD928B6E4ADFA36225627B047F9059EBA52AAC67F6FD1A3117",
        ),
        (
            2,
            1113,
            "B23AB3E957B60D1807F7A659C59D88613EC088FBB8AD643F33FF49D391697C93",
        ),
        (
            3,
            1055,
            "BAA7E02AD05E518C1FD59ED388DCDBD98A97558CA58182FC242BD9BD1080A3B5",
        ),
        (
            4,
            1121,
            "83F2B07CD964A3B9A402A16E5489EC6ABB84CD63BBEAD2BCAC4721D565E15745",
        ),
        (
            5,
            1004,
            "9A06C31D403B323D92E12190CE16C3FF03DE87AE40D1176034867980387393DD",
        ),
        (
            6,
            1213,
            "7EF8FE83FA9DF57B2DD858D3C07CFBD81BD43E5F48FE8F263F6DCD9412CD0D58",
        ),
        (
            7,
            1066,
            "B2B6443B4CBD2E703FE1FD18C5BC96557051B6055A1EBD39E2D3CBA9BBBF48FF",
        ),
    ] {
        let (archive, bytes) = reader.read(&format!("66-{heading}.spr"));
        assert_eq!(archive, SharedArchiveKind::Sprite);
        assert_eq!(bytes.len(), length);
        assert_eq!(hex(&Sha256::digest(&bytes)), sha);
        let decoded = decode_classic_spr_with_limits(&bytes, SpriteLimits::default()).unwrap();
        assert_eq!(decoded.frames.len(), 3);
        sprites.insert(heading, decoded);
    }
    for (heading, index, bounds, rgba_bytes, sha) in [
        (
            0,
            0,
            (0, -48, 48, 72),
            13824,
            "876939BB8DF55A566747E79839C00756DD575D38878D737DA62A7DC923E7360D",
        ),
        (
            0,
            1,
            (0, -36, 48, 60),
            11520,
            "34AEB1CAA43DE3DD2AB9828227D561949BC4E7F5E5CE7D98E0562C90B12FAC0C",
        ),
        (
            0,
            2,
            (0, -12, 48, 36),
            6912,
            "FF1276E415B3587A56527560D68563DDDD6BD344C72CC501DE4C816358263E0D",
        ),
        (
            1,
            0,
            (0, -48, 48, 72),
            13824,
            "B3C323DD8D4EC301AF8850C24053FA2594018E7A69BD64EEFF90B1698FCF14DD",
        ),
        (
            1,
            1,
            (0, -24, 48, 48),
            9216,
            "43892F2F8C9336E10AF5BF95737C6E8B45ECBC98EC44D5D5215CD53B93885BBD",
        ),
        (
            1,
            2,
            (0, 0, 48, 24),
            4608,
            "0057D70AE3916658F7F23BD49ACF54AE704037BA173635E9CDECF4BF01BDD569",
        ),
        (
            2,
            0,
            (0, -60, 48, 84),
            16128,
            "298034EF5E37674846665F321F763CCAD16436E8784BCCA417AD2648B2136251",
        ),
        (
            2,
            1,
            (0, -36, 48, 60),
            11520,
            "B6EB62B05E3C7981225FC51F264A2D521FEC123010B33626F673091F0B3AB5C2",
        ),
        (
            2,
            2,
            (0, -12, 48, 48),
            9216,
            "70DC039CE1333B7A70302631DD34EB81A28795CC9B90FEE053566154F409B0E6",
        ),
        (
            3,
            0,
            (-24, -48, 72, 72),
            20736,
            "5BAB6A38B703A37BF08CC12590BA8F21A1F9E0B3102000B6E2E9DB7717C16D06",
        ),
        (
            3,
            1,
            (0, -24, 48, 48),
            9216,
            "CF0EA94CB3438695509FA56907AA24FAB57A65612221DE9267957EC2278181C3",
        ),
        (
            3,
            2,
            (0, -12, 48, 36),
            6912,
            "70CB09C5BAED4B972AA0B9544357C438A4116109B4CA1AE2A1C2ADF81ECC77FC",
        ),
        (
            4,
            0,
            (0, -48, 48, 72),
            13824,
            "F08618094E065CE98BEFC711A64344060F868AE5947BF09FB7910ECEC111406D",
        ),
        (
            4,
            1,
            (0, -36, 48, 60),
            11520,
            "0434E3CC68F9C265D9190C1488A6B782BE0C070B61D5AD5728DA943D34BBB68A",
        ),
        (
            4,
            2,
            (0, -12, 48, 48),
            9216,
            "658E58E940938BF97D35857130972DFE3AABD33C3ACD3AB7D06A7242400996CB",
        ),
        (
            5,
            0,
            (0, -48, 48, 72),
            13824,
            "0FCAB0C0CEE054ACF63DD292F358D218BD5EC8CDDCEF8580765E45126EA711E2",
        ),
        (
            5,
            1,
            (0, -36, 48, 60),
            11520,
            "B7C9D565D338C837BBD722ED4803E3E38880B70389F57A41A405DCF8F05EBA88",
        ),
        (
            5,
            2,
            (0, -12, 48, 36),
            6912,
            "689C00002FA8772C4EADD4588DD727D01F7E3E5EB777B718852C0DF411AC1A76",
        ),
        (
            6,
            0,
            (-24, -48, 72, 84),
            24192,
            "8CF28F889B883A6C15ABB47AE0EE8F6E660A006642153F4F17F510BB268D0C12",
        ),
        (
            6,
            1,
            (0, -36, 48, 72),
            13824,
            "2C2933BEA642C8481A7BA2A3573326E3D93D931704089E2FE0E05C728C5A7454",
        ),
        (
            6,
            2,
            (0, -12, 48, 48),
            9216,
            "37097F23B1C7EE35963B2CF5E5F3C07CFE312C1DB3C7351050637DAB4133AAEA",
        ),
        (
            7,
            0,
            (-24, -48, 72, 72),
            20736,
            "46765CE8BB5A59899ACC5A8D9E2F8C4F4C2BAC2A57286DE9CBA551F187AECC52",
        ),
        (
            7,
            1,
            (0, -36, 48, 60),
            11520,
            "83338D2FE232B0701125E16E865D5689A7D190424947EC1862BBB2AD4191FD6F",
        ),
        (
            7,
            2,
            (0, -12, 48, 36),
            6912,
            "703E7A9191D22D17F15595A1F2C91D0BDF8449754E4E3B4D09A8C2C21C1D12E0",
        ),
    ] {
        let frame = &sprites[&heading].frames[index];
        // Inspection decoding intentionally omits world-interleaving fragments.
        // Compare all legacy metadata and every RGBA byte after checking that
        // the production world frame retained its separate fragment payload.
        let mut world_frame = assets.sprite_frames(heading).unwrap()[index].clone();
        assert!(world_frame.fragments.take().is_some());
        assert_eq!(frame, &world_frame);
        assert_eq!(
            (frame.x_offset, frame.y_offset, frame.width, frame.height),
            bounds
        );
        assert_eq!(frame.rgba.len(), rgba_bytes);
        assert_eq!(hex(&Sha256::digest(&frame.rgba)), sha);
    }
    let mut icons = SharedInventoryIcons::open(&root).unwrap();
    let icon = icons.load(96, &cancel).unwrap();
    let image = icon.image();
    assert_eq!(
        (image.raw_offset(), image.width(), image.height()),
        ([1, 2], 30, 29)
    );
    assert_eq!(
        hex(&Sha256::digest(image.reference_rgba().unwrap())),
        "2161D43E9C38B89BD1C7737A1F83527E02C7BC2B3FF161CD9FA758E394ECBB6D"
    );
    // Real qualified pixels through the production compositor, not a dummy
    // placeholder sprite or a timing test inferred only from the frame count.
    use super::{
        compositor::Canvas,
        ranged_effect::RangedPath,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let from = WorldCell { x: 32724, y: 32784 };
    let projection = ViewProjection {
        anchor_cell: from,
        anchor_pixel: PixelPoint { x: 200, y: 200 },
    };
    let mut distinct = std::collections::BTreeSet::new();
    for (heading, (dx, dy)) in [
        (0, (0, -3)),
        (1, (3, -3)),
        (2, (3, 0)),
        (3, (3, 3)),
        (4, (0, 3)),
        (5, (-3, 3)),
        (6, (-3, 0)),
        (7, (-3, -3)),
    ] {
        let path = RangedPath::new(
            from,
            WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            },
        )
        .unwrap();
        let sample = assets.effect_frame(&path, Duration::ZERO).unwrap();
        assert_eq!(sample.sprite_name, format!("66-{heading}.spr"));
        assert_eq!(
            (sample.source_frame, sample.subframe, sample.finished),
            (1, 0, false)
        );
        let mut canvas = Canvas::new(400, 400).unwrap();
        let blank = canvas.rgba().to_vec();
        assert!(assets
            .draw_effect(&mut canvas, &path, projection, Duration::ZERO)
            .unwrap());
        assert_ne!(canvas.rgba(), blank);
        distinct.insert(hex(&Sha256::digest(canvas.rgba())));
        canvas.clear([0, 0, 0]);
        assert!(!assets
            .draw_effect(&mut canvas, &path, projection, path.finished_at())
            .unwrap());
        assert_eq!(
            canvas.rgba(),
            blank,
            "terminal0 is unlinked, not a new impact"
        );
        assert!(!assets
            .draw_effect(&mut canvas, &path, projection, Duration::MAX)
            .unwrap());
        assert_eq!(canvas.rgba(), blank);
    }
    assert_eq!(distinct.len(), 8, "no four-heading fold");
}
