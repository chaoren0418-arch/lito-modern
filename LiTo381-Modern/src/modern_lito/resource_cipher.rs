//! Shared Text.idx/Text.pak L1 cipher, ported from the local PakViewer
//! `Lin.Helper.Core/Pak/PakTools.cs` (Coder/DecodeBlock and its helpers).
//!
//! The permutations and S-boxes below were reduced from its embedded Map1..4
//! tables, and ROUND_KEYS are the exact 96 bytes of Map5.bin. This is not a
//! guessed DES password or the separate OldDesHandler format. Decoding reverses
//! the sixteen round keys, swaps the final halves, and preserves a final short
//! block. IDX keeps its four-byte count outside the cipher; PAK entries start
//! their cipher at byte zero, independently of their offset in the archive.

use std::io::{self, Read};

const INITIAL_BIT_DEST: [u8; 64] = [
    39, 7, 47, 15, 55, 23, 63, 31, 38, 6, 46, 14, 54, 22, 62, 30, 37, 5, 45, 13, 53, 21, 61, 29,
    36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11, 51, 19, 59, 27, 34, 2, 42, 10, 50, 18, 58, 26,
    33, 1, 41, 9, 49, 17, 57, 25, 32, 0, 40, 8, 48, 16, 56, 24,
];
const FINAL_BIT_DEST: [u8; 64] = [
    57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35, 27, 19, 11, 3, 61, 53, 45, 37, 29, 21, 13, 5, 63,
    55, 47, 39, 31, 23, 15, 7, 56, 48, 40, 32, 24, 16, 8, 0, 58, 50, 42, 34, 26, 18, 10, 2, 60, 52,
    44, 36, 28, 20, 12, 4, 62, 54, 46, 38, 30, 22, 14, 6,
];
const ROUND_BIT_DEST: [u8; 32] = [
    8, 16, 22, 30, 12, 27, 1, 17, 23, 15, 29, 5, 25, 19, 9, 0, 7, 13, 24, 2, 3, 28, 10, 18, 31, 11,
    21, 6, 4, 26, 14, 20,
];
const ROUND_KEYS: [u8; 96] = [
    0x50, 0x9C, 0xA6, 0x16, 0x8A, 0x25, 0x80, 0xAE, 0x20, 0x30, 0x40, 0x9E, 0xC0, 0x3E, 0x36, 0xC5,
    0x30, 0x87, 0xE4, 0xB6, 0x40, 0xA6, 0x22, 0xE9, 0x42, 0xD6, 0x62, 0x32, 0x9B, 0x47, 0xE8, 0xD1,
    0x52, 0x16, 0x85, 0xB2, 0x24, 0xC3, 0x4B, 0x4D, 0x2D, 0x41, 0x23, 0x51, 0x13, 0x6A, 0xE0, 0x58,
    0x1E, 0x52, 0x49, 0xD2, 0x8C, 0x6B, 0x0B, 0x51, 0x58, 0x0E, 0x9F, 0x18, 0x0C, 0x49, 0xCD, 0x19,
    0x75, 0x70, 0x13, 0x69, 0x09, 0x69, 0xC8, 0x20, 0x09, 0x2D, 0xA1, 0xC0, 0x6C, 0x1E, 0xD1, 0x2C,
    0x8D, 0xAD, 0x12, 0x9C, 0x51, 0xA6, 0x80, 0x91, 0x52, 0xE3, 0xF0, 0x28, 0x2C, 0x79, 0x6B, 0x04,
];
const SBOXES: [[u8; 64]; 8] = [
    [
        14, 0, 4, 15, 13, 7, 1, 4, 2, 14, 15, 2, 11, 13, 8, 1, 3, 10, 10, 6, 6, 12, 12, 11, 5, 9,
        9, 5, 0, 3, 7, 8, 4, 15, 1, 12, 14, 8, 8, 2, 13, 4, 6, 9, 2, 1, 11, 7, 15, 5, 12, 11, 9, 3,
        7, 14, 3, 10, 10, 0, 5, 6, 0, 13,
    ],
    [
        15, 3, 1, 13, 8, 4, 14, 7, 6, 15, 11, 2, 3, 8, 4, 14, 9, 12, 7, 0, 2, 1, 13, 10, 12, 6, 0,
        9, 5, 11, 10, 5, 0, 13, 14, 8, 7, 10, 11, 1, 10, 3, 4, 15, 13, 4, 1, 2, 5, 11, 8, 6, 12, 7,
        6, 12, 9, 0, 3, 5, 2, 14, 15, 9,
    ],
    [
        10, 13, 0, 7, 9, 0, 14, 9, 6, 3, 3, 4, 15, 6, 5, 10, 1, 2, 13, 8, 12, 5, 7, 14, 11, 12, 4,
        11, 2, 15, 8, 1, 13, 1, 6, 10, 4, 13, 9, 0, 8, 6, 15, 9, 3, 8, 0, 7, 11, 4, 1, 15, 2, 14,
        12, 3, 5, 11, 10, 5, 14, 2, 7, 12,
    ],
    [
        7, 13, 13, 8, 14, 11, 3, 5, 0, 6, 6, 15, 9, 0, 10, 3, 1, 4, 2, 7, 8, 2, 5, 12, 11, 1, 12,
        10, 4, 14, 15, 9, 10, 3, 6, 15, 9, 0, 0, 6, 12, 10, 11, 1, 7, 13, 13, 8, 15, 9, 1, 4, 3, 5,
        14, 11, 5, 12, 2, 7, 8, 2, 4, 14,
    ],
    [
        2, 14, 12, 11, 4, 2, 1, 12, 7, 4, 10, 7, 11, 13, 6, 1, 8, 5, 5, 0, 3, 15, 15, 10, 13, 3, 0,
        9, 14, 8, 9, 6, 4, 11, 2, 8, 1, 12, 11, 7, 10, 1, 13, 14, 7, 2, 8, 13, 15, 6, 9, 15, 12, 0,
        5, 9, 6, 10, 3, 4, 0, 5, 14, 3,
    ],
    [
        12, 10, 1, 15, 10, 4, 15, 2, 9, 7, 2, 12, 6, 9, 8, 5, 0, 6, 13, 1, 3, 13, 4, 14, 14, 0, 7,
        11, 5, 3, 11, 8, 9, 4, 14, 3, 15, 2, 5, 12, 2, 9, 8, 5, 12, 15, 3, 10, 7, 11, 0, 14, 4, 1,
        10, 7, 1, 6, 13, 0, 11, 8, 6, 13,
    ],
    [
        4, 13, 11, 0, 2, 11, 14, 7, 15, 4, 0, 9, 8, 1, 13, 10, 3, 14, 12, 3, 9, 5, 7, 12, 5, 2, 10,
        15, 6, 8, 1, 6, 1, 6, 4, 11, 11, 13, 13, 8, 12, 1, 3, 4, 7, 10, 14, 7, 10, 9, 15, 5, 6, 0,
        8, 15, 0, 14, 5, 2, 9, 3, 2, 12,
    ],
    [
        13, 1, 2, 15, 8, 13, 4, 8, 6, 10, 15, 3, 11, 7, 1, 4, 10, 12, 9, 5, 3, 6, 14, 11, 5, 0, 0,
        14, 12, 9, 7, 2, 7, 2, 11, 1, 4, 14, 1, 7, 9, 4, 12, 10, 14, 8, 2, 13, 0, 15, 6, 12, 10, 9,
        13, 0, 15, 3, 3, 5, 5, 6, 8, 11,
    ],
];

