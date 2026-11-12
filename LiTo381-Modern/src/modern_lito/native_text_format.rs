//! Native inline text format and item-name presentation rules.
//!
//! Static evidence only: recovered PE SHA256 4271D92F…5C2AB6E9, decoded with
//! Capstone; the original binary was never executed.
//! - 4318E0 / 431030 / 431E50: the text writers split a string at `\f?`, `\b?`,
//!   `\i?`, `\k`, `\j`, `\c` and `\\`; only `\f` changes the colour.
//! - 503440..503524: 57 RGB555 colours; `\fX` selects entry `X - '0'`
//!   (`dword[0x503380 + X*4]`).
//! - 442980: an inventory name is drawn with `word_834F34[identified ?
//!   blessing : 3]`, filled at 4EC338 from palette entries 24, 14, 17 and 7.
//! - 4437F0 / 44A680: a name ending in `(digits)` gets thousands separators
//!   when the digit run is 4..=9 characters long.
//!
//! Everything here is presentation only; wire fields are never changed.

/// `dword_503440`: RGB555, red in the high bits.
pub const PALETTE: [u16; 57] = [
    0x0000, 0x001f, 0x03e0, 0x7c00, 0x509e, 0x3421, 0x50a5, 0x5ef7, 0x2108, 0x3def, 0x5f7f, 0x63b6,
    0x7afe, 0x7fe0, 0x7fff, 0x6318, 0x4200, 0x5ca7, 0x1577, 0x7db2, 0x4a3f, 0x1eb2, 0x02e0, 0x4a52,
    0x7fd6, 0x4e74, 0x662e, 0x6737, 0x14a5, 0x5fbc, 0x675f, 0x733f, 0x7f39, 0x7f99, 0x5e8d, 0x2e91,
    0x3e8c, 0x3696, 0x525a, 0x5e38, 0x5a36, 0x6631, 0x767d, 0x102d, 0x57bd, 0x7df0, 0x5000, 0x7e1f,
    0x564e, 0x3dff, 0x0014, 0x5949, 0x3214, 0x41ff, 0x0014, 0x43ef, 0x0280,
];
/// Palette entry 14, the normal inventory name colour.
pub const WHITE: u16 = PALETTE[14];
/// `word_834F34`: blessed, normal, cursed, unidentified.
pub const NAME_COLORS: [u16; 4] = [PALETTE[24], PALETTE[14], PALETTE[17], PALETTE[7]];

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Run {
    pub text: String,
    pub color: u16,
}

/// Splits `text` the way the native writers do. `\fX` (`'0' <= X < 'i'`)
/// switches to palette entry `X - '0'`; `\bX` (same range) and `\i0`..`\i5`
/// are consumed without a colour change; `\k`, `\j` and `\c` (code page
/// switches in the original) are consumed; `\\` draws one backslash. Any other
/// backslash is literal. Adjacent runs of one colour are merged.
///
/// The original scans CP950 bytes and skips double-byte characters, so a 0x5C
/// trail byte is never an escape; decoded Unicode text has the same property.
pub fn runs(text: &str, base: u16) -> Vec<Run> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Run> = Vec::new();
    let mut color = base;
    let mut pending = String::new();
    let mut at = 0;
    fn flush(out: &mut Vec<Run>, pending: &mut String, color: u16) {
        if pending.is_empty() {
            return;
        }
        match out.last_mut() {
            Some(last) if last.color == color => last.text.push_str(pending),
            _ => out.push(Run {
                text: pending.clone(),
                color,
            }),
        }
        pending.clear();
    }
    while at < chars.len() {
        let Some(&kind) = chars.get(at + 1).filter(|_| chars[at] == '\\') else {
            pending.push(chars[at]);
            at += 1;
            continue;
        };
        let argument = chars.get(at + 2).copied();
        let ranged = argument.filter(|c| ('0'..'i').contains(c));
        match kind {
            'f' if ranged.is_some() => {
                flush(&mut out, &mut pending, color);
                color = PALETTE[ranged.unwrap() as usize - '0' as usize];
                at += 3;
            }
            'b' if ranged.is_some() => at += 3,
            'i' if argument.is_some_and(|c| ('0'..='5').contains(&c)) => at += 3,
            'k' | 'j' | 'c' => at += 2,
            '\\' => {
                pending.push('\\');
                at += 2;
            }
            _ => {
                pending.push('\\');
                at += 1;
            }
        }
    }
    flush(&mut out, &mut pending, color);
    out
}

/// Text with every native escape removed.
pub fn plain(text: &str) -> String {
    runs(text, 0).into_iter().map(|r| r.text).collect()
}

/// 442980: `None` when the original would index past its four-entry table
/// (an identified item whose blessing byte is above 3).
pub fn name_color(identified: u8, blessing: u8) -> Option<u16> {
    let index = if identified != 0 { blessing } else { 3 };
    NAME_COLORS.get(usize::from(index)).copied()
}

/// 44A680: thousands separators for a run of 4..=9 ASCII digits.
pub fn group_digits(digits: &str) -> String {
    if !(4..=9).contains(&digits.len()) || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return digits.to_owned();
    }
    let lead = digits.len() % 3;
    let mut out = String::with_capacity(digits.len() + 3);
    out.push_str(&digits[..lead]);
    for (group, chunk) in digits.as_bytes()[lead..].chunks(3).enumerate() {
        if lead > 0 || group > 0 {
            out.push(',');
        }
        out.push_str(std::str::from_utf8(chunk).unwrap_or_default());
    }
    out
}

