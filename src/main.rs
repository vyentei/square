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

                let segments: Vec<_> = path.segments().collect();

                if glyph_name == "zoitei.sonorant.q" {
                    dbg!(&segments);
                }

                let Some(path) = reverse_path_segments(&path) else {
                    panic!("Failed to reverse path")
                };
                let segments: Vec<_> = path.segments().collect();

                if glyph_name == "zoitei.sonorant.q" {
                    dbg!(&segments);
                }

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

fn main() {
    use std::io::{BufRead, Write};

    let input_font = std::fs::File::open("./Square.sfd").unwrap();
    let input_font = std::io::BufReader::new(input_font);
    let output_font = std::fs::File::create("./.Square.sfd").unwrap();
    let mut output_font = std::io::BufWriter::new(output_font);
    let mut current_spline: Option<String> = None;
    let mut in_fore = false;
    let mut lines = input_font.lines();

    while let Some(Ok(mut line)) = lines.next() {
        if let Some(name) = line.strip_prefix("StartChar: ") {
            if name.starts_with("zoitei.") {
                current_spline = Some(read(name));
            }
        } else if line == "EndChar" {
            current_spline = None;
            in_fore = false;
        } else if line == "Fore"
            && let Some(spline) = current_spline.take()
        {
            in_fore = true;
            line.push_str("\nSplineSet\n");
            line.push_str(spline.as_str());
            line.push_str("EndSplineSet\n");
            output_font.write_all(line.as_bytes()).unwrap();
        }

        if !in_fore {
            line.push('\n');
            output_font.write_all(line.as_bytes()).unwrap();
        }
    }

    drop((lines, output_font));
    std::fs::rename("./.Square.sfd", "./Square.sfd").unwrap();
}
