use lito_modern::modern_lito::{
    native_light_color::{Channels, Format, Remap}, native_light_mask::Mask,
};
use sha2::{Digest, Sha256};

const CAPTURE: &[u8] = include_bytes!("fixtures/native-light-color-original.bin");
const CHANNELS: usize = 20 + 1024;
const SELECTIONS: usize = CHANNELS + 12288;
const RASTER: usize = SELECTIONS + 104 * 45;
fn u16_at(at: usize) -> u16 { u16::from_le_bytes(CAPTURE[at..at + 2].try_into().unwrap()) }
fn u32_at(at: usize) -> u32 { u32::from_le_bytes(CAPTURE[at..at + 4].try_into().unwrap()) }
fn i32_at(at: usize) -> i32 { i32::from_le_bytes(CAPTURE[at..at + 4].try_into().unwrap()) }
fn retained() {
    assert_eq!(&CAPTURE[..8], b"LTLIT01\0");
    assert_eq!((u32_at(8), u32_at(12), u32_at(16)), (12288, 104, 11968));
    assert_eq!(CAPTURE.len(), RASTER + 11968 * 9);
    assert_eq!(format!("{:X}", Sha256::digest(CAPTURE)),
        "3A646B0EC052BEA9001B958683419BB148B0D5FA773B4C8BB397B641E8BF9564");
}
fn plane(width: usize, height: usize, bytes: &[u8]) -> Mask {
    let mut mask = Mask::new(width, height).unwrap();
    mask.compose_plane(bytes, width, height, 0, 0).unwrap();
    mask
}
fn identity() -> [u8; 32] { std::array::from_fn(|i| i as u8) }

#[test]
fn all_original_selector_calls_preserve_level_only_cache_semantics() {
    retained();
    let mut cache = Remap::default();
    for call in 0..104 {
        let at = SELECTIONS + call * 45;
        let before = i32_at(at + 5);
        if before == -1 { cache = Remap::default(); }
        else { assert_eq!(cache.level().map(i32::from), Some(before), "call{call}"); }
        assert_eq!(cache.select(u32_at(at), CAPTURE[at + 4] != 0),
            &CAPTURE[at + 13..at + 45], "original selector call{call}");
        assert_eq!(cache.level().map(i32::from), Some(i32_at(at + 9)));
    }
}

#[test]
fn every_generated_channel_entry_matches_captured_original_writer_output() {
    retained();
    let row = identity();
    let mut count = 0;
    for (mode, format, shifts) in [(0, Format::Rgb555, [0, 5, 10]), (1, Format::Rgb565, [0, 6, 11])] {
        let channels = Channels::new(format);
        for (component, shift) in shifts.into_iter().enumerate() {
            for color in 0..32 { for level in 0..32 {
                let expected = u16_at(CHANNELS + mode * 6144 + (component * 1024 + color * 32 + level) * 2);
                let mut pixels = [(color as u16) << shift; 4];
                let mask = plane(1, 1, &[(level as u8) << 3]);
                channels.apply_words(&mut pixels, 2, [0, 0], [2, 2], &mask, &row, None).unwrap();
                assert_eq!(pixels, [expected; 4], "format{mode} component{component} color{color} level{level}");
                count += 1;
            }}
        }
    }
    assert_eq!(count, 6144);
}

fn adversarial(format: Format) -> Channels {
    let shifts = if format == Format::Rgb555 { [0, 5, 10] } else { [0, 6, 11] };
    let mut bytes = Vec::new();
    // These are the declared inputs of the retained original x86 oracle, not
    // expected outputs calculated by the new kernel under test.
    for (shift, bias) in shifts.into_iter().zip([3, 11, 19]) {
        for color in 0..32 { for level in 0..32 {
            let value = (((color * 17 + level * 7 + bias) & 31) as u16) << shift;
            bytes.extend_from_slice(&value.to_le_bytes());
        }}
    }
    Channels::from_bytes(format, &bytes).unwrap()
}