/// 4437F0: the first `(` is paired with a final `)`; when everything between
/// them is a digit, that number is regrouped by [`group_digits`].
pub fn inventory_view_name(name: &str) -> String {
    let Some(body) = name.strip_suffix(')') else {
        return name.to_owned();
    };
    let Some(open) = body.find('(') else {
        return name.to_owned();
    };
    let digits = &body[open + 1..];
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return name.to_owned();
    }
    format!("{}({})", &body[..open], group_digits(digits))
}

/// Five-bit channels shifted left by three, as the native text path hands
/// colours to GDI. Returns `[red, green, blue]`.
pub fn rgb(color: u16) -> [u8; 3] {
    [
        (((color >> 10) & 31) << 3) as u8,
        (((color >> 5) & 31) << 3) as u8,
        ((color & 31) << 3) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(text: &str, color: u16) -> Run {
        Run {
            text: text.into(),
            color,
        }
    }
    #[test]
    fn colour_escapes_follow_the_native_writer() {
        assert_eq!(runs("治癒藥水", 7), [run("治癒藥水", 7)]);
        // `\fT` is entry 0x54 - 0x30 = 36; `\f2` is entry 2; `\fY` is entry 41.
        assert_eq!(runs("\\fT試煉的騎士", 1), [run("試煉的騎士", 0x3e8c)]);
        assert_eq!(
            runs("a\\f2b\\fYc", 1),
            [run("a", 1), run("b", 0x03e0), run("c", 0x6631)]
        );
        // Last selectable entry is 'h' (56); 'i' and '/' stay literal.
        assert_eq!(runs("\\fhx", 1), [run("x", 0x0280)]);
        assert_eq!(runs("\\fix", 1), [run("\\fix", 1)]);
        assert_eq!(runs("\\f/x", 1), [run("\\f/x", 1)]);
        assert_eq!(runs("x\\f", 1), [run("x\\f", 1)]);
        assert_eq!(runs("x\\", 1), [run("x\\", 1)]);
    }
    #[test]
    fn other_escapes_are_consumed_without_changing_colour() {
        assert_eq!(runs("a\\b3b\\i5c\\kd\\je\\cf", 9), [run("abcdef", 9)]);
        assert_eq!(runs("a\\i6b", 9), [run("a\\i6b", 9)]);
        assert_eq!(runs("a\\bib", 9), [run("a\\bib", 9)]);
        assert_eq!(runs("a\\\\fTb", 9), [run("a\\fTb", 9)]);
        assert_eq!(runs("a\\zb", 9), [run("a\\zb", 9)]);
        assert!(runs("\\f3\\f3", 9).is_empty());
        assert_eq!(plain("\\fT試煉\\f2的\\b0騎士"), "試煉的騎士");
        assert_eq!(plain(""), "");
    }
    #[test]
    fn name_colour_table_matches_the_native_initialiser() {
        assert_eq!(NAME_COLORS, [0x7fd6, 0x7fff, 0x5ca7, 0x5ef7]);
        assert_eq!(name_color(1, 0), Some(0x7fd6));
        assert_eq!(name_color(1, 1), Some(0x7fff));
        assert_eq!(name_color(1, 2), Some(0x5ca7));
        assert_eq!(name_color(1, 3), Some(0x5ef7));
        // Unidentified always uses entry 3, whatever the blessing byte says.
        for blessing in [0, 1, 2, 3, 128, 255] {
            assert_eq!(name_color(0, blessing), Some(0x5ef7));
        }
        assert_eq!(name_color(1, 4), None);
        assert_eq!(name_color(1, 129), None);
        assert_eq!(rgb(0x7fd6), [248, 240, 176]);
        assert_eq!(rgb(0x5ca7), [184, 40, 56]);
    }
    #[test]
    fn trailing_counts_are_grouped_like_the_native_item_constructor() {
        assert_eq!(group_digits("123"), "123");
        assert_eq!(group_digits("1234"), "1,234");
        assert_eq!(group_digits("12345"), "12,345");
        assert_eq!(group_digits("123456"), "123,456");
        assert_eq!(group_digits("1234567"), "1,234,567");
        assert_eq!(group_digits("123456789"), "123,456,789");
        assert_eq!(group_digits("1234567890"), "1234567890");
        assert_eq!(group_digits("12a4"), "12a4");
        assert_eq!(inventory_view_name("金幣 (50000)"), "金幣 (50,000)");
        assert_eq!(inventory_view_name("金幣 (500)"), "金幣 (500)");
        assert_eq!(inventory_view_name("金幣"), "金幣");
        assert_eq!(inventory_view_name("金幣 (50000) "), "金幣 (50000) ");
        // The first `(` is used, so a charge count before the stack count
        // leaves non-digits in between and nothing is regrouped.
        assert_eq!(inventory_view_name("魔杖 (5000) (3000)"), "魔杖 (5000) (3000)");
        assert_eq!(inventory_view_name("()"), "()");
        assert_eq!(inventory_view_name(")"), ")");
    }
}
