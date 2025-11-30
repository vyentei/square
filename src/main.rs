use usvg::tiny_skia_path::{PathSegment, Point};

const IMPORT: &[&str] = &[
    "zoitei.silence.H",
    // "zoitei.silence.HH",
    // "zoitei.silence.HHH",
    // "zoitei.silence.HHHH",
];

fn read(glyph_name: &str) {
    let glyph_path = glyph_name.split('.').collect::<Vec<_>>().join("/");
    let input_svg =
        std::fs::read_to_string(format!("./{glyph_path}.svg")).unwrap();
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(&input_svg, &opt).unwrap();
    let nodes = tree.root().children();

    println!(
        "StartChar: {glyph_name}\n\
         Encoding: 1009728 1009728 1009728\n\
         Width: 2048\n\
         Flags: W\n\
         LayerCount: 2\n\
         Fore\n\
         SplineSet",
    );

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

                            println!("{x} {y} m 0")
                        }
                        PathSegment::LineTo(Point { x, y }) => {
                            let y = 1556.0 - y;

                            println!("{x} {y} l 0")
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

    println!(
        "EndSplineSet\n\
         EndChar",
    );
}

fn main() {
    for i in IMPORT {
        read(i);
    }
}
