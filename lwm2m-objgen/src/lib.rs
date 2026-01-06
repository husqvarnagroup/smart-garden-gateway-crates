//! Convert LWM2M object definitions to rust traits.

#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

use anyhow::anyhow;
use convert_case::Casing as _;
use quote::quote;
use quote::ToTokens as _;
use serde::Deserialize as _;
use std::io::Write as _;

fn escapestr(string: &str) -> String {
    let s = string.replace(['/', '(', ')', ',', '.', '&', '+', '|'], "");
    if s.chars().next().is_some_and(char::is_numeric) {
        format!("num{s}")
    } else if string.to_lowercase() == "type" {
        "type_escaped".to_string()
    } else {
        s
    }
}

fn str2upperident(string: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(
        &escapestr(string).to_case(convert_case::Case::UpperSnake),
        proc_macro2::Span::call_site(),
    )
}

fn str2ident(string: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(
        &escapestr(string).to_case(convert_case::Case::Snake),
        proc_macro2::Span::call_site(),
    )
}

fn str2typeident(string: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(
        &escapestr(string).to_case(convert_case::Case::Pascal),
        proc_macro2::Span::call_site(),
    )
}

/// giant hack to work around quick-xml enum bugs
fn deserialize_enum<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    for<'de2> T: serde::Deserialize<'de2>,
{
    // deserialize it as string. That's the part quick-xml does correctly
    let s = String::deserialize(deserializer)?;

    // now serialize that string into a proper json document
    let json = serde_json::Value::String(s).to_string();

    // now deserialize into the enum using serde_json
    let val: T = serde_json::from_str(&json).map_err(serde::de::Error::custom)?;

    Ok(val)
}

#[derive(Debug, serde::Deserialize)]
enum MultipleInstances {
    Multiple,
    Single,
}

#[derive(Debug, serde::Deserialize)]
enum Mandatory {
    Mandatory,
    Optional,
}

#[derive(Debug, serde::Deserialize)]
enum Operations {
    #[serde(rename = "R")]
    ReadOnly,
    #[serde(rename = "W")]
    WriteOnly,
    #[serde(rename = "RW")]
    ReadWrite,
    #[serde(rename = "E")]
    Execute,
    #[serde(rename = "")]
    None,
}

