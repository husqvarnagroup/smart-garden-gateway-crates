// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use convert_case::Casing as _;
use quote::quote;
use std::io::Write as _;

lazy_static::lazy_static! {
    static ref RE: regex::Regex = regex::Regex::new(r"^\d+(-\d+_\d+)?\.xml$").unwrap();
}

fn str2ident(string: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(string, proc_macro2::Span::call_site())
}

fn process_file<P: AsRef<std::path::Path> + std::fmt::Debug, W: std::io::Write>(
    objects: &mut Vec<lwm2m_objgen::Object>,
    writer: &mut W,
    path: P,
) {
    eprintln!("{path:?}");

    let xmlfile = std::fs::File::open(&path).expect("can't open file");
    let reader = std::io::BufReader::new(xmlfile);
    let object = lwm2m_objgen::Object::from_reader(reader).expect("can't parse xsd");

    let code = format!("{object}");
    let code =
        lwm2m_objgen::rustfmt_generated_string(&code).unwrap_or(std::borrow::Cow::Borrowed(&code));
    write!(
        writer,
        "// + {path:?}\n#[allow(clippy::doc_markdown)]\n{code}\n// - {path:?}\n\n"
    )
    .unwrap();

    objects.push(object);
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut f = std::fs::File::create(out_path.join("objects.rs")).unwrap();
    let mut objects = Vec::new();

    let dir = "../third_party/lwm2m-registry/version_history";
    println!("cargo:rerun-if-changed={dir}");
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_file()
            || !path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| RE.is_match(s))
        {
            continue;
        }

        // NOTE: parsing is incredibly slow. we don't want to parse all of them
        //       everytime we change some code.
        if !matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some("3-1_1.xml" | "5-1_1.xml")
        ) {
            continue;
        }

        process_file(&mut objects, &mut f, path);
    }

    let dir = "../third_party/bnw-ipso-registry";
    println!("cargo:rerun-if-changed={dir}");
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_file() || !matches!(path.extension().and_then(|s| s.to_str()), Some("xml")) {
            continue;
        }

        if !matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some(
                "connection-status.xml"
                    | "data-download.xml"
                    | "includable-device.xml"
                    | "status-message.xml"
            )
        ) {
            continue;
        }

        process_file(&mut objects, &mut f, path);
    }

    let mut f = std::fs::File::create(out_path.join("misc.rs")).unwrap();
    let mut oids: Vec<_> = objects.iter().map(lwm2m_objgen::Object::strid).collect();
    let mut urns: Vec<_> = objects.iter().map(lwm2m_objgen::Object::urn).collect();

    // NOTE: There's no IPSO specification for this since the resources are
    //       generated dynamically from lemonbeat values.
    //       Since we use an enum for the object type we need to add it here
    //       though.
    oids.push("lemonbeat".to_string());
    urns.push("urn:oma:lwm2m:x:31000");

    let oid_variants: Vec<_> = oids
        .iter()
        .map(|oid| str2ident(&oid.to_case(convert_case::Case::Pascal)))
        .collect();

    let code = quote! {
            #[doc = "Generated object type enum."]
            #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
            pub enum ObjectType {
                #(#oid_variants,)*
            }

            impl ::std::str::FromStr for ObjectType {
                type Err = Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        #(#oids => Ok(Self::#oid_variants),)*
                        other => Err(Error::Anyhow(::anyhow::anyhow!("unsupported object ID `{other}`"))),
                    }
                }
            }

            impl ObjectType {
                pub fn as_str(self) -> &'static str {
                    match self {
                        #(Self::#oid_variants => #oids,)*
                    }
                }

                pub fn urn(self) -> &'static str {
                    match self {
                        #(Self::#oid_variants => #urns,)*
                    }
                }
            }
        }
        .to_string();
    let code =
        lwm2m_objgen::rustfmt_generated_string(&code).unwrap_or(std::borrow::Cow::Borrowed(&code));
    f.write_all(code.as_bytes()).unwrap();
}
