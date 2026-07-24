//! A `const fn` TTF subsetter for icon fonts.
//!
//! Surviving glyphs are renumbered into a dense `0..N` range, which keeps
//! `loca` and `hmtx` proportional to the subset rather than to the source font
//! — worth about 9 KB. So `glyf`, `loca`, `hmtx`, `cmap` and `GSUB` are all
//! rebuilt, `hhea` and `maxp` are patched with the new glyph count, and only
//! `OS/2` copies verbatim. `gasp`, `name` and `post` are dropped.
//!
//! Renumbering is why composite glyphs are rejected: they embed the ids of
//! their components. Phosphor has none.
//!
//! # Why `GSUB` is kept
//!
//! Phosphor's `GSUB` is a single `liga` lookup implementing "type `gear`, get
//! the gear icon". Its side effect is that every icon glyph is reachable from
//! the Latin letters a-z, which is how skrifa's autohinter classifies icons as
//! Latin and gives them proper blue zones. Drop `GSUB` and icons render
//! noticeably blurrier below ~20px. So the ligatures for the icons we keep are
//! carried over, along with the letter glyphs they are reached from — together
//! about 1.4 KB, and rendering stays pixel-identical to the full font.

const MAX_GLYPHS: usize = 8192;
const KEEP_WORDS: usize = MAX_GLYPHS / 64;
const NTABLES: usize = 9;

/// Output tables, in the ascending-tag order the spec requires.
const TAGS: [&[u8; 4]; NTABLES] = [
    b"GSUB", b"OS/2", b"cmap", b"glyf", b"head", b"hhea", b"hmtx", b"loca", b"maxp",
];
const T_GSUB: usize = 0;
const T_OS2: usize = 1;
const T_CMAP: usize = 2;
const T_GLYF: usize = 3;
const T_HEAD: usize = 4;
const T_HHEA: usize = 5;
const T_HMTX: usize = 6;
const T_LOCA: usize = 7;
const T_MAXP: usize = 8;

// ---------------------------------------------------------------- byte reads

const fn be16(d: &[u8], o: usize) -> u16 {
    (d[o] as u16) << 8 | d[o + 1] as u16
}

const fn be32(d: &[u8], o: usize) -> u32 {
    (d[o] as u32) << 24 | (d[o + 1] as u32) << 16 | (d[o + 2] as u32) << 8 | d[o + 3] as u32
}

/// Codepoint of the first `char` of `s`, so callers can pass the crate's
/// existing `&str` icon constants straight through.
pub const fn cp(s: &str) -> u32 {
    let b = s.as_bytes();
    let b0 = b[0] as u32;
    if b0 < 0x80 {
        b0
    } else if b0 < 0xE0 {
        (b0 & 0x1F) << 6 | (b[1] as u32 & 0x3F)
    } else if b0 < 0xF0 {
        (b0 & 0x0F) << 12 | (b[1] as u32 & 0x3F) << 6 | (b[2] as u32 & 0x3F)
    } else {
        (b0 & 0x07) << 18
            | (b[1] as u32 & 0x3F) << 12
            | (b[2] as u32 & 0x3F) << 6
            | (b[3] as u32 & 0x3F)
    }
}

// ------------------------------------------------------------- source lookup

const fn find_table(src: &[u8], tag: &[u8; 4]) -> (usize, usize) {
    let (o, l) = opt_table(src, tag);
    if o == usize::MAX {
        panic!("egui-phosphor: required table missing from source font");
    }
    (o, l)
}

const fn opt_table(src: &[u8], tag: &[u8; 4]) -> (usize, usize) {
    let num = be16(src, 4) as usize;
    let mut i = 0;
    while i < num {
        let r = 12 + 16 * i;
        if src[r] == tag[0] && src[r + 1] == tag[1] && src[r + 2] == tag[2] && src[r + 3] == tag[3]
        {
            return (be32(src, r + 8) as usize, be32(src, r + 12) as usize);
        }
        i += 1;
    }
    (usize::MAX, 0)
}

const fn loca_at(src: &[u8], loca: usize, long: bool, i: usize) -> usize {
    if long {
        be32(src, loca + 4 * i) as usize
    } else {
        be16(src, loca + 2 * i) as usize * 2
    }
}