#[derive(Debug, serde::Deserialize)]
enum Type {
    String,
    Integer,
    Float,
    Boolean,
    Opaque,
    Time,
    Objlnk,
    #[serde(rename = "")]
    None,
    #[serde(rename = "Unsigned Integer")]
    UnsignedInteger,
    Corelnk,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceItem {
    #[serde(rename = "ID")]
    id: u16,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Operations", deserialize_with = "deserialize_enum")]
    operations: Operations,
    #[serde(rename = "MultipleInstances", deserialize_with = "deserialize_enum")]
    multiple_instances: MultipleInstances,
    #[serde(rename = "Mandatory", deserialize_with = "deserialize_enum")]
    mandatory: Mandatory,
    #[serde(rename = "Type", deserialize_with = "deserialize_enum")]
    resource_type: Type,
    #[serde(rename = "RangeEnumeration")]
    range_enumeration: String,
    #[serde(rename = "Units")]
    units: String,
    #[serde(rename = "Description")]
    description: String,
}

impl ResourceItem {
    fn quoted_rust_type(&self) -> proc_macro2::TokenStream {
        match &self.resource_type {
            Type::String => quote! {String},
            Type::Integer => quote! {i64},
            Type::UnsignedInteger => quote! {u64},
            Type::Float => quote! {f64},
            Type::Boolean => quote! {bool},
            Type::Opaque => quote! {Vec<u8>},
            Type::Time => quote! {std::time::SystemTime},
            Type::Objlnk => quote! {ObjectLink},
            Type::Corelnk => quote! {CoreLink},
            Type::None => quote! {()},
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Resources {
    #[serde(rename = "Item", default = "Vec::new")]
    items: Vec<ResourceItem>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    #[allow(clippy::struct_field_names)]
    #[serde(rename = "ObjectType")]
    object_type: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Description1")]
    description1: String,
    #[serde(rename = "Description2")]
    description2: String,
    #[serde(rename = "ObjectID")]
    id: u16,
    #[serde(rename = "ObjectURN")]
    urn: String,
    #[serde(rename = "LWM2MVersion")]
    lwm2m_version: String,
    #[serde(rename = "ObjectVersion")]
    version: String,
    #[serde(rename = "MultipleInstances", deserialize_with = "deserialize_enum")]
    multiple_instances: MultipleInstances,
    #[serde(rename = "Mandatory", deserialize_with = "deserialize_enum")]
    mandatory: Mandatory,
    #[serde(rename = "Resources")]
    resources: Resources,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Lwm2m {
    #[serde(rename = "xmlns:xsi")]
    xmlns_xsi: String,
    #[serde(rename = "xsi:noNamespaceSchemaLocation")]
    no_ns_schema_location: String,

    #[serde(rename = "Object")]
    objects: Vec<Object>,
}

impl quote::ToTokens for Object {
    #[allow(clippy::too_many_lines)]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = str2typeident(&self.name);
        let ident_handler = str2typeident(&format!("{}Handler", self.name));
        let ident_makejson = str2ident(&format!("make_{}_json", self.name));
        let doc = format!("{}\n\n{}", self.description1, self.description2)
            .replace('[', "(")
            .replace(']', ")");
        let urn = &self.urn;

        let mut fields = Vec::new();
        let mut resource_ids_consts = Vec::new();
        let mut read_matches = Vec::new();
        let mut write_matches = Vec::new();
        let mut exec_matches = Vec::new();
        let mut resource_names = Vec::new();
        let mut resource_ids = Vec::new();
        let mut resource_operations = Vec::new();
        let mut resource_isarray = Vec::new();
        for r in &self.resources.items {
            let ident_upper = str2upperident(&format!("{}_{}", ident, r.name));
            let ident = str2ident(&r.name);
            let ident_set = str2ident(&format!("set_{}", r.name));
            let ident_exec = str2ident(&r.name);
            let doc = &r.description;
            let id = r.id as usize;
            let name = &r.name;
            let ty = r.quoted_rust_type();

            let traitimpl = match &r.mandatory {
                Mandatory::Mandatory => quote! {;},
                Mandatory::Optional => quote! {
                    { Err(Error::UnsupportedOptionalResource) }
                },
            };

            let fn_get = match &r.operations {
                Operations::ReadWrite | Operations::ReadOnly => Some(quote! {
                    #[doc=#doc]
                    async fn #ident(
                        &self,
                        _object: usize,
                        _resource: usize
                    ) -> Result<TimedData<#ty>, Error>
                        #traitimpl
                }),
                _ => None,
            };

            let fn_set = match &r.operations {
                Operations::ReadWrite | Operations::WriteOnly => Some(quote! {
                    #[doc=#doc]
                    async fn #ident_set(
                        &mut self,
                        _object: usize,
                        _resource: usize,
                        _value: #ty
                    ) -> Result<(), Error>
                        #traitimpl
                }),
                _ => None,
            };

            let fn_exec = match &r.operations {
                Operations::Execute => Some(quote! {
                    #[doc=#doc]
                    async fn #ident_exec(
                        &mut self,
                        _object: usize,
                        _resource: usize,
                        _args: Option<Vec<String>>
                    ) -> Result<(), Error>
                        #traitimpl
                }),
                _ => None,
            };

            fields.push(quote! {
                #fn_get
                #fn_set
                #fn_exec
            });
            resource_ids_consts.push(quote! {
                pub const #ident_upper: usize = #id;
            });
            resource_names.push(name.to_case(convert_case::Case::Snake));
            resource_ids.push(id);
            resource_isarray.push(match &r.multiple_instances {
                MultipleInstances::Multiple => true,
                MultipleInstances::Single => false,
            });

            let operations = match &r.operations {
                Operations::ReadOnly => lwm2m_types::OP_READ,
                Operations::WriteOnly => lwm2m_types::OP_WRITE,
                Operations::ReadWrite => lwm2m_types::OP_READ | lwm2m_types::OP_WRITE,
                Operations::Execute => lwm2m_types::OP_EXEC,
                Operations::None => 0x0,
            };
            resource_operations.push(operations);

            if fn_get.is_some() {
                read_matches.push(quote! {
                    #ident_upper => Ok(self
                        .t
                        .#ident(object_instance, instance)
                        .await?
                        .into()),
                });
            }

            if fn_set.is_some() {
                write_matches.push(quote! {
                    #ident_upper => {
                        self
                            .t
                            .#ident_set(object_instance, instance, value.data.try_into()?)
                            .await?;
                        Ok(())
                    },
                });
            }

