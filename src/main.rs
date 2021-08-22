mod geom2d;

fn main() {
    let args = clap::App::new("dxf2svg")
        .arg(clap::Arg::with_name("dxf").required(true))
        .arg(clap::Arg::with_name("svg").required(true))
        .get_matches();
    let path = args.value_of("dxf").unwrap();
    let drawing = dxf::Drawing::load_file(path).unwrap();
    std::fs::write("from_dxf.json", serde_json::to_string(&drawing).unwrap()).unwrap();
    std::fs::write("from_dxf.yaml", serde_yaml::to_string(&drawing).unwrap()).unwrap();
    let view_box = {
        let min = &drawing.header.minimum_drawing_limits;
        let max = &drawing.header.maximum_drawing_limits;
        (min.x, min.y, max.x - min.x, max.y - min.y)
    };
    let mut svg = svg::Document::new().set("viewBox", view_box).add(
        svg::node::element::Rectangle::new()
            .set("fill", "white")
            .set("x", view_box.0)
            .set("y", view_box.0)
            .set("width", view_box.2 - view_box.0)
            .set("height", view_box.3 - view_box.1),
    );
    svg = draw_entities(svg, &drawing.entities, &drawing, &|p| dxf::Point {
        x: p.x,
        y: 2.0 * view_box.1 + view_box.3 - p.y,
        z: p.z,
    });
    svg::save(args.value_of("svg").unwrap(), &svg).unwrap();
}

fn draw_entities(
    mut svg: svg::Document,
    entities: &[dxf::entities::Entity],
    drawing: &dxf::Drawing,
    transform: &dyn Fn(&dxf::Point) -> dxf::Point,
) -> svg::Document {
    for e in entities {
        match &e.specific {
            dxf::entities::EntityType::Insert(insert) => {
                println!("INSERT: {}", insert.name);
                println!(
                    "scale_factor = ({}, {}, {})",
                    insert.x_scale_factor, insert.y_scale_factor, insert.z_scale_factor
                );
                if let Some(block) = drawing
                    .blocks
                    .iter()
                    .find(|block| block.name == insert.name)
                {
                    let (cos, sin) = {
                        let theta = insert.rotation * std::f64::consts::PI / 180.0;
                        (theta.cos(), theta.sin())
                    };
                    let transform = |p: &dxf::Point| {
                        let p = dxf::Point {
                            x: insert.x_scale_factor * p.x,
                            y: insert.y_scale_factor * p.y,
                            z: insert.z_scale_factor * p.z,
                        };
                        let p = dxf::Point {
                            x: cos * p.x - sin * p.y,
                            y: sin * p.x + cos * p.y,
                            z: p.z,
                        };
                        let p = dxf::Point {
                            x: insert.location.x + p.x,
                            y: insert.location.y + p.y,
                            z: insert.location.z + p.z,
                        };
                        transform(&p)
                    };
                    svg = draw_entities(svg, &block.entities, drawing, &transform);
                } else {
                    println!("block not found: name = {}", insert.name);
                }
            }
            dxf::entities::EntityType::Line(line) => {
                svg = draw_line(svg, line, transform);
            }
            dxf::entities::EntityType::Circle(circle) => {
                println!("CIRCLE");
            }
            dxf::entities::EntityType::Polyline(pol) => {
                println!("POLYLINE");
                svg = draw_polyline(svg, pol, transform);
            }
            dxf::entities::EntityType::RotatedDimension(dim) => {
                svg = draw_rotated_dimension(svg, dim, transform);
            }
            _ => println!("{:?}", e.specific),
        }
    }
    svg
}

fn draw_line(
    svg: svg::Document,
    line: &dxf::entities::Line,
    transform: impl Fn(&dxf::Point) -> dxf::Point,
) -> svg::Document {
    line_strip(svg, &[transform(&line.p1), transform(&line.p2)])
}

fn draw_polyline(
    svg: svg::Document,
    pol: &dxf::entities::Polyline,
    transform: impl Fn(&dxf::Point) -> dxf::Point,
) -> svg::Document {
    let points = pol
        .vertices
        .iter()
        .map(|v| transform(&v.location))
        .collect::<Vec<_>>();
    line_strip(svg, &points)
}

fn draw_rotated_dimension(
    svg: svg::Document,
    dim: &dxf::entities::RotatedDimension,
    transform: impl Fn(&dxf::Point) -> dxf::Point,
) -> svg::Document {
    let p1 = &dim.dimension_base.definition_point_1;
    let p2 = &dim.definition_point_2;
    let p3 = &dim.definition_point_3;
    let p4 = {
        let theta = dim.rotation_angle * std::f64::consts::PI / 180.0;
        let line1 = geom2d::Line {
            p: p1.into(),
            v: geom2d::UnitVec::of_angle(theta),
        };
        let line2 = geom2d::Line {
            p: p2.into(),
            v: (geom2d::Pos::from(p1) - geom2d::Pos::from(p3))
                .normalize()
                .unwrap(),
        };
        line1.intersection_pos(&line2).unwrap()
    };
    line_strip(
        svg,
        &[
            transform(p2),
            transform(&p4.into()),
            transform(p1),
            transform(p3),
        ],
    )
}

fn line_strip(svg: svg::Document, pol: &[dxf::Point]) -> svg::Document {
    if pol.len() >= 2 {
        let data = svg::node::element::path::Data::new().move_to((pol[0].x, pol[0].y));
        let data = (1..pol.len())
            .map(|i| (pol[i].x - pol[i - 1].x, pol[i].y - pol[i - 1].y))
            .fold(data, |data, v| data.line_by(v));
        let path = svg::node::element::Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1)
            .set("d", data);
        svg.add(path)
    } else {
        svg
    }
}
