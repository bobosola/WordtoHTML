//! Resolve Word's `w:sym` symbol-font glyphs to Unicode.
//!
//! Word writes a symbol-font character as `<w:sym w:font="Wingdings"
//! w:char="F04A"/>`. `w:char` is not a Unicode code point: it is an index into
//! the named font, normally written with an `0xF0` high byte, so `0xF04A` means
//! "glyph 0x4A of Wingdings" — a smiley face. Dropping the element loses the
//! character; emitting `w:char` verbatim yields a private-use code point that
//! no browser can draw. Each font therefore needs its own table, and the three
//! spelling variants below are all accepted:
//!
//! - `0xF04A` (Word's usual form: `0xF0` prefix + font index)
//! - `0x4A`   (bare font index, written by some tools)
//! - `0x263A` (a literal Unicode code point, written by others)

/// The character Word displayed, or `None` if it cannot be resolved.
pub fn to_char(font: &str, code: u32) -> Option<char> {
    let index = match code {
        c if c >> 8 == 0xF0 => Some((c & 0xFF) as u8),
        c if c <= 0xFF => Some(c as u8),
        _ => None, // already a Unicode code point
    };

    if let (Some(index), Some(table)) = (index, table_for(font)) {
        return table
            .binary_search_by_key(&index, |&(key, _)| key)
            .ok()
            .map(|pos| table[pos].1)
            // A gap in the table (a space, say) is still a real character, so
            // fall back to the plain one rather than dropping it.
            .or_else(|| visible(index));
    }

    // An unknown font has no reliable table, so keep the code point itself.
    index.map_or_else(|| char::from_u32(code), visible)
}

/// Font name -> glyph table. Names vary in case and spacing between producers.
fn table_for(font: &str) -> Option<&'static [(u8, char)]> {
    match font.trim().to_ascii_lowercase().as_str() {
        "symbol" => Some(SYMBOL),
        "wingdings" => Some(WINGDINGS),
        "wingdings 2" | "wingdings2" => Some(WINGDINGS2),
        "wingdings 3" | "wingdings3" => Some(WINGDINGS3),
        "webdings" => Some(WEBDINGS),
        _ => None,
    }
}

/// Map a font index straight through as a character, but never let control
/// codes (which would be invisible or corrupt the output) through.
fn visible(index: u8) -> Option<char> {
    Some(index as char).filter(|c| !c.is_control())
}

