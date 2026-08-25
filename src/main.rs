#![allow(dead_code, unused_variables)]

use usvg::tiny_skia_path::{Path, PathBuilder, PathSegment, PathVerb, Point};

mod spacing {
    /// Maximum height for a vowel without an onglide
    pub const VOWEL_MAX_HEIGHT: i32 = 1276;
    /// Height of the 'y-' onglide glyph (54 spacing)
    pub const Y_ONGLIDE_HEIGHT: i32 = 486;
    /// Height of the 'u-' onglide glyph (540 without overlap)
    pub const U_ONGLIDE_HEIGHT: i32 = 720;
    /// Maximum height (including spacing) a glyph can be underneath the capline
    pub const MAX_HEIGHT_UNDER_CAPLINE: i32 = 1880;
    /// Maximum height for onglide glyphs (not including overlap)
    pub const MAX_ONGLIDE_HEIGHT: i32 = 604;
    /// Distance between glpyh and onglide
    pub const Y_ONGLIDE_DISTANCE_ABOVE: i32 = 60 + 12;
    /// Distance between glpyh and onglide
    pub const U_ONGLIDE_DISTANCE_ABOVE: i32 = -180 + 12;
}

// Private use area order font list
const CONSONANT_LIST: &[&str] = &[
    "l", "p", "f", "th", "c", "t", "s", "sh", "r", "k", "x", "lh", "m", "b",
    "v", "w", "n", "d", "z", "zh", "q", "g", "nh", "rh",
];

// Font order (x / y swapped from unicode private use order)
const CONSONANT_LIST_INDICES: &[usize] = &[
    0, 4, 8, 12, 16, 20, //
    1, 5, 9, 13, 17, 21, //
    2, 6, 10, 14, 18, 22, //
    3, 7, 11, 15, 19, 23, //
];

// Vowel list (same in font and unicode private use)
const VOWEL_LIST: &[&str] = &[
    "yh", "ae", "ih", "iy", "ah", "ia", "eh", "ea", "uh", "ou", "oh", "io",
    "ay", "ai", "ey", "ei", "oy", "oi", "iu", "au", "ao", "eu", "eo", "oa",
    "yie", "yae", "yih", "yiy", "yah", "yia", "yeh", "yea", "yuh", "you",
    "yoh", "yio", "yay", "yai", "yey", "yei", "yoy", "yoi", "yiu", "yau",
    "yao", "yeu", "yeo", "yoa", "uyh", "uae", "uih", "uiy", "uah", "uia",
    "ueh", "uea", "uoe", "uou", "uoh", "uio", "uay", "uai", "uey", "uei",
    "uoy", "uoi", "uiu", "uau", "uao", "ueu", "ueo", "uoa",
];

// Offset to move onglide to (0, 0) with spacing
const ONGLIDE_Y: &[i32] = &[
    -695 + spacing::Y_ONGLIDE_DISTANCE_ABOVE, // y
    -578 + spacing::U_ONGLIDE_DISTANCE_ABOVE, // u
];

// Vowel y positions (used to position onglides)
const VOWEL_Y: &[i32] = &[
    72 + 2048 - ((239 - 82) + 747),     // yh
    72 + 2048 - ((239 - 82) + 604),     // ae
    72 + 2048 - ((239 - 82) + 747),     // ih
    72 + 2048 - ((239 - 82) + 747),     // ay
    72 + 2048 - ((239 - 82) + 604),     // ah
    72 + 2048 - ((239 - 82) + 702),     // ia
    72 + 2048 - ((239 - 82) + 747),     // eh
    72 + 2048 - ((239 - 82) + 702),     // ea
    72 + 2048 - ((239 - 82) + 661 + 4), // uh
    72 + 2048 - ((239 - 82) + 428 + 4), // ou
    72 + 2048 - ((239 - 82) + 792),     // oh
    72 + 2048 - ((239 - 82) + 559),     // io
    72 + 2048 - ((239 - 82) + 559),     // ay
    72 + 2048 - ((239 - 82) + 702),     // ai
    72 + 2048 - ((239 - 82) + 559),     // ey
    72 + 2048 - ((239 - 82) + 702),     // ei
    72 + 2048 - ((239 - 82) + 616),     // oy
    72 + 2048 - ((239 - 82) + 383),     // oi
    72 + 2048 - ((239 - 82) + 428 + 4), // iu
    72 + 2048 - ((239 - 82) + 428 + 4), // au
    72 + 2048 - ((239 - 82) + 383 + 4), // ao
    72 + 2048 - ((239 - 82) + 428 + 4), // eu
    72 + 2048 - ((239 - 82) + 616 + 4), // eo
    72 + 2048 - ((239 - 82) + 428 + 4), // oa
];

