//! Convert XSD files to rust structs that can be converted to/from.
//!
//! Right now, this only works for a few lemonbeat XSDs because this crate is
//! far from feature complete.
//!
//! While this crate could and should be generic and not related to lemonbeatd
//! at all - we failed horribly at achieving that goal.
//! - most functionality lives inside the `Network` trait
//! - we generate code for `go_to_sleep`

use quote::quote;
use quote::ToTokens as _;
use std::io::Write as _;

fn str2ident(string: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(string, proc_macro2::Span::call_site())
}

fn str2bytestring(string: &str) -> proc_macro2::Literal {
    proc_macro2::Literal::byte_string(string.as_bytes())
}

fn type_xsd2rust<S: AsRef<str>>(xsd: S) -> Result<&'static str, std::fmt::Error> {
    Ok(match xsd.as_ref() {
        "xs:unsignedInt" => "u32",
        "xs:unsignedLong" => "u64",
        "xs:unsignedByte" => "u8",
        "xs:int" => "i32",
        "xs:string" => "String",
        "xs:double" => "f64",
        "xs:hexBinary" => "String",
        _ => return Err(std::fmt::Error),
    })
}

fn convert_fieldname<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    match s {
        "virtual" => "virtual_".to_string(),
        other => other.to_string(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub enum Use {
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "use")]
    use_: Use,
}