// Greek letters, mathematical operators, and arrows.
#[rustfmt::skip]
const SYMBOL: &[(u8, char)] = &[
    (0x20, '\u{00A0}'), (0x21, '\u{0021}'), (0x22, '\u{2200}'),
    (0x23, '\u{0023}'), (0x24, '\u{2203}'), (0x25, '\u{0025}'),
    (0x26, '\u{0026}'), (0x27, '\u{220B}'), (0x28, '\u{0028}'),
    (0x29, '\u{0029}'), (0x2A, '\u{2217}'), (0x2B, '\u{002B}'),
    (0x2C, '\u{002C}'), (0x2D, '\u{2212}'), (0x2E, '\u{002E}'),
    (0x2F, '\u{002F}'), (0x30, '\u{0030}'), (0x31, '\u{0031}'),
    (0x32, '\u{0032}'), (0x33, '\u{0033}'), (0x34, '\u{0034}'),
    (0x35, '\u{0035}'), (0x36, '\u{0036}'), (0x37, '\u{0037}'),
    (0x38, '\u{0038}'), (0x39, '\u{0039}'), (0x3A, '\u{003A}'),
    (0x3B, '\u{003B}'), (0x3C, '\u{003C}'), (0x3D, '\u{003D}'),
    (0x3E, '\u{003E}'), (0x3F, '\u{003F}'), (0x40, '\u{2245}'),
    (0x41, '\u{0391}'), (0x42, '\u{0392}'), (0x43, '\u{03A7}'),
    (0x44, '\u{2206}'), (0x45, '\u{0395}'), (0x46, '\u{03A6}'),
    (0x47, '\u{0393}'), (0x48, '\u{0397}'), (0x49, '\u{0399}'),
    (0x4A, '\u{03D1}'), (0x4B, '\u{039A}'), (0x4C, '\u{039B}'),
    (0x4D, '\u{039C}'), (0x4E, '\u{039D}'), (0x4F, '\u{039F}'),
    (0x50, '\u{03A0}'), (0x51, '\u{0398}'), (0x52, '\u{03A1}'),
    (0x53, '\u{03A3}'), (0x54, '\u{03A4}'), (0x55, '\u{03A5}'),
    (0x56, '\u{03C2}'), (0x57, '\u{2126}'), (0x58, '\u{039E}'),
    (0x59, '\u{03A8}'), (0x5A, '\u{0396}'), (0x5B, '\u{005B}'),
    (0x5C, '\u{2234}'), (0x5D, '\u{005D}'), (0x5E, '\u{22A5}'),
    (0x5F, '\u{005F}'), (0x60, '\u{F8E5}'), (0x61, '\u{03B1}'),
    (0x62, '\u{03B2}'), (0x63, '\u{03C7}'), (0x64, '\u{03B4}'),
    (0x65, '\u{03B5}'), (0x66, '\u{03C6}'), (0x67, '\u{03B3}'),
    (0x68, '\u{03B7}'), (0x69, '\u{03B9}'), (0x6A, '\u{03D5}'),
    (0x6B, '\u{03BA}'), (0x6C, '\u{03BB}'), (0x6D, '\u{03BC}'),
    (0x6E, '\u{03BD}'), (0x6F, '\u{03BF}'), (0x70, '\u{03C0}'),
    (0x71, '\u{03B8}'), (0x72, '\u{03C1}'), (0x73, '\u{03C3}'),
    (0x74, '\u{03C4}'), (0x75, '\u{03C5}'), (0x76, '\u{03D6}'),
    (0x77, '\u{03C9}'), (0x78, '\u{03BE}'), (0x79, '\u{03C8}'),
    (0x7A, '\u{03B6}'), (0x7B, '\u{007B}'), (0x7C, '\u{007C}'),
    (0x7D, '\u{007D}'), (0x7E, '\u{223C}'), (0xA0, '\u{20AC}'),
    (0xA1, '\u{03D2}'), (0xA2, '\u{2032}'), (0xA3, '\u{2264}'),
    (0xA4, '\u{2215}'), (0xA5, '\u{221E}'), (0xA6, '\u{0192}'),
    (0xA7, '\u{2663}'), (0xA8, '\u{2666}'), (0xA9, '\u{2665}'),
    (0xAA, '\u{2660}'), (0xAB, '\u{2194}'), (0xAC, '\u{2190}'),
    (0xAD, '\u{2191}'), (0xAE, '\u{2192}'), (0xAF, '\u{2193}'),
    (0xB0, '\u{00B0}'), (0xB1, '\u{00B1}'), (0xB2, '\u{2033}'),
    (0xB3, '\u{2265}'), (0xB4, '\u{00D7}'), (0xB5, '\u{221D}'),
    (0xB6, '\u{2202}'), (0xB7, '\u{2022}'), (0xB8, '\u{00F7}'),
    (0xB9, '\u{2260}'), (0xBA, '\u{2261}'), (0xBB, '\u{2248}'),
    (0xBC, '\u{2026}'), (0xBD, '\u{F8E6}'), (0xBE, '\u{F8E7}'),
    (0xBF, '\u{21B5}'), (0xC0, '\u{2135}'), (0xC1, '\u{2111}'),
    (0xC2, '\u{211C}'), (0xC3, '\u{2118}'), (0xC4, '\u{2297}'),
    (0xC5, '\u{2295}'), (0xC6, '\u{2205}'), (0xC7, '\u{2229}'),
    (0xC8, '\u{222A}'), (0xC9, '\u{2283}'), (0xCA, '\u{2287}'),
    (0xCB, '\u{2284}'), (0xCC, '\u{2282}'), (0xCD, '\u{2286}'),
    (0xCE, '\u{2208}'), (0xCF, '\u{2209}'), (0xD0, '\u{2220}'),
    (0xD1, '\u{2207}'), (0xD2, '\u{F6DA}'), (0xD3, '\u{F6D9}'),
    (0xD4, '\u{F6DB}'), (0xD5, '\u{220F}'), (0xD6, '\u{221A}'),
    (0xD7, '\u{22C5}'), (0xD8, '\u{00AC}'), (0xD9, '\u{2227}'),
    (0xDA, '\u{2228}'), (0xDB, '\u{21D4}'), (0xDC, '\u{21D0}'),
    (0xDD, '\u{21D1}'), (0xDE, '\u{21D2}'), (0xDF, '\u{21D3}'),
    (0xE0, '\u{25CA}'), (0xE1, '\u{2329}'), (0xE2, '\u{F8E8}'),
    (0xE3, '\u{F8E9}'), (0xE4, '\u{F8EA}'), (0xE5, '\u{2211}'),
    (0xE6, '\u{F8EB}'), (0xE7, '\u{F8EC}'), (0xE8, '\u{F8ED}'),
    (0xE9, '\u{F8EE}'), (0xEA, '\u{F8EF}'), (0xEB, '\u{F8F0}'),
    (0xEC, '\u{F8F1}'), (0xED, '\u{F8F2}'), (0xEE, '\u{F8F3}'),
    (0xEF, '\u{F8F4}'), (0xF1, '\u{232A}'), (0xF2, '\u{222B}'),
    (0xF3, '\u{2320}'), (0xF4, '\u{F8F5}'), (0xF5, '\u{2321}'),
    (0xF6, '\u{F8F6}'), (0xF7, '\u{F8F7}'), (0xF8, '\u{F8F8}'),
    (0xF9, '\u{F8F9}'), (0xFA, '\u{F8FA}'), (0xFB, '\u{F8FB}'),
    (0xFC, '\u{F8FC}'), (0xFD, '\u{F8FD}'), (0xFE, '\u{F8FE}'),
];