// Rebuild the original nibble/byte permutation lookup tables at compile time.
// The runtime performs no allocations per block or per round.
const fn nibble_table(destinations: [u8; 64]) -> [[u64; 16]; 16] {
    let mut result = [[0; 16]; 16];
    let mut nibble = 0;
    while nibble < 16 {
        let mut value = 0;
        while value < 16 {
            let mut bit = 0;
            while bit < 4 {
                if value & (1 << (3 - bit)) != 0 {
                    result[nibble][value] |= 1u64 << (63 - destinations[nibble * 4 + bit]);
                }
                bit += 1;
            }
            value += 1;
        }
        nibble += 1;
    }
    result
}

const fn round_table() -> [[u32; 256]; 4] {
    let mut result = [[0; 256]; 4];
    let mut byte = 0;
    while byte < 4 {
        let mut value = 0;
        while value < 256 {
            let mut bit = 0;
            while bit < 8 {
                if value & (128 >> bit) != 0 {
                    result[byte][value] |= 1u32 << (31 - ROUND_BIT_DEST[byte * 8 + bit]);
                }
                bit += 1;
            }
            value += 1;
        }
        byte += 1;
    }
    result
}

static INITIAL_TABLE: [[u64; 16]; 16] = nibble_table(INITIAL_BIT_DEST);
static FINAL_TABLE: [[u64; 16]; 16] = nibble_table(FINAL_BIT_DEST);
#[cfg(test)]
static ROUND_TABLE: [[u32; 256]; 4] = round_table();

