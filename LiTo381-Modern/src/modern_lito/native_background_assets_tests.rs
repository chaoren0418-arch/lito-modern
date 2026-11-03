use super::*;

fn source() -> &'static Path {
    Path::new("D:/lineage開發/3-lito開發/lito脫殼資源")
}

#[test]
fn modern_bin_20261007_selector_all_bytes_and_unknown_no_io() {
    assert_eq!(select(None), None);
    // A nonexistent relative source must not even be checked for unknown input.
    assert!(load_selected(Path::new("absent-relative-source"), None)
        .unwrap()
        .is_none());
    for byte in 0..=255u8 {
        assert_eq!(
            select(Some(byte)).unwrap().number(),
            if byte == 0 { 6314 } else { 7482 }
        );
    }
    assert!(load_selected(Path::new("relative-source"), Some(0)).is_err());
}

#[test]
fn modern_bin_20261007_real_assets_through_existing_img_and_canvas() {
    for flag in [0, 255] {
        let asset = load_selected(source(), Some(flag)).unwrap().unwrap();
        let image = asset.image();
        assert_eq!(
            (image.width, image.height, image.rgba.len()),
            (640, 480, BYTES)
        );
        assert_eq!(asset.id(), select(Some(flag)).unwrap());
        assert_eq!(
            format!("{:X}", Sha256::digest(&image.rgba)),
            asset.id().rgba_sha256()
        );
        let mut canvas = Canvas::new(639, 479).unwrap();
        canvas.clear([7, 13, 19]);
        // Explicit inspection origin clips one top/left source pixel. Independently
        // compare every output pixel; this does not assert Native backdrop geometry.
        asset.blit_preview(&mut canvas, [-1, -1]).unwrap();
        for y in 0..479usize {
            for x in 0..639usize {
                let src = ((y + 1) * 640 + x + 1) * 4;
                let dst = (y * 639 + x) * 4;
                let pixel = &image.rgba[src..src + 4];
                assert!(pixel[3] == 0 || pixel[3] == 255);
                let expected = if pixel[3] == 255 {
                    [pixel[0], pixel[1], pixel[2], 255]
                } else {
                    [7, 13, 19, 255]
                };
                assert_eq!(&canvas.rgba()[dst..dst + 4], &expected);
            }
        }
    }
}

#[test]
fn modern_bin_20261007_wrong_identity_corruption_lengths_and_path_refused() {
    let asset = BackgroundAsset::load(source(), BackgroundId::Image6314).unwrap();
    let original = &asset.image().rgba;
    assert!(BackgroundAsset::from_rgba(BackgroundId::Image7482, original.clone()).is_err());
    for bytes in [
        vec![],
        original[..BYTES - 1].to_vec(),
        [original.as_slice(), &[0]].concat(),
    ] {
        assert!(BackgroundAsset::from_rgba(BackgroundId::Image6314, bytes).is_err());
    }
    let mut corrupt = original.clone();
    corrupt[0] ^= 1;
    assert!(BackgroundAsset::from_rgba(BackgroundId::Image6314, corrupt).is_err());
    assert!(BackgroundAsset::load(&source().join(".."), BackgroundId::Image6314).is_err());
    assert_eq!(
        format!("{:X}", Sha256::digest(original)),
        BackgroundId::Image6314.rgba_sha256()
    );
}