// Faces, hands, office and computer pictograms. Pandoc and the vendor charts
// give 0x4A as U+263A (WHITE SMILING FACE), but that code point defaults to a
// monochrome text presentation, so it is mapped to a colour emoji instead.
#[rustfmt::skip]
const WINGDINGS: &[(u8, char)] = &[
    (0x21, '\u{1F589}'), (0x22, '\u{2702}'), (0x23, '\u{2701}'),
    (0x24, '\u{1F453}'), (0x25, '\u{1F56D}'), (0x26, '\u{1F56E}'),
    (0x27, '\u{1F56F}'), (0x28, '\u{1F57F}'), (0x29, '\u{2706}'),
    (0x2A, '\u{1F582}'), (0x2B, '\u{1F583}'), (0x2C, '\u{1F4EA}'),
    (0x2D, '\u{1F4EB}'), (0x2E, '\u{1F4EC}'), (0x2F, '\u{1F4ED}'),
    (0x30, '\u{1F4C1}'), (0x31, '\u{1F4C2}'), (0x32, '\u{1F4C4}'),
    (0x33, '\u{1F5CF}'), (0x34, '\u{1F5D0}'), (0x35, '\u{1F5C4}'),
    (0x36, '\u{231B}'), (0x37, '\u{1F5AE}'), (0x38, '\u{1F5B0}'),
    (0x39, '\u{1F5B2}'), (0x3A, '\u{1F5B3}'), (0x3B, '\u{1F5B4}'),
    (0x3C, '\u{1F5AB}'), (0x3D, '\u{1F5AC}'), (0x3E, '\u{2707}'),
    (0x3F, '\u{270D}'), (0x40, '\u{1F58E}'), (0x41, '\u{270C}'),
    (0x42, '\u{1F44C}'), (0x43, '\u{1F44D}'), (0x44, '\u{1F44E}'),
    (0x45, '\u{261C}'), (0x46, '\u{261E}'), (0x47, '\u{261D}'),
    (0x48, '\u{261F}'), (0x49, '\u{1F590}'), (0x4A, '\u{1F642}'),
    (0x4B, '\u{1F610}'), (0x4C, '\u{2639}'), (0x4D, '\u{1F4A3}'),
    (0x4E, '\u{2620}'), (0x4F, '\u{1F3F3}'), (0x50, '\u{1F3F1}'),
    (0x51, '\u{2708}'), (0x52, '\u{263C}'), (0x53, '\u{1F4A7}'),
    (0x54, '\u{2744}'), (0x55, '\u{1F546}'), (0x56, '\u{271E}'),
    (0x57, '\u{1F548}'), (0x58, '\u{2720}'), (0x59, '\u{2721}'),
    (0x5A, '\u{262A}'), (0x5B, '\u{262F}'), (0x5C, '\u{0950}'),
    (0x5D, '\u{2638}'), (0x5E, '\u{2648}'), (0x5F, '\u{2649}'),
    (0x60, '\u{264A}'), (0x61, '\u{264B}'), (0x62, '\u{264C}'),
    (0x63, '\u{264D}'), (0x64, '\u{264E}'), (0x65, '\u{264F}'),
    (0x66, '\u{2650}'), (0x67, '\u{2651}'), (0x68, '\u{2652}'),
    (0x69, '\u{2653}'), (0x6A, '\u{1F670}'), (0x6B, '\u{1F675}'),
    (0x6C, '\u{25CF}'), (0x6D, '\u{1F53E}'), (0x6E, '\u{25A0}'),
    (0x6F, '\u{25A1}'), (0x70, '\u{1F790}'), (0x71, '\u{2751}'),
    (0x72, '\u{2752}'), (0x73, '\u{2B27}'), (0x74, '\u{29EB}'),
    (0x75, '\u{25C6}'), (0x76, '\u{2756}'), (0x77, '\u{2B25}'),
    (0x78, '\u{2327}'), (0x79, '\u{2BB9}'), (0x7A, '\u{2318}'),
    (0x7B, '\u{1F3F5}'), (0x7C, '\u{1F3F6}'), (0x7D, '\u{1F676}'),
    (0x7E, '\u{1F677}'), (0x80, '\u{24EA}'), (0x81, '\u{2460}'),
    (0x82, '\u{2461}'), (0x83, '\u{2462}'), (0x84, '\u{2463}'),
    (0x85, '\u{2464}'), (0x86, '\u{2465}'), (0x87, '\u{2466}'),
    (0x88, '\u{2467}'), (0x89, '\u{2468}'), (0x8A, '\u{2469}'),
    (0x8B, '\u{24FF}'), (0x8C, '\u{2776}'), (0x8D, '\u{2777}'),
    (0x8E, '\u{2778}'), (0x8F, '\u{2779}'), (0x90, '\u{277A}'),
    (0x91, '\u{277B}'), (0x92, '\u{277C}'), (0x93, '\u{277D}'),
    (0x94, '\u{277E}'), (0x95, '\u{277F}'), (0x96, '\u{1F662}'),
    (0x97, '\u{1F660}'), (0x98, '\u{1F661}'), (0x99, '\u{1F663}'),
    (0x9A, '\u{1F65E}'), (0x9B, '\u{1F65C}'), (0x9C, '\u{1F65D}'),
    (0x9D, '\u{1F65F}'), (0x9E, '\u{00B7}'), (0x9F, '\u{2022}'),
    (0xA1, '\u{26AA}'), (0xA2, '\u{1F786}'), (0xA3, '\u{1F788}'),
    (0xA4, '\u{25C9}'), (0xA5, '\u{25CE}'), (0xA6, '\u{1F53F}'),
    (0xA7, '\u{25AA}'), (0xA8, '\u{25FB}'), (0xA9, '\u{1F7C2}'),
    (0xAA, '\u{2726}'), (0xAB, '\u{2605}'), (0xAC, '\u{2736}'),
    (0xAD, '\u{2734}'), (0xAE, '\u{2739}'), (0xAF, '\u{2735}'),
    (0xB0, '\u{2BD0}'), (0xB1, '\u{2316}'), (0xB2, '\u{27E1}'),
    (0xB3, '\u{2311}'), (0xB4, '\u{2BD1}'), (0xB5, '\u{272A}'),
    (0xB6, '\u{2730}'), (0xB7, '\u{1F550}'), (0xB8, '\u{1F551}'),
    (0xB9, '\u{1F552}'), (0xBA, '\u{1F553}'), (0xBB, '\u{1F554}'),
    (0xBC, '\u{1F555}'), (0xBD, '\u{1F556}'), (0xBE, '\u{1F557}'),
    (0xBF, '\u{1F558}'), (0xC0, '\u{1F559}'), (0xC1, '\u{1F55A}'),
    (0xC2, '\u{1F55B}'), (0xC3, '\u{2BB0}'), (0xC4, '\u{2BB1}'),
    (0xC5, '\u{2BB2}'), (0xC6, '\u{2BB3}'), (0xC7, '\u{2BB4}'),
    (0xC8, '\u{2BB5}'), (0xC9, '\u{2BB6}'), (0xCA, '\u{2BB7}'),
    (0xCB, '\u{1F66A}'), (0xCC, '\u{1F66B}'), (0xCD, '\u{1F655}'),
    (0xCE, '\u{1F654}'), (0xCF, '\u{1F657}'), (0xD0, '\u{1F656}'),
    (0xD1, '\u{1F650}'), (0xD2, '\u{1F651}'), (0xD3, '\u{1F652}'),
    (0xD4, '\u{1F653}'), (0xD5, '\u{232B}'), (0xD6, '\u{2326}'),
    (0xD7, '\u{2B98}'), (0xD8, '\u{2B9A}'), (0xD9, '\u{2B99}'),
    (0xDA, '\u{2B9B}'), (0xDB, '\u{2B88}'), (0xDC, '\u{2B8A}'),
    (0xDD, '\u{2B89}'), (0xDE, '\u{2B8B}'), (0xDF, '\u{1F868}'),
    (0xE0, '\u{1F86A}'), (0xE1, '\u{1F869}'), (0xE2, '\u{1F86B}'),
    (0xE3, '\u{1F86C}'), (0xE4, '\u{1F86D}'), (0xE5, '\u{1F86F}'),
    (0xE6, '\u{1F86E}'), (0xE7, '\u{1F878}'), (0xE8, '\u{1F87A}'),
    (0xE9, '\u{1F879}'), (0xEA, '\u{1F87B}'), (0xEB, '\u{1F87C}'),
    (0xEC, '\u{1F87D}'), (0xED, '\u{1F87F}'), (0xEE, '\u{1F87E}'),
    (0xEF, '\u{21E6}'), (0xF0, '\u{21E8}'), (0xF1, '\u{21E7}'),
    (0xF2, '\u{21E9}'), (0xF3, '\u{2B04}'), (0xF4, '\u{21F3}'),
    (0xF5, '\u{2B00}'), (0xF6, '\u{2B01}'), (0xF7, '\u{2B03}'),
    (0xF8, '\u{2B02}'), (0xF9, '\u{1F8AC}'), (0xFA, '\u{1F8AD}'),
    (0xFB, '\u{1F5F6}'), (0xFC, '\u{2714}'), (0xFD, '\u{1F5F7}'),
    (0xFE, '\u{1F5F9}'),
];