// The Y offset to subtract from the vowel (added height / 2)
const ONGLIDE_OFFSET: i32 = 300;

/// Replace `Close` with `LineTo` back to the last `MoveTo` position
fn replace_close_segments(path: &Path) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut start = None;
    let mut last = None;

    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(point) => {
                start = Some(point);
                last = Some(point);
                builder.move_to(point.x, point.y);
            }
            PathSegment::LineTo(point) => {
                last = Some(point);
                builder.line_to(point.x, point.y);
            }
            PathSegment::QuadTo(pt1, pt2) => {
                last = Some(pt2);
                builder.quad_to(pt1.x, pt1.y, pt2.x, pt2.y);
            }
            PathSegment::CubicTo(pt1, pt2, pt3) => {
                last = Some(pt3);
                builder.cubic_to(pt1.x, pt1.y, pt2.x, pt2.y, pt3.x, pt3.y);
            }
            PathSegment::Close => {
                let point = start.expect("no start");

                if start != last {
                    builder.line_to(point.x, point.y);
                }
            }
        }
    }

    builder.finish()
}

fn reverse_path_segments(path: &Path) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut points = path.points().iter().rev();

    // The first point
    let Some(first_point) = points.next() else {
        return builder.finish();
    };

    assert_eq!(path.verbs()[0], PathVerb::Move);
    builder.move_to(first_point.x, first_point.y);

    let mut last_point = first_point;
    let mut last_move = first_point;
    let mut has_moved = false;

    for verb in path.verbs()[1..].iter().rev() {
        match verb {
            PathVerb::Move => {
                if has_moved && last_move != last_point {
                    builder.line_to(last_move.x, last_move.y);
                }

                let pt = points.next()?;

                last_move = pt;
                has_moved = true;
                builder.move_to(pt.x, pt.y);
                last_point = pt;
            }
            PathVerb::Line => {
                let pt = points.next()?;

                builder.line_to(pt.x, pt.y);
                last_point = pt;
            }
            PathVerb::Quad => {
                let pt1 = points.next()?;
                let pt2 = points.next()?;

                builder.quad_to(pt1.x, pt1.y, pt2.x, pt2.y);
                last_point = pt2;
            }
            PathVerb::Cubic => {
                let pt1 = points.next()?;
                let pt2 = points.next()?;
                let pt3 = points.next()?;

                builder.cubic_to(pt1.x, pt1.y, pt2.x, pt2.y, pt3.x, pt3.y);
                last_point = pt3;
            }
            PathVerb::Close => {}
        }
    }

    if last_move != last_point {
        builder.line_to(last_move.x, last_move.y);
    }

    builder.finish()
}

