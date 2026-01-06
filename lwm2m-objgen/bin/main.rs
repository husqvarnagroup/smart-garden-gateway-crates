fn main() {
    let args: Vec<_> = std::env::args().collect();

    let xmlfile = std::fs::File::open(&args[1]).expect("can't open file");
    let reader = std::io::BufReader::new(xmlfile);
    let object =
        lwm2m_objgen::Object::from_reader(reader).expect("can't parse lwm2m object definition");
    eprintln!("Object: \n{object:#?}");

    let code = format!("{object}");
    let code =
        lwm2m_objgen::rustfmt_generated_string(&code).unwrap_or(std::borrow::Cow::Borrowed(&code));
    println!("{code}");
}