impl quote::ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = str2ident(&convert_fieldname(&self.name));
        let ty = str2ident(type_xsd2rust(&self.type_).unwrap());
        let ty = match self.use_ {
            Use::Required => quote! { #ty },
            Use::Optional => quote! { Option<#ty> },
        };
        tokens.extend(quote! {
            pub #name: #ty,
        })
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Element {
    name: String,
    #[serde(rename = "minOccurs", default)]
    min_occurs: Occurs,
    #[serde(rename = "maxOccurs", default)]
    max_occurs: Occurs,
    #[serde(rename = "type")]
    type_: Option<String>,
    #[serde(rename = "complexType", default)]
    complex_type: Option<ComplexType>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    #[serde(rename = "minOccurs", default)]
    min_occurs: Occurs,
    #[serde(rename = "maxOccurs", default)]
    max_occurs: Occurs,
    #[serde(rename = "element", default)]
    elements: Vec<Element>,
    #[serde(rename = "choice", default)]
    choices: Vec<Choice>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Occurs {
    Unbounded,
    Number(usize),
}

impl Default for Occurs {
    fn default() -> Self {
        Self::Number(1)
    }
}

impl<'de> serde::Deserialize<'de> for Occurs {
    fn deserialize<D>(deserializer: D) -> Result<Occurs, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "unbounded" => Ok(Occurs::Unbounded),
            _ => Ok(Occurs::Number(s.parse().map_err(serde::de::Error::custom)?)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    #[serde(rename = "minOccurs", default)]
    min_occurs: Occurs,
    #[serde(rename = "maxOccurs", default)]
    max_occurs: Occurs,
    #[serde(rename = "element", default)]
    elements: Vec<Element>,
    #[serde(rename = "sequence", default)]
    sequences: Vec<Sequence>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extension {
    base: String,
    #[serde(rename = "attribute", default)]
    attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimpleContent {
    extension: Extension,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexType {
    pub name: Option<String>,
    #[serde(rename = "attribute", default)]
    attributes: Vec<Attribute>,
    #[serde(rename = "sequence", default)]
    sequences: Vec<Sequence>,
    #[serde(rename = "choice", default)]
    choices: Vec<Choice>,
    #[serde(rename = "simpleContent", default)]
    simple_contents: Vec<SimpleContent>,
    #[serde(default)]
    content_type_: Option<String>,
}

pub enum ComplexTypeInner<'a> {
    Sequence(&'a Sequence),
    Choice(&'a Choice),
    SimpleContent(&'a SimpleContent),
}

impl ComplexType {
    fn inner(&self) -> Option<ComplexTypeInner<'_>> {
        assert!(self.sequences.len() < 2);
        assert!(self.choices.len() < 2);
        assert!(self.simple_contents.len() < 2);

        if let Some(sequence) = self.sequences.first() {
            assert!(self.choices.is_empty());
            assert!(self.simple_contents.is_empty());
            Some(ComplexTypeInner::Sequence(sequence))
        } else if let Some(choice) = self.choices.first() {
            assert!(self.sequences.is_empty());
            assert!(self.simple_contents.is_empty());
            Some(ComplexTypeInner::Choice(choice))
        } else if let Some(simple) = self.simple_contents.first() {
            assert!(self.choices.is_empty());
            assert!(self.sequences.is_empty());
            Some(ComplexTypeInner::SimpleContent(simple))
        } else {
            None
        }
    }
}

impl quote::ToTokens for ComplexType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name_str = self.name.as_ref().unwrap();
        let name = str2ident(name_str);
        let attributes = &self.attributes;

        let inner = self.inner();
        let subtypes: Vec<_> = inner
            .as_ref()
            .map(|inner| match inner {
                ComplexTypeInner::Sequence(sequence) => sequence.elements.iter().collect(),
                ComplexTypeInner::Choice(choice) => choice.elements.iter().collect(),
                ComplexTypeInner::SimpleContent(_) => unreachable!(),
            })
            .unwrap_or_default();
        let subtype_names: Vec<_> = subtypes
            .iter()
            .map(|e| proc_macro2::Literal::byte_string(e.name.as_bytes()))
            .collect();
        let subtype_idents: Vec<_> = subtypes
            .iter()
            .map(|e| {
                let ty = e.type_.as_ref().unwrap();
                str2ident(type_xsd2rust(ty).unwrap_or(ty))
            })
            .collect();
        let attribute_matches: Vec<_> = attributes
            .iter()
            .map(|a| {
                let fieldname = convert_fieldname(&a.name);
                let ident = str2ident(&fieldname);
                let string = str2bytestring(&a.name);
                let val = quote! { ::std::str::from_utf8(&attribute.value)?.parse()? };
                let val = match &a.use_ {
                    Use::Required => val,
                    Use::Optional => quote! { Some(#val) },
                };
                quote! {
                    #string => { o.#ident = #val; },
                }
            })
            .collect();
        let attribute_toxmls: Vec<_> = attributes
            .iter()
            .map(|a| {
                let fieldname = convert_fieldname(&a.name);
                let ident = str2ident(&fieldname);
                let string = str2bytestring(&a.name);
                match &a.use_ {
                    Use::Required => quote! {
                        e.push_attribute(::quick_xml::events::attributes::Attribute::from((&#string[..], self.#ident.to_string().as_bytes())));
                    },
                    Use::Optional => quote! {
                        if let Some(v) = self.#ident.as_ref() {
                            e.push_attribute(::quick_xml::events::attributes::Attribute::from((&#string[..], v.to_string().as_bytes())));
                        }
                    },
                }
            })
            .collect();

        let (inner_enum, inner_field, inner_pushes, inner_writetags) = match inner {
            Some(ComplexTypeInner::Sequence(sequence)) => {
                if !sequence.choices.is_empty() {
                    panic!("UNSUPPORTED: sequence with choices: {sequence:#?}");
                }
                if sequence.elements.len() != 1 {
                    panic!(
                        "UNSUPPORTED: sequence with {} element(s): {:#?}",
                        sequence.elements.len(),
                        sequence
                    );
                }

                let ty = &subtype_idents[0];
                let element = sequence.elements.first().unwrap();
                let elemdoc = format!("list of `{}`", &element.name);
                let elemname = &str2bytestring(&element.name);
                (
                    quote! {},
                    quote! {
                        #[doc=#elemdoc]
                        pub inner: Vec<#ty>,
                    },
                    vec![quote! { self.inner.push(o); }],
                    quote! {
                        for v in &self.inner {
                            let mut start = ::quick_xml::events::BytesStart::borrowed_name(#elemname);
                            v.get_xml_attributes(&mut start)?;

                            if v.has_tags() {
                                writer.write_event(::quick_xml::events::Event::Start(start))?;
                                v.write_xml_tags(writer)?;
                                writer.write_event(::quick_xml::events::Event::End(::quick_xml::events::BytesEnd::borrowed(#elemname)))?;
                            } else {
                                writer.write_event(::quick_xml::events::Event::Empty(start))?;
                            }
                        }
                    },
                )
            }
            Some(ComplexTypeInner::Choice(choice)) => {
                if !choice.sequences.is_empty() {
                    panic!("UNSUPPORTED: choice with sequences: {choice:#?}");
                }

                let name = str2ident(&format!("{name}Inner"));
                let variant_names: Vec<_> =
                    choice.elements.iter().map(|e| str2ident(&e.name)).collect();
                let variant_names_bytestr: Vec<_> = choice
                    .elements
                    .iter()
                    .map(|e| str2bytestring(&e.name))
                    .collect();
                (
                    quote! {
                        #[derive(Debug, Clone, PartialEq)]
                        pub enum #name {
                            #( #variant_names(#subtype_idents), )*
                        }
                    },
                    quote! { pub inner: Vec<#name>, },
                    choice
                        .elements
                        .iter()
                        .zip(variant_names.iter())
                        .map(|(_e, variant)| quote! { self.inner.push(#name::#variant(o)); })
                        .collect(),
                    quote! {
                        for v in &self.inner {
                            match v {
                            #(
                                #name::#variant_names(v) => {
                                    let mut start = ::quick_xml::events::BytesStart::borrowed_name(#variant_names_bytestr);
                                    v.get_xml_attributes(&mut start)?;

                                    if v.has_tags() {
                                        writer.write_event(::quick_xml::events::Event::Start(start))?;
                                        v.write_xml_tags(writer)?;
                                        writer.write_event(::quick_xml::events::Event::End(
                                            ::quick_xml::events::BytesEnd::borrowed(#variant_names_bytestr)
                                        ))?;
                                    } else {
                                        writer.write_event(::quick_xml::events::Event::Empty(start))?;
                                    }
                                }
                            )*
                            }
                        }
                    },
                )
            }
            Some(ComplexTypeInner::SimpleContent(_)) => unreachable!(),
            None => (quote! {}, quote! {}, vec![], quote! {}),
        };

        let hastags = !subtypes.is_empty() || self.content_type_.is_some();

        let (value_field, handle_text_content, value_writetags) =
            if let Some(content_type) = &self.content_type_ {
                let content_type = type_xsd2rust(content_type).unwrap();
                let content_type_ident = str2ident(content_type);

                (
                    quote! {
                        pub value: #content_type_ident,
                    },
                    quote! {
                        Ok(::quick_xml::events::Event::Text(ref e)) => {
                            self.value = std::str::from_utf8(e)?.to_string();
                        },
                    },
                    quote! {
                        writer.write_event(quick_xml::events::Event::Text(
                            quick_xml::events::BytesText::from_plain(self.value.as_bytes()),
                        ))?;
                    },
                )
            } else {
                (quote! {}, quote! {}, quote! {})
            };

        tokens.extend(quote! {
            #inner_enum

            #[allow(clippy::derive_partial_eq_without_eq)] // Eq cannot be derived for all structs
            #[derive(Debug, Default, Clone, PartialEq)]
            pub struct #name {
                #(#attributes)*
                #inner_field
                #value_field
            }

            impl XmlType for #name {
                fn has_tags(&self) -> bool {
                    #hastags
                }

                fn from_xml_attributes(e: &::quick_xml::events::BytesStart<'_>) -> Result<Self, Error> {
                    let mut o = #name::default();

                    for attribute in e.attributes() {
                        let attribute = attribute?;
                        match attribute.key {
                            #(#attribute_matches)*
                            b"xmlns:xsi" | b"xmlns" | b"xsi:noNamespaceSchemaLocation" => (),
                            other => {
                                eprintln!(
                                    "[{}] unsupported attribute: {}",
                                    #name_str,
                                    ::std::str::from_utf8(other).unwrap(),
                                );
                                return Err(Error::UnsupportedAttribute);
                            }
                        }
                    }

                    Ok(o)
                }

                fn read_xml_tags<B: ::std::io::BufRead>(
                    &mut self,
                    ctx: &mut crate::ReadContext<B>,
                    tagname: &[u8],
                ) -> Result<(), Error> {
                    loop {
                        match ctx.reader.read_event(&mut ctx.buf) {
                            Ok(::quick_xml::events::Event::Start(ref e)) => match e.name() {
                                #( #subtype_names => {
                                    let mut o = #subtype_idents::from_xml_attributes(e)?;
                                    o.read_xml_tags(ctx, #subtype_names)?;
                                    #inner_pushes
                                }, )*
                                other => {
                                    eprintln!(
                                        "[{}] unsupported start element: {}",
                                        #name_str,
                                        std::str::from_utf8(other).unwrap(),
                                    );
                                    return Err(Error::UnsupportedElement);
                                }
                            },
                            Ok(::quick_xml::events::Event::Empty(ref e)) => match e.name() {
                                #( #subtype_names => {
                                    let o = #subtype_idents::from_xml_attributes(e)?;
                                    #inner_pushes
                                }, )*
                                other => {
                                    eprintln!(
                                        "[{}] unsupported empty element: {}",
                                        #name_str,
                                        std::str::from_utf8(other).unwrap(),
                                    );
                                    return Err(Error::UnsupportedElement);
                                }
                            },
                            Ok(::quick_xml::events::Event::End(e)) => {
                                if e.name() == tagname {
                                    return Ok(());
                                } else {
                                    eprintln!(
                                        "[{}] unsupported end element: {}",
                                        #name_str,
                                        std::str::from_utf8(e.name()).unwrap(),
                                    );
                                    return Err(Error::UnsupportedElement);
                                }
                            }
                            Ok(::quick_xml::events::Event::Comment(_)) => (),
                            #handle_text_content
                            other => {
                                eprintln!("[{}] unexpected event: {:?}", #name_str, other);
                                return Err(Error::UnexpectedEvent);
                            }
                        }
                        ctx.buf.clear();
                    }
                }

                fn get_xml_attributes(&self, e: &mut ::quick_xml::events::BytesStart<'_>) -> Result<(), Error> {
                    #( #attribute_toxmls )*
                    Ok(())
                }

                fn write_xml_tags<W: ::std::io::Write>(&self, writer: &mut ::quick_xml::Writer<W>) -> Result<(), Error> {
                    #inner_writetags
                    #value_writetags
                    Ok(())
                }
            }
        })
    }
}

impl std::fmt::Display for ComplexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ts = quote! {};
        self.to_tokens(&mut ts);

        f.write_str(&ts.to_string())
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Xsd {
    #[serde(rename = "targetNamespace")]
    target_namespace: String,
    xmlns: String,
    #[serde(rename = "xmlns:xs")]
    xmlns_xs: String,
    #[serde(rename = "elementFormDefault")]
    element_form_default: String,
    #[serde(rename = "complexType", default)]
    pub complex_types: Vec<ComplexType>,
    element: Element,
}

impl quote::ToTokens for Xsd {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name_str = &self.element.name;
        let name_bytestring = str2bytestring(name_str);
        let structname = str2ident(self.element.type_.as_ref().unwrap());
        let types = &self.complex_types;
        let xmlns = str2bytestring(&self.xmlns);
        tokens.extend(quote! {
            use crate::Error;
            use crate::XmlType;

            #(#types)*

            impl crate::Network for #structname {
                fn from_xml_readctx<B: ::std::io::BufRead>(ctx: &mut crate::ReadContext<B>) -> Result<Self, Error> {
                    let mut ret = None;

                    loop {
                        match ctx.reader.read_event(&mut ctx.buf) {
                            Ok(::quick_xml::events::Event::Start(ref e)) => match e.name() {
                                #name_bytestring => {
                                    if ret.is_some() {
                                        eprintln!("[root:{}] duplicate root element: {:?}", #name_str, e);
                                        return Err(Error::DuplicateRootElement);
                                    }

                                    let mut o = Self::from_xml_attributes(e)?;

                                    let mut gotns = false;
                                    for attribute in e.attributes() {
                                        let attribute = attribute?;
                                        match attribute.key {
                                            b"xmlns" => {
                                                if gotns {
                                                    return Err(Error::DuplicateXmlNamespace);
                                                }
                                                if attribute.value != &#xmlns[..] {
                                                    return Err(Error::WrongXmlNamespace);
                                                }
                                                gotns = true;
                                            }
                                            _ => (),
                                        }
                                    }
                                    if !gotns {
                                        return Err(Error::NoXmlNamespace);
                                    }

                                    o.read_xml_tags(ctx, #name_bytestring)?;
                                    ret = Some(o);
                                }
                                other => {
                                    eprintln!(
                                        "[root:{}] unsupported start element: {}",
                                        #name_str,
                                        ::std::str::from_utf8(other).unwrap()
                                    );
                                    return Err(Error::UnsupportedElement);
                                }
                            },
                            Ok(::quick_xml::events::Event::Empty(ref e)) => match e.name() {
                                other => {
                                    eprintln!(
                                        "[root:{}] unsupported empty element: {}",
                                        #name_str,
                                        ::std::str::from_utf8(other).unwrap()
                                    );
                                    return Err(Error::UnsupportedElement);
                                }
                            },
                            Ok(::quick_xml::events::Event::Eof) => {
                                break;
                            }
                            Ok(::quick_xml::events::Event::Decl(_)) => (),
                            Ok(::quick_xml::events::Event::Comment(_)) => (),
                            other => {
                                eprintln!("[root:{}] unexpected event: {:?}", #name_str, other);
                                return Err(Error::UnexpectedEvent);
                            }
                        }
                        ctx.buf.clear();
                    }
                    ret.ok_or(Error::NoRootElement)
                }

                fn to_xml_writer<W: ::std::io::Write>(&self, writer: &mut ::quick_xml::Writer<W>) -> Result<(), Error> {
                    let mut decl = ::quick_xml::events::BytesDecl::new(b"1.0", Some(b"UTF-8"), None);
                    writer.write_event(::quick_xml::events::Event::Decl(decl))?;

                    let mut start = ::quick_xml::events::BytesStart::borrowed_name(#name_bytestring);
                    start.push_attribute(::quick_xml::events::attributes::Attribute::from(
                        (&b"xmlns:xsi"[..], &b"http://www.w3.org/2001/XMLSchema-instance"[..])
                    ));
                    start.push_attribute(::quick_xml::events::attributes::Attribute::from((&b"xmlns"[..], &#xmlns[..])));
                    self.get_xml_attributes(&mut start)?;
                    writer.write_event(::quick_xml::events::Event::Start(start))?;

                    self.write_xml_tags(writer)?;

                    writer.write_event(::quick_xml::events::Event::End(::quick_xml::events::BytesEnd::borrowed(#name_bytestring)))?;
                    Ok(())
                }

                fn go_to_sleep_raw(&self) -> Option<u32> {
                    self.inner.iter().filter_map(|device_type| device_type.go_to_sleep).min()
                }

                fn set_go_to_sleep_raw(&mut self, value: Option<u32>) {
                    for device_type in &mut self.inner {
                        device_type.go_to_sleep = value;
                    }
                }
            }
        });
    }
}

impl std::fmt::Display for Xsd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ts = quote! {};
        self.to_tokens(&mut ts);

        f.write_str(&ts.to_string())
    }
}

/// rust representation of a parsed XSD.
///
/// This type implements `quote::ToTokens` for converting it to rust code.
impl Xsd {
    fn extract_complex_type(element: &mut Element) -> Option<ComplexType> {
        if let Some(mut ct) = element.complex_type.take() {
            assert!(element.type_.is_none());
            let ty = format!("{}Type", element.name);
            ct.name = Some(ty.clone());
            element.type_ = Some(ty);
            Some(ct)
        } else {
            None
        }
    }

    fn extract_ct_from_sequence(sequence: &mut Sequence, extracted_types: &mut Vec<ComplexType>) {
        for element in &mut sequence.elements {
            if let Some(ct) = Self::extract_complex_type(element) {
                extracted_types.push(ct);
            }
        }

        for choice in &mut sequence.choices {
            Self::extract_ct_from_choice(choice, extracted_types);
        }
    }

    fn extract_ct_from_choice(choice: &mut Choice, extracted_types: &mut Vec<ComplexType>) {
        for element in &mut choice.elements {
            if let Some(ct) = Self::extract_complex_type(element) {
                extracted_types.push(ct);
            }
        }

        for sequence in &mut choice.sequences {
            Self::extract_ct_from_sequence(sequence, extracted_types);
        }
    }

    pub fn from_reader<R: std::io::BufRead>(reader: R) -> Result<Self, quick_xml::de::DeError> {
        let mut xsd: Xsd = quick_xml::de::from_reader(reader)?;

        if let Some(ct) = Self::extract_complex_type(&mut xsd.element) {
            xsd.complex_types.push(ct);
        }

        loop {
            let mut extracted_types = Vec::new();

            for ct in &mut xsd.complex_types {
                for sequence in &mut ct.sequences {
                    Self::extract_ct_from_sequence(sequence, &mut extracted_types);
                }

                for choice in &mut ct.choices {
                    Self::extract_ct_from_choice(choice, &mut extracted_types);
                }
            }

            if extracted_types.is_empty() {
                break;
            }

            xsd.complex_types.append(&mut extracted_types);
        }

        for ct in &mut xsd.complex_types {
            if let Some(ComplexTypeInner::SimpleContent(_)) = ct.inner() {
                let mut simple = ct.simple_contents.pop().unwrap();
                for attribute in simple.extension.attributes.drain(..) {
                    ct.attributes.push(attribute);
                }
                ct.content_type_ = Some(simple.extension.base);
            }
        }

        // simplify XSD by removing sub-sequences and sub-choices that don't
        // change validation.
        // - simplifies our data structures
        // - works around us having to implement sub sequences/choices in our
        //   code generator
        for ct in &mut xsd.complex_types {
            loop {
                if let Some(ComplexTypeInner::Choice(choice)) = ct.inner() {
                    if choice.elements.is_empty()
                        && choice.sequences.len() == 1
                        && choice.min_occurs == Occurs::Number(1)
                        && choice.max_occurs == Occurs::Number(1)
                    {
                        std::mem::swap(&mut ct.sequences, &mut ct.choices.remove(0).sequences);
                        continue;
                    }
                }

                if let Some(ComplexTypeInner::Sequence(sequence)) = ct.inner() {
                    if sequence.elements.is_empty()
                        && sequence.choices.len() == 1
                        && sequence.min_occurs == Occurs::Number(1)
                        && sequence.max_occurs == Occurs::Number(1)
                    {
                        std::mem::swap(&mut ct.choices, &mut ct.sequences.remove(0).choices);
                        continue;
                    }
                }

                break;
            }
        }

        Ok(xsd)
    }
}

pub fn rustfmt_generated_string(source: &str) -> std::io::Result<std::borrow::Cow<'_, str>> {
    let rustfmt = "rustfmt";
    let mut cmd = std::process::Command::new(rustfmt);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());

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