fn read(glyph_name: &str) -> String {
    let glyph_path = glyph_name.split('.').collect::<Vec<_>>().join("/");
    let svg_path = format!("./{glyph_path}.svg");
    let input_svg = std::fs::read_to_string(&svg_path)
        .unwrap_or_else(|e| panic!("Failed to open {svg_path}: {e}"));
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(&input_svg, &opt).unwrap();
    let output_svg = tree.to_string(&Default::default());

    if input_svg != output_svg {
        eprintln!("Formatting {svg_path}");
        std::fs::write(svg_path, output_svg).unwrap();
    }

    let nodes = tree.root().children();
    let mut output = String::new();

    for node in nodes {
        match node {
            usvg::Node::Path(path) => {
                let Some(path) = replace_close_segments(path.data()) else {
                    panic!("failed to contruct path without close")
                };
                let Some(path) = reverse_path_segments(&path) else {
                    panic!("Failed to reverse path")
                };
                let segments: Vec<_> = path.segments().collect();
                let mut last_position = None;

                // Render segments
                for segment in segments.iter() {
                    match segment {
                        PathSegment::MoveTo(Point { x, y }) => {
                            let x = x.round();
                            let y = 1628.0 - y.round();

                            last_position = Some((x, y));
                            output.push_str(&format!("{x} {y} m 0\n"));
                        }
                        PathSegment::LineTo(Point { x, y }) => {
                            let x = x.round();
                            let y = 1628.0 - y.round();

                            if last_position != Some((x, y)) {
                                last_position = Some((x, y));
                                output.push_str(&format!("{x} {y} l 0\n"));
                            }
                        }
                        PathSegment::QuadTo(
                            Point { x: _x1, y: _y1 },
                            Point { x: _x2, y: _y2 },
                        ) => unimplemented!("quad"),
                        PathSegment::CubicTo(
                            Point { x: x1, y: y1 },
                            Point { x: x2, y: y2 },
                            Point { x: x3, y: y3 },
                        ) => {
                            let x1 = x1.round();
                            let x2 = x2.round();
                            let x3 = x3.round();
                            let y1 = 1628.0 - y1.round();
                            let y2 = 1628.0 - y2.round();
                            let y3 = 1628.0 - y3.round();

                            last_position = Some((x3, y3));
                            output.push_str(&format!(
                                "{x1} {y1} {x2} {y2} {x3} {y3} c 0\n"
                            ));
                        }
                        PathSegment::Close => {}
                    }
                }
            }
            _ => panic!("Unsupported node: {node:?}"),
        }
    }

    output
}