            if fn_exec.is_some() {
                exec_matches.push(quote! {
                    #ident_upper => {
                        self
                            .t
                            .#ident(object_instance, instance, args)
                            .await?;
                        Ok(())
                    },
                });
            }
        }

        let makejson_args = self
            .resources
            .items
            .iter()
            .filter(|r| matches!(&r.operations, Operations::ReadWrite | Operations::ReadOnly))
            .map(|r| {
                let name = str2ident(&r.name);
                let ty = r.quoted_rust_type();
                let ty = match (&r.multiple_instances, &r.mandatory) {
                    (MultipleInstances::Multiple, _) => quote! { Vec<Option<#ty>> },
                    (MultipleInstances::Single, Mandatory::Optional) => quote! { Option<#ty> },
                    (MultipleInstances::Single, Mandatory::Mandatory) => quote! { #ty },
                };
                quote! {
                    #name: #ty,
                }
            });

        let makejson_assignments = self
            .resources
            .items
            .iter()
            .filter(|r| matches!(&r.operations, Operations::ReadWrite | Operations::ReadOnly))
            .map(|r| {
                let name = &r.name;
                let ident = str2ident(&r.name);
                let q = quote! {
                    res.insert(#name, Value {
                        data: ValueData::from(#ident),
                        time,
                    });
                };
                match (&r.multiple_instances, &r.mandatory) {
                    (MultipleInstances::Single, Mandatory::Optional) => quote! {
                        if let Some(#ident) = #ident {
                            #q
                        }
                    },
                    _ => q,
                }
            });

        tokens.extend(quote! {
            #[doc=#doc]
            #[::async_trait::async_trait]
            pub trait #ident {
                #(#fields)*

                async fn handle_partial_write(
                    &mut self,
                    _object_instance: usize,
                    _values: std::collections::HashMap<String, Value>,
                ) -> Result<(), Error> {
                    Err(Error::UnsupportedPartialWrite)
                }
            }

            #(#resource_ids_consts)*

            pub struct #ident_handler<'a, T> {
                t: &'a mut T,
            }

            impl<'a, T: Send + Sync + #ident> #ident_handler<'a, T> {
                pub fn new(t: &'a mut T) -> Self {
                    Self { t }
                }
            }

            #[::async_trait::async_trait]
            impl<T: Send + Sync + #ident> Object for #ident_handler<'_, T> {
                fn urn(&self) -> &'static str {
                    #urn
                }

                async fn read_resource(
                    &self,
                    object_instance: usize,
                    id: usize,
                    instance: usize,
                ) -> Result<Value, Error>
                {
                    match id {
                        #(#read_matches)*
                        _ => Err(Error::Anyhow(::anyhow::anyhow!(
                            "read: unknown resource id `{}`",
                            id
                        ))),
                    }
                }

                async fn write_resource(
                    &mut self,
                    object_instance: usize,
                    id: usize,
                    instance: usize,
                    value: Value
                ) -> Result<(), Error>
                {
                    match id {
                        #(#write_matches)*
                        _ => Err(Error::Anyhow(::anyhow::anyhow!(
                            "write: unknown resource id `{}`",
                            id
                        ))),
                    }
                }

                async fn exec(
                    &mut self,
                    object_instance: usize,
                    id: usize,
                    instance: usize,
                    args: Option<Vec<String>>
                ) -> Result<(), Error>
                {
                    match id {
                        #(#exec_matches)*
                        _ => Err(Error::Anyhow(::anyhow::anyhow!(
                            "write: unknown resource id `{}`",
                            id
                        ))),
                    }
                }

                fn parse_resource_name(&self, name: &str) -> Result<usize, Error> {
                    match name {
                        #(#resource_names => Ok(#resource_ids),)*
                        _ => Err(Error::Anyhow(::anyhow::anyhow!(
                            "unknown resource name `{}`",
                            name
                        ))),
                    }
                }

                fn get_resource_name(&self, id: usize) -> Result<&str, Error> {
                    match id {
                        #(#resource_ids => Ok(#resource_names),)*
                        _ => Err(Error::Anyhow(::anyhow::anyhow!(
                            "unknown resource id `{}`",
                            id
                        ))),
                    }
                }

                fn supported_resource_operations(&self, id: usize) -> Result<usize, Error> {
                    #[allow(clippy::match_same_arms)]
                    match id {
                        #(#resource_ids => Ok(#resource_operations),)*
                        _ => Err(Error::Anyhow(::anyhow::anyhow!(
                            "unknown resource id `{}`",
                            id
                        ))),
                    }
                }

                fn is_array_resource(&self, id: usize) -> Result<bool, Error> {
                    #[allow(clippy::match_same_arms)]
                    match id {
                        #(#resource_ids => Ok(#resource_isarray),)*
                        _ => Err(Error::Anyhow(::anyhow::anyhow!(
                            "unknown resource id `{}`",
                            id
                        ))),
                    }
                }

                async fn handle_partial_write(
                    &mut self,
                    object_instance: usize,
                    values: std::collections::HashMap<String, Value>,
                ) -> Result<(), Error> {
                    self.t.handle_partial_write(object_instance, values).await
                }
            }

            #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
            pub fn #ident_makejson(
                time: Option<std::time::SystemTime>,
                #(#makejson_args)*
            ) -> ::serde_json::Value {
                let mut res = std::collections::HashMap::<&str, Value>::new();
                #(#makejson_assignments)*
                let mut json = ::serde_json::json!(res);

                json.as_object_mut().unwrap().insert("_urn".to_string(), #urn.into());

                json
            }
        });
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ts = quote! {};
        self.to_tokens(&mut ts);

        f.write_str(&ts.to_string())
    }
}