// A pair's two six-bit S-box inputs determine one permuted output byte.
// Fuse those operations at compile time: 4 * 4096 * 4 = 64 KiB, shared by
// every loader. No archive/root cache, dynamic initialization or new cipher key.
const fn paired_substitution_table() -> [[u32; 4096]; 4] {
    let permutation = round_table();
    let mut table = [[0; 4096]; 4];
    let mut pair = 0;
    while pair < 4 {
        let mut input = 0;
        while input < 4096 {
            let value = (SBOXES[pair * 2][input >> 6] << 4) | SBOXES[pair * 2 + 1][input & 63];
            table[pair][input] = permutation[pair][value as usize];
            input += 1;
        }
        pair += 1;
    }
    table
}

const fn paired_round_keys() -> [[usize; 4]; 16] {
    let mut keys = [[0; 4]; 16];
    let mut round = 0;
    while round < 16 {
        let start = round * 6;
        keys[round] = [
            ((ROUND_KEYS[start] as usize) << 4) | ((ROUND_KEYS[start + 1] as usize) >> 4),
            (((ROUND_KEYS[start + 1] as usize) & 15) << 8) | ROUND_KEYS[start + 2] as usize,
            ((ROUND_KEYS[start + 3] as usize) << 4) | ((ROUND_KEYS[start + 4] as usize) >> 4),
            (((ROUND_KEYS[start + 4] as usize) & 15) << 8) | ROUND_KEYS[start + 5] as usize,
        ];
        round += 1;
    }
    keys
}

static PAIRED_SUBSTITUTION: [[u32; 4096]; 4] = paired_substitution_table();
static PAIRED_KEYS: [[usize; 4]; 16] = paired_round_keys();

#[inline]
fn permute_word(input: u64, table: &[[u64; 16]; 16]) -> u64 {
    table[0][(input >> 60) as usize]
        | table[1][((input >> 56) & 15) as usize]
        | table[2][((input >> 52) & 15) as usize]
        | table[3][((input >> 48) & 15) as usize]
        | table[4][((input >> 44) & 15) as usize]
        | table[5][((input >> 40) & 15) as usize]
        | table[6][((input >> 36) & 15) as usize]
        | table[7][((input >> 32) & 15) as usize]
        | table[8][((input >> 28) & 15) as usize]
        | table[9][((input >> 24) & 15) as usize]
        | table[10][((input >> 20) & 15) as usize]
        | table[11][((input >> 16) & 15) as usize]
        | table[12][((input >> 12) & 15) as usize]
        | table[13][((input >> 8) & 15) as usize]
        | table[14][((input >> 4) & 15) as usize]
        | table[15][(input & 15) as usize]
}

