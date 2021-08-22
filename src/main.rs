fn main() {
    let args = clap::App::new("dxf2svg")
        .arg(clap::Arg::with_name("dxf").required(true))
        .arg(clap::Arg::with_name("svg").required(true))
        .get_matches();
    let path = args.value_of("dxf").unwrap();
    let drawing = dxf::Drawing::load_file(path).unwrap();
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
                        let x = cos * p.x - sin * p.y;
                        let y = sin * p.x + cos * p.y;
                        let p = dxf::Point {
                            x: insert.location.x + x,
                            y: insert.location.y + y,
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
            _ => (),
        }
    }
    svg
}

fn draw_line(
    svg: svg::Document,
    line: &dxf::entities::Line,
    transform: impl Fn(&dxf::Point) -> dxf::Point,
) -> svg::Document {
    let p1 = transform(&line.p1);
    let p2 = transform(&line.p2);
    let data = svg::node::element::path::Data::new()
        .move_to((p1.x, p1.y))
        .line_by((p2.x - p1.x, p2.y - p1.y));
    let path = svg::node::element::Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("d", data);
    svg.add(path)
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
    if points.len() >= 2 {
        let data = (1..points.len())
            .map(|i| (points[i].x - points[i - 1].x, points[i].y - points[i - 1].y))
            .fold(
                svg::node::element::path::Data::new().move_to((points[0].x, points[0].y)),
                |data, v| data.line_by(v),
            );
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
