use usvg::tiny_skia_path::{PathSegment, Point};

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
                let mut segments: Vec<_> = path.data().segments().collect();
                let first = segments.first_mut().unwrap();

                match *first {
                    PathSegment::MoveTo(pt) => {
                        let last = PathSegment::MoveTo(pt);

                        *first = PathSegment::LineTo(pt);
                        segments.push(last);
                    }
                    _ => panic!("Unsupported first: {node:?}"),
                }

                for segment in segments.iter().rev() {
                    match segment {
                        PathSegment::MoveTo(Point { x, y }) => {
                            let y = 1556.0 - y;

                            output.push_str(&format!("{x} {y} m 0\n"));
                        }
                        PathSegment::LineTo(Point { x, y }) => {
                            let y = 1556.0 - y;

                            output.push_str(&format!("{x} {y} l 0\n"));
                        }
                        PathSegment::QuadTo(
                            Point { x: _x1, y: _y1 },
                            Point { x: _x2, y: _y2 },
                        ) => todo!(),
                        PathSegment::CubicTo(
                            Point { x: _x1, y: _y1 },
                            Point { x: _x2, y: _y2 },
                            Point { x: _x3, y: _y3 },
                        ) => todo!(),
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
        } else if line == "Fore" && let Some(spline) = current_spline.take() {
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
