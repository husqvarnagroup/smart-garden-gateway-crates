// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

//! Safe Rust bindings to liblsdl-serializer
//!
//! This crate doesn't just provide safe wrappers around the C functions.
//! It also provides Rust types for all lemonbeat payloads you can find in the
//! specification. These were generated using `xsd2rust`.

use std::convert::TryInto as _;

/// holds all types generated from the XSD specifications
pub mod xsd {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(unused_mut)]
    #![allow(unused_variables)]
    #![allow(clippy::match_single_binding)]
    #![allow(clippy::print_literal)]
    #![allow(clippy::single_match)]
    #![allow(unused_imports)]

    include!(concat!(env!("OUT_DIR"), "/xsd.rs"));
}

/// context to be used when decoding xml
pub struct ReadContext<B: std::io::BufRead> {
    pub reader: quick_xml::Reader<B>,
    pub buf: Vec<u8>,
}

/// implemented by types that can be converted to/from XML
trait XmlType {
    /// returns true if this type supports xml tags
    fn has_tags(&self) -> bool;

    /// parse XML attributes and return a new instance of this type
    fn from_xml_attributes(e: &::quick_xml::events::BytesStart<'_>) -> Result<Self, Error>
    where
        Self: Sized;

    /// parse XML tags into this type
    fn read_xml_tags<B: ::std::io::BufRead>(
        &mut self,
        ctx: &mut crate::ReadContext<B>,
        tagname: &[u8],
    ) -> Result<(), Error>;

    /// convert this type to XML attributes
    fn get_xml_attributes(&self, e: &mut ::quick_xml::events::BytesStart<'_>) -> Result<(), Error>;

    /// convert this type to XML tags
    fn write_xml_tags<W: ::std::io::Write>(
        &self,
        writer: &mut ::quick_xml::Writer<W>,
    ) -> Result<(), Error>;
}

/// implemented by all XSD network types
pub trait Network {
    /// decode XML into this network type
    fn from_xml_readctx<B: ::std::io::BufRead>(
        ctx: &mut crate::ReadContext<B>,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    /// encode this network type as XML
    fn to_xml_writer<W: ::std::io::Write>(
        &self,
        writer: &mut ::quick_xml::Writer<W>,
    ) -> Result<(), Error>;

    /// returns the lowest value of all inner device messages, if any
    fn go_to_sleep_raw(&self) -> Option<u32>;

    fn go_to_sleep(&self) -> Option<std::time::Duration> {
        self.go_to_sleep_raw()
            .map(|millis| std::time::Duration::from_millis(millis.into()))
    }

    fn set_go_to_sleep_raw(&mut self, value: Option<u32>);

    fn set_go_to_sleep(&mut self, duration: Option<std::time::Duration>) -> Result<(), Error> {
        self.set_go_to_sleep_raw(if let Some(duration) = duration {
            Some(duration.as_millis().try_into()?)
        } else {
            None
        });
        Ok(())
    }
}

impl XmlType for String {
    fn has_tags(&self) -> bool {
        true
    }

    fn from_xml_attributes(_e: &::quick_xml::events::BytesStart<'_>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(String::new())
    }

    fn read_xml_tags<B: ::std::io::BufRead>(
        &mut self,
        ctx: &mut crate::ReadContext<B>,
        tagname: &[u8],
    ) -> Result<(), Error> {
        loop {
            match ctx.reader.read_event(&mut ctx.buf) {
                Ok(::quick_xml::events::Event::End(e)) => {
                    if e.name() == tagname {
                        return Ok(());
                    } else {
                        return Err(Error::UnsupportedElement);
                    }
                }
                Ok(::quick_xml::events::Event::Text(ref e)) => {
                    self.push_str(std::str::from_utf8(e)?)
                }
                Ok(::quick_xml::events::Event::Comment(_)) => (),
                other => {
                    eprintln!("[statusReportType] unexpected event: {other:?}");
                    return Err(Error::UnexpectedEvent);
                }
            }
            ctx.buf.clear();
        }
    }

    fn get_xml_attributes(
        &self,
        _e: &mut ::quick_xml::events::BytesStart<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn write_xml_tags<W: ::std::io::Write>(
        &self,
        writer: &mut ::quick_xml::Writer<W>,
    ) -> Result<(), Error> {
        writer.write_event(quick_xml::events::Event::Text(
            quick_xml::events::BytesText::from_plain(self.as_bytes()),
        ))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error(transparent)]
    QuickXml(#[from] quick_xml::Error),
    #[error(transparent)]
    QuickXmlAttr(#[from] quick_xml::events::attributes::AttrError),

    #[error("native LSDL error: {0}")]
    Native(::std::os::raw::c_ulong),
    #[error("unsupported element")]
    UnsupportedElement,
    #[error("unsupported attribute")]
    UnsupportedAttribute,
    #[error("unexpected event")]
    UnexpectedEvent,
    #[error("duplicate root element")]
    DuplicateRootElement,
    #[error("no root element")]
    NoRootElement,
    #[error("wrong xml namespace")]
    WrongXmlNamespace,
    #[error("no xml namespace")]
    NoXmlNamespace,
    #[error("duplicate xml namespace")]
    DuplicateXmlNamespace,
}

/// Trait for getting port info without knowing the network type.
///
/// Each network type is usually communicated to/from a UDP server
/// listening on a certain port.
pub trait NetworkPort {
    /// return port for this service.
    fn get_port() -> u16;
}

macro_rules! impl_networkport {
    ($ty:ident, $port:expr) => {
        impl NetworkPort for xsd::$ty::networkType {
            fn get_port() -> u16 {
                $port
            }
        }
    };
}

impl_networkport!(value, 20000);
impl_networkport!(device_description, 20001);
impl_networkport!(public_key, 20002);
impl_networkport!(network_management, 20003);
impl_networkport!(value_description, 20004);
impl_networkport!(service_description, 20005);
impl_networkport!(memory_information, 20006);
impl_networkport!(partner_information, 20007);
impl_networkport!(action, 20008);
//impl_networkport!(calculation, 20009);
impl_networkport!(timer, 20010);
impl_networkport!(calendar, 20011);
//impl_networkport!(state_machine, 20012);
impl_networkport!(firmware_update, 20013);
//impl_networkport!(channel_scan, 20014);
impl_networkport!(status, 20015);
impl_networkport!(configuration, 20016);

/// safe wrapper for lsdl-serializer's `compressXML`
pub fn compress_xml(
    port: u16,
    xml: &[u8],
    exi: &mut [u8],
    bit_offset: u64,
) -> Result<usize, Error> {
    let rc = unsafe {
        // the types may be different on other platforms
        #[allow(clippy::useless_conversion)]
        lsdl_sys::compressXML(
            port.try_into()?,
            xml.as_ptr() as *mut ::std::os::raw::c_uchar,
            xml.len().try_into()?,
            exi.as_mut_ptr(),
            exi.len().try_into()?,
            bit_offset.try_into()?,
        )
    };
    let rcusize: usize = rc.try_into()?;
    if rc == 0 || rcusize >= exi.len() {
        return Err(Error::Native(rc));
    }

    Ok(rcusize)
}

/// safe wrapper for lsdl-serializer's `decompressEXI`
pub fn decompress_exi(
    port: u16,
    exi: &[u8],
    xml: &mut [u8],
    bit_offset: u64,
) -> Result<usize, Error> {
    let rc = unsafe {
        // the types may be different on other platforms
        #[allow(clippy::useless_conversion)]
        lsdl_sys::decompressEXI(
            port.try_into()?,
            exi.as_ptr() as *mut ::std::os::raw::c_uchar,
            exi.len().try_into()?,
            xml.as_mut_ptr(),
            xml.len().try_into()?,
            bit_offset.try_into()?,
        )
    };
    let rcusize: usize = rc.try_into()?;
    if rc == 0 || rcusize >= xml.len() {
        return Err(Error::Native(rc));
    }

    Ok(rcusize)
}

/// safe wrapper for lsdl-serializer's `lsdlconv_getVersion`
pub fn get_version() -> &'static str {
    let version = unsafe { lsdl_sys::lsdlconv_getVersion() };
    assert!(!version.is_null());
    unsafe { std::ffi::CStr::from_ptr(version) }
        .to_str()
        .unwrap()
}

/// construct a new device message
#[macro_export]
macro_rules! device_message {
    ($ty:ident, $device_id:expr, $go_to_sleep:expr, $inner:expr) => {{
        // this is needed to prevent evaluating them within the closure which
        // has a different return type
        let device_id = $device_id;
        let go_to_sleep: Option<std::time::Duration> = $go_to_sleep;
        let inner = $inner;

        || -> ::anyhow::Result<lsdl::xsd::$ty::networkType> {
            Ok(lsdl::xsd::$ty::networkType {
                version: 1,
                inner: ::std::vec![lsdl::xsd::$ty::deviceType {
                    version: 1,
                    device_id: device_id,
                    go_to_sleep: {
                        if let Some(duration) = go_to_sleep {
                            Some(
                                duration
                                    .as_millis()
                                    .try_into()
                                    .context("can't convert go_to_sleep to millis")?,
                            )
                        } else {
                            None
                        }
                    },
                    inner,
                }],
            })
        }()
    }};
}

/// get all results from a lemonbeat response
///
/// This does the following things:
/// - check version fields
/// - check that all `inner` arrays contain exactly 1 element
/// - return `Error::MalformedLsdlResponse` if anything doesn't match
#[macro_export]
macro_rules! get_responses {
    ($network:ident, $ty:ident) => {
        if $network.version != 1 {
            Err(anyhow!("unsupported network version: {}", $network.version))
        } else if $network.inner.len() != 1 {
            Err(anyhow!("expected 1 device, got {}", $network.inner.len()))
        } else {
            let devicetype = &mut $network.inner[0];
            if devicetype.version != 1 {
                Err(anyhow!(
                    "unsupported network device version: {}",
                    devicetype.version
                ))
            } else {
                Ok(&mut devicetype.inner)
            }
        }
    };
}

/// get result from a lemonbeat response
///
/// This does the following things:
/// - check version fields
/// - check that all `inner` arrays contain exactly 1 element
/// - match for and return `lsdl::xsd::$ty::deviceTypeInner::$variant`
/// - return `Error::MalformedLsdlResponse` if anything doesn't match
#[macro_export]
macro_rules! get_response {
    ($network:ident, $ty:ident, $variant:ident) => {
        if $network.version != 1 {
            Err(anyhow!("unsupported network version: {}", $network.version))
        } else if $network.inner.len() != 1 {
            Err(anyhow!("expected 1 device, got {}", $network.inner.len()))
        } else {
            let devicetype = &mut $network.inner[0];
            if devicetype.version != 1 {
                Err(anyhow!(
                    "unsupported network device version: {}",
                    devicetype.version
                ))
            } else if devicetype.inner.len() != 1 {
                Err(anyhow!("expected 1 answer, got {}", devicetype.inner.len()))
            } else {
                let inner = &mut devicetype.inner[0];
                match inner {
                    lsdl::xsd::$ty::deviceTypeInner::$variant(v) => Ok(v),
                    _ => Err(anyhow!("unsupported answer variant")),
                }
            }
        }
    };
}

mod types;
pub use types::*;