/// Generate a glyph by referencing other glyphs
fn generate(glyph: &str) -> String {
    use std::fmt::Write;

    let mut g = String::new();
    let refer_at = |gn: &mut String, number: u32, (x, y): (i32, i32)| {
        writeln!(gn, "Refer: {number} -1 N 1 0 0 1 {x} {y} 1").unwrap();
    };
    let refer = |gn: &mut String, number: u32| {
        refer_at(gn, number, (0, 0));
    };
    let vowel = |gn: &mut String, vowel: u32, onglide: u32| {
        refer_at(gn, 1114169 + vowel, (0, -ONGLIDE_OFFSET));
        refer_at(
            gn,
            1114313 + onglide,
            (
                0,
                VOWEL_Y[usize::try_from(vowel).unwrap()]
                    + ONGLIDE_Y[usize::try_from(onglide).unwrap()]
                    - ONGLIDE_OFFSET,
            ),
        );
    };
    let sonorant_modifier = |gn: &mut String, sonorant: u32, modifier: u32| {
        refer(gn, 1114112 + sonorant);
        refer(gn, 1114136 + modifier);
    };
    let fw_sonorant_modifier =
        |gn: &mut String, sonorant: u32, modifier: u32| {
            refer(gn, 1114139 + sonorant);
            refer(gn, 1114163 + modifier);
        };
    let syllable_consonant = |gn: &mut String, index: u32| {
        // Fullwidth
        refer(gn, 1114139 + index);
        // Middle / Standalone Cap line
        refer(gn, 1114167);
    };
    let generate_syllable = |gn: &mut String, syllable: &str| {};

    match glyph {
        // generated combined vowel glyphs (y-)
        "zoitei.vowel.yie" => vowel(&mut g, 0, 0),
        "zoitei.vowel.yae" => vowel(&mut g, 1, 0),
        "zoitei.vowel.yih" => vowel(&mut g, 2, 0),
        "zoitei.vowel.yiy" => vowel(&mut g, 3, 0),
        "zoitei.vowel.yah" => vowel(&mut g, 4, 0),
        "zoitei.vowel.yia" => vowel(&mut g, 5, 0),
        "zoitei.vowel.yeh" => vowel(&mut g, 6, 0),
        "zoitei.vowel.yea" => vowel(&mut g, 7, 0),
        "zoitei.vowel.yuh" => vowel(&mut g, 8, 0),
        "zoitei.vowel.you" => vowel(&mut g, 9, 0),
        "zoitei.vowel.yoh" => vowel(&mut g, 10, 0),
        "zoitei.vowel.yio" => vowel(&mut g, 11, 0),
        "zoitei.vowel.yay" => vowel(&mut g, 12, 0),
        "zoitei.vowel.yai" => vowel(&mut g, 13, 0),
        "zoitei.vowel.yey" => vowel(&mut g, 14, 0),
        "zoitei.vowel.yei" => vowel(&mut g, 15, 0),
        "zoitei.vowel.yoy" => vowel(&mut g, 16, 0),
        "zoitei.vowel.yoi" => vowel(&mut g, 17, 0),
        "zoitei.vowel.yiu" => vowel(&mut g, 18, 0),
        "zoitei.vowel.yau" => vowel(&mut g, 19, 0),
        "zoitei.vowel.yao" => vowel(&mut g, 20, 0),
        "zoitei.vowel.yeu" => vowel(&mut g, 21, 0),
        "zoitei.vowel.yeo" => vowel(&mut g, 22, 0),
        "zoitei.vowel.yoa" => vowel(&mut g, 23, 0),
        // generated combined vowel glyphs (u-)
        "zoitei.vowel.uyh" => vowel(&mut g, 0, 1),
        "zoitei.vowel.uae" => vowel(&mut g, 1, 1),
        "zoitei.vowel.uih" => vowel(&mut g, 2, 1),
        "zoitei.vowel.uiy" => vowel(&mut g, 3, 1),
        "zoitei.vowel.uah" => vowel(&mut g, 4, 1),
        "zoitei.vowel.uia" => vowel(&mut g, 5, 1),
        "zoitei.vowel.ueh" => vowel(&mut g, 6, 1),
        "zoitei.vowel.uea" => vowel(&mut g, 7, 1),
        "zoitei.vowel.uoe" => vowel(&mut g, 8, 1),
        "zoitei.vowel.uou" => vowel(&mut g, 9, 1),
        "zoitei.vowel.uoh" => vowel(&mut g, 10, 1),
        "zoitei.vowel.uio" => vowel(&mut g, 11, 1),
        "zoitei.vowel.uay" => vowel(&mut g, 12, 1),
        "zoitei.vowel.uai" => vowel(&mut g, 13, 1),
        "zoitei.vowel.uey" => vowel(&mut g, 14, 1),
        "zoitei.vowel.uei" => vowel(&mut g, 15, 1),
        "zoitei.vowel.uoy" => vowel(&mut g, 16, 1),
        "zoitei.vowel.uoi" => vowel(&mut g, 17, 1),
        "zoitei.vowel.uiu" => vowel(&mut g, 18, 1),
        "zoitei.vowel.uau" => vowel(&mut g, 19, 1),
        "zoitei.vowel.uao" => vowel(&mut g, 20, 1),
        "zoitei.vowel.ueu" => vowel(&mut g, 21, 1),
        "zoitei.vowel.ueo" => vowel(&mut g, 22, 1),
        "zoitei.vowel.uoa" => vowel(&mut g, 23, 1),
        // generated base consonant glyphs
        "zoitei.plosive.p" => sonorant_modifier(&mut g, 0, 0),
        "zoitei.plosive.t" => sonorant_modifier(&mut g, 1, 0),
        "zoitei.plosive.k" => sonorant_modifier(&mut g, 2, 0),
        "zoitei.plosive.b" => sonorant_modifier(&mut g, 3, 0),
        "zoitei.plosive.d" => sonorant_modifier(&mut g, 4, 0),
        "zoitei.plosive.g" => sonorant_modifier(&mut g, 5, 0),
        "zoitei.unpalatalized.f" => sonorant_modifier(&mut g, 0, 1),
        "zoitei.unpalatalized.s" => sonorant_modifier(&mut g, 1, 1),
        "zoitei.unpalatalized.x" => sonorant_modifier(&mut g, 2, 1),
        "zoitei.unpalatalized.v" => sonorant_modifier(&mut g, 3, 1),
        "zoitei.unpalatalized.z" => sonorant_modifier(&mut g, 4, 1),
        "zoitei.unpalatalized.nh" => sonorant_modifier(&mut g, 5, 1),
        "zoitei.palatalized.th" => sonorant_modifier(&mut g, 0, 2),
        "zoitei.palatalized.sh" => sonorant_modifier(&mut g, 1, 2),
        "zoitei.palatalized.lh" => sonorant_modifier(&mut g, 2, 2),
        "zoitei.palatalized.w" => sonorant_modifier(&mut g, 3, 2),
        "zoitei.palatalized.zh" => sonorant_modifier(&mut g, 4, 2),
        "zoitei.palatalized.rh" => sonorant_modifier(&mut g, 5, 2),
        // generated fullwidth consonants
        "zoitei.fullwidth.plosive.p" => fw_sonorant_modifier(&mut g, 0, 0),
        "zoitei.fullwidth.plosive.t" => fw_sonorant_modifier(&mut g, 1, 0),
        "zoitei.fullwidth.plosive.k" => fw_sonorant_modifier(&mut g, 2, 0),
        "zoitei.fullwidth.plosive.b" => fw_sonorant_modifier(&mut g, 3, 0),
        "zoitei.fullwidth.plosive.d" => fw_sonorant_modifier(&mut g, 4, 0),
        "zoitei.fullwidth.plosive.g" => fw_sonorant_modifier(&mut g, 5, 0),
        "zoitei.fullwidth.unpalatalized.f" => {
            fw_sonorant_modifier(&mut g, 0, 1)
        }
        "zoitei.fullwidth.unpalatalized.s" => {
            fw_sonorant_modifier(&mut g, 1, 1)
        }
        "zoitei.fullwidth.unpalatalized.x" => {
            fw_sonorant_modifier(&mut g, 2, 1)
        }
        "zoitei.fullwidth.unpalatalized.v" => {
            fw_sonorant_modifier(&mut g, 3, 1)
        }
        "zoitei.fullwidth.unpalatalized.z" => {
            fw_sonorant_modifier(&mut g, 4, 1)
        }
        "zoitei.fullwidth.unpalatalized.nh" => {
            fw_sonorant_modifier(&mut g, 5, 1)
        }
        "zoitei.fullwidth.palatalized.th" => fw_sonorant_modifier(&mut g, 0, 2),
        "zoitei.fullwidth.palatalized.sh" => fw_sonorant_modifier(&mut g, 1, 2),
        "zoitei.fullwidth.palatalized.lh" => fw_sonorant_modifier(&mut g, 2, 2),
        "zoitei.fullwidth.palatalized.w" => fw_sonorant_modifier(&mut g, 3, 2),
        "zoitei.fullwidth.palatalized.zh" => fw_sonorant_modifier(&mut g, 4, 2),
        "zoitei.fullwidth.palatalized.rh" => fw_sonorant_modifier(&mut g, 5, 2),
        // generated syllable consonants
        "zoitei.syllable.l" => syllable_consonant(&mut g, 0),
        "zoitei.syllable.c" => syllable_consonant(&mut g, 1),
        "zoitei.syllable.r" => syllable_consonant(&mut g, 2),
        "zoitei.syllable.m" => syllable_consonant(&mut g, 3),
        "zoitei.syllable.n" => syllable_consonant(&mut g, 4),
        "zoitei.syllable.q" => syllable_consonant(&mut g, 5),
        "zoitei.syllable.p" => syllable_consonant(&mut g, 6),
        "zoitei.syllable.t" => syllable_consonant(&mut g, 7),
        "zoitei.syllable.k" => syllable_consonant(&mut g, 8),
        "zoitei.syllable.b" => syllable_consonant(&mut g, 9),
        "zoitei.syllable.d" => syllable_consonant(&mut g, 10),
        "zoitei.syllable.g" => syllable_consonant(&mut g, 11),
        "zoitei.syllable.f" => syllable_consonant(&mut g, 12),
        "zoitei.syllable.s" => syllable_consonant(&mut g, 13),
        "zoitei.syllable.x" => syllable_consonant(&mut g, 14),
        "zoitei.syllable.v" => syllable_consonant(&mut g, 15),
        "zoitei.syllable.z" => syllable_consonant(&mut g, 16),
        "zoitei.syllable.nh" => syllable_consonant(&mut g, 17),
        "zoitei.syllable.th" => syllable_consonant(&mut g, 18),
        "zoitei.syllable.sh" => syllable_consonant(&mut g, 19),
        "zoitei.syllable.lh" => syllable_consonant(&mut g, 20),
        "zoitei.syllable.w" => syllable_consonant(&mut g, 21),
        "zoitei.syllable.zh" => syllable_consonant(&mut g, 22),
        "zoitei.syllable.rh" => syllable_consonant(&mut g, 23),
        // generated consonant + vowel
        x if x.starts_with("zoitei.syllable.") => generate_syllable(
            &mut g,
            x.strip_prefix("zoitei.syllable.").unwrap(),
        ),
        _ => panic!("unknown generated glyph {glyph}"),
    }

    g
}