// Circled numbers, stars, and checkbox-style glyphs.
#[rustfmt::skip]
const WINGDINGS2: &[(u8, char)] = &[
    (0x21, '\u{1F58A}'), (0x22, '\u{1F58B}'), (0x23, '\u{1F58C}'),
    (0x24, '\u{1F58D}'), (0x25, '\u{2704}'), (0x26, '\u{2700}'),
    (0x27, '\u{1F57E}'), (0x28, '\u{1F57D}'), (0x29, '\u{1F5C5}'),
    (0x2A, '\u{1F5C6}'), (0x2B, '\u{1F5C7}'), (0x2C, '\u{1F5C8}'),
    (0x2D, '\u{1F5C9}'), (0x2E, '\u{1F5CA}'), (0x2F, '\u{1F5CB}'),
    (0x30, '\u{1F5CC}'), (0x31, '\u{1F5CD}'), (0x32, '\u{1F4CB}'),
    (0x33, '\u{1F5D1}'), (0x34, '\u{1F5D4}'), (0x35, '\u{1F5B5}'),
    (0x36, '\u{1F5B6}'), (0x37, '\u{1F5B7}'), (0x38, '\u{1F5B8}'),
    (0x39, '\u{1F5AD}'), (0x3A, '\u{1F5AF}'), (0x3B, '\u{1F5B1}'),
    (0x3C, '\u{1F592}'), (0x3D, '\u{1F593}'), (0x3E, '\u{1F598}'),
    (0x3F, '\u{1F599}'), (0x40, '\u{1F59A}'), (0x41, '\u{1F59B}'),
    (0x42, '\u{1F448}'), (0x43, '\u{1F449}'), (0x44, '\u{1F59C}'),
    (0x45, '\u{1F59D}'), (0x46, '\u{1F59E}'), (0x47, '\u{1F59F}'),
    (0x48, '\u{1F5A0}'), (0x49, '\u{1F5A1}'), (0x4A, '\u{1F446}'),
    (0x4B, '\u{1F447}'), (0x4C, '\u{1F5A2}'), (0x4D, '\u{1F5A3}'),
    (0x4E, '\u{1F591}'), (0x4F, '\u{1F5F4}'), (0x50, '\u{2713}'),
    (0x51, '\u{1F5F5}'), (0x52, '\u{2611}'), (0x53, '\u{2612}'),
    (0x54, '\u{2612}'), (0x55, '\u{2BBE}'), (0x56, '\u{2BBF}'),
    (0x57, '\u{29B8}'), (0x58, '\u{29B8}'), (0x59, '\u{1F671}'),
    (0x5A, '\u{1F674}'), (0x5B, '\u{1F672}'), (0x5C, '\u{1F673}'),
    (0x5D, '\u{203D}'), (0x5E, '\u{1F679}'), (0x5F, '\u{1F67A}'),
    (0x60, '\u{1F67B}'), (0x61, '\u{1F666}'), (0x62, '\u{1F664}'),
    (0x63, '\u{1F665}'), (0x64, '\u{1F667}'), (0x65, '\u{1F65A}'),
    (0x66, '\u{1F658}'), (0x67, '\u{1F659}'), (0x68, '\u{1F65B}'),
    (0x69, '\u{24EA}'), (0x6A, '\u{2460}'), (0x6B, '\u{2461}'),
    (0x6C, '\u{2462}'), (0x6D, '\u{2463}'), (0x6E, '\u{2464}'),
    (0x6F, '\u{2465}'), (0x70, '\u{2466}'), (0x71, '\u{2467}'),
    (0x72, '\u{2468}'), (0x73, '\u{2469}'), (0x74, '\u{24FF}'),
    (0x75, '\u{2776}'), (0x76, '\u{2777}'), (0x77, '\u{2778}'),
    (0x78, '\u{2779}'), (0x79, '\u{277A}'), (0x7A, '\u{277B}'),
    (0x7B, '\u{277C}'), (0x7C, '\u{277D}'), (0x7D, '\u{277E}'),
    (0x7E, '\u{277F}'), (0x80, '\u{2609}'), (0x81, '\u{1F315}'),
    (0x82, '\u{263D}'), (0x83, '\u{263E}'), (0x84, '\u{2E3F}'),
    (0x85, '\u{271D}'), (0x86, '\u{1F547}'), (0x87, '\u{1F55C}'),
    (0x88, '\u{1F55D}'), (0x89, '\u{1F55E}'), (0x8A, '\u{1F55F}'),
    (0x8B, '\u{1F560}'), (0x8C, '\u{1F561}'), (0x8D, '\u{1F562}'),
    (0x8E, '\u{1F563}'), (0x8F, '\u{1F564}'), (0x90, '\u{1F565}'),
    (0x91, '\u{1F566}'), (0x92, '\u{1F567}'), (0x93, '\u{1F668}'),
    (0x94, '\u{1F669}'), (0x95, '\u{2022}'), (0x96, '\u{25CF}'),
    (0x97, '\u{26AB}'), (0x98, '\u{2B24}'), (0x99, '\u{1F785}'),
    (0x9A, '\u{1F786}'), (0x9B, '\u{1F787}'), (0x9C, '\u{1F788}'),
    (0x9D, '\u{1F78A}'), (0x9E, '\u{29BF}'), (0x9F, '\u{25FE}'),
    (0xA1, '\u{25FC}'), (0xA2, '\u{2B1B}'), (0xA3, '\u{2B1C}'),
    (0xA4, '\u{1F791}'), (0xA5, '\u{1F792}'), (0xA6, '\u{1F793}'),
    (0xA7, '\u{1F794}'), (0xA8, '\u{25A3}'), (0xA9, '\u{1F795}'),
    (0xAA, '\u{1F796}'), (0xAB, '\u{1F797}'), (0xAC, '\u{2B29}'),
    (0xAD, '\u{2B25}'), (0xAE, '\u{25C6}'), (0xAF, '\u{25C7}'),
    (0xB0, '\u{1F79A}'), (0xB1, '\u{25C8}'), (0xB2, '\u{1F79B}'),
    (0xB3, '\u{1F79C}'), (0xB4, '\u{1F79D}'), (0xB5, '\u{2B2A}'),
    (0xB6, '\u{2B27}'), (0xB7, '\u{29EB}'), (0xB8, '\u{25CA}'),
    (0xB9, '\u{1F7A0}'), (0xBA, '\u{25D6}'), (0xBB, '\u{25D7}'),
    (0xBC, '\u{2BCA}'), (0xBD, '\u{2BCB}'), (0xBE, '\u{25FC}'),
    (0xBF, '\u{2B25}'), (0xC0, '\u{2B1F}'), (0xC1, '\u{2BC2}'),
    (0xC2, '\u{2B23}'), (0xC3, '\u{2B22}'), (0xC4, '\u{2BC3}'),
    (0xC5, '\u{2BC4}'), (0xC6, '\u{1F7A1}'), (0xC7, '\u{1F7A2}'),
    (0xC8, '\u{1F7A3}'), (0xC9, '\u{1F7A4}'), (0xCA, '\u{1F7A5}'),
    (0xCB, '\u{1F7A6}'), (0xCC, '\u{1F7A7}'), (0xCD, '\u{1F7A8}'),
    (0xCE, '\u{1F7A9}'), (0xCF, '\u{1F7AA}'), (0xD0, '\u{1F7AB}'),
    (0xD1, '\u{1F7AC}'), (0xD2, '\u{1F7AD}'), (0xD3, '\u{1F7AE}'),
    (0xD4, '\u{1F7AF}'), (0xD5, '\u{1F7B0}'), (0xD6, '\u{1F7B1}'),
    (0xD7, '\u{1F7B2}'), (0xD8, '\u{1F7B3}'), (0xD9, '\u{1F7B4}'),
    (0xDA, '\u{1F7B5}'), (0xDB, '\u{1F7B6}'), (0xDC, '\u{1F7B7}'),
    (0xDD, '\u{1F7B8}'), (0xDE, '\u{1F7B9}'), (0xDF, '\u{1F7BA}'),
    (0xE0, '\u{1F7BB}'), (0xE1, '\u{1F7BC}'), (0xE2, '\u{1F7BD}'),
    (0xE3, '\u{1F7BE}'), (0xE4, '\u{1F7BF}'), (0xE5, '\u{1F7C0}'),
    (0xE6, '\u{1F7C2}'), (0xE7, '\u{1F7C4}'), (0xE8, '\u{2726}'),
    (0xE9, '\u{1F7C9}'), (0xEA, '\u{2605}'), (0xEB, '\u{2736}'),
    (0xEC, '\u{1F7CB}'), (0xED, '\u{2737}'), (0xEE, '\u{1F7CF}'),
    (0xEF, '\u{1F7D2}'), (0xF0, '\u{2739}'), (0xF1, '\u{1F7C3}'),
    (0xF2, '\u{1F7C7}'), (0xF3, '\u{272F}'), (0xF4, '\u{1F7CD}'),
    (0xF5, '\u{1F7D4}'), (0xF6, '\u{2BCC}'), (0xF7, '\u{2BCD}'),
    (0xF8, '\u{203B}'), (0xF9, '\u{2042}'),
];

