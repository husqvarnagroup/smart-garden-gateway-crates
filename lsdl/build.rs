// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use std::io::Write as _;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let xsddir = "../third_party/lsdl-specification-w3c/xsd";

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut f = std::fs::File::create(out_path.join("xsd.rs")).unwrap();

    let mut infotype = None;

    println!("cargo:rerun-if-changed={xsddir}");
    for entry in std::fs::read_dir(xsddir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_file() || path.extension() != Some(std::ffi::OsStr::new("xsd")) {
            continue;
        }

        let filename = path.file_name().unwrap();
        if filename == "mac.xsd"
            || filename == "phy.xsd"
            || filename == "calculation.xsd"
            || filename == "statemachine.xsd"
        {
            continue;
        }

        eprintln!("{path:?}");
        let xmlfile = std::fs::File::open(&path).expect("can't open file");
        let reader = std::io::BufReader::new(xmlfile);
        let mut xsd = xsd2rust::Xsd::from_reader(reader).expect("can't parse xsd");

        if let Some((i, ct)) = xsd
            .complex_types
            .iter()
            .enumerate()
            .find(|(_, ct)| matches!(ct.name.as_deref(), Some("infoType")))
        {
            match &mut infotype {
                Some(infotype) => {
                    if infotype != ct {
                        panic!("unequal infoType's");
                    }
                }
                None => infotype = Some((*ct).clone()),
            }

            xsd.complex_types.swap_remove(i);
        }

        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let code = format!("pub mod {file_stem} {{ use super::common::infoType; {xsd} }}");
        let code =
            xsd2rust::rustfmt_generated_string(&code).unwrap_or(std::borrow::Cow::Borrowed(&code));
        writeln!(f, "{code}").unwrap();
    }

    let code = format!(
        "pub mod common {{ use crate::Error; use crate::XmlType; {} }}",
        infotype.unwrap()
    );
    let code =
        xsd2rust::rustfmt_generated_string(&code).unwrap_or(std::borrow::Cow::Borrowed(&code));
    writeln!(f, "{code}").unwrap();
}
