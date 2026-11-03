//! Exact opaque RGBA -> BGRA conversion. SIMD is only a CPU implementation
//! detail: no reduced resolution, colour quantization or render-state cache.

pub(super) fn copy(source: &[u8], target: &mut [u8]) {
    assert_eq!(source.len(), target.len());
    assert_eq!(source.len() % 4, 0);
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if std::is_x86_feature_detected!("sse2") {
        // SAFETY: CPU support is checked; equal whole-pixel slices are checked
        // above. The implementation only loads/stores complete 16-byte chunks.
        unsafe { copy_sse2(source, target) };
        return;
    }
    copy_scalar(source, target);
}

fn copy_scalar(source: &[u8], target: &mut [u8]) {
    for (src, dst) in source.chunks_exact(4).zip(target.chunks_exact_mut(4)) {
        dst.copy_from_slice(&[src[2], src[1], src[0], 255]);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn copy_sse2(source: &[u8], target: &mut [u8]) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let low = _mm_set1_epi32(0xff);
    let green = _mm_set1_epi32(0xff00);
    let opaque = _mm_set1_epi32(0xff000000u32 as i32);
    let mut source_chunks = source.chunks_exact(16);
    let mut target_chunks = target.chunks_exact_mut(16);
    for (src, dst) in source_chunks.by_ref().zip(target_chunks.by_ref()) {
        // SAFETY: chunks_exact supplies exactly 16 accessible bytes; unaligned
        // loads/stores impose no additional alignment. Source and target have
        // disjoint Rust borrows. No access reaches into another chunk or tail.
        let rgba = _mm_loadu_si128(src.as_ptr().cast());
        let red = _mm_slli_epi32::<16>(_mm_and_si128(rgba, low));
        let blue = _mm_and_si128(_mm_srli_epi32::<16>(rgba), low);
        let bgra = _mm_or_si128(
            _mm_or_si128(red, blue),
            _mm_or_si128(_mm_and_si128(rgba, green), opaque),
        );
        _mm_storeu_si128(dst.as_mut_ptr().cast(), bgra);
    }
    copy_scalar(source_chunks.remainder(), target_chunks.into_remainder());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dib_conversion_matches_scalar_for_unaligned_chunks_tails_and_guards() {
        for pixels in 0..65 {
            for source_offset in 0..16 {
                for target_offset in 0..16 {
                    let mut source = vec![0x42; source_offset + pixels * 4 + 17];
                    for (i, b) in source[source_offset..source_offset + pixels * 4]
                        .iter_mut()
                        .enumerate()
                    {
                        *b = (i * 137 + pixels * 41 + source_offset) as u8;
                    }
                    let original = source.clone();
                    let src = &source[source_offset..source_offset + pixels * 4];
                    let mut target = vec![0x9b; target_offset + pixels * 4 + 17];
                    let mut expected = target.clone();
                    copy_scalar(
                        src,
                        &mut expected[target_offset..target_offset + pixels * 4],
                    );
                    copy(src, &mut target[target_offset..target_offset + pixels * 4]);
                    assert_eq!(
                        target, expected,
                        "pixels={pixels}, offsets={source_offset}/{target_offset}"
                    );
                    assert_eq!(source, original);
                }
            }
        }
    }

    #[test]
    fn dib_conversion_preserves_every_red_blue_pair_and_forces_opaque_alpha() {
        let source: Vec<_> = (0..=255u8)
            .flat_map(|red| {
                (0..=255u8).flat_map(move |blue| [red, red.wrapping_add(blue), blue, red ^ blue])
            })
            .collect();
        let mut expected = vec![0; source.len()];
        copy_scalar(&source, &mut expected);
        let mut actual = vec![0; source.len()];
        copy(&source, &mut actual);
        assert_eq!(actual, expected);
        assert!(actual.chunks_exact(4).all(|p| p[3] == 255));
    }
}
