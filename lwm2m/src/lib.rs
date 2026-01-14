// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

//! Gardena-specific LWM2M implementation
//!
//! The idea here is to provide a Rust trait for every IPSO object so users can
//! simply implement those on any struct and use our API to handle requests on
//! the struct.\
//! The traits are generated using the crate `lwm2m_objgen`.
//!
//! Unfortunately this needs quite a few dynamic-dispatch calls due to many
//! trait functions being async.
//! That situation may improve when/if Rust natively starts supporting async
//! traits though.
//!
//! Since we can have multiple instances of the same resource and/or object and
//! we may also want to have different resources available depending on the
//! data inside the implementing struct we have to add instance-ID arguments to
//! all resource callbacks and implement trait functions which return the list
//! of available instances.\
//! The latter puts responsibility on the user of this crate to verify that the
//! resource implementations `know` about the same instances as the function
//! that returns the list since we can't check that at compile-time.
//!
//! SG-20455: unit-test includable-id generation

#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

pub mod bnw_consumer;
mod bnw_protocol;
mod endpoint;
mod error;
mod firmware_status;
mod pub_service;
pub mod raw;
mod rep_service;
mod request;
mod value;

// TODO: export modules instead of flat re-exporting
pub use bnw_protocol::*;
pub use endpoint::*;
pub use error::*;
pub use firmware_status::*;
pub use pub_service::*;
pub use rep_service::*;
pub use request::*;
pub use value::*;

pub mod lwm2mserver {
    pub const URL_PREFIX: &str = "/tmp/lwm2mserver";
    pub const SERVICE_NAME: &str = "lwm2mserver";
}

/// All objects generated from specifications.
#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(clippy::match_single_binding)]
#[allow(rustdoc::invalid_html_tags)]
pub mod objects {
    use super::CoreLink;
    use super::Error;
    use super::Object;
    use super::ObjectLink;
    use super::TimedData;
    use super::Value;
    use super::ValueData;
    use std::convert::TryInto;

    include!(concat!(env!("OUT_DIR"), "/objects.rs"));
}

include!(concat!(env!("OUT_DIR"), "/misc.rs"));