#[inline]
fn round_word(input: u32, round: usize) -> u32 {
    let a = input >> 24;
    let b = (input >> 16) & 255;
    let c = (input >> 8) & 255;
    let d = input & 255;
    // The same six bytes as C# ExpandInput, promoted before shifts and masked
    // exactly where the reference truncates to u8. Pairing commutes with XOR.
    let e0 = (d << 7 | ((a & 249 | a >> 2 & 6) >> 1)) & 255;
    let e1 = (((a & 1 | a << 2) << 3) | ((b >> 2 | b & 135) >> 3)) & 255;
    let e2 = (c >> 7 | ((b & 31 | (b & 248) << 2) << 1)) & 255;
    let e3 = (b << 7 | ((c & 249 | c >> 2 & 6) >> 1)) & 255;
    let e4 = (((c & 1 | c << 2) << 3) | ((d >> 2 | d & 135) >> 3)) & 255;
    let e5 = (a >> 7 | ((d & 31 | (d & 248) << 2) << 1)) & 255;
    let keys = &PAIRED_KEYS[round];
    PAIRED_SUBSTITUTION[0][((e0 << 4 | e1 >> 4) as usize) ^ keys[0]]
        | PAIRED_SUBSTITUTION[1][(((e1 & 15) << 8 | e2) as usize) ^ keys[1]]
        | PAIRED_SUBSTITUTION[2][((e3 << 4 | e4 >> 4) as usize) ^ keys[2]]
        | PAIRED_SUBSTITUTION[3][(((e4 & 15) << 8 | e5) as usize) ^ keys[3]]
}

fn decode_block(input: [u8; 8]) -> [u8; 8] {
    let state = permute_word(u64::from_be_bytes(input), &INITIAL_TABLE);
    let mut left = (state >> 32) as u32;
    let mut right = state as u32;
    let mut round = 16;
    while round != 0 {
        round -= 1;
        let next = left ^ round_word(right, round);
        left = right;
        right = next;
    }
    permute_word((u64::from(right) << 32) | u64::from(left), &FINAL_TABLE).to_be_bytes()
}

// Retain the original byte/round implementation solely as a structurally
// independent differential oracle. Existing C# golden tests remain unchanged.
#[cfg(test)]
fn permute(input: [u8; 8], table: &[[u64; 16]; 16]) -> [u8; 8] {
    let mut value = 0u64;
    for (byte, input) in input.into_iter().enumerate() {
        value |=
            table[byte * 2][(input >> 4) as usize] | table[byte * 2 + 1][(input & 15) as usize];
    }
    value.to_be_bytes()
}

#[cfg(test)]
fn round_function(input: [u8; 4], round: usize) -> [u8; 4] {
    let [a, b, c, d] = input.map(u32::from);
    // Integer promotion and final truncation match C# ExpandInput exactly.
    let mut expanded = [
        (d << 7 | ((a & 249 | a >> 2 & 6) >> 1)) as u8,
        (((a & 1 | a << 2) << 3) | ((b >> 2 | b & 135) >> 3)) as u8,
        (c >> 7 | ((b & 31 | (b & 248) << 2) << 1)) as u8,
        (b << 7 | ((c & 249 | c >> 2 & 6) >> 1)) as u8,
        (((c & 1 | c << 2) << 3) | ((d >> 2 | d & 135) >> 3)) as u8,
        (a >> 7 | ((d & 31 | (d & 248) << 2) << 1)) as u8,
    ];
    for (byte, value) in expanded.iter_mut().enumerate() {
        *value ^= ROUND_KEYS[round * 6 + byte];
    }
    let e = expanded.map(usize::from);
    let pairs = [
        e[0] << 4 | e[1] >> 4,
        (e[1] & 15) << 8 | e[2],
        e[3] << 4 | e[4] >> 4,
        (e[4] & 15) << 8 | e[5],
    ];
    let mut output = 0u32;
    for (pair, index) in pairs.into_iter().enumerate() {
        let substituted = SBOXES[pair * 2][index >> 6] << 4 | SBOXES[pair * 2 + 1][index & 63];
        output |= ROUND_TABLE[pair][substituted as usize];
    }
    output.to_be_bytes()
}

#[cfg(test)]
fn reference_decode_block(input: [u8; 8]) -> [u8; 8] {
    let mut state = permute(input, &INITIAL_TABLE);
    for round in (0..16).rev() {
        let right = [state[4], state[5], state[6], state[7]];
        let value = round_function(right, round);
        state = [
            right[0],
            right[1],
            right[2],
            right[3],
            value[0] ^ state[0],
            value[1] ^ state[1],
            value[2] ^ state[2],
            value[3] ^ state[3],
        ];
    }
    permute(
        [
            state[4], state[5], state[6], state[7], state[0], state[1], state[2], state[3],
        ],
        &FINAL_TABLE,
    )
}