/// Offset of the format 4 subtable inside `cmap`, preferring (3,1) then (0,3).
const fn cmap4(src: &[u8], cmap: usize) -> usize {
    let n = be16(src, cmap + 2) as usize;
    let mut best = usize::MAX;
    let mut i = 0;
    while i < n {
        let r = cmap + 4 + 8 * i;
        let plat = be16(src, r);
        let enc = be16(src, r + 2);
        let off = cmap + be32(src, r + 4) as usize;
        if be16(src, off) == 4 {
            if plat == 3 && enc == 1 {
                return off;
            }
            if best == usize::MAX && plat == 0 && enc == 3 {
                best = off;
            }
        }
        i += 1;
    }
    if best == usize::MAX {
        panic!("egui-phosphor: source font has no format 4 cmap subtable");
    }
    best
}

/// Glyph id for codepoint `c` using segment `i`, which must already contain `c`.
const fn gid_in_seg(src: &[u8], sub: usize, seg_count: usize, i: usize, c: u16) -> u16 {
    let end = sub + 14;
    let start = end + seg_count * 2 + 2;
    let delta = start + seg_count * 2;
    let range = delta + seg_count * 2;
    let ro = be16(src, range + 2 * i) as usize;
    if ro == 0 {
        return c.wrapping_add(be16(src, delta + 2 * i));
    }
    let g = be16(
        src,
        range + 2 * i + ro + 2 * (c - be16(src, start + 2 * i)) as usize,
    );
    if g == 0 {
        return 0;
    }
    g.wrapping_add(be16(src, delta + 2 * i))
}