fn main() {
    use std::io::{BufRead, Write};

    // Generate file with all the font's characters
    let mut chars = String::new();

    chars.push_str("Character List for ");
    // Unicode equivalent of <bdo dir = "rtl">
    chars.extend([
        '\u{202A}', // FSI - For embedding
        '\u{202E}', // RLO
        'z', 'o', 'i', 't', 'e', 'i', '\u{202C}', // PDF
        '\u{2069}', // PDI - For embedding
        ':', '\n', '\u{202E}', // RLO
    ]);

    let input_font = std::fs::File::open("./Square.sfd").unwrap();
    let input_font = std::io::BufReader::new(input_font);
    let output_font = std::fs::File::create("./.Square.sfd").unwrap();
    let mut output_font = std::io::BufWriter::new(output_font);
    let mut current_spline: Option<String> = None;
    let mut in_fore = false;
    let mut lines = input_font.lines();
    let mut is_zoitei = false;
    let mut generated = None;

    while let Some(Ok(mut line)) = lines.next() {
        if let Some(name) = line.strip_prefix("StartChar: ") {
            generated = None;

            if name.starts_with("zoitei.") {
                current_spline = Some(read(name));
                is_zoitei = true;
            } else if let Some(glyph) = name.strip_prefix("generated.") {
                is_zoitei = true;
                current_spline = Some(String::new());
                generated = Some(generate(glyph));
            }
        } else if line == "EndChar" {
            current_spline = None;
            in_fore = false;
        } else if line == "Fore"
            && let Some(spline) = current_spline.take()
        {
            in_fore = true;

            if let Some(ref generated) = generated {
                line.push('\n');
                line.push_str(generated);
            } else {
                line.push_str("\nSplineSet\n");
                line.push_str(spline.as_str());
                line.push_str("EndSplineSet\n");
            }

            output_font.write_all(line.as_bytes()).unwrap();
        } else if let Some(encoding) = line.strip_prefix("Encoding: ")
            && is_zoitei
        {
            let encoding = encoding.split(' ').next().unwrap().parse().unwrap();

            if let Some(character) = char::from_u32(encoding) {
                chars.push(character);
            }

            // Where to start a new paragraph
            if matches!(encoding, 0xF600B | 0xF6017) {
                // PDF, newline, RLO
                chars.extend(['\u{202C}', '\n', '\u{202E}']);
            }
        }

        if !in_fore {
            line.push('\n');
            output_font.write_all(line.as_bytes()).unwrap();
        }
    }

    drop((lines, output_font));
    std::fs::rename("./.Square.sfd", "./Square.sfd").unwrap();

    // PDF
    chars.push('\u{202C}');
    std::fs::write("chars_zoitei.txt", chars).unwrap();
}
