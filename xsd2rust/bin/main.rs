fn main() {
    let args: Vec<_> = std::env::args().collect();

    let xmlfile = std::fs::File::open(&args[1]).expect("can't open file");
    let reader = std::io::BufReader::new(xmlfile);
    let xsd = xsd2rust::Xsd::from_reader(reader).expect("can't parse xsd");
    eprintln!("XSD: \n{xsd:#?}");

    let code = format!("{xsd}");
    let code =
        xsd2rust::rustfmt_generated_string(&code).unwrap_or(std::borrow::Cow::Borrowed(&code));
    println!("{code}");
}