/// Glyph id for `c` via a format 4 subtable, or 0.
const fn lookup(src: &[u8], sub: usize, c: u32) -> u16 {
    if c > 0xFFFF {
        return 0;
    }
    let c = c as u16;
    let seg_count = be16(src, sub + 6) as usize / 2;
    let end = sub + 14;
    let start = end + seg_count * 2 + 2;

    // endCode[] is sorted ascending: binary search for the first segment whose
    // endCode >= c. A linear scan here trips `long_running_const_eval` once a
    // caller asks for a few thousand codepoints.
    let mut lo = 0;
    let mut hi = seg_count;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if be16(src, end + 2 * mid) >= c {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    if lo >= seg_count || be16(src, start + 2 * lo) > c {
        return 0;
    }
    gid_in_seg(src, sub, seg_count, lo, c)
}

// ---------------------------------------------------------------- GSUB reads

/// Offset of the single `LigatureSubst` format 1 subtable, or `usize::MAX`.
const fn lig_subtable(src: &[u8], gsub: usize) -> usize {
    if gsub == usize::MAX {
        return usize::MAX;
    }
    let ll = gsub + be16(src, gsub + 8) as usize;
    let count = be16(src, ll) as usize;
    let mut i = 0;
    while i < count {
        let lo = ll + be16(src, ll + 2 + 2 * i) as usize;
        if be16(src, lo) == 4 && be16(src, lo + 4) >= 1 {
            let so = lo + be16(src, lo + 6) as usize;
            if be16(src, so) == 1 {
                return so;
            }
        }
        i += 1;
    }
    usize::MAX
}

const fn cov_count(src: &[u8], cov: usize) -> usize {
    let fmt = be16(src, cov);
    if fmt == 1 {
        be16(src, cov + 2) as usize
    } else {
        let n = be16(src, cov + 2) as usize;
        let mut total = 0;
        let mut i = 0;
        while i < n {
            let r = cov + 4 + 6 * i;
            total += (be16(src, r + 2) - be16(src, r)) as usize + 1;
            i += 1;
        }
        total
    }
}

/// The `i`-th glyph in coverage-index order.
const fn cov_glyph(src: &[u8], cov: usize, i: usize) -> u16 {
    let fmt = be16(src, cov);
    if fmt == 1 {
        return be16(src, cov + 4 + 2 * i);
    }
    let n = be16(src, cov + 2) as usize;
    let mut seen = 0;
    let mut r = 0;
    while r < n {
        let o = cov + 4 + 6 * r;
        let (s, e) = (be16(src, o), be16(src, o + 2));
        let len = (e - s) as usize + 1;
        if i < seen + len {
            return s + (i - seen) as u16;
        }
        seen += len;
        r += 1;
    }
    0
}

// ---------------------------------------------------------------------- plan

struct Plan {
    keep: [u64; KEEP_WORDS],
    num_glyphs: usize,
    /// Glyph count after renumbering.
    num_kept: usize,
    long_loca: bool,
    seg_count: usize,
    /// Number of first-glyph coverage entries whose set retains a ligature.
    lig_sets: usize,
    off: [usize; NTABLES],
    len: [usize; NTABLES],
    total: usize,
}

const fn is_kept(keep: &[u64; KEEP_WORDS], g: usize) -> bool {
    keep[g / 64] & (1u64 << (g % 64)) != 0
}

/// Renumbered id of `old`: the number of kept glyphs below it.
///
/// Ids are assigned in ascending source order, so relative order is preserved
/// and `GSUB` coverage tables stay sorted without re-sorting. Counting bits
/// costs ~128 iterations and avoids carrying an 8192-entry map through const
/// evaluation.
const fn new_gid(keep: &[u64; KEEP_WORDS], old: usize) -> u16 {
    let mut n = 0u32;
    let w = old / 64;
    let mut i = 0;
    while i < w {
        n += keep[i].count_ones();
        i += 1;
    }
    let bit = old % 64;
    if bit > 0 {
        n += (keep[w] & ((1u64 << bit) - 1)).count_ones();
    }
    n as u16
}

const fn count_kept(keep: &[u64; KEEP_WORDS]) -> usize {
    let mut n = 0u32;
    let mut i = 0;
    while i < KEEP_WORDS {
        n += keep[i].count_ones();
        i += 1;
    }
    n as usize
}

const fn align4(n: usize) -> usize {
    (n + 3) & !3
}

const fn plan(src: &[u8], icons: &[&str]) -> Plan {
    let (head, head_len) = find_table(src, b"head");
    let (maxp, maxp_len) = find_table(src, b"maxp");
    let (cmap, _) = find_table(src, b"cmap");
    let (loca, _) = find_table(src, b"loca");
    let (_, os2_len) = find_table(src, b"OS/2");
    let (_, hhea_len) = find_table(src, b"hhea");
    let (gsub, _) = opt_table(src, b"GSUB");

    let num_glyphs = be16(src, maxp + 4) as usize;
    if num_glyphs > MAX_GLYPHS {
        panic!("egui-phosphor: source font has too many glyphs for the subsetter");
    }
    let src_long = be16(src, head + 50) == 1;
    let sub = cmap4(src, cmap);

    // 1. Requested codepoints -> kept glyph ids. .notdef is always kept.
    let mut keep = [0u64; KEEP_WORDS];
    keep[0] |= 1;
    let mut i = 0;
    while i < icons.len() {
        let g = lookup(src, sub, cp(icons[i])) as usize;
        if g != 0 && g < num_glyphs {
            keep[g / 64] |= 1u64 << (g % 64);
        }
        i += 1;
    }

    // 2. Keep every ASCII-mapped glyph. These are the letters the `liga`
    //    ligatures start from, and the reason the autohinter treats the icons
    //    as Latin. ~30 simple glyphs, about 550 bytes.
    let seg_count_src = be16(src, sub + 6) as usize / 2;
    let s_end = sub + 14;
    let s_start = s_end + seg_count_src * 2 + 2;
    let mut i = 0;
    while i < seg_count_src {
        let ss = be16(src, s_start + 2 * i) as u32;
        let se = be16(src, s_end + 2 * i) as u32;
        if ss == 0xFFFF {
            break;
        }
        if ss < 0x80 {
            let mut c = ss;
            while c <= se && c < 0x80 {
                let g = gid_in_seg(src, sub, seg_count_src, i, c as u16) as usize;
                if g != 0 && g < num_glyphs {
                    keep[g / 64] |= 1u64 << (g % 64);
                }
                c += 1;
            }
        }
        i += 1;
    }

    // 3. glyf: kept records copied, each padded to 4 bytes.
    let (glyf, _) = find_table(src, b"glyf");
    let mut glyf_len = 0;
    let mut g = 0;
    while g < num_glyphs {
        if is_kept(&keep, g) {
            let s = loca_at(src, loca, src_long, g);
            let e = loca_at(src, loca, src_long, g + 1);
            if e > s {
                // Composite glyphs embed the ids of their components, which
                // renumbering would invalidate. Phosphor has none; refuse
                // rather than emit a silently broken outline.
                if be16(src, glyf + s) >= 0x8000 {
                    panic!("egui-phosphor: composite glyphs are not supported");
                }
                glyf_len += align4(e - s);
            }
        }
        g += 1;
    }

    // Glyphs are renumbered, so loca and hmtx cover only what survived.
    let num_kept = count_kept(&keep);

    // Short loca stores offset/2 in a u16, so it caps out at 131070 bytes.
    let long_loca = glyf_len > 0xFFFF * 2;
    let loca_len = if long_loca {
        4 * (num_kept + 1)
    } else {
        2 * (num_kept + 1)
    };

    let seg_count = count_segments(src, sub, &keep, num_glyphs) + 1; // + 0xFFFF terminator
    let cmap_len = 12 + 16 + 8 * seg_count;

    let (lig_sets, gsub_len) = gsub_size(src, lig_subtable(src, gsub), &keep);

    let mut len = [0usize; NTABLES];
    len[T_GSUB] = gsub_len;
    len[T_OS2] = os2_len;
    len[T_CMAP] = cmap_len;
    len[T_GLYF] = glyf_len;
    len[T_HEAD] = head_len;
    len[T_HHEA] = hhea_len;
    len[T_HMTX] = 4 * num_kept; // every glyph gets a long metric
    len[T_LOCA] = loca_len;
    len[T_MAXP] = maxp_len;

    let mut off = [0usize; NTABLES];
    let mut pos = 12 + 16 * NTABLES;
    let mut t = 0;
    while t < NTABLES {
        off[t] = pos;
        pos += align4(len[t]);
        t += 1;
    }

    Plan {
        keep,
        num_glyphs,
        num_kept,
        long_loca,
        seg_count,
        lig_sets,
        off,
        len,
        total: pos,
    }
}

/// Merged format 4 segments needed for every kept glyph reachable from `cmap`.
const fn count_segments(src: &[u8], sub: usize, keep: &[u64; KEEP_WORDS], ng: usize) -> usize {
    let mut segs = 0;
    let mut prev_c = 0u32;
    let mut prev_g = 0u32;
    let mut open = false;
    let seg_count = be16(src, sub + 6) as usize / 2;
    let end = sub + 14;
    let start = end + seg_count * 2 + 2;

    let mut i = 0;
    while i < seg_count {
        let s = be16(src, start + 2 * i) as u32;
        let e = be16(src, end + 2 * i) as u32;
        if s == 0xFFFF {
            break;
        }
        let mut c = s;
        while c <= e {
            let g = gid_in_seg(src, sub, seg_count, i, c as u16) as usize;
            if g != 0 && g < ng && is_kept(keep, g) {
                let g = new_gid(keep, g) as usize;
                if open && c == prev_c + 1 && g as u32 == prev_g + 1 {
                    // extends the current run
                } else {
                    segs += 1;
                }
                prev_c = c;
                prev_g = g as u32;
                open = true;
            }
            c += 1;
        }
        i += 1;
    }
    segs
}

/// Whether a ligature's output glyph and every component glyph survived.
/// After renumbering a dangling reference would point at the wrong glyph.
const fn lig_kept(src: &[u8], lg: usize, keep: &[u64; KEEP_WORDS]) -> bool {
    let lig_g = be16(src, lg) as usize;
    if lig_g >= MAX_GLYPHS || !is_kept(keep, lig_g) {
        return false;
    }
    let comp_c = be16(src, lg + 2) as usize;
    let mut i = 1;
    while i < comp_c {
        let c = be16(src, lg + 2 + 2 * i) as usize;
        if c >= MAX_GLYPHS || !is_kept(keep, c) {
            return false;
        }
        i += 1;
    }
    true
}

/// Bytes in one rebuilt `LigatureSet`, and how many ligatures it retains.
const fn lig_set_size(src: &[u8], lso: usize, keep: &[u64; KEEP_WORDS]) -> (usize, usize) {
    let n = be16(src, lso) as usize;
    let mut kept = 0;
    let mut body = 0;
    let mut j = 0;
    while j < n {
        let l = lso + be16(src, lso + 2 + 2 * j) as usize;
        if lig_kept(src, l, keep) {
            kept += 1;
            body += 4 + 2 * (be16(src, l + 2) as usize - 1);
        }
        j += 1;
    }
    if kept == 0 {
        return (0, 0);
    }
    (kept, 2 + 2 * kept + body)
}

/// `(retained coverage entries, total GSUB size)`. Size 0 means "emit no GSUB".
const fn gsub_size(src: &[u8], so: usize, keep: &[u64; KEEP_WORDS]) -> (usize, usize) {
    if so == usize::MAX {
        return (0, 0);
    }
    let cov = so + be16(src, so + 2) as usize;
    let n_sets = be16(src, so + 4) as usize;
    let n_cov = cov_count(src, cov);

    let mut sets = 0;
    let mut sets_bytes = 0;
    let mut i = 0;
    while i < n_sets && i < n_cov {
        let lso = so + be16(src, so + 6 + 2 * i) as usize;
        let (kept, bytes) = lig_set_size(src, lso, keep);
        if kept > 0 {
            sets += 1;
            sets_bytes += bytes;
        }
        i += 1;
    }
    if sets == 0 {
        return (0, 0);
    }
    let cov_bytes = 4 + 2 * sets;
    let subtable = 6 + 2 * sets + cov_bytes + sets_bytes;
    // header + ScriptList(20) + FeatureList(14) + LookupList(4 + 8 + subtable)
    (sets, 10 + 20 + 14 + 4 + 8 + subtable)
}

// ---------------------------------------------------------------------- emit

/// Size in bytes of the font [`subset_into`] would build from `icons`.
///
/// `icons` are the crate's icon constants, e.g. `regular::GEAR`.
pub const fn subset_len(src: &[u8], icons: &[&str]) -> usize {
    plan(src, icons).total
}

/// Build a font containing only `icons`. `N` must be [`subset_len`] for the
/// same arguments.
pub const fn subset_into<const N: usize>(src: &[u8], icons: &[&str]) -> [u8; N] {
    let p = plan(src, icons);
    if p.total != N {
        panic!("egui-phosphor: subset buffer size mismatch");
    }
    let mut out = [0u8; N];
    emit(src, &p, &mut out);
    out
}

const fn put16(out: &mut [u8], o: usize, v: u16) {
    out[o] = (v >> 8) as u8;
    out[o + 1] = v as u8;
}

const fn put32(out: &mut [u8], o: usize, v: u32) {
    out[o] = (v >> 24) as u8;
    out[o + 1] = (v >> 16) as u8;
    out[o + 2] = (v >> 8) as u8;
    out[o + 3] = v as u8;
}

const fn copy(out: &mut [u8], dst: usize, src: &[u8], s: usize, n: usize) {
    let mut i = 0;
    while i < n {
        out[dst + i] = src[s + i];
        i += 1;
    }
}

const fn emit(src: &[u8], p: &Plan, out: &mut [u8]) {
    let (head, head_len) = find_table(src, b"head");
    let (maxp, maxp_len) = find_table(src, b"maxp");
    let (cmap, _) = find_table(src, b"cmap");
    let (loca, _) = find_table(src, b"loca");
    let (glyf, _) = find_table(src, b"glyf");
    let (os2, os2_len) = find_table(src, b"OS/2");
    let (hhea, hhea_len) = find_table(src, b"hhea");
    let (hmtx, _) = find_table(src, b"hmtx");
    let src_long = be16(src, head + 50) == 1;
    let sub = cmap4(src, cmap);

    // --- offset table. GSUB may be absent, in which case its record is skipped.
    let n_tables = if p.len[T_GSUB] == 0 {
        NTABLES - 1
    } else {
        NTABLES
    };
    let mut p2 = 1;
    while p2 * 2 <= n_tables {
        p2 *= 2;
    }
    let mut es = 0;
    let mut q = p2;
    while q > 1 {
        q /= 2;
        es += 1;
    }
    put32(out, 0, 0x0001_0000);
    put16(out, 4, n_tables as u16);
    put16(out, 6, (p2 * 16) as u16);
    put16(out, 8, es as u16);
    put16(out, 10, (n_tables * 16 - p2 * 16) as u16);

    let mut t = 0;
    let mut rec = 0;
    while t < NTABLES {
        if p.len[t] > 0 {
            let r = 12 + 16 * rec;
            out[r] = TAGS[t][0];
            out[r + 1] = TAGS[t][1];
            out[r + 2] = TAGS[t][2];
            out[r + 3] = TAGS[t][3];
            put32(out, r + 4, 0); // checksum: not validated by skrifa
            put32(out, r + 8, p.off[t] as u32);
            put32(out, r + 12, p.len[t] as u32);
            rec += 1;
        }
        t += 1;
    }

    // --- OS/2 is glyph-id free and copies verbatim
    copy(out, p.off[T_OS2], src, os2, os2_len);

    // --- hhea and maxp carry glyph counts that renumbering changes
    copy(out, p.off[T_HHEA], src, hhea, hhea_len);
    put16(out, p.off[T_HHEA] + 34, p.num_kept as u16); // numberOfHMetrics
    copy(out, p.off[T_MAXP], src, maxp, maxp_len);
    put16(out, p.off[T_MAXP] + 4, p.num_kept as u16); // numGlyphs

    // --- hmtx, rebuilt in new-id order as all-long metrics
    let src_nhm = be16(src, hhea + 34) as usize;
    let mo = p.off[T_HMTX];
    let mut g = 0;
    while g < p.num_glyphs {
        if is_kept(&p.keep, g) {
            let n = new_gid(&p.keep, g) as usize;
            // Past numberOfHMetrics the advance repeats the last entry and the
            // side bearings continue in a trailing array.
            let (adv, lsb) = if g < src_nhm {
                (be16(src, hmtx + 4 * g), be16(src, hmtx + 4 * g + 2))
            } else {
                (
                    be16(src, hmtx + 4 * (src_nhm - 1)),
                    be16(src, hmtx + 4 * src_nhm + 2 * (g - src_nhm)),
                )
            };
            put16(out, mo + 4 * n, adv);
            put16(out, mo + 4 * n + 2, lsb);
        }
        g += 1;
    }

    // --- head, with indexToLocFormat patched to match the new loca
    copy(out, p.off[T_HEAD], src, head, head_len);
    put32(out, p.off[T_HEAD] + 8, 0); // checkSumAdjustment
    put16(out, p.off[T_HEAD] + 50, if p.long_loca { 1 } else { 0 });

    // --- glyf + loca, indexed by new id
    let go = p.off[T_GLYF];
    let lo = p.off[T_LOCA];
    let mut cur = 0usize;
    let mut n = 0usize;
    let mut g = 0;
    while g < p.num_glyphs {
        if is_kept(&p.keep, g) {
            if p.long_loca {
                put32(out, lo + 4 * n, cur as u32);
            } else {
                put16(out, lo + 2 * n, (cur / 2) as u16);
            }
            let s = loca_at(src, loca, src_long, g);
            let e = loca_at(src, loca, src_long, g + 1);
            if e > s {
                copy(out, go + cur, src, glyf + s, e - s);
                cur += align4(e - s);
            }
            n += 1;
        }
        g += 1;
    }
    if p.long_loca {
        put32(out, lo + 4 * n, cur as u32);
    } else {
        put16(out, lo + 2 * n, (cur / 2) as u16);
    }

    emit_cmap(src, sub, p, out);
    if p.len[T_GSUB] > 0 {
        let (gsub, _) = opt_table(src, b"GSUB");
        emit_gsub(src, lig_subtable(src, gsub), p, out);
    }
}

const fn emit_cmap(src: &[u8], sub: usize, p: &Plan, out: &mut [u8]) {
    let co = p.off[T_CMAP];
    put16(out, co, 0);
    put16(out, co + 2, 1);
    put16(out, co + 4, 3);
    put16(out, co + 6, 1);
    put32(out, co + 8, 12);

    let f = co + 12;
    let n = p.seg_count;
    put16(out, f, 4);
    put16(out, f + 2, (16 + 8 * n) as u16);
    put16(out, f + 4, 0);
    put16(out, f + 6, (n * 2) as u16);
    let mut p2 = 1;
    while p2 * 2 <= n {
        p2 *= 2;
    }
    put16(out, f + 8, (p2 * 2) as u16);
    let mut es = 0;
    let mut q = p2;
    while q > 1 {
        q /= 2;
        es += 1;
    }
    put16(out, f + 10, es as u16);
    put16(out, f + 12, (n * 2 - p2 * 2) as u16);

    let end_a = f + 14;
    let start_a = end_a + n * 2 + 2;
    let arrays = SegArrays {
        end: end_a,
        start: start_a,
        delta: start_a + n * 2,
        range: start_a + n * 4,
    };
    put16(out, end_a + n * 2, 0); // reservedPad

    // Re-walk the source cmap, emitting merged segments in codepoint order.
    let mut seg = 0;
    let mut run_start = 0u32;
    let mut run_start_g = 0u32;
    let mut prev_c = 0u32;
    let mut prev_g = 0u32;
    let mut open = false;

    let src_segs = be16(src, sub + 6) as usize / 2;
    let s_end = sub + 14;
    let s_start = s_end + src_segs * 2 + 2;
    let mut i = 0;
    while i < src_segs {
        let ss = be16(src, s_start + 2 * i) as u32;
        let se = be16(src, s_end + 2 * i) as u32;
        if ss == 0xFFFF {
            break;
        }
        let mut c = ss;
        while c <= se {
            let gid = gid_in_seg(src, sub, src_segs, i, c as u16) as usize;
            if gid != 0 && gid < p.num_glyphs && is_kept(&p.keep, gid) {
                let gid = new_gid(&p.keep, gid) as usize;
                if open && c == prev_c + 1 && gid as u32 == prev_g + 1 {
                    // extend
                } else {
                    if open {
                        arrays.write(out, seg, run_start, prev_c, run_start_g);
                        seg += 1;
                    }
                    run_start = c;
                    run_start_g = gid as u32;
                }
                prev_c = c;
                prev_g = gid as u32;
                open = true;
            }
            c += 1;
        }
        i += 1;
    }
    if open {
        arrays.write(out, seg, run_start, prev_c, run_start_g);
        seg += 1;
    }
    put16(out, arrays.end + 2 * seg, 0xFFFF);
    put16(out, arrays.start + 2 * seg, 0xFFFF);
    put16(out, arrays.delta + 2 * seg, 1);
    put16(out, arrays.range + 2 * seg, 0);
}

/// Base offsets of the four parallel arrays in a format 4 subtable.
struct SegArrays {
    end: usize,
    start: usize,
    delta: usize,
    range: usize,
}

impl SegArrays {
    /// idDelta is `gid - codepoint` mod 65536, and glyph ids are preserved, so
    /// a run has a constant delta and idRangeOffset is always 0.
    const fn write(&self, out: &mut [u8], seg: usize, start_c: u32, end_c: u32, start_g: u32) {
        put16(out, self.end + 2 * seg, end_c as u16);
        put16(out, self.start + 2 * seg, start_c as u16);
        put16(
            out,
            self.delta + 2 * seg,
            (start_g as u16).wrapping_sub(start_c as u16),
        );
        put16(out, self.range + 2 * seg, 0);
    }
}

/// Rebuild `GSUB` as one DFLT script / `liga` feature / LigatureSubst lookup
/// holding only the ligatures whose output glyph survived.
const fn emit_gsub(src: &[u8], so: usize, p: &Plan, out: &mut [u8]) {
    let base = p.off[T_GSUB];
    let sets = p.lig_sets;

    let script_o = 10;
    let feature_o = script_o + 20;
    let lookup_o = feature_o + 14;
    put16(out, base, 1);
    put16(out, base + 2, 0);
    put16(out, base + 4, script_o as u16);
    put16(out, base + 6, feature_o as u16);
    put16(out, base + 8, lookup_o as u16);

    // ScriptList: 1 record, DFLT, default LangSys referencing feature 0.
    let s = base + script_o;
    put16(out, s, 1);
    out[s + 2] = b'D';
    out[s + 3] = b'F';
    out[s + 4] = b'L';
    out[s + 5] = b'T';
    put16(out, s + 6, 8);
    put16(out, s + 8, 4); // defaultLangSysOffset
    put16(out, s + 10, 0); // langSysCount
    put16(out, s + 12, 0); // lookupOrderOffset
    put16(out, s + 14, 0xFFFF); // requiredFeatureIndex
    put16(out, s + 16, 1); // featureIndexCount
    put16(out, s + 18, 0);

    // FeatureList: 1 record, `liga`, referencing lookup 0.
    let f = base + feature_o;
    put16(out, f, 1);
    out[f + 2] = b'l';
    out[f + 3] = b'i';
    out[f + 4] = b'g';
    out[f + 5] = b'a';
    put16(out, f + 6, 8);
    put16(out, f + 8, 0); // featureParamsOffset
    put16(out, f + 10, 1); // lookupIndexCount
    put16(out, f + 12, 0);

    // LookupList: 1 lookup, type 4, 1 subtable.
    let l = base + lookup_o;
    put16(out, l, 1);
    put16(out, l + 2, 4);
    let lk = l + 4;
    put16(out, lk, 4); // lookupType = LigatureSubst
    put16(out, lk + 2, 0); // lookupFlag
    put16(out, lk + 4, 1); // subTableCount
    put16(out, lk + 6, 8);

    // LigatureSubst format 1.
    let st = lk + 8;
    let cov_off = 6 + 2 * sets;
    put16(out, st, 1);
    put16(out, st + 2, cov_off as u16);
    put16(out, st + 4, sets as u16);

    let cov = st + cov_off;
    put16(out, cov, 1);
    put16(out, cov + 2, sets as u16);

    // Walk the source sets in coverage order, copying surviving ligatures.
    let src_cov = so + be16(src, so + 2) as usize;
    let n_sets = be16(src, so + 4) as usize;
    let n_cov = cov_count(src, src_cov);
    let mut body = cov_off + 4 + 2 * sets; // first LigatureSet, relative to `st`
    let mut k = 0;
    let mut i = 0;
    while i < n_sets && i < n_cov {
        let lso = so + be16(src, so + 6 + 2 * i) as usize;
        let (kept, bytes) = lig_set_size(src, lso, &p.keep);
        if kept > 0 {
            put16(
                out,
                cov + 4 + 2 * k,
                new_gid(&p.keep, cov_glyph(src, src_cov, i) as usize),
            );
            put16(out, st + 6 + 2 * k, body as u16);

            let dst = st + body;
            put16(out, dst, kept as u16);
            let mut w = 2 + 2 * kept; // running offset within this LigatureSet
            let mut m = 0;
            let n = be16(src, lso) as usize;
            let mut j = 0;
            while j < n {
                let lg = lso + be16(src, lso + 2 + 2 * j) as usize;
                let comp_c = be16(src, lg + 2) as usize;
                if lig_kept(src, lg, &p.keep) {
                    put16(out, dst + 2 + 2 * m, w as u16);
                    put16(out, dst + w, new_gid(&p.keep, be16(src, lg) as usize));
                    put16(out, dst + w + 2, comp_c as u16);
                    let mut c = 1;
                    while c < comp_c {
                        let old = be16(src, lg + 2 + 2 * c) as usize;
                        put16(out, dst + w + 2 + 2 * c, new_gid(&p.keep, old));
                        c += 1;
                    }
                    w += 4 + 2 * (comp_c - 1);
                    m += 1;
                }
                j += 1;
            }
            body += bytes;
            k += 1;
        }
        i += 1;
    }
}

// -------------------------------------------------------------------- macro

/// Build a font containing only the icons you name.
///
/// The subset is computed during const evaluation, so the full ~490 KB source
/// font never reaches your binary — only the glyphs you asked for do. There is
/// no build script and no proc macro involved.
///
/// Each `use` line becomes a submodule holding that variant's icons and its own
/// font:
///
/// ```
/// egui_phosphor::subset! {
///     /// Icons used by this app.
///     pub mod icons {
///         use regular::{GEAR, HOUSE, TRASH};
///     }
/// }
///
/// # fn demo(fonts: &mut egui::FontDefinitions) {
/// icons::regular::add_to_fonts(fonts);
/// let label = format!("{} Settings", icons::regular::GEAR);
/// # let _ = label;
/// # }
/// ```
///
/// # Using more than one variant
///
/// All variants share codepoints, so `regular::GEAR` and `fill::GEAR` are the
/// same `&str`, and proportional text can only resolve it to one of them. Add
/// the first with [`add_to_fonts`] and the rest with [`add_as_family`], then
/// select those by family at the call site.
///
/// ```
/// egui_phosphor::subset! {
///     pub mod icons {
///         use regular::{GEAR, HOUSE};
///         use fill::{GEAR, HEART};
///     }
/// }
///
/// # fn demo(fonts: &mut egui::FontDefinitions) -> egui::RichText {
/// icons::regular::add_to_fonts(fonts);  // inline with ordinary text
/// icons::fill::add_as_family(fonts);    // selected explicitly
///
/// egui::RichText::new(icons::fill::GEAR).family(icons::fill::family())
/// # }
/// ```
///
/// [`add_to_fonts`]: crate::add_font_bytes_to_fonts
/// [`add_as_family`]: crate::add_font_bytes_as_family
#[macro_export]
macro_rules! subset {
    (
        $(#[$attr:meta])* $vis:vis mod $name:ident {
            $(use $variant:ident::{$($icon:ident),* $(,)?};)*
        }
    ) => {
        $(#[$attr])*
        $vis mod $name {
            $(
                pub mod $variant {
                    pub use $crate::variants::$variant::{$($icon),*};

                    const ICONS: &[&str] = &[$($crate::variants::$variant::$icon),*];
                    const SOURCE: &[u8] = $crate::variants::bytes::$variant::FONT;

                    /// Size of [`FONT`], for naming its type in const contexts.
                    pub const FONT_LEN: usize = $crate::subset::subset_len(SOURCE, ICONS);

                    /// The subsetted font, containing only the icons named
                    /// above. A complete, valid TTF.
                    pub static FONT: [u8; FONT_LEN] =
                        $crate::subset::subset_into::<FONT_LEN>(SOURCE, ICONS);

                    /// The name this font is registered under, unique per
                    /// module and variant so several subsets can coexist.
                    pub const FONT_NAME: &str =
                        concat!(stringify!($name), "-", stringify!($variant));

                    /// The family these icons are guaranteed to render from,
                    /// whichever other variants are also loaded.
                    pub fn family() -> $crate::egui::FontFamily {
                        $crate::egui::FontFamily::Name(FONT_NAME.into())
                    }

                    /// Add this subset as a fallback for proportional text, so
                    /// its icons can be used inline in ordinary labels, and
                    /// register [`family`].
                    pub fn add_to_fonts(fonts: &mut $crate::egui::FontDefinitions) {
                        $crate::add_font_bytes_to_fonts(fonts, FONT_NAME, &FONT);
                    }

                    /// Register [`family`] only, leaving the proportional fonts
                    /// untouched. Use this for every variant after the first.
                    pub fn add_as_family(fonts: &mut $crate::egui::FontDefinitions) {
                        $crate::add_font_bytes_as_family(fonts, FONT_NAME, &FONT);
                    }
                }
            )*
        }
    };
}