// Arrows and brackets.
#[rustfmt::skip]
const WINGDINGS3: &[(u8, char)] = &[
    (0x21, '\u{2B60}'), (0x22, '\u{2B62}'), (0x23, '\u{2B61}'),
    (0x24, '\u{2B63}'), (0x25, '\u{2B66}'), (0x26, '\u{2B67}'),
    (0x27, '\u{2B69}'), (0x28, '\u{2B68}'), (0x29, '\u{2B70}'),
    (0x2A, '\u{2B72}'), (0x2B, '\u{2B71}'), (0x2C, '\u{2B73}'),
    (0x2D, '\u{2B76}'), (0x2E, '\u{2B78}'), (0x2F, '\u{2B7B}'),
    (0x30, '\u{2B7D}'), (0x31, '\u{2B64}'), (0x32, '\u{2B65}'),
    (0x33, '\u{2B6A}'), (0x34, '\u{2B6C}'), (0x35, '\u{2B6B}'),
    (0x36, '\u{2B6D}'), (0x37, '\u{2B4D}'), (0x38, '\u{2BA0}'),
    (0x39, '\u{2BA1}'), (0x3A, '\u{2BA2}'), (0x3B, '\u{2BA3}'),
    (0x3C, '\u{2BA4}'), (0x3D, '\u{2BA5}'), (0x3E, '\u{2BA6}'),
    (0x3F, '\u{2BA7}'), (0x40, '\u{2B90}'), (0x41, '\u{2B91}'),
    (0x42, '\u{2B92}'), (0x43, '\u{2B93}'), (0x44, '\u{2B80}'),
    (0x45, '\u{2B83}'), (0x46, '\u{2B7E}'), (0x47, '\u{2B7F}'),
    (0x48, '\u{2B84}'), (0x49, '\u{2B86}'), (0x4A, '\u{2B85}'),
    (0x4B, '\u{2B87}'), (0x4C, '\u{2B8F}'), (0x4D, '\u{2B8D}'),
    (0x4E, '\u{2B8E}'), (0x4F, '\u{2B8C}'), (0x50, '\u{2B6E}'),
    (0x51, '\u{2B6F}'), (0x52, '\u{238B}'), (0x53, '\u{2324}'),
    (0x54, '\u{2303}'), (0x55, '\u{2325}'), (0x56, '\u{23B5}'),
    (0x57, '\u{237D}'), (0x58, '\u{21EA}'), (0x59, '\u{2BB8}'),
    (0x5A, '\u{1F8A0}'), (0x5B, '\u{1F8A1}'), (0x5C, '\u{1F8A2}'),
    (0x5D, '\u{1F8A3}'), (0x5E, '\u{1F8A4}'), (0x5F, '\u{1F8A5}'),
    (0x60, '\u{1F8A6}'), (0x61, '\u{1F8A7}'), (0x62, '\u{1F8A8}'),
    (0x63, '\u{1F8A9}'), (0x64, '\u{1F8AA}'), (0x65, '\u{1F8AB}'),
    (0x66, '\u{2190}'), (0x67, '\u{2192}'), (0x68, '\u{2191}'),
    (0x69, '\u{2193}'), (0x6A, '\u{2196}'), (0x6B, '\u{2197}'),
    (0x6C, '\u{2199}'), (0x6D, '\u{2198}'), (0x6E, '\u{1F858}'),
    (0x6F, '\u{1F859}'), (0x70, '\u{25B2}'), (0x71, '\u{25BC}'),
    (0x72, '\u{25B3}'), (0x73, '\u{25BD}'), (0x74, '\u{25C4}'),
    (0x75, '\u{25BA}'), (0x76, '\u{25C1}'), (0x77, '\u{25B7}'),
    (0x78, '\u{25E3}'), (0x79, '\u{25E2}'), (0x7A, '\u{25E4}'),
    (0x7B, '\u{25E5}'), (0x7C, '\u{1F780}'), (0x7D, '\u{1F782}'),
    (0x7E, '\u{1F781}'), (0x80, '\u{1F783}'), (0x81, '\u{25B2}'),
    (0x82, '\u{25BC}'), (0x83, '\u{25C0}'), (0x84, '\u{25B6}'),
    (0x85, '\u{2B9C}'), (0x86, '\u{2B9E}'), (0x87, '\u{2B9D}'),
    (0x88, '\u{2B9F}'), (0x89, '\u{1F810}'), (0x8A, '\u{1F812}'),
    (0x8B, '\u{1F811}'), (0x8C, '\u{1F813}'), (0x8D, '\u{1F814}'),
    (0x8E, '\u{1F816}'), (0x8F, '\u{1F815}'), (0x90, '\u{1F817}'),
    (0x91, '\u{1F818}'), (0x92, '\u{1F81A}'), (0x93, '\u{1F819}'),
    (0x94, '\u{1F81B}'), (0x95, '\u{1F81C}'), (0x96, '\u{1F81E}'),
    (0x97, '\u{1F81D}'), (0x98, '\u{1F81F}'), (0x99, '\u{1F800}'),
    (0x9A, '\u{1F802}'), (0x9B, '\u{1F801}'), (0x9C, '\u{1F803}'),
    (0x9D, '\u{1F804}'), (0x9E, '\u{1F806}'), (0x9F, '\u{1F805}'),
    (0xA1, '\u{1F808}'), (0xA2, '\u{1F80A}'), (0xA3, '\u{1F809}'),
    (0xA4, '\u{1F80B}'), (0xA5, '\u{1F820}'), (0xA6, '\u{1F822}'),
    (0xA7, '\u{1F824}'), (0xA8, '\u{1F826}'), (0xA9, '\u{1F828}'),
    (0xAA, '\u{1F82A}'), (0xAB, '\u{1F82C}'), (0xAC, '\u{1F89C}'),
    (0xAD, '\u{1F89D}'), (0xAE, '\u{1F89E}'), (0xAF, '\u{1F89F}'),
    (0xB0, '\u{1F82E}'), (0xB1, '\u{1F830}'), (0xB2, '\u{1F832}'),
    (0xB3, '\u{1F834}'), (0xB4, '\u{1F836}'), (0xB5, '\u{1F838}'),
    (0xB6, '\u{1F83A}'), (0xB7, '\u{1F839}'), (0xB8, '\u{1F83B}'),
    (0xB9, '\u{1F898}'), (0xBA, '\u{1F89A}'), (0xBB, '\u{1F899}'),
    (0xBC, '\u{1F89B}'), (0xBD, '\u{1F83C}'), (0xBE, '\u{1F83E}'),
    (0xBF, '\u{1F83D}'), (0xC0, '\u{1F83F}'), (0xC1, '\u{1F840}'),
    (0xC2, '\u{1F842}'), (0xC3, '\u{1F841}'), (0xC4, '\u{1F843}'),
    (0xC5, '\u{1F844}'), (0xC6, '\u{1F846}'), (0xC7, '\u{1F845}'),
    (0xC8, '\u{1F847}'), (0xC9, '\u{2BA8}'), (0xCA, '\u{2BA9}'),
    (0xCB, '\u{2BAA}'), (0xCC, '\u{2BAB}'), (0xCD, '\u{2BAC}'),
    (0xCE, '\u{2BAD}'), (0xCF, '\u{2BAE}'), (0xD0, '\u{2BAF}'),
    (0xD1, '\u{1F860}'), (0xD2, '\u{1F862}'), (0xD3, '\u{1F861}'),
    (0xD4, '\u{1F863}'), (0xD5, '\u{1F864}'), (0xD6, '\u{1F865}'),
    (0xD7, '\u{1F867}'), (0xD8, '\u{1F866}'), (0xD9, '\u{1F870}'),
    (0xDA, '\u{1F872}'), (0xDB, '\u{1F871}'), (0xDC, '\u{1F873}'),
    (0xDD, '\u{1F874}'), (0xDE, '\u{1F875}'), (0xDF, '\u{1F877}'),
    (0xE0, '\u{1F876}'), (0xE1, '\u{1F880}'), (0xE2, '\u{1F882}'),
    (0xE3, '\u{1F881}'), (0xE4, '\u{1F883}'), (0xE5, '\u{1F884}'),
    (0xE6, '\u{1F885}'), (0xE7, '\u{1F887}'), (0xE8, '\u{1F886}'),
    (0xE9, '\u{1F890}'), (0xEA, '\u{1F892}'), (0xEB, '\u{1F891}'),
    (0xEC, '\u{1F893}'), (0xED, '\u{1F894}'), (0xEE, '\u{1F896}'),
    (0xEF, '\u{1F895}'), (0xF0, '\u{1F897}'),
];

