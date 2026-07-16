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

    /* The following code in this 'objects' module was originally generated
    by 'build.rs'. No future changes regarding supported objects are expected.
    Therefore, the generated code is added here directly.
    This also removes the dependency on the IPSO registries. */

    // + "../third_party/lwm2m-registry/version_history/3-1_1.xml"
    #[allow(clippy::doc_markdown)]
    #[doc = "This LwM2M Object provides a range of device related information which can be queried by the LwM2M Server, and a device reboot and factory reset function.\n\n"]
    #[::async_trait::async_trait]
    pub trait Device {
        #[doc = "Human readable manufacturer name"]
        async fn manufacturer(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "A model identifier (manufacturer specified string)"]
        async fn model_number(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Serial Number"]
        async fn serial_number(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Current firmware version of the Device.The Firmware Management function could rely on this resource."]
        async fn firmware_version(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Reboot the LwM2M Device to restore the Device from unexpected firmware failure."]
        async fn reboot(
            &mut self,
            _object: usize,
            _resource: usize,
            _args: Option<Vec<String>>,
        ) -> Result<(), Error>;
        #[doc = "Perform factory reset of the LwM2M Device to make the LwM2M Device to go through initial deployment sequence where provisioning and bootstrap sequence is performed. This requires client ensuring post factory reset to have minimal information to allow it to carry out one of the bootstrap methods specified in section 5.2.3. \r\nWhen this Resource is executed, \"De-register\" operation MAY be sent to the LwM2M Server(s) before factory reset of the LwM2M Device."]
        async fn factory_reset(
            &mut self,
            _object: usize,
            _resource: usize,
            _args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "0: DC power\r\n1: Internal Battery\r\n2: External Battery\r\n3: Fuel Cell\r\n4: Power over Ethernet\r\n5: USB\r\n6: AC (Mains) power\r\n7: Solar\r\nThe same Resource Instance ID MUST be used to associate a given Power Source (Resource ID:6) with its Present Voltage (Resource ID:7) and its Present Current (Resource ID:8)"]
        async fn available_power_sources(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Present voltage for each Available Power Sources Resource Instance. The unit used for this resource is in mV."]
        async fn power_source_voltage(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Present current for each Available Power Source. The unit used for this resource is in mA."]
        async fn power_source_current(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Contains the current battery level as a percentage (with a range from 0 to 100). This value is only valid for the Device internal Battery if present (one Available Power Sources Resource Instance is 1)."]
        async fn battery_level(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Estimated current available amount of storage space which can store data and software in the LwM2M Device (expressed in kilobytes)."]
        async fn memory_free(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "0=No error\r\n1=Low battery power\r\n2=External power supply off\r\n3=GPS module failure\r\n4=Low received signal strength\r\n5=Out of memory\r\n6=SMS failure\r\n7=IP connectivity failure\r\n8=Peripheral malfunction\r\n\r\nWhen the single Device Object Instance is initiated, there is only one error code Resource Instance whose value is equal to 0 that means no error. When the first error happens, the LwM2M Client changes error code Resource Instance to any non-zero value to indicate the error type. When any other error happens, a new error code Resource Instance is created. When an error associated with a Resource Instance is no longer present, that Resource Instance is deleted. When the single existing error is no longer present, the LwM2M Client returns to the original no error state where Instance 0 has value 0.\r\nThis error code Resource MAY be observed by the LwM2M Server. How to deal with LwM2M Client’s error report depends on the policy of the LwM2M Server."]
        async fn error_code(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error>;
        #[doc = "Delete all error code Resource Instances and create only one zero-value error code that implies no error, then re-evaluate all error conditions and update and create Resources Instances to capture all current error conditions."]
        async fn reset_error_code(
            &mut self,
            _object: usize,
            _resource: usize,
            _args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Current UNIX time of the LwM2M Client.\r\nThe LwM2M Client should be responsible to increase this time value as every second elapses.\r\nThe LwM2M Server is able to write this Resource to make the LwM2M Client synchronized with the LwM2M Server."]
        async fn current_time(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<std::time::SystemTime>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Current UNIX time of the LwM2M Client.\r\nThe LwM2M Client should be responsible to increase this time value as every second elapses.\r\nThe LwM2M Server is able to write this Resource to make the LwM2M Client synchronized with the LwM2M Server."]
        async fn set_current_time(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: std::time::SystemTime,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Indicates the UTC offset currently in effect for this LwM2M Device. UTC+X [ISO 8601]."]
        async fn utc_offset(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Indicates the UTC offset currently in effect for this LwM2M Device. UTC+X [ISO 8601]."]
        async fn set_utc_offset(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: String,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Indicates in which time zone the LwM2M Device is located, in IANA Timezone (TZ) database format."]
        async fn timezone(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Indicates in which time zone the LwM2M Device is located, in IANA Timezone (TZ) database format."]
        async fn set_timezone(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: String,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Indicates which bindings and modes are supported in the LwM2M Client. The possible values are those listed in the LwM2M Core Specification."]
        async fn supported_binding_and_modes(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error>;
        #[doc = "Type of the device (manufacturer specified string: e.g. smart meters / dev Class / ...)"]
        async fn device_type(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Current hardware version of the device"]
        async fn hardware_version(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Current software version of the device (manufacturer specified string). On elaborated LwM2M device, SW could be split in 2 parts: a firmware one and a higher level software on top.\r\nBoth pieces of Software are together managed by LwM2M Firmware Update Object (Object ID 5)"]
        async fn software_version(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "This value is only valid for the Device Internal Battery if present (one Available Power Sources Resource Instance value is 1).\r\nBattery\r\nStatus\tMeaning\tDescription\r\n0\tNormal\tThe battery is operating normally and not on power.\r\n1\tCharging\tThe battery is currently charging.\r\n2\tCharge Complete\tThe battery is fully charged and still on power.\r\n3\tDamaged\tThe battery has some problem.\r\n4\tLow Battery\tThe battery is low on charge.\r\n5\tNot Installed\tThe battery is not installed.\r\n6\tUnknown\tThe battery information is not available."]
        async fn battery_status(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Total amount of storage space which can store data and software in the LwM2M Device (expressed in kilobytes)."]
        async fn memory_total(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Reference to external \"Device\" object instance containing information. For example, such an external device can be a Host Device, which is a device into which the Device containing the LwM2M client is embedded. This Resource may be used to retrieve information about the Host Device."]
        async fn ext_dev_info(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<ObjectLink>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        async fn handle_partial_write(
            &mut self,
            _object_instance: usize,
            _values: std::collections::HashMap<String, Value>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedPartialWrite)
        }
    }
    pub const DEVICE_MANUFACTURER: usize = 0usize;
    pub const DEVICE_MODEL_NUMBER: usize = 1usize;
    pub const DEVICE_SERIAL_NUMBER: usize = 2usize;
    pub const DEVICE_FIRMWARE_VERSION: usize = 3usize;
    pub const DEVICE_REBOOT: usize = 4usize;
    pub const DEVICE_FACTORY_RESET: usize = 5usize;
    pub const DEVICE_AVAILABLE_POWER_SOURCES: usize = 6usize;
    pub const DEVICE_POWER_SOURCE_VOLTAGE: usize = 7usize;
    pub const DEVICE_POWER_SOURCE_CURRENT: usize = 8usize;
    pub const DEVICE_BATTERY_LEVEL: usize = 9usize;
    pub const DEVICE_MEMORY_FREE: usize = 10usize;
    pub const DEVICE_ERROR_CODE: usize = 11usize;
    pub const DEVICE_RESET_ERROR_CODE: usize = 12usize;
    pub const DEVICE_CURRENT_TIME: usize = 13usize;
    pub const DEVICE_UTC_OFFSET: usize = 14usize;
    pub const DEVICE_TIMEZONE: usize = 15usize;
    pub const DEVICE_SUPPORTED_BINDING_AND_MODES: usize = 16usize;
    pub const DEVICE_DEVICE_TYPE: usize = 17usize;
    pub const DEVICE_HARDWARE_VERSION: usize = 18usize;
    pub const DEVICE_SOFTWARE_VERSION: usize = 19usize;
    pub const DEVICE_BATTERY_STATUS: usize = 20usize;
    pub const DEVICE_MEMORY_TOTAL: usize = 21usize;
    pub const DEVICE_EXT_DEV_INFO: usize = 22usize;
    pub struct DeviceHandler<'a, T> {
        t: &'a mut T,
    }
    impl<'a, T: Send + Sync + Device> DeviceHandler<'a, T> {
        pub fn new(t: &'a mut T) -> Self {
            Self { t }
        }
    }
    #[::async_trait::async_trait]
    impl<T: Send + Sync + Device> Object for DeviceHandler<'_, T> {
        fn urn(&self) -> &'static str {
            "urn:oma:lwm2m:oma:3:1.1"
        }
        async fn read_resource(
            &self,
            object_instance: usize,
            id: usize,
            instance: usize,
        ) -> Result<Value, Error> {
            match id {
                DEVICE_MANUFACTURER => {
                    Ok(self.t.manufacturer(object_instance, instance).await?.into())
                }
                DEVICE_MODEL_NUMBER => {
                    Ok(self.t.model_number(object_instance, instance).await?.into())
                }
                DEVICE_SERIAL_NUMBER => Ok(self
                    .t
                    .serial_number(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_FIRMWARE_VERSION => Ok(self
                    .t
                    .firmware_version(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_AVAILABLE_POWER_SOURCES => Ok(self
                    .t
                    .available_power_sources(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_POWER_SOURCE_VOLTAGE => Ok(self
                    .t
                    .power_source_voltage(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_POWER_SOURCE_CURRENT => Ok(self
                    .t
                    .power_source_current(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_BATTERY_LEVEL => Ok(self
                    .t
                    .battery_level(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_MEMORY_FREE => {
                    Ok(self.t.memory_free(object_instance, instance).await?.into())
                }
                DEVICE_ERROR_CODE => Ok(self.t.error_code(object_instance, instance).await?.into()),
                DEVICE_CURRENT_TIME => {
                    Ok(self.t.current_time(object_instance, instance).await?.into())
                }
                DEVICE_UTC_OFFSET => Ok(self.t.utc_offset(object_instance, instance).await?.into()),
                DEVICE_TIMEZONE => Ok(self.t.timezone(object_instance, instance).await?.into()),
                DEVICE_SUPPORTED_BINDING_AND_MODES => Ok(self
                    .t
                    .supported_binding_and_modes(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_DEVICE_TYPE => {
                    Ok(self.t.device_type(object_instance, instance).await?.into())
                }
                DEVICE_HARDWARE_VERSION => Ok(self
                    .t
                    .hardware_version(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_SOFTWARE_VERSION => Ok(self
                    .t
                    .software_version(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_BATTERY_STATUS => Ok(self
                    .t
                    .battery_status(object_instance, instance)
                    .await?
                    .into()),
                DEVICE_MEMORY_TOTAL => {
                    Ok(self.t.memory_total(object_instance, instance).await?.into())
                }
                DEVICE_EXT_DEV_INFO => {
                    Ok(self.t.ext_dev_info(object_instance, instance).await?.into())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "read: unknown resource id `{id}`"
                ))),
            }
        }
        async fn write_resource(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            value: Value,
        ) -> Result<(), Error> {
            match id {
                DEVICE_CURRENT_TIME => {
                    self.t
                        .set_current_time(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                DEVICE_UTC_OFFSET => {
                    self.t
                        .set_utc_offset(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                DEVICE_TIMEZONE => {
                    self.t
                        .set_timezone(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        async fn exec(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            match id {
                DEVICE_REBOOT => {
                    self.t.reboot(object_instance, instance, args).await?;
                    Ok(())
                }
                DEVICE_FACTORY_RESET => {
                    self.t
                        .factory_reset(object_instance, instance, args)
                        .await?;
                    Ok(())
                }
                DEVICE_RESET_ERROR_CODE => {
                    self.t
                        .reset_error_code(object_instance, instance, args)
                        .await?;
                    Ok(())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        fn parse_resource_name(&self, name: &str) -> Result<usize, Error> {
            match name {
                "manufacturer" => Ok(0usize),
                "model_number" => Ok(1usize),
                "serial_number" => Ok(2usize),
                "firmware_version" => Ok(3usize),
                "reboot" => Ok(4usize),
                "factory_reset" => Ok(5usize),
                "available_power_sources" => Ok(6usize),
                "power_source_voltage" => Ok(7usize),
                "power_source_current" => Ok(8usize),
                "battery_level" => Ok(9usize),
                "memory_free" => Ok(10usize),
                "error_code" => Ok(11usize),
                "reset_error_code" => Ok(12usize),
                "current_time" => Ok(13usize),
                "utc_offset" => Ok(14usize),
                "timezone" => Ok(15usize),
                "supported_binding_and_modes" => Ok(16usize),
                "device_type" => Ok(17usize),
                "hardware_version" => Ok(18usize),
                "software_version" => Ok(19usize),
                "battery_status" => Ok(20usize),
                "memory_total" => Ok(21usize),
                "ext_dev_info" => Ok(22usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource name `{name}`"
                ))),
            }
        }
        fn get_resource_name(&self, id: usize) -> Result<&str, Error> {
            match id {
                0usize => Ok("manufacturer"),
                1usize => Ok("model_number"),
                2usize => Ok("serial_number"),
                3usize => Ok("firmware_version"),
                4usize => Ok("reboot"),
                5usize => Ok("factory_reset"),
                6usize => Ok("available_power_sources"),
                7usize => Ok("power_source_voltage"),
                8usize => Ok("power_source_current"),
                9usize => Ok("battery_level"),
                10usize => Ok("memory_free"),
                11usize => Ok("error_code"),
                12usize => Ok("reset_error_code"),
                13usize => Ok("current_time"),
                14usize => Ok("utc_offset"),
                15usize => Ok("timezone"),
                16usize => Ok("supported_binding_and_modes"),
                17usize => Ok("device_type"),
                18usize => Ok("hardware_version"),
                19usize => Ok("software_version"),
                20usize => Ok("battery_status"),
                21usize => Ok("memory_total"),
                22usize => Ok("ext_dev_info"),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn supported_resource_operations(&self, id: usize) -> Result<usize, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                0usize => Ok(1usize),
                1usize => Ok(1usize),
                2usize => Ok(1usize),
                3usize => Ok(1usize),
                4usize => Ok(20usize),
                5usize => Ok(20usize),
                6usize => Ok(1usize),
                7usize => Ok(1usize),
                8usize => Ok(1usize),
                9usize => Ok(1usize),
                10usize => Ok(1usize),
                11usize => Ok(1usize),
                12usize => Ok(20usize),
                13usize => Ok(3usize),
                14usize => Ok(3usize),
                15usize => Ok(3usize),
                16usize => Ok(1usize),
                17usize => Ok(1usize),
                18usize => Ok(1usize),
                19usize => Ok(1usize),
                20usize => Ok(1usize),
                21usize => Ok(1usize),
                22usize => Ok(1usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn is_array_resource(&self, id: usize) -> Result<bool, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                0usize => Ok(false),
                1usize => Ok(false),
                2usize => Ok(false),
                3usize => Ok(false),
                4usize => Ok(false),
                5usize => Ok(false),
                6usize => Ok(true),
                7usize => Ok(true),
                8usize => Ok(true),
                9usize => Ok(false),
                10usize => Ok(false),
                11usize => Ok(true),
                12usize => Ok(false),
                13usize => Ok(false),
                14usize => Ok(false),
                15usize => Ok(false),
                16usize => Ok(false),
                17usize => Ok(false),
                18usize => Ok(false),
                19usize => Ok(false),
                20usize => Ok(false),
                21usize => Ok(false),
                22usize => Ok(true),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
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
    pub fn make_device_json(
        time: Option<std::time::SystemTime>,
        manufacturer: Option<String>,
        model_number: Option<String>,
        serial_number: Option<String>,
        firmware_version: Option<String>,
        available_power_sources: Vec<Option<i64>>,
        power_source_voltage: Vec<Option<i64>>,
        power_source_current: Vec<Option<i64>>,
        battery_level: Option<i64>,
        memory_free: Option<i64>,
        error_code: Vec<Option<i64>>,
        current_time: Option<std::time::SystemTime>,
        utc_offset: Option<String>,
        timezone: Option<String>,
        supported_binding_and_modes: String,
        device_type: Option<String>,
        hardware_version: Option<String>,
        software_version: Option<String>,
        battery_status: Option<i64>,
        memory_total: Option<i64>,
        ext_dev_info: Vec<Option<ObjectLink>>,
    ) -> ::serde_json::Value {
        let mut res = std::collections::HashMap::<&str, Value>::new();
        if let Some(manufacturer) = manufacturer {
            res.insert(
                "Manufacturer",
                Value {
                    data: ValueData::from(manufacturer),
                    time,
                },
            );
        }
        if let Some(model_number) = model_number {
            res.insert(
                "Model Number",
                Value {
                    data: ValueData::from(model_number),
                    time,
                },
            );
        }
        if let Some(serial_number) = serial_number {
            res.insert(
                "Serial Number",
                Value {
                    data: ValueData::from(serial_number),
                    time,
                },
            );
        }
        if let Some(firmware_version) = firmware_version {
            res.insert(
                "Firmware Version",
                Value {
                    data: ValueData::from(firmware_version),
                    time,
                },
            );
        }
        res.insert(
            "Available Power Sources",
            Value {
                data: ValueData::from(available_power_sources),
                time,
            },
        );
        res.insert(
            "Power Source Voltage",
            Value {
                data: ValueData::from(power_source_voltage),
                time,
            },
        );
        res.insert(
            "Power Source Current",
            Value {
                data: ValueData::from(power_source_current),
                time,
            },
        );
        if let Some(battery_level) = battery_level {
            res.insert(
                "Battery Level",
                Value {
                    data: ValueData::from(battery_level),
                    time,
                },
            );
        }
        if let Some(memory_free) = memory_free {
            res.insert(
                "Memory Free",
                Value {
                    data: ValueData::from(memory_free),
                    time,
                },
            );
        }
        res.insert(
            "Error Code",
            Value {
                data: ValueData::from(error_code),
                time,
            },
        );
        if let Some(current_time) = current_time {
            res.insert(
                "Current Time",
                Value {
                    data: ValueData::from(current_time),
                    time,
                },
            );
        }
        if let Some(utc_offset) = utc_offset {
            res.insert(
                "UTC Offset",
                Value {
                    data: ValueData::from(utc_offset),
                    time,
                },
            );
        }
        if let Some(timezone) = timezone {
            res.insert(
                "Timezone",
                Value {
                    data: ValueData::from(timezone),
                    time,
                },
            );
        }
        res.insert(
            "Supported Binding and Modes",
            Value {
                data: ValueData::from(supported_binding_and_modes),
                time,
            },
        );
        if let Some(device_type) = device_type {
            res.insert(
                "Device Type",
                Value {
                    data: ValueData::from(device_type),
                    time,
                },
            );
        }
        if let Some(hardware_version) = hardware_version {
            res.insert(
                "Hardware Version",
                Value {
                    data: ValueData::from(hardware_version),
                    time,
                },
            );
        }
        if let Some(software_version) = software_version {
            res.insert(
                "Software Version",
                Value {
                    data: ValueData::from(software_version),
                    time,
                },
            );
        }
        if let Some(battery_status) = battery_status {
            res.insert(
                "Battery Status",
                Value {
                    data: ValueData::from(battery_status),
                    time,
                },
            );
        }
        if let Some(memory_total) = memory_total {
            res.insert(
                "Memory Total",
                Value {
                    data: ValueData::from(memory_total),
                    time,
                },
            );
        }
        res.insert(
            "ExtDevInfo",
            Value {
                data: ValueData::from(ext_dev_info),
                time,
            },
        );
        let mut json = ::serde_json::json!(res);
        json.as_object_mut()
            .unwrap()
            .insert("_urn".to_string(), "urn:oma:lwm2m:oma:3:1.1".into());
        json
    }

    // - "../third_party/lwm2m-registry/version_history/3-1_1.xml"

    // + "../third_party/lwm2m-registry/version_history/5-1_1.xml"
    #[allow(clippy::doc_markdown)]
    #[doc = "This LwM2M Object enables management of firmware which is to be updated. This Object includes installing a firmware package, updating firmware, and performing actions after updating firmware. The firmware update MAY require to reboot the device; it will depend on a number of factors, such as the operating system architecture and the extent of the updated software.\r\nThe envisioned functionality is to allow a LwM2M Client to connect to any LwM2M Server to obtain a firmware image using the object and resource structure defined in this section experiencing communication security protection using TLS/DTLS. There are, however, other design decisions that need to be taken into account to allow a manufacturer of a device to securely install firmware on a device. Examples for such design decisions are how to manage the firmware update repository at the server side (which may include user interface considerations), the techniques to provide additional application layer security protection of the firmware image, how many versions of firmware images to store on the device, and how to execute the firmware update process considering the hardware specific details of a given IoT hardware product. These aspects are considered to be outside the scope of this version of the specification.\r\nA LwM2M Server may also instruct a LwM2M Client to fetch a firmware image from a dedicated server (instead of pushing firmware images to the LwM2M Client). The Package URI resource is contained in the Firmware object and can be used for this purpose.\r\nA LwM2M Client MUST support block-wise transfer (CoAP_Blockwise) if it implements the Firmware Update object.\r\nA LwM2M Server MUST support block-wise transfer. Other protocols, such as HTTP/HTTPs, MAY also be used for downloading firmware updates (via the Package URI resource). For constrained devices it is, however, RECOMMENDED to use CoAP for firmware downloads to avoid the need for additional protocol implementations.\n\n"]
    #[::async_trait::async_trait]
    pub trait FirmwareUpdate {
        #[doc = "Firmware package"]
        async fn set_package(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: Vec<u8>,
        ) -> Result<(), Error>;
        #[doc = "URI from where the device can download the firmware package by an alternative mechanism. As soon the device has received the Package URI it performs the download at the next practical opportunity. \r\nThe URI format is defined in RFC 3986. For example, coaps://example.org/firmware is a syntactically valid URI. The URI scheme determines the protocol to be used. For CoAP this endpoint MAY be a LwM2M Server but does not necessarily need to be. A CoAP server implementing block-wise transfer is sufficient as a server hosting a firmware repository and the expectation is that this server merely serves as a separate file server making firmware images available to LwM2M Clients."]
        async fn package_uri(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error>;
        #[doc = "URI from where the device can download the firmware package by an alternative mechanism. As soon the device has received the Package URI it performs the download at the next practical opportunity. \r\nThe URI format is defined in RFC 3986. For example, coaps://example.org/firmware is a syntactically valid URI. The URI scheme determines the protocol to be used. For CoAP this endpoint MAY be a LwM2M Server but does not necessarily need to be. A CoAP server implementing block-wise transfer is sufficient as a server hosting a firmware repository and the expectation is that this server merely serves as a separate file server making firmware images available to LwM2M Clients."]
        async fn set_package_uri(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: String,
        ) -> Result<(), Error>;
        #[doc = "Updates firmware by using the firmware package stored in Package, or, by using the firmware downloaded from the Package URI.\r\nThis Resource is only executable when the value of the State Resource is Downloaded."]
        async fn update(
            &mut self,
            _object: usize,
            _resource: usize,
            _args: Option<Vec<String>>,
        ) -> Result<(), Error>;
        #[doc = "Indicates current state with respect to this firmware update. This value is set by the LwM2M Client.\r\n0: Idle (before downloading or after successful updating)\r\n1: Downloading (The data sequence is on the way)\r\n2: Downloaded\r\n3: Updating\r\nIf writing the firmware package to Package Resource has completed, or, if the device has downloaded the firmware package from the Package URI the state changes to Downloaded.\r\nWriting an empty string to Package URI Resource or setting the Package Resource to NULL (‘\\0’), resets the Firmware Update State Machine: the State Resource value is set to Idle and the Update Result Resource value is set to 0.\r\nThe device should remove the downloaded firmware image when the state is reset to Idle.\r\nWhen in Downloaded state, and the executable Resource Update is triggered, the state changes to Updating if the update starts immediately. For devices that support a user interface and the deferred update functionality, the user may be allowed to defer the firmware update to a later time. In this case, the state stays in Downloaded state and the Update Result is set to 11. Once a user accepted the firmware update, the state changes to Updating.\r\nWhen the user deferred the update, the device will continue operations normally until the user approves the firmware update or an automatic update starts. It will not block any operation on the device.\r\nIf the Update Resource failed, the state may return to either Downloaded or Idle depending on the underlying reason of update failure, e.g. Integrity Check Failure results in the client moving to the Idle state.\r\nIf performing the Update or Cancel operation was successful, the state changes to Idle. \r\nThe firmware update state machine is illustrated in the respective LwM2M specification."]
        async fn state(&self, _object: usize, _resource: usize) -> Result<TimedData<i64>, Error>;
        #[doc = "Contains the result of downloading or updating the firmware\r\n0: Initial value. Once the updating process is initiated (Download /Update), this Resource MUST be reset to Initial value.\r\n1: Firmware updated successfully.\r\n2: Not enough flash memory for the new firmware package.\r\n3: Out of RAM during downloading process.\r\n4: Connection lost during downloading process.\r\n5: Integrity check failure for new downloaded package.\r\n6: Unsupported package type.\r\n7: Invalid URI.\r\n8: Firmware update failed.\r\n9: Unsupported protocol. A LwM2M client indicates the failure to retrieve the firmware image using the URI provided in the Package URI resource by writing the value 9 to the /5/0/5 (Update Result resource) when the URI contained a URI scheme unsupported by the client. Consequently, the LwM2M Client is unable to retrieve the firmware image using the URI provided by the LwM2M Server in the Package URI when it refers to an unsupported protocol.\r\n10: Firmware update cancelled. A Cancel operation has been executed successfully.\r\n11: Firmware update deferred."]
        async fn update_result(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error>;
        #[doc = "Name of the Firmware Package"]
        async fn pkg_name(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Version of the Firmware package"]
        async fn pkg_version(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "This resource indicates what protocols the LwM2M Client implements to retrieve firmware images. The LwM2M server uses this information to decide what URI to include in the Package URI. A LwM2M Server MUST NOT include a URI in the Package URI object that uses a protocol that is unsupported by the LwM2M client.\r\nFor example, if a LwM2M client indicates that it supports CoAP and CoAPS then a LwM2M Server must not provide an HTTP URI in the Packet URI.\r\nThe following values are defined by this version of the specification:\r\n0: CoAP (as defined in RFC 7252) with the additional support for block-wise transfer. CoAP is the default setting.\r\n1: CoAPS (as defined in RFC 7252) with the additional support for block-wise transfer\r\n2: HTTP 1.1 (as defined in RFC 7230)\r\n3: HTTPS 1.1 (as defined in RFC 7230)\r\n4: CoAP over TCP (as defined in RFC 8323)\r\n5: CoAP over TLS (as defined in RFC 8323)\r\nAdditional values MAY be defined in the future. Any value not understood by the LwM2M Server MUST be ignored."]
        async fn firmware_update_protocol_support(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "The LwM2M Client uses this resource to indicate its support for transferring firmware images to the client either via the Package Resource (=push) or via the Package URI Resource (=pull) mechanism.\r\n0: Pull only\r\n1: Push only\r\n2: Both. In this case the LwM2M Server MAY choose the preferred mechanism for conveying the firmware image to the LwM2M Client."]
        async fn firmware_update_delivery_method(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error>;
        #[doc = "Cancels firmware update.\r\nCancel can be executed if the device has not initiated the Update process. If the device is in the process of installing the firmware or has already completed installation it MUST respond with Method Not Allowed error code.\r\nUpon successful Cancel operation, Update Result Resource is set to 10 and State is set to 0 by the device."]
        async fn cancel(
            &mut self,
            _object: usize,
            _resource: usize,
            _args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Severity of the firmware image.\r\n0: Critical\r\n1: Mandatory\r\n2: Optional\r\nThis information is useful when the device provides option for the deferred update. Default value is 1."]
        async fn severity(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "Severity of the firmware image.\r\n0: Critical\r\n1: Mandatory\r\n2: Optional\r\nThis information is useful when the device provides option for the deferred update. Default value is 1."]
        async fn set_severity(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: i64,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "This resource stores the time when the State resource is changed. Device updates this resource before making any change to the State."]
        async fn last_state_change_time(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<std::time::SystemTime>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "The number of seconds a user can defer the software update.\r\nWhen this time period is over, the device will not prompt the user for update and install it automatically.\r\nIf the value is 0, a deferred update is not allowed."]
        async fn maximum_defer_period(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<u64>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        #[doc = "The number of seconds a user can defer the software update.\r\nWhen this time period is over, the device will not prompt the user for update and install it automatically.\r\nIf the value is 0, a deferred update is not allowed."]
        async fn set_maximum_defer_period(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: u64,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        async fn handle_partial_write(
            &mut self,
            _object_instance: usize,
            _values: std::collections::HashMap<String, Value>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedPartialWrite)
        }
    }
    pub const FIRMWARE_UPDATE_PACKAGE: usize = 0usize;
    pub const FIRMWARE_UPDATE_PACKAGE_URI: usize = 1usize;
    pub const FIRMWARE_UPDATE_UPDATE: usize = 2usize;
    pub const FIRMWARE_UPDATE_STATE: usize = 3usize;
    pub const FIRMWARE_UPDATE_UPDATE_RESULT: usize = 5usize;
    pub const FIRMWARE_UPDATE_PKG_NAME: usize = 6usize;
    pub const FIRMWARE_UPDATE_PKG_VERSION: usize = 7usize;
    pub const FIRMWARE_UPDATE_FIRMWARE_UPDATE_PROTOCOL_SUPPORT: usize = 8usize;
    pub const FIRMWARE_UPDATE_FIRMWARE_UPDATE_DELIVERY_METHOD: usize = 9usize;
    pub const FIRMWARE_UPDATE_CANCEL: usize = 10usize;
    pub const FIRMWARE_UPDATE_SEVERITY: usize = 11usize;
    pub const FIRMWARE_UPDATE_LAST_STATE_CHANGE_TIME: usize = 12usize;
    pub const FIRMWARE_UPDATE_MAXIMUM_DEFER_PERIOD: usize = 13usize;
    pub struct FirmwareUpdateHandler<'a, T> {
        t: &'a mut T,
    }
    impl<'a, T: Send + Sync + FirmwareUpdate> FirmwareUpdateHandler<'a, T> {
        pub fn new(t: &'a mut T) -> Self {
            Self { t }
        }
    }
    #[::async_trait::async_trait]
    impl<T: Send + Sync + FirmwareUpdate> Object for FirmwareUpdateHandler<'_, T> {
        fn urn(&self) -> &'static str {
            "urn:oma:lwm2m:oma:5:1.1"
        }
        async fn read_resource(
            &self,
            object_instance: usize,
            id: usize,
            instance: usize,
        ) -> Result<Value, Error> {
            match id {
                FIRMWARE_UPDATE_PACKAGE_URI => {
                    Ok(self.t.package_uri(object_instance, instance).await?.into())
                }
                FIRMWARE_UPDATE_STATE => Ok(self.t.state(object_instance, instance).await?.into()),
                FIRMWARE_UPDATE_UPDATE_RESULT => Ok(self
                    .t
                    .update_result(object_instance, instance)
                    .await?
                    .into()),
                FIRMWARE_UPDATE_PKG_NAME => {
                    Ok(self.t.pkg_name(object_instance, instance).await?.into())
                }
                FIRMWARE_UPDATE_PKG_VERSION => {
                    Ok(self.t.pkg_version(object_instance, instance).await?.into())
                }
                FIRMWARE_UPDATE_FIRMWARE_UPDATE_PROTOCOL_SUPPORT => Ok(self
                    .t
                    .firmware_update_protocol_support(object_instance, instance)
                    .await?
                    .into()),
                FIRMWARE_UPDATE_FIRMWARE_UPDATE_DELIVERY_METHOD => Ok(self
                    .t
                    .firmware_update_delivery_method(object_instance, instance)
                    .await?
                    .into()),
                FIRMWARE_UPDATE_SEVERITY => {
                    Ok(self.t.severity(object_instance, instance).await?.into())
                }
                FIRMWARE_UPDATE_LAST_STATE_CHANGE_TIME => Ok(self
                    .t
                    .last_state_change_time(object_instance, instance)
                    .await?
                    .into()),
                FIRMWARE_UPDATE_MAXIMUM_DEFER_PERIOD => Ok(self
                    .t
                    .maximum_defer_period(object_instance, instance)
                    .await?
                    .into()),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "read: unknown resource id `{id}`"
                ))),
            }
        }
        async fn write_resource(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            value: Value,
        ) -> Result<(), Error> {
            match id {
                FIRMWARE_UPDATE_PACKAGE => {
                    self.t
                        .set_package(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                FIRMWARE_UPDATE_PACKAGE_URI => {
                    self.t
                        .set_package_uri(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                FIRMWARE_UPDATE_SEVERITY => {
                    self.t
                        .set_severity(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                FIRMWARE_UPDATE_MAXIMUM_DEFER_PERIOD => {
                    self.t
                        .set_maximum_defer_period(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        async fn exec(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            match id {
                FIRMWARE_UPDATE_UPDATE => {
                    self.t.update(object_instance, instance, args).await?;
                    Ok(())
                }
                FIRMWARE_UPDATE_CANCEL => {
                    self.t.cancel(object_instance, instance, args).await?;
                    Ok(())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        fn parse_resource_name(&self, name: &str) -> Result<usize, Error> {
            match name {
                "package" => Ok(0usize),
                "package_uri" => Ok(1usize),
                "update" => Ok(2usize),
                "state" => Ok(3usize),
                "update_result" => Ok(5usize),
                "pkg_name" => Ok(6usize),
                "pkg_version" => Ok(7usize),
                "firmware_update_protocol_support" => Ok(8usize),
                "firmware_update_delivery_method" => Ok(9usize),
                "cancel" => Ok(10usize),
                "severity" => Ok(11usize),
                "last_state_change_time" => Ok(12usize),
                "maximum_defer_period" => Ok(13usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource name `{name}`"
                ))),
            }
        }
        fn get_resource_name(&self, id: usize) -> Result<&str, Error> {
            match id {
                0usize => Ok("package"),
                1usize => Ok("package_uri"),
                2usize => Ok("update"),
                3usize => Ok("state"),
                5usize => Ok("update_result"),
                6usize => Ok("pkg_name"),
                7usize => Ok("pkg_version"),
                8usize => Ok("firmware_update_protocol_support"),
                9usize => Ok("firmware_update_delivery_method"),
                10usize => Ok("cancel"),
                11usize => Ok("severity"),
                12usize => Ok("last_state_change_time"),
                13usize => Ok("maximum_defer_period"),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn supported_resource_operations(&self, id: usize) -> Result<usize, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                0usize => Ok(2usize),
                1usize => Ok(3usize),
                2usize => Ok(20usize),
                3usize => Ok(1usize),
                5usize => Ok(1usize),
                6usize => Ok(1usize),
                7usize => Ok(1usize),
                8usize => Ok(1usize),
                9usize => Ok(1usize),
                10usize => Ok(20usize),
                11usize => Ok(3usize),
                12usize => Ok(1usize),
                13usize => Ok(3usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn is_array_resource(&self, id: usize) -> Result<bool, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                0usize => Ok(false),
                1usize => Ok(false),
                2usize => Ok(false),
                3usize => Ok(false),
                5usize => Ok(false),
                6usize => Ok(false),
                7usize => Ok(false),
                8usize => Ok(true),
                9usize => Ok(false),
                10usize => Ok(false),
                11usize => Ok(false),
                12usize => Ok(false),
                13usize => Ok(false),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
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
    pub fn make_firmware_update_json(
        time: Option<std::time::SystemTime>,
        package_uri: String,
        state: i64,
        update_result: i64,
        pkg_name: Option<String>,
        pkg_version: Option<String>,
        firmware_update_protocol_support: Vec<Option<i64>>,
        firmware_update_delivery_method: i64,
        severity: Option<i64>,
        last_state_change_time: Option<std::time::SystemTime>,
        maximum_defer_period: Option<u64>,
    ) -> ::serde_json::Value {
        let mut res = std::collections::HashMap::<&str, Value>::new();
        res.insert(
            "Package URI",
            Value {
                data: ValueData::from(package_uri),
                time,
            },
        );
        res.insert(
            "State",
            Value {
                data: ValueData::from(state),
                time,
            },
        );
        res.insert(
            "Update Result",
            Value {
                data: ValueData::from(update_result),
                time,
            },
        );
        if let Some(pkg_name) = pkg_name {
            res.insert(
                "PkgName",
                Value {
                    data: ValueData::from(pkg_name),
                    time,
                },
            );
        }
        if let Some(pkg_version) = pkg_version {
            res.insert(
                "PkgVersion",
                Value {
                    data: ValueData::from(pkg_version),
                    time,
                },
            );
        }
        res.insert(
            "Firmware Update Protocol Support",
            Value {
                data: ValueData::from(firmware_update_protocol_support),
                time,
            },
        );
        res.insert(
            "Firmware Update Delivery Method",
            Value {
                data: ValueData::from(firmware_update_delivery_method),
                time,
            },
        );
        if let Some(severity) = severity {
            res.insert(
                "Severity",
                Value {
                    data: ValueData::from(severity),
                    time,
                },
            );
        }
        if let Some(last_state_change_time) = last_state_change_time {
            res.insert(
                "Last State Change Time",
                Value {
                    data: ValueData::from(last_state_change_time),
                    time,
                },
            );
        }
        if let Some(maximum_defer_period) = maximum_defer_period {
            res.insert(
                "Maximum Defer Period",
                Value {
                    data: ValueData::from(maximum_defer_period),
                    time,
                },
            );
        }
        let mut json = ::serde_json::json!(res);
        json.as_object_mut()
            .unwrap()
            .insert("_urn".to_string(), "urn:oma:lwm2m:oma:5:1.1".into());
        json
    }

    // - "../third_party/lwm2m-registry/version_history/5-1_1.xml"

    // + "../third_party/bnw-ipso-registry/includable-device.xml"
    #[allow(clippy::doc_markdown)]
    #[doc = "Represent a device that can be included into the network of the gateway\n\n"]
    #[::async_trait::async_trait]
    pub trait IncludableDevice {
        #[doc = "Examples:\r\n3034F8EE90126D4000000033"]
        async fn identifier(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<String>, Error>;
        #[doc = "1=lwm2m\r\n2=lemonbeat"]
        async fn protocol(&self, _object: usize, _resource: usize)
            -> Result<TimedData<i64>, Error>;
        #[doc = "Start inclusion (semantics dependent on protocol)\r\n\r\nlwm2m: allow bootstrap & registration (inclusion_allowed = True)\r\n\r\nlemonbeat: start inclusion process (inclusion_allowed = True & inclusion_started = True)\r\n"]
        async fn include(
            &mut self,
            _object: usize,
            _resource: usize,
            _args: Option<Vec<String>>,
        ) -> Result<(), Error>;
        #[doc = "Has the device started the inclusion process?"]
        async fn inclusion_started(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<bool>, Error>;
        #[doc = "Has the device completed the inclusion process and is now part of gateway network?"]
        async fn inclusion_completed(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<bool>, Error>;
        #[doc = "0=no error\r\n1=inclusion start failure\r\n\r\nfurther error codes might be protocol specific / dependent\r\n\r\nlwm2m:\r\n1: device could not be whitelisted"]
        async fn inclusion_error(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error>;
        async fn handle_partial_write(
            &mut self,
            _object_instance: usize,
            _values: std::collections::HashMap<String, Value>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedPartialWrite)
        }
    }
    pub const INCLUDABLE_DEVICE_IDENTIFIER: usize = 1usize;
    pub const INCLUDABLE_DEVICE_PROTOCOL: usize = 2usize;
    pub const INCLUDABLE_DEVICE_INCLUDE: usize = 5usize;
    pub const INCLUDABLE_DEVICE_INCLUSION_STARTED: usize = 7usize;
    pub const INCLUDABLE_DEVICE_INCLUSION_COMPLETED: usize = 8usize;
    pub const INCLUDABLE_DEVICE_INCLUSION_ERROR: usize = 9usize;
    pub struct IncludableDeviceHandler<'a, T> {
        t: &'a mut T,
    }
    impl<'a, T: Send + Sync + IncludableDevice> IncludableDeviceHandler<'a, T> {
        pub fn new(t: &'a mut T) -> Self {
            Self { t }
        }
    }
    #[::async_trait::async_trait]
    impl<T: Send + Sync + IncludableDevice> Object for IncludableDeviceHandler<'_, T> {
        fn urn(&self) -> &'static str {
            "urn:oma:lwm2m:x:28170:0.2"
        }
        async fn read_resource(
            &self,
            object_instance: usize,
            id: usize,
            instance: usize,
        ) -> Result<Value, Error> {
            match id {
                INCLUDABLE_DEVICE_IDENTIFIER => {
                    Ok(self.t.identifier(object_instance, instance).await?.into())
                }
                INCLUDABLE_DEVICE_PROTOCOL => {
                    Ok(self.t.protocol(object_instance, instance).await?.into())
                }
                INCLUDABLE_DEVICE_INCLUSION_STARTED => Ok(self
                    .t
                    .inclusion_started(object_instance, instance)
                    .await?
                    .into()),
                INCLUDABLE_DEVICE_INCLUSION_COMPLETED => Ok(self
                    .t
                    .inclusion_completed(object_instance, instance)
                    .await?
                    .into()),
                INCLUDABLE_DEVICE_INCLUSION_ERROR => Ok(self
                    .t
                    .inclusion_error(object_instance, instance)
                    .await?
                    .into()),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "read: unknown resource id `{id}`"
                ))),
            }
        }
        async fn write_resource(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            value: Value,
        ) -> Result<(), Error> {
            match id {
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        async fn exec(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            match id {
                INCLUDABLE_DEVICE_INCLUDE => {
                    self.t.include(object_instance, instance, args).await?;
                    Ok(())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        fn parse_resource_name(&self, name: &str) -> Result<usize, Error> {
            match name {
                "identifier" => Ok(1usize),
                "protocol" => Ok(2usize),
                "include" => Ok(5usize),
                "inclusion_started" => Ok(7usize),
                "inclusion_completed" => Ok(8usize),
                "inclusion_error" => Ok(9usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource name `{name}`"
                ))),
            }
        }
        fn get_resource_name(&self, id: usize) -> Result<&str, Error> {
            match id {
                1usize => Ok("identifier"),
                2usize => Ok("protocol"),
                5usize => Ok("include"),
                7usize => Ok("inclusion_started"),
                8usize => Ok("inclusion_completed"),
                9usize => Ok("inclusion_error"),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn supported_resource_operations(&self, id: usize) -> Result<usize, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(1usize),
                2usize => Ok(1usize),
                5usize => Ok(20usize),
                7usize => Ok(1usize),
                8usize => Ok(1usize),
                9usize => Ok(1usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn is_array_resource(&self, id: usize) -> Result<bool, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(false),
                2usize => Ok(false),
                5usize => Ok(false),
                7usize => Ok(false),
                8usize => Ok(false),
                9usize => Ok(false),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
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
    pub fn make_includable_device_json(
        time: Option<std::time::SystemTime>,
        identifier: String,
        protocol: i64,
        inclusion_started: bool,
        inclusion_completed: bool,
        inclusion_error: i64,
    ) -> ::serde_json::Value {
        let mut res = std::collections::HashMap::<&str, Value>::new();
        res.insert(
            "identifier",
            Value {
                data: ValueData::from(identifier),
                time,
            },
        );
        res.insert(
            "protocol",
            Value {
                data: ValueData::from(protocol),
                time,
            },
        );
        res.insert(
            "inclusion_started",
            Value {
                data: ValueData::from(inclusion_started),
                time,
            },
        );
        res.insert(
            "inclusion_completed",
            Value {
                data: ValueData::from(inclusion_completed),
                time,
            },
        );
        res.insert(
            "inclusion_error",
            Value {
                data: ValueData::from(inclusion_error),
                time,
            },
        );
        let mut json = ::serde_json::json!(res);
        json.as_object_mut()
            .unwrap()
            .insert("_urn".to_string(), "urn:oma:lwm2m:x:28170:0.2".into());
        json
    }

    // - "../third_party/bnw-ipso-registry/includable-device.xml"

    // + "../third_party/bnw-ipso-registry/status-message.xml"
    #[allow(clippy::doc_markdown)]
    #[doc = "Logging capability for unexpected device events. Used for debugging purposes only. \n\n"]
    #[::async_trait::async_trait]
    pub trait LemonbeatStatusMessage {
        #[doc = "The type of the status.\n1: Public Key\n2: Memory Information\n3: Device Description\n4: Value Description\n5: Value\n6: Partner Information\n7: Action\n8: Calculation\n9: Timer\n10: Calendar\n11: State machine\n12: Firmware update\n13: Configuration\n100: Exi\n101: System\n200: Application"]
        async fn type_escaped(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<u64>, Error>;
        #[doc = "Status code in the context of the status type."]
        async fn code(&self, _object: usize, _resource: usize) -> Result<TimedData<u64>, Error>;
        #[doc = "Severity level\n0: Disabled\n1: Important\n2: Error\n3: Warning\n4: Info\n5: Debug"]
        async fn level(&self, _object: usize, _resource: usize) -> Result<TimedData<u64>, Error>;
        #[doc = "Additional information in the context of type and code. Up to 4 bytes encoded as HEX string."]
        async fn data(&self, _object: usize, _resource: usize) -> Result<TimedData<String>, Error> {
            Err(Error::UnsupportedOptionalResource)
        }
        async fn handle_partial_write(
            &mut self,
            _object_instance: usize,
            _values: std::collections::HashMap<String, Value>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedPartialWrite)
        }
    }
    pub const LEMONBEAT_STATUS_MESSAGE_TYPE: usize = 1usize;
    pub const LEMONBEAT_STATUS_MESSAGE_CODE: usize = 2usize;
    pub const LEMONBEAT_STATUS_MESSAGE_LEVEL: usize = 3usize;
    pub const LEMONBEAT_STATUS_MESSAGE_DATA: usize = 4usize;
    pub struct LemonbeatStatusMessageHandler<'a, T> {
        t: &'a mut T,
    }
    impl<'a, T: Send + Sync + LemonbeatStatusMessage> LemonbeatStatusMessageHandler<'a, T> {
        pub fn new(t: &'a mut T) -> Self {
            Self { t }
        }
    }
    #[::async_trait::async_trait]
    impl<T: Send + Sync + LemonbeatStatusMessage> Object for LemonbeatStatusMessageHandler<'_, T> {
        fn urn(&self) -> &'static str {
            "urn:oma:lwm2m:x:28173"
        }
        async fn read_resource(
            &self,
            object_instance: usize,
            id: usize,
            instance: usize,
        ) -> Result<Value, Error> {
            match id {
                LEMONBEAT_STATUS_MESSAGE_TYPE => {
                    Ok(self.t.type_escaped(object_instance, instance).await?.into())
                }
                LEMONBEAT_STATUS_MESSAGE_CODE => {
                    Ok(self.t.code(object_instance, instance).await?.into())
                }
                LEMONBEAT_STATUS_MESSAGE_LEVEL => {
                    Ok(self.t.level(object_instance, instance).await?.into())
                }
                LEMONBEAT_STATUS_MESSAGE_DATA => {
                    Ok(self.t.data(object_instance, instance).await?.into())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "read: unknown resource id `{id}`"
                ))),
            }
        }
        async fn write_resource(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            value: Value,
        ) -> Result<(), Error> {
            match id {
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        async fn exec(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            match id {
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        fn parse_resource_name(&self, name: &str) -> Result<usize, Error> {
            match name {
                "type" => Ok(1usize),
                "code" => Ok(2usize),
                "level" => Ok(3usize),
                "data" => Ok(4usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource name `{name}`"
                ))),
            }
        }
        fn get_resource_name(&self, id: usize) -> Result<&str, Error> {
            match id {
                1usize => Ok("type"),
                2usize => Ok("code"),
                3usize => Ok("level"),
                4usize => Ok("data"),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn supported_resource_operations(&self, id: usize) -> Result<usize, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(1usize),
                2usize => Ok(1usize),
                3usize => Ok(1usize),
                4usize => Ok(1usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn is_array_resource(&self, id: usize) -> Result<bool, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(false),
                2usize => Ok(false),
                3usize => Ok(false),
                4usize => Ok(false),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
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
    pub fn make_lemonbeat_status_message_json(
        time: Option<std::time::SystemTime>,
        type_escaped: u64,
        code: u64,
        level: u64,
        data: Option<String>,
    ) -> ::serde_json::Value {
        let mut res = std::collections::HashMap::<&str, Value>::new();
        res.insert(
            "Type",
            Value {
                data: ValueData::from(type_escaped),
                time,
            },
        );
        res.insert(
            "Code",
            Value {
                data: ValueData::from(code),
                time,
            },
        );
        res.insert(
            "Level",
            Value {
                data: ValueData::from(level),
                time,
            },
        );
        if let Some(data) = data {
            res.insert(
                "Data",
                Value {
                    data: ValueData::from(data),
                    time,
                },
            );
        }
        let mut json = ::serde_json::json!(res);
        json.as_object_mut()
            .unwrap()
            .insert("_urn".to_string(), "urn:oma:lwm2m:x:28173".into());
        json
    }

    // - "../third_party/bnw-ipso-registry/status-message.xml"

    // + "../third_party/bnw-ipso-registry/data-download.xml"
    #[allow(clippy::doc_markdown)]
    #[doc = "LONA garden map and zone transfer.\nValid download request requires all four writable resources to be set within single write.\n\n"]
    #[::async_trait::async_trait]
    pub trait DataDownload {
        #[doc = "Binary data"]
        async fn set_data(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: Vec<u8>,
        ) -> Result<(), Error>;
        #[doc = "File type identifier\n258: garden map\n260: zone"]
        async fn slot(&self, _object: usize, _resource: usize) -> Result<TimedData<i64>, Error>;
        #[doc = "File type identifier\n258: garden map\n260: zone"]
        async fn set_slot(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: i64,
        ) -> Result<(), Error>;
        #[doc = "CRC16-XModem checksum over the whole content of the data resource. \nSame algorithm as Lemonbeat uses so the value can directly be passed on to firmware_init."]
        async fn checksum(&self, _object: usize, _resource: usize)
            -> Result<TimedData<i64>, Error>;
        #[doc = "CRC16-XModem checksum over the whole content of the data resource. \nSame algorithm as Lemonbeat uses so the value can directly be passed on to firmware_init."]
        async fn set_checksum(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: i64,
        ) -> Result<(), Error>;
        #[doc = "The content tag is a 32 bit value with the meaning for the backend of identifying the data content (similar to HTTP-ETag). Semantics of the Content Tag is up to the user."]
        async fn content_tag(
            &self,
            _object: usize,
            _resource: usize,
        ) -> Result<TimedData<i64>, Error>;
        #[doc = "The content tag is a 32 bit value with the meaning for the backend of identifying the data content (similar to HTTP-ETag). Semantics of the Content Tag is up to the user."]
        async fn set_content_tag(
            &mut self,
            _object: usize,
            _resource: usize,
            _value: i64,
        ) -> Result<(), Error>;
        #[doc = "Current state of data download process:\n0: <empty>: not active\n1: uploading: transfer to device in progress\n2: activating: activating data\n3: activated: process completed successfully\n4: upload_failed: failure during data upload\n5: activation_failed: data not activated"]
        async fn status(&self, _object: usize, _resource: usize) -> Result<TimedData<i64>, Error>;
        async fn handle_partial_write(
            &mut self,
            _object_instance: usize,
            _values: std::collections::HashMap<String, Value>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedPartialWrite)
        }
    }
    pub const DATA_DOWNLOAD_DATA: usize = 1usize;
    pub const DATA_DOWNLOAD_SLOT: usize = 2usize;
    pub const DATA_DOWNLOAD_CHECKSUM: usize = 3usize;
    pub const DATA_DOWNLOAD_CONTENT_TAG: usize = 4usize;
    pub const DATA_DOWNLOAD_STATUS: usize = 5usize;
    pub struct DataDownloadHandler<'a, T> {
        t: &'a mut T,
    }
    impl<'a, T: Send + Sync + DataDownload> DataDownloadHandler<'a, T> {
        pub fn new(t: &'a mut T) -> Self {
            Self { t }
        }
    }
    #[::async_trait::async_trait]
    impl<T: Send + Sync + DataDownload> Object for DataDownloadHandler<'_, T> {
        fn urn(&self) -> &'static str {
            "urn:oma:lwm2m:x:28174"
        }
        async fn read_resource(
            &self,
            object_instance: usize,
            id: usize,
            instance: usize,
        ) -> Result<Value, Error> {
            match id {
                DATA_DOWNLOAD_SLOT => Ok(self.t.slot(object_instance, instance).await?.into()),
                DATA_DOWNLOAD_CHECKSUM => {
                    Ok(self.t.checksum(object_instance, instance).await?.into())
                }
                DATA_DOWNLOAD_CONTENT_TAG => {
                    Ok(self.t.content_tag(object_instance, instance).await?.into())
                }
                DATA_DOWNLOAD_STATUS => Ok(self.t.status(object_instance, instance).await?.into()),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "read: unknown resource id `{id}`"
                ))),
            }
        }
        async fn write_resource(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            value: Value,
        ) -> Result<(), Error> {
            match id {
                DATA_DOWNLOAD_DATA => {
                    self.t
                        .set_data(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                DATA_DOWNLOAD_SLOT => {
                    self.t
                        .set_slot(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                DATA_DOWNLOAD_CHECKSUM => {
                    self.t
                        .set_checksum(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                DATA_DOWNLOAD_CONTENT_TAG => {
                    self.t
                        .set_content_tag(object_instance, instance, value.data.try_into()?)
                        .await?;
                    Ok(())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        async fn exec(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            match id {
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        fn parse_resource_name(&self, name: &str) -> Result<usize, Error> {
            match name {
                "data" => Ok(1usize),
                "slot" => Ok(2usize),
                "checksum" => Ok(3usize),
                "content_tag" => Ok(4usize),
                "status" => Ok(5usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource name `{name}`"
                ))),
            }
        }
        fn get_resource_name(&self, id: usize) -> Result<&str, Error> {
            match id {
                1usize => Ok("data"),
                2usize => Ok("slot"),
                3usize => Ok("checksum"),
                4usize => Ok("content_tag"),
                5usize => Ok("status"),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn supported_resource_operations(&self, id: usize) -> Result<usize, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(2usize),
                2usize => Ok(3usize),
                3usize => Ok(3usize),
                4usize => Ok(3usize),
                5usize => Ok(1usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn is_array_resource(&self, id: usize) -> Result<bool, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(false),
                2usize => Ok(false),
                3usize => Ok(false),
                4usize => Ok(false),
                5usize => Ok(false),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
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
    pub fn make_data_download_json(
        time: Option<std::time::SystemTime>,
        slot: i64,
        checksum: i64,
        content_tag: i64,
        status: i64,
    ) -> ::serde_json::Value {
        let mut res = std::collections::HashMap::<&str, Value>::new();
        res.insert(
            "Slot",
            Value {
                data: ValueData::from(slot),
                time,
            },
        );
        res.insert(
            "Checksum",
            Value {
                data: ValueData::from(checksum),
                time,
            },
        );
        res.insert(
            "Content Tag",
            Value {
                data: ValueData::from(content_tag),
                time,
            },
        );
        res.insert(
            "Status",
            Value {
                data: ValueData::from(status),
                time,
            },
        );
        let mut json = ::serde_json::json!(res);
        json.as_object_mut()
            .unwrap()
            .insert("_urn".to_string(), "urn:oma:lwm2m:x:28174".into());
        json
    }

    // - "../third_party/bnw-ipso-registry/data-download.xml"

    // + "../third_party/bnw-ipso-registry/connection-status.xml"
    #[allow(clippy::doc_markdown)]
    #[doc = "Status of radio communication\n\n"]
    #[::async_trait::async_trait]
    pub trait ConnectionStatus {
        #[doc = "Recent successful communication"]
        async fn online(&self, _object: usize, _resource: usize) -> Result<TimedData<bool>, Error>;
        #[doc = "Force communication to test if device can be reached"]
        async fn check(
            &mut self,
            _object: usize,
            _resource: usize,
            _args: Option<Vec<String>>,
        ) -> Result<(), Error>;
        async fn handle_partial_write(
            &mut self,
            _object_instance: usize,
            _values: std::collections::HashMap<String, Value>,
        ) -> Result<(), Error> {
            Err(Error::UnsupportedPartialWrite)
        }
    }
    pub const CONNECTION_STATUS_ONLINE: usize = 1usize;
    pub const CONNECTION_STATUS_CHECK: usize = 2usize;
    pub struct ConnectionStatusHandler<'a, T> {
        t: &'a mut T,
    }
    impl<'a, T: Send + Sync + ConnectionStatus> ConnectionStatusHandler<'a, T> {
        pub fn new(t: &'a mut T) -> Self {
            Self { t }
        }
    }
    #[::async_trait::async_trait]
    impl<T: Send + Sync + ConnectionStatus> Object for ConnectionStatusHandler<'_, T> {
        fn urn(&self) -> &'static str {
            "urn:oma:lwm2m:x:28171"
        }
        async fn read_resource(
            &self,
            object_instance: usize,
            id: usize,
            instance: usize,
        ) -> Result<Value, Error> {
            match id {
                CONNECTION_STATUS_ONLINE => {
                    Ok(self.t.online(object_instance, instance).await?.into())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "read: unknown resource id `{id}`"
                ))),
            }
        }
        async fn write_resource(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            value: Value,
        ) -> Result<(), Error> {
            match id {
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        async fn exec(
            &mut self,
            object_instance: usize,
            id: usize,
            instance: usize,
            args: Option<Vec<String>>,
        ) -> Result<(), Error> {
            match id {
                CONNECTION_STATUS_CHECK => {
                    self.t.check(object_instance, instance, args).await?;
                    Ok(())
                }
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "write: unknown resource id `{id}`"
                ))),
            }
        }
        fn parse_resource_name(&self, name: &str) -> Result<usize, Error> {
            match name {
                "online" => Ok(1usize),
                "check" => Ok(2usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource name `{name}`"
                ))),
            }
        }
        fn get_resource_name(&self, id: usize) -> Result<&str, Error> {
            match id {
                1usize => Ok("online"),
                2usize => Ok("check"),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn supported_resource_operations(&self, id: usize) -> Result<usize, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(1usize),
                2usize => Ok(20usize),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
                ))),
            }
        }
        fn is_array_resource(&self, id: usize) -> Result<bool, Error> {
            #[allow(clippy::match_same_arms)]
            match id {
                1usize => Ok(false),
                2usize => Ok(false),
                _ => Err(Error::Anyhow(::anyhow::anyhow!(
                    "unknown resource id `{id}`"
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
    pub fn make_connection_status_json(
        time: Option<std::time::SystemTime>,
        online: bool,
    ) -> ::serde_json::Value {
        let mut res = std::collections::HashMap::<&str, Value>::new();
        res.insert(
            "Online",
            Value {
                data: ValueData::from(online),
                time,
            },
        );
        let mut json = ::serde_json::json!(res);
        json.as_object_mut()
            .unwrap()
            .insert("_urn".to_string(), "urn:oma:lwm2m:x:28171".into());
        json
    }

    // - "../third_party/bnw-ipso-registry/connection-status.xml"
}

/* The rest of the code in this file was originally generated by 'build.rs'.
No future changes regarding supported objects are expected.
Therefore the generated code is add here directly.
This also removes the dependency on the IPSO registries. */
#[doc = "Generated object type enum."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ObjectType {
    Device,
    FirmwareUpdate,
    IncludableDevice,
    LemonbeatStatusMessage,
    DataDownload,
    ConnectionStatus,
    Lemonbeat,
}
impl ::std::str::FromStr for ObjectType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "device" => Ok(Self::Device),
            "firmware_update" => Ok(Self::FirmwareUpdate),
            "includable_device" => Ok(Self::IncludableDevice),
            "lemonbeat_status_message" => Ok(Self::LemonbeatStatusMessage),
            "data_download" => Ok(Self::DataDownload),
            "connection_status" => Ok(Self::ConnectionStatus),
            "lemonbeat" => Ok(Self::Lemonbeat),
            other => Err(Error::Anyhow(::anyhow::anyhow!(
                "unsupported object ID `{other}`"
            ))),
        }
    }
}
impl ObjectType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Device => "device",
            Self::FirmwareUpdate => "firmware_update",
            Self::IncludableDevice => "includable_device",
            Self::LemonbeatStatusMessage => "lemonbeat_status_message",
            Self::DataDownload => "data_download",
            Self::ConnectionStatus => "connection_status",
            Self::Lemonbeat => "lemonbeat",
        }
    }
    pub fn urn(self) -> &'static str {
        match self {
            Self::Device => "urn:oma:lwm2m:oma:3:1.1",
            Self::FirmwareUpdate => "urn:oma:lwm2m:oma:5:1.1",
            Self::IncludableDevice => "urn:oma:lwm2m:x:28170:0.2",
            Self::LemonbeatStatusMessage => "urn:oma:lwm2m:x:28173",
            Self::DataDownload => "urn:oma:lwm2m:x:28174",
            Self::ConnectionStatus => "urn:oma:lwm2m:x:28171",
            Self::Lemonbeat => "urn:oma:lwm2m:x:31000",
        }
    }
}