#[test]
fn all_retained_native_kernel_samples_match_with_pitch_guards_and_byte_gates() {
    retained();
    let original = [Channels::from_bytes(Format::Rgb555, &CAPTURE[CHANNELS..CHANNELS + 6144]).unwrap(),
        Channels::from_bytes(Format::Rgb565, &CAPTURE[CHANNELS + 6144..CHANNELS + 12288]).unwrap()];
    let adversarial = [adversarial(Format::Rgb555), adversarial(Format::Rgb565)];
    for case in 0..11968 {
        let at = RASTER + case * 9;
        let mode = usize::from(CAPTURE[at]);
        let pass = CAPTURE[at + 1];
        let raw = CAPTURE[at + 2];
        let color = u16_at(at + 3);
        let gate = [CAPTURE[at + 5]; 4];
        let expected = u16_at(at + 7);
        let row = if pass == 0 { identity() }
                  else { std::array::from_fn(|i| ((i * 13 + 7) & 31) as u8) };
        assert_eq!(row[usize::from(raw >> 3)], CAPTURE[at + 6]);
        let mask = plane(1, 1, &[raw]);
        let mut pixels = [0xa55a; 60];
        for index in [15, 16, 27, 28] { pixels[index] = color; }
        let table = if pass == 0 { &original[mode] } else { &adversarial[mode] };
        let receipt = table.apply_words(&mut pixels, 12, [3, 1], [2, 2], &mask, &row,
            (pass != 0).then_some(gate.as_slice())).unwrap();
        assert_eq!(receipt.sampled, 4);
        for (index, actual) in pixels.into_iter().enumerate() {
            assert_eq!(actual, if [15, 16, 27, 28].contains(&index) { expected } else { 0xa55a },
                "original native sample{case}, backing index{index}");
        }
    }
}

#[test]
fn rgba_adapter_uses_native_555_entries_preserves_alpha_and_identity_bytes() {
    retained();
    let channels = Channels::new(Format::Rgb555);
    let mask = plane(3, 2, &[0, 120, 255, 64, 32, 192]);
    let row = identity();
    let gate: Vec<_> = (0..15).map(|i| u8::from(i != 7)).collect();
    let mut pixels: Vec<_> = (0..60).map(|i| ((i * 41 + 17) & 255) as u8).collect();
    let original = pixels.clone();
    let receipt = channels.apply_rgba555(&mut pixels, [5, 3], &mask, &row, Some(&gate)).unwrap();
    assert_eq!(receipt.sampled, 15);
    assert!(receipt.identity > 0 && receipt.transformed > 0 && receipt.gated > 0);
    for y in 0..3 { for x in 0..5 {
        let index = y * 5 + x;
        let at = index * 4;
        let level = usize::from(mask.pixels()[(y / 2) * 3 + x / 2] >> 3);
        let mut expected = original[at..at + 4].to_vec();
        if level != 31 && gate[index] != 0 {
            for (component, shift) in [10, 5, 0].into_iter().enumerate() {
                let color = usize::from(original[at + component] >> 3);
                let native = u16_at(CHANNELS + ((2 - component) * 1024 + color * 32 + level) * 2);
                let value = ((native >> shift) & 31) as u8;
                expected[component] = (value << 3) | (value >> 2);
            }
        }
        assert_eq!(&pixels[at..at + 4], expected);
    }}
}

#[test]
fn malformed_inputs_fail_before_any_surface_mutation() {
    let channels = Channels::new(Format::Rgb555);
    let mut pixels = [0xbeef; 16];
    let initial = pixels;
    let mask = plane(1, 1, &[127]);
    let mut row = identity(); row[7] = 32;
    assert!(channels.apply_words(&mut pixels, 4, [0, 0], [2, 2], &mask, &row, None).is_err());
    assert_eq!(pixels, initial);
    for (pitch, origin, size, gate) in [(4, [0, 0], [0, 2], None),
        (4, [0, 0], [3, 2], None), (1, [0, 0], [2, 2], None),
        (4, [usize::MAX, 0], [2, 2], None), (4, [0, usize::MAX], [2, 2], None),
        (4, [0, 3], [2, 2], None), (4, [0, 0], [2, 2], Some(&[1u8][..]))] {
        assert!(channels.apply_words(&mut pixels, pitch, origin, size, &mask, &identity(), gate).is_err());
        assert_eq!(pixels, initial);
    }
    let mut rgba = vec![17; 16]; let before = rgba.clone();
    assert!(Channels::new(Format::Rgb565).apply_rgba555(&mut rgba, [2, 2], &mask, &identity(), None).is_err());
    assert_eq!(rgba, before);
    assert!(channels.apply_rgba555(&mut rgba, [1, 2], &mask, &identity(), None).is_err());
    assert_eq!(rgba, before);
    let mut invalid = vec![0; 6144]; invalid[1] = 0x80;
    assert!(Channels::from_bytes(Format::Rgb555, &invalid).is_err());
}
