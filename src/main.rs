fn main() {
    let args = clap::App::new("dxf2svg")
        .arg(clap::Arg::with_name("dxf"))
        .get_matches();
    let path = args.value_of("dxf").unwrap();
    let drawing = dxf::Drawing::load_file(path).unwrap();
    for e in drawing.entities {
        println!("{:?}", e.specific);
    }
}
