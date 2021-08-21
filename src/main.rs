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
    svg = draw_entities(svg, &drawing.entities, &drawing);
    svg::save(args.value_of("svg").unwrap(), &svg).unwrap();
}

fn draw_entities(
    mut svg: svg::Document,
    entities: &[dxf::entities::Entity],
    drawing: &dxf::Drawing,
) -> svg::Document {
    for e in entities {
        match &e.specific {
            dxf::entities::EntityType::Line(line) => {
                svg = draw_line(svg, line);
            }
            dxf::entities::EntityType::Insert(insert) => {
                println!("INSERT: {}", insert.name);
                println!("{:?}", insert);
                if let Some(block) = drawing
                    .blocks
                    .iter()
                    .find(|block| block.name == insert.name)
                {
                    svg = draw_entities(svg, &block.entities, drawing);
                } else {
                    println!("block not found: name = {}", insert.name);
                }
            }
            dxf::entities::EntityType::Circle(circle) => {
                println!("CIRCLE");
            }
            dxf::entities::EntityType::Polyline(pol) => {
                println!("POLYLINE");
            }
            _ => (),
        }
    }
    svg
}

fn draw_line(svg: svg::Document, line: &dxf::entities::Line) -> svg::Document {
    let data = svg::node::element::path::Data::new()
        .move_to((line.p1.x, line.p1.y))
        .line_by((line.p2.x - line.p1.x, line.p2.y - line.p1.y));
    let path = svg::node::element::Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("d", data);
    svg.add(path)
}