/// Decode a single already-bounded PAK entry in place. The last 0..7 bytes
/// remain unchanged, exactly like the reference's non-padding block format.
pub(crate) fn decode_payload_in_place(bytes: &mut [u8]) {
    for block in bytes.chunks_exact_mut(8) {
        let decoded = decode_block(block.try_into().unwrap());
        block.copy_from_slice(&decoded);
    }
}

/// Streaming IDX decoder: count stays plaintext, then a single continuous
/// cipher stream covers all records. In particular the 28-byte record boundary
/// must not reset the eight-byte cipher blocks. Buffer storage is eight bytes.
pub(crate) struct L1IndexReader<R> {
    inner: R,
    header_remaining: usize,
    block: [u8; 8],
    next: usize,
    length: usize,
    finished: bool,
}

impl<R: Read> L1IndexReader<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner,
            header_remaining: 4,
            block: [0; 8],
            next: 0,
            length: 0,
            finished: false,
        }
    }
}

impl<R: Read> Read for L1IndexReader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.header_remaining != 0 {
            let limit = output.len().min(self.header_remaining);
            let length = self.inner.read(&mut output[..limit])?;
            self.header_remaining -= length;
            return Ok(length);
        }
        if self.next == self.length {
            if self.finished {
                return Ok(0);
            }
            let mut length = 0;
            while length < 8 {
                match self.inner.read(&mut self.block[length..]) {
                    Ok(0) => {
                        self.finished = true;
                        break;
                    }
                    Ok(read) => length += read,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(error),
                }
            }
            if length == 8 {
                self.block = decode_block(self.block);
            }
            self.next = 0;
            self.length = length;
        }
        let count = output.len().min(self.length - self.next);
        output[..count].copy_from_slice(&self.block[self.next..self.next + count]);
        self.next += count;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Generated once with the existing C# PakTools.Decode(byte[0..31], 0),
    // independently of this Rust implementation (2026-09-14).
    const KNOWN_DECODED: [u8; 32] = [
        0x48, 0xDB, 0xEF, 0x8C, 0xDA, 0x28, 0x47, 0x9F, 0x37, 0x25, 0xEE, 0x7E, 0x1A, 0x97, 0x92,
        0xEA, 0xAF, 0xF0, 0xA5, 0xBA, 0x7B, 0x60, 0x88, 0xA5, 0xA0, 0xB0, 0xAE, 0x0A, 0x56, 0x92,
        0xD5, 0x42,
    ];

    #[test]
    fn csharp_known_answer_four_blocks() {
        let mut bytes: Vec<u8> = (0..32).collect();
        decode_payload_in_place(&mut bytes);
        assert_eq!(bytes, KNOWN_DECODED);
    }

    fn fixed_random(state: &mut u64) -> u64 {
        // Reproducible test input only, not a cryptographic or gameplay RNG.
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn fused_tables_and_all_rounds_match_original_byte_operations() {
        for pair in 0..4 {
            for input in 0..4096 {
                let substituted =
                    SBOXES[pair * 2][input >> 6] << 4 | SBOXES[pair * 2 + 1][input & 63];
                assert_eq!(
                    PAIRED_SUBSTITUTION[pair][input],
                    ROUND_TABLE[pair][substituted as usize]
                );
            }
        }
        for round in 0..16 {
            for lane in 0..4 {
                for byte in 0..=255u32 {
                    let input = byte << (lane * 8);
                    assert_eq!(
                        round_word(input, round),
                        u32::from_be_bytes(round_function(input.to_be_bytes(), round))
                    );
                    let inverse = !input;
                    assert_eq!(
                        round_word(inverse, round),
                        u32::from_be_bytes(round_function(inverse.to_be_bytes(), round))
                    );
                }
            }
        }
        let mut seed = 0x8172_e44a_9913_560du64;
        for _ in 0..4096 {
            let input = fixed_random(&mut seed) as u32;
            for round in 0..16 {
                assert_eq!(
                    round_word(input, round),
                    u32::from_be_bytes(round_function(input.to_be_bytes(), round))
                );
            }
        }
    }

    #[test]
    fn word_feistel_matches_reference_blocks_permutations_and_byte_boundaries() {
        let check = |input: [u8; 8]| {
            assert_eq!(decode_block(input), reference_decode_block(input));
            for table in [&INITIAL_TABLE, &FINAL_TABLE] {
                assert_eq!(
                    permute_word(u64::from_be_bytes(input), table).to_be_bytes(),
                    permute(input, table)
                );
            }
        };
        for repeated in [0, 1, 0x55, 0xaa, 0xfe, 0xff] {
            check([repeated; 8]);
        }
        for lane in 0..8 {
            for byte in 0..=255u8 {
                let mut input = [0; 8];
                input[lane] = byte;
                check(input);
                let mut inverse = [255; 8];
                inverse[lane] = !byte;
                check(inverse);
            }
        }
        let mut seed = 0xd13a_76bc_294f_5801u64;
        for _ in 0..4096 {
            check(fixed_random(&mut seed).to_be_bytes());
        }
    }

    #[test]
    fn optimized_payload_and_unaligned_index_stream_preserve_every_tail() {
        let mut seed = 0x19fa_740b_5ca8_21d3u64;
        for blocks in [0, 1, 2, 3, 17] {
            for tail in 0..8 {
                let length = blocks * 8 + tail;
                let encoded: Vec<u8> = (0..length).map(|_| fixed_random(&mut seed) as u8).collect();
                let mut expected = encoded.clone();
                for block in expected.chunks_exact_mut(8) {
                    let decoded = reference_decode_block(block.try_into().unwrap());
                    block.copy_from_slice(&decoded);
                }
                let mut actual = encoded.clone();
                decode_payload_in_place(&mut actual);
                assert_eq!(actual, expected);
                assert_eq!(&actual[blocks * 8..], &encoded[blocks * 8..]);
                for source_chunk in [1, 3, 7, 9, 28] {
                    for output_chunk in [1, 5, 28, 31] {
                        let mut input = 37_496u32.to_le_bytes().to_vec();
                        input.extend_from_slice(&encoded);
                        let mut reader = L1IndexReader::new(ShortReads {
                            inner: Cursor::new(input),
                            maximum: source_chunk,
                        });
                        let mut output = Vec::new();
                        let mut buffer = vec![0; output_chunk];
                        loop {
                            let count = reader.read(&mut buffer).unwrap();
                            if count == 0 {
                                break;
                            }
                            output.extend_from_slice(&buffer[..count]);
                        }
                        assert_eq!(&output[..4], &37_496u32.to_le_bytes());
                        assert_eq!(&output[4..], &expected);
                        assert_eq!(reader.read(&mut buffer).unwrap(), 0);
                    }
                }
            }
        }
    }

    #[test]
    fn every_short_tail_is_unchanged_without_padding_or_crossing() {
        for length in 0..=32 {
            let mut bytes: Vec<u8> = (0..length as u8).collect();
            decode_payload_in_place(&mut bytes);
            let complete = length / 8 * 8;
            assert_eq!(&bytes[..complete], &KNOWN_DECODED[..complete]);
            assert_eq!(
                &bytes[complete..],
                &(complete as u8..length as u8).collect::<Vec<_>>()
            );
        }
    }

    struct ShortReads<R> {
        inner: R,
        maximum: usize,
    }
    impl<R: Read> Read for ShortReads<R> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = output.len().min(self.maximum);
            self.inner.read(&mut output[..count])
        }
    }

    #[test]
    fn index_count_is_not_decrypted_and_arbitrary_read_splits_are_safe() {
        for payload_length in 0..=32 {
            for maximum in 1..=9 {
                let mut input = 37_496u32.to_le_bytes().to_vec();
                input.extend(0..payload_length as u8);
                let source = ShortReads {
                    inner: Cursor::new(input),
                    maximum,
                };
                let mut reader = L1IndexReader::new(source);
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).unwrap();
                assert_eq!(&bytes[..4], &37_496u32.to_le_bytes());
                let complete = payload_length / 8 * 8;
                assert_eq!(&bytes[4..4 + complete], &KNOWN_DECODED[..complete]);
                assert_eq!(
                    &bytes[4 + complete..],
                    &(complete as u8..payload_length as u8).collect::<Vec<_>>()
                );
                assert_eq!(reader.read(&mut [0u8; 4]).unwrap(), 0);
            }
        }
    }
}