impl Object {
    pub fn from_reader<R: std::io::BufRead>(reader: R) -> Result<Object, anyhow::Error> {
        let mut lwm2m: Lwm2m = quick_xml::de::from_reader(reader)?;

        if lwm2m.xmlns_xsi != "http://www.w3.org/2001/XMLSchema-instance" {
            return Err(anyhow!("unsupported xmlns:xsi: {}", lwm2m.xmlns_xsi));
        }

        if !matches!(
            lwm2m.no_ns_schema_location.as_str(),
            "http://www.openmobilealliance.org/tech/profiles/LWM2M-v1_1.xsd"
                | "http://openmobilealliance.org/tech/profiles/LWM2M-v1_1.xsd"
                | "http://www.openmobilealliance.org/tech/profiles/LWM2M.xsd"
                | "http://openmobilealliance.org/tech/profiles/LWM2M.xsd"
        ) {
            return Err(anyhow!(
                "unsupported xsi:noNamespaceSchemaLocation: {}",
                lwm2m.no_ns_schema_location
            ));
        }

        if lwm2m.objects.len() != 1 {
            return Err(anyhow!(
                "invalid number of objects: {}",
                lwm2m.objects.len()
            ));
        }

        for object in &lwm2m.objects {
            if object.object_type != "MODefinition" {
                return Err(anyhow!("unsupported object type: {}", object.object_type));
            }

            if !matches!(object.lwm2m_version.as_str(), "1.0" | "1.1" | "1.2") {
                return Err(anyhow!(
                    "unsupported object lwm2m-version: {}",
                    object.lwm2m_version
                ));
            }
        }

        Ok(lwm2m.objects.remove(0))
    }

    pub fn strid(&self) -> String {
        self.name.to_case(convert_case::Case::Snake)
    }

    pub fn urn(&self) -> &str {
        &self.urn
    }
}

// The actual source is unknown, but it seems to be an adaption of this:
// https://github.com/rust-lang/rust-bindgen/blob/4f9fa49ca907b831fdc3aecdfaec36b16d03c8d8/bindgen/lib.rs#L986
//
// See LICENSE.rust-bindgen for licensing information.
pub fn rustfmt_generated_string(source: &str) -> std::io::Result<std::borrow::Cow<'_, str>> {
    let rustfmt = "rustfmt";
    let mut cmd = std::process::Command::new(rustfmt);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .arg("--edition=2018");

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let source = source.to_owned();

    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let stdin_handle = ::std::thread::spawn(move || {
        let _ = child_stdin.write_all(source.as_bytes());
        source
    });

    let mut output = vec![];
    std::io::copy(&mut child_stdout, &mut output)?;

    let status = child.wait()?;
    let source = stdin_handle.join().expect(
        "The thread writing to rustfmt's stdin doesn't do \
             anything that could panic",
    );

    match String::from_utf8(output) {
        Ok(bindings) => match status.code() {
            Some(0) => Ok(std::borrow::Cow::Owned(bindings)),
            Some(2) => Err(std::io::Error::other("Rustfmt parsing errors.".to_string())),
            Some(3) => {
                eprintln!("Rustfmt could not format some lines.");
                Ok(std::borrow::Cow::Owned(bindings))
            }
            _ => Err(std::io::Error::other("Internal rustfmt error".to_string())),
        },
        _ => Ok(std::borrow::Cow::Owned(source)),
    }
}