// Small pictograms used for icons and bullets.
#[rustfmt::skip]
const WEBDINGS: &[(u8, char)] = &[
    (0x21, '\u{1F577}'), (0x22, '\u{1F578}'), (0x23, '\u{1F572}'),
    (0x24, '\u{1F576}'), (0x25, '\u{1F3C6}'), (0x26, '\u{1F396}'),
    (0x27, '\u{1F587}'), (0x28, '\u{1F5E8}'), (0x29, '\u{1F5E9}'),
    (0x2A, '\u{1F5F0}'), (0x2B, '\u{1F5F1}'), (0x2C, '\u{1F336}'),
    (0x2D, '\u{1F397}'), (0x2E, '\u{1F67E}'), (0x2F, '\u{1F67C}'),
    (0x30, '\u{1F5D5}'), (0x31, '\u{1F5D6}'), (0x32, '\u{1F5D7}'),
    (0x33, '\u{23F4}'), (0x34, '\u{23F5}'), (0x35, '\u{23F6}'),
    (0x36, '\u{23F7}'), (0x37, '\u{23EA}'), (0x38, '\u{23E9}'),
    (0x39, '\u{23EE}'), (0x3A, '\u{23ED}'), (0x3B, '\u{23F8}'),
    (0x3C, '\u{23F9}'), (0x3D, '\u{23FA}'), (0x3E, '\u{1F5DA}'),
    (0x3F, '\u{1F5F3}'), (0x40, '\u{1F6E0}'), (0x41, '\u{1F3D7}'),
    (0x42, '\u{1F3D8}'), (0x43, '\u{1F3D9}'), (0x44, '\u{1F3DA}'),
    (0x45, '\u{1F3DC}'), (0x46, '\u{1F3ED}'), (0x47, '\u{1F3DB}'),
    (0x48, '\u{1F3E0}'), (0x49, '\u{1F3D6}'), (0x4A, '\u{1F3DD}'),
    (0x4B, '\u{1F6E3}'), (0x4C, '\u{1F50D}'), (0x4D, '\u{1F3D4}'),
    (0x4E, '\u{1F441}'), (0x4F, '\u{1F442}'), (0x50, '\u{1F3DE}'),
    (0x51, '\u{1F3D5}'), (0x52, '\u{1F6E4}'), (0x53, '\u{1F3DF}'),
    (0x54, '\u{1F6F3}'), (0x55, '\u{1F56C}'), (0x56, '\u{1F56B}'),
    (0x57, '\u{1F568}'), (0x58, '\u{1F508}'), (0x59, '\u{1F394}'),
    (0x5A, '\u{1F395}'), (0x5B, '\u{1F5EC}'), (0x5C, '\u{1F67D}'),
    (0x5D, '\u{1F5ED}'), (0x5E, '\u{1F5EA}'), (0x5F, '\u{1F5EB}'),
    (0x60, '\u{2B94}'), (0x61, '\u{2714}'), (0x62, '\u{1F6B2}'),
    (0x63, '\u{25A1}'), (0x64, '\u{1F6E1}'), (0x65, '\u{1F4E6}'),
    (0x66, '\u{1F6F1}'), (0x67, '\u{25A0}'), (0x68, '\u{1F691}'),
    (0x69, '\u{1F6C8}'), (0x6A, '\u{1F6E9}'), (0x6B, '\u{1F6F0}'),
    (0x6C, '\u{1F7C8}'), (0x6D, '\u{1F574}'), (0x6E, '\u{26AB}'),
    (0x6F, '\u{1F6E5}'), (0x70, '\u{1F694}'), (0x71, '\u{1F5D8}'),
    (0x72, '\u{1F5D9}'), (0x73, '\u{2753}'), (0x74, '\u{1F6F2}'),
    (0x75, '\u{1F687}'), (0x76, '\u{1F68D}'), (0x77, '\u{26F3}'),
    (0x78, '\u{1F6C7}'), (0x79, '\u{2296}'), (0x7A, '\u{1F6AD}'),
    (0x7B, '\u{1F5EE}'), (0x7C, '\u{007C}'), (0x7D, '\u{1F5EF}'),
    (0x7E, '\u{1F5F2}'), (0x80, '\u{1F6B9}'), (0x81, '\u{1F6BA}'),
    (0x82, '\u{1F6C9}'), (0x83, '\u{1F6CA}'), (0x84, '\u{1F6BC}'),
    (0x85, '\u{1F47D}'), (0x86, '\u{1F3CB}'), (0x87, '\u{26F7}'),
    (0x88, '\u{1F3C2}'), (0x89, '\u{1F3CC}'), (0x8A, '\u{1F3CA}'),
    (0x8B, '\u{1F3C4}'), (0x8C, '\u{1F3CD}'), (0x8D, '\u{1F3CE}'),
    (0x8E, '\u{1F698}'), (0x8F, '\u{1F5E0}'), (0x90, '\u{1F6E2}'),
    (0x91, '\u{1F4B0}'), (0x92, '\u{1F3F7}'), (0x93, '\u{1F4B3}'),
    (0x94, '\u{1F46A}'), (0x95, '\u{1F5E1}'), (0x96, '\u{1F5E2}'),
    (0x97, '\u{1F5E3}'), (0x98, '\u{272F}'), (0x99, '\u{1F584}'),
    (0x9A, '\u{1F585}'), (0x9B, '\u{1F583}'), (0x9C, '\u{1F586}'),
    (0x9D, '\u{1F5B9}'), (0x9E, '\u{1F5BA}'), (0x9F, '\u{1F5BB}'),
    (0xA1, '\u{1F570}'), (0xA2, '\u{1F5BD}'), (0xA3, '\u{1F5BE}'),
    (0xA4, '\u{1F4CB}'), (0xA5, '\u{1F5D2}'), (0xA6, '\u{1F5D3}'),
    (0xA7, '\u{1F4D6}'), (0xA8, '\u{1F4DA}'), (0xA9, '\u{1F5DE}'),
    (0xAA, '\u{1F5DF}'), (0xAB, '\u{1F5C3}'), (0xAC, '\u{1F5C2}'),
    (0xAD, '\u{1F5BC}'), (0xAE, '\u{1F3AD}'), (0xAF, '\u{1F39C}'),
    (0xB0, '\u{1F398}'), (0xB1, '\u{1F399}'), (0xB2, '\u{1F3A7}'),
    (0xB3, '\u{1F4BF}'), (0xB4, '\u{1F39E}'), (0xB5, '\u{1F4F7}'),
    (0xB6, '\u{1F39F}'), (0xB7, '\u{1F3AC}'), (0xB8, '\u{1F4FD}'),
    (0xB9, '\u{1F4F9}'), (0xBA, '\u{1F4FE}'), (0xBB, '\u{1F4FB}'),
    (0xBC, '\u{1F39A}'), (0xBD, '\u{1F39B}'), (0xBE, '\u{1F4FA}'),
    (0xBF, '\u{1F4BB}'), (0xC0, '\u{1F5A5}'), (0xC1, '\u{1F5A6}'),
    (0xC2, '\u{1F5A7}'), (0xC3, '\u{1F579}'), (0xC4, '\u{1F3AE}'),
    (0xC5, '\u{1F57B}'), (0xC6, '\u{1F57C}'), (0xC7, '\u{1F4DF}'),
    (0xC8, '\u{1F581}'), (0xC9, '\u{1F580}'), (0xCA, '\u{1F5A8}'),
    (0xCB, '\u{1F5A9}'), (0xCC, '\u{1F5BF}'), (0xCD, '\u{1F5AA}'),
    (0xCE, '\u{1F5DC}'), (0xCF, '\u{1F512}'), (0xD0, '\u{1F513}'),
    (0xD1, '\u{1F5DD}'), (0xD2, '\u{1F4E5}'), (0xD3, '\u{1F4E4}'),
    (0xD4, '\u{1F573}'), (0xD5, '\u{1F323}'), (0xD6, '\u{1F324}'),
    (0xD7, '\u{1F325}'), (0xD8, '\u{1F326}'), (0xD9, '\u{2601}'),
    (0xDA, '\u{1F327}'), (0xDB, '\u{1F328}'), (0xDC, '\u{1F329}'),
    (0xDD, '\u{1F32A}'), (0xDE, '\u{1F32C}'), (0xDF, '\u{1F32B}'),
    (0xE0, '\u{1F31C}'), (0xE1, '\u{1F321}'), (0xE2, '\u{1F6CB}'),
    (0xE3, '\u{1F6CF}'), (0xE4, '\u{1F37D}'), (0xE5, '\u{1F378}'),
    (0xE6, '\u{1F6CE}'), (0xE7, '\u{1F6CD}'), (0xE8, '\u{24C5}'),
    (0xE9, '\u{267F}'), (0xEA, '\u{1F6C6}'), (0xEB, '\u{1F588}'),
    (0xEC, '\u{1F393}'), (0xED, '\u{1F5E4}'), (0xEE, '\u{1F5E5}'),
    (0xEF, '\u{1F5E6}'), (0xF0, '\u{1F5E7}'), (0xF1, '\u{1F6EA}'),
    (0xF2, '\u{1F43F}'), (0xF3, '\u{1F426}'), (0xF4, '\u{1F41F}'),
    (0xF5, '\u{1F415}'), (0xF6, '\u{1F408}'), (0xF7, '\u{1F66C}'),
    (0xF8, '\u{1F66E}'), (0xF9, '\u{1F66D}'), (0xFA, '\u{1F66F}'),
    (0xFB, '\u{1F5FA}'), (0xFC, '\u{1F30D}'), (0xFD, '\u{1F30F}'),
    (0xFE, '\u{1F30E}'), (0xFF, '\u{1F54A}'),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wingdings_smiley_becomes_a_colour_face() {
        // The case this all exists for: Word's Wingdings "J" is a smiley.
        assert_eq!(to_char("Wingdings", 0xF04A), Some('\u{1F642}'));
        assert_eq!(to_char("Wingdings", 0x4A), Some('\u{1F642}'));
        assert_eq!(to_char("wingdings", 0x4A), Some('\u{1F642}'));
    }

    #[test]
    fn other_symbol_fonts_are_recognised() {
        assert_eq!(to_char("Symbol", 0xF061), Some('\u{03B1}')); // alpha
        assert_eq!(to_char("Wingdings 2", 0xF04A), Some('\u{1F446}'));
        assert_eq!(to_char("Webdings", 0xF04C), Some('\u{1F50D}')); // magnifier
    }

    #[test]
    fn literal_code_points_pass_through() {
        assert_eq!(to_char("Wingdings", 0x263A), Some('\u{263A}'));
    }

    #[test]
    fn tables_are_sorted_and_unique_for_binary_search() {
        for table in [SYMBOL, WINGDINGS, WINGDINGS2, WINGDINGS3, WEBDINGS] {
            for pair in table.windows(2) {
                assert!(pair[0].0 < pair[1].0, "table is not sorted: {pair:?}");
            }
        }
    }
}
