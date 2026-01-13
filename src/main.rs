use usvg::tiny_skia_path::{Path, PathBuilder, PathSegment, PathVerb, Point};

/// Replace `Close` with `LineTo` back to the last `MoveTo` position
fn replace_close_segments(path: &Path) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut start = None;

    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(point) => {
                start = Some(point);
                builder.move_to(point.x, point.y);
            }
            PathSegment::LineTo(point) => {
                builder.line_to(point.x, point.y);
            }
            PathSegment::QuadTo(pt1, pt2) => {
                builder.quad_to(pt1.x, pt1.y, pt2.x, pt2.y);
            }
            PathSegment::CubicTo(pt1, pt2, pt3) => {
                builder.cubic_to(pt1.x, pt1.y, pt2.x, pt2.y, pt3.x, pt3.y);
            }
            PathSegment::Close => {
                let point = start.expect("no start");

                builder.line_to(point.x, point.y);
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

    builder.move_to(first_point.x, first_point.y);

    let mut last_move = first_point;

    for verb in path.verbs()[1..].iter().rev() {
        match verb {
            PathVerb::Move => {
                let pt = points.next()?;

                last_move = pt;
                builder.move_to(pt.x, pt.y);
            }
            PathVerb::Line => {
                let pt = points.next()?;

                builder.line_to(pt.x, pt.y);
            }
            PathVerb::Quad => {
                let pt1 = points.next()?;
                let pt2 = points.next()?;

                builder.quad_to(pt1.x, pt1.y, pt2.x, pt2.y);
            }
            PathVerb::Cubic => {
                let pt1 = points.next()?;
                let pt2 = points.next()?;
                let pt3 = points.next()?;

                builder.cubic_to(pt1.x, pt1.y, pt2.x, pt2.y, pt3.x, pt3.y);
            }
            PathVerb::Close => {}
        }
    }

    builder.line_to(last_move.x, last_move.y);
    builder.finish()
}

fn read(glyph_name: &str) -> String {
    let glyph_path = glyph_name.split('.').collect::<Vec<_>>().join("/");
    let input_svg =
        std::fs::read_to_string(format!("./{glyph_path}.svg")).unwrap();
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(&input_svg, &opt).unwrap();
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

    let mut generated = String::new();
    let mut refer = |number: u32| {
        writeln!(&mut generated, "Refer: {number} -1 N 1 0 0 1 0 0 1").unwrap();
    };
    let mut sonorant_modifier = |sonorant: u32, modifier: u32| {
        refer(1114112 + sonorant);
        refer(1114136 + modifier);
    };

    match glyph {
        "zoitei.plosive.p" => sonorant_modifier(0, 0),
        "zoitei.plosive.t" => sonorant_modifier(1, 0),
        "zoitei.plosive.k" => sonorant_modifier(2, 0),
        "zoitei.plosive.b" => sonorant_modifier(3, 0),
        "zoitei.plosive.d" => sonorant_modifier(4, 0),
        "zoitei.plosive.g" => sonorant_modifier(5, 0),
        "zoitei.unpalatalized.f" => sonorant_modifier(0, 1),
        "zoitei.unpalatalized.s" => sonorant_modifier(1, 1),
        "zoitei.unpalatalized.x" => sonorant_modifier(2, 1),
        "zoitei.unpalatalized.v" => sonorant_modifier(3, 1),
        "zoitei.unpalatalized.z" => sonorant_modifier(4, 1),
        "zoitei.unpalatalized.nh" => sonorant_modifier(5, 1),
        "zoitei.palatalized.th" => sonorant_modifier(0, 2),
        "zoitei.palatalized.sh" => sonorant_modifier(1, 2),
        "zoitei.palatalized.lh" => sonorant_modifier(2, 2),
        "zoitei.palatalized.w" => sonorant_modifier(3, 2),
        "zoitei.palatalized.zh" => sonorant_modifier(4, 2),
        "zoitei.palatalized.rh" => sonorant_modifier(5, 2),
        _ => panic!("unknown generated glyph {glyph}"),
    }

    generated
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
            } else if name.starts_with("composite.zoitei") {
                is_zoitei = true;
            } else if let Some(glyph) = name.strip_prefix("generated.") {
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
                line.push_str("\n");
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
        }

        if !in_fore {
            line.push('\n');
            output_font.write_all(line.as_bytes()).unwrap();
        }
    }

    drop((lines, output_font));
    std::fs::rename("./.Square.sfd", "./Square.sfd").unwrap();

    // PDF
    chars.extend(['\u{202C}']);
    std::fs::write("chars_zoitei.txt", chars).unwrap();
}
