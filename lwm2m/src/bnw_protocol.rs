// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use crate::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IPC method.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    #[default]
    None,
    Read,
    Write,
    Execute,
    Update,
    Overwrite,
    Delete,
}

impl Method {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EntityKind {
    Gateway {
        /// name of the service to talk to
        service: String,
    },
    Device {
        /// GARDENA device
        device: String,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Entity {
    /// resource of the device
    pub path: std::path::PathBuf,
    /// destination service
    #[serde(flatten)]
    pub kind: EntityKind,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata<'a> {
    pub source: std::borrow::Cow<'a, str>,
    pub sequence: u64,
}

type ValueMap = HashMap<String, Value>;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Payload {
    Value(Value),
    Values(ValueMap),
    MultiResourceValues(HashMap<String, ValueMap>),
}

impl Payload {
    pub fn is_big(&self) -> bool {
        match self {
            Payload::Value(value) => value.is_big(),
            Payload::Values(values) => values.values().any(Value::is_big),
            Payload::MultiResourceValues(values) => {
                values.values().any(|vals| vals.values().any(Value::is_big))
            }
        }
    }
}

/// Request that can be sent via IPC/LWM2M Req socket.
#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    /// action to execute or that was executed
    pub op: Method,
    /// entity the action is performed on.
    ///
    /// Destination for requests, source for events.
    pub entity: Entity,
    // TODO: Add None in Payload enum and remove Option here
    pub payload: Option<Payload>,
}

impl Request {
    pub fn is_big(&self) -> bool {
        self.payload.as_ref().is_some_and(Payload::is_big)
    }
}

/// Response as received on IPC/LWM2M Rep sockets.
#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub success: bool,
    pub entity: Option<Entity>,
    pub payload: T,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Event as received on IPC/LWM2M Sub sockets.
#[derive(Debug, Deserialize, Serialize)]
pub struct Event<'a> {
    /// action to execute or that was executed
    pub op: Method,
    /// entity the action is performed on.
    ///
    /// Destination for requests, source for events.
    pub entity: Entity,
    pub payload: Option<std::borrow::Cow<'a, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata<'a>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Source(String),
    ErrorSource(String),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct OwnedMetadata {
    #[serde(flatten)]
    pub source: Source,
    pub sequence: Option<u64>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct PayloadWithUrn {
    #[serde(flatten)]
    pub data: Payload,
    #[serde(rename = "_urn")]
    pub urn: Option<String>,
}

type PayloadMap = HashMap<String, PayloadWithUrn>;

pub trait Payloads: Default + serde::Serialize + serde::de::DeserializeOwned {
    fn is_none(&self) -> bool {
        false
    }

    fn get_single_string(&self) -> Option<&str> {
        None
    }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ObjectsPayload {
    #[default]
    None,
    Single(PayloadWithUrn),
    Multi(PayloadMap),
}

impl Payloads for ObjectsPayload {
    fn is_none(&self) -> bool {
        self == &Self::None
    }

    fn get_single_string(&self) -> Option<&str> {
        match self {
            ObjectsPayload::Single(PayloadWithUrn {
                data:
                    Payload::Value(Value {
                        data: crate::ValueData::String(Some(s)),
                        ..
                    }),
                ..
            }) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct DevicesPayload(pub HashMap<String, PayloadMap>);

impl Payloads for DevicesPayload {}

impl Payloads for () {
    fn is_none(&self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Message<T: Payloads> {
    #[serde(rename = "op", default, skip_serializing_if = "Method::is_none")]
    pub operation: Method,
    pub entity: Option<Entity>,
    #[serde(
        default,
        skip_serializing_if = "Payloads::is_none",
        bound(deserialize = "T: Payloads")
    )]
    pub payload: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OwnedMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
}

macro_rules! impl_message_try_from {
    ($single_name:ident, $vec_name:ident, $serde_name:ident, $typ:ty) => {
        pub fn $single_name(input: $typ) -> Result<Self, serde_json::Error> {
            serde_json::$serde_name(input)
        }
        pub fn $vec_name(input: $typ) -> Result<Vec<Self>, serde_json::Error> {
            serde_json::$serde_name(input)
        }
    };
}

#[rustfmt::skip]
impl<T: Payloads> Message<T> {
    impl_message_try_from!(try_from_json_str, try_vec_from_json_str, from_str, &str);
    impl_message_try_from!(try_from_json_slice, try_vec_from_json_slice, from_slice, &[u8]);
    impl_message_try_from!(try_from_json_reader, try_vec_from_json_reader, from_reader, impl std::io::Read);
}

#[allow(clippy::unreadable_literal)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwm2mserver;
    use crate::Integer::Unsigned;
    use crate::IntegerArray;
    use crate::ValueData::{self, Boolean, Integer, Opaque, Time};
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::PathBuf;

    fn ts_to_system_time(time_stamp: u64) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(time_stamp)
    }

    #[test]
    fn deserialize_includable_devices_message() {
        const MSG: &str = "[{\
            \"op\":\"update\",\
            \"entity\":{\"service\":\"lwm2mserver\",\"path\":\"includable_device/5\"},\
            \"payload\":{\
                \"identifier\":{\"vs\":\"301400000000004000000047\",\"ts\":1739807546},\
                \"protocol\":{\"vi\":1,\"ts\":1739807546},\
                \"inclusion_started\":{\"vb\":false,\"ts\":1739807546},\
                \"inclusion_completed\":{\"vb\":false,\"ts\":1739807546},\
                \"inclusion_error\":{\"vi\":0,\"ts\":1739807546},\
                \"_urn\":\"urn:oma:lwm2m:x:28170:0.1\"\
            },\
            \"metadata\":{\"source\":\"lwm2mserver\",\"sequence\":149}\
        }]";
        let messages = Message::<ObjectsPayload>::try_vec_from_json_str(MSG);
        assert!(messages.is_ok(), "{}", messages.unwrap_err());
        assert_eq!(
            messages.unwrap(),
            vec![Message {
                operation: Method::Update,
                entity: Some(Entity {
                    path: PathBuf::from("includable_device/5"),
                    kind: EntityKind::Gateway {
                        service: lwm2mserver::SERVICE_NAME.to_string(),
                    },
                }),
                payload: ObjectsPayload::Single(PayloadWithUrn {
                    data: Payload::Values(HashMap::from([
                        (
                            "identifier".to_string(),
                            Value {
                                data: ValueData::String(Some(
                                    "301400000000004000000047".to_string()
                                )),
                                time: Some(ts_to_system_time(1739807546)),
                            },
                        ),
                        (
                            "protocol".to_string(),
                            Value {
                                data: Integer(Some(Unsigned(1))),
                                time: Some(ts_to_system_time(1739807546)),
                            },
                        ),
                        (
                            "inclusion_started".to_string(),
                            Value {
                                data: Boolean(Some(false)),
                                time: Some(ts_to_system_time(1739807546)),
                            },
                        ),
                        (
                            "inclusion_completed".to_string(),
                            Value {
                                data: Boolean(Some(false)),
                                time: Some(ts_to_system_time(1739807546)),
                            },
                        ),
                        (
                            "inclusion_error".to_string(),
                            Value {
                                data: Integer(Some(Unsigned(0))),
                                time: Some(ts_to_system_time(1739807546)),
                            },
                        ),
                    ])),
                    urn: Some("urn:oma:lwm2m:x:28170:0.1".to_string()),
                }),
                metadata: Some(OwnedMetadata {
                    source: Source::Source(lwm2mserver::SERVICE_NAME.to_string()),
                    sequence: Some(149),
                }),
                success: None,
            }],
        );
    }

    #[test]
    fn deserialize_connection_status() {
        const MSG: &str = "[{\
            \"op\":\"update\",\
            \"entity\":{\"device\":\"301400000000004000000047\",\"path\":\"connection_status\"},\
            \"payload\":{\
                \"0\":{\"online\":{\"vb\":true,\"ts\":1739979071}},\
                \"_urn\":\"urn:oma:lwm2m:x:28171\"\
            },\
            \"metadata\":{\"source\":\"lwm2mserver\",\"sequence\":244}\
        }]";
        let messages = Message::<ObjectsPayload>::try_vec_from_json_str(MSG);
        assert!(messages.is_ok(), "{}", messages.unwrap_err());
        assert_eq!(
            messages.unwrap(),
            vec![Message {
                operation: Method::Update,
                entity: Some(Entity {
                    path: PathBuf::from("connection_status"),
                    kind: EntityKind::Device {
                        device: "301400000000004000000047".to_string()
                    }
                }),
                payload: ObjectsPayload::Single(PayloadWithUrn {
                    data: Payload::MultiResourceValues(HashMap::from([(
                        "0".to_string(),
                        HashMap::from([(
                            "online".to_string(),
                            Value {
                                data: Boolean(Some(true)),
                                time: Some(ts_to_system_time(1739979071))
                            }
                        )])
                    )])),
                    urn: Some("urn:oma:lwm2m:x:28171".to_string()),
                }),
                metadata: Some(OwnedMetadata {
                    source: Source::Source(lwm2mserver::SERVICE_NAME.to_string()),
                    sequence: Some(244),
                }),
                success: None
            }]
        );
    }

    #[test]
    fn deserialize_exclusion_notification() {
        const MSG: &str = "[{\
            \"op\":\"delete\",\
            \"entity\":{\"device\":\"301400000000004000000047\",\"path\":\"\"},\
            \"metadata\":{\"source\":\"lwm2mserver\",\"sequence\":266}\
        }]";
        let messages = Message::<ObjectsPayload>::try_vec_from_json_str(MSG);
        assert!(messages.is_ok(), "{}", messages.unwrap_err());
        assert_eq!(
            messages.unwrap(),
            vec![Message {
                operation: Method::Delete,
                entity: Some(Entity {
                    path: PathBuf::from(""),
                    kind: EntityKind::Device {
                        device: "301400000000004000000047".to_string()
                    }
                }),
                payload: ObjectsPayload::None,
                metadata: Some(OwnedMetadata {
                    source: Source::Source(lwm2mserver::SERVICE_NAME.to_string()),
                    sequence: Some(266),
                }),
                success: None
            }]
        );
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn deserialize_multiple_objects() {
        const MSG: &str = "[{\
            \"op\":\"update\",\
            \"entity\":{\"device\":\"301400000000004000000047\",\"path\":\"\"},\
            \"payload\":{\
                \"device\":{\
                    \"_urn\":\"urn:oma:lwm2m:oma:3\",\
                    \"0\":{\
                        \"manufacturer\":{\"vs\":\"Gardena\",\"ts\":1739981623},\
                        \"model_number\":{\"vs\":\"1\",\"ts\":1739981623},\
                        \"serial_number\":{\"vs\":\"00000071\",\"ts\":1739981623},\
                        \"firmware_version\":{\"vs\":\"0.0.0-3.7.0\",\"ts\":1739981623},\
                        \"battery_level\":{\"vi\":0,\"ts\":1739981623},\
                        \"current_time\":{\"vt\":0,\"ts\":1739981623},\
                        \"utc_offset\":{\"vs\":\"\",\"ts\":1739981623},\
                        \"timezone\":{\"vs\":\"CET-1CEST,M3.5.0,M10.5.0/3\",\"ts\":1739981623},\
                        \"supported_binding_and_modes\":{\"vs\":\"U\",\"ts\":1739981623},\
                        \"device_type\":{\"vs\":\"Mower CBT11\",\"ts\":1739981623},\
                        \"hardware_version\":{\"vs\":\"0.0.0\",\"ts\":1739981623},\
                        \"software_version\":{\
                            \"vs\":\"36.x_Com-App-T11-Posix-Xdebugnativesim_H586325_250214-174226\",\
                            \"ts\":1739981623\
                        },\
                        \"battery_status\":{\"vi\":6,\"ts\":1739981623},\
                        \"available_power_sources\":{\"ai\":[1],\"ts\":1739981623},\
                        \"error_code\":{\"ai\":[0],\"ts\":1739981623}\
                    }\
                },\
                \"connectivity_monitoring\":{\
                    \"_urn\":\"urn:oma:lwm2m:oma:4:1.2\",\
                    \"0\":{\
                        \"network_bearer\":{\"vi\":42,\"ts\":1739981623},\
                        \"radio_signal_strength\":{\"vi\":0,\"ts\":1739981623},\
                        \"link_quality\":{\"vi\":0,\"ts\":1739981623},\
                        \"cell_id\":{\"vi\":0,\"ts\":1739981623},\
                        \"smnc\":{\"vi\":0,\"ts\":1739981623},\
                        \"smcc\":{\"vi\":0,\"ts\":1739981623},\
                        \"signalsnr\":{\"vi\":0,\"ts\":1739981623},\
                        \"lac\":{\"vi\":0,\"ts\":1739981623}\
                    }\
                }\
            },\
            \"metadata\":{\"source\":\"lwm2mserver\",\"sequence\":249}\
        }]";
        let messages = Message::<ObjectsPayload>::try_vec_from_json_str(MSG);
        assert!(messages.is_ok(), "{}", messages.unwrap_err());
        assert_eq!(
            messages.unwrap(),
            vec![Message {
                operation: Method::Update,
                entity: Some(Entity {
                    path: PathBuf::from(""),
                    kind: EntityKind::Device { device: "301400000000004000000047".to_string() }
                }),
                payload: ObjectsPayload::Multi(HashMap::from([
                    ("device".to_string(), PayloadWithUrn {
                        urn: Some("urn:oma:lwm2m:oma:3".to_string()),
                        data: Payload::MultiResourceValues(HashMap::from([
                            ("0".to_string(), HashMap::from([
                                ("manufacturer".to_string(), Value {
                                    data: ValueData::String(Some("Gardena".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("model_number".to_string(), Value {
                                    data: ValueData::String(Some("1".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("serial_number".to_string(), Value {
                                    data: ValueData::String(Some("00000071".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("firmware_version".to_string(), Value {
                                    data: ValueData::String(Some("0.0.0-3.7.0".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("battery_level".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("current_time".to_string(), Value {
                                    data: Time(ts_to_system_time(0)),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("utc_offset".to_string(), Value {
                                    data: ValueData::String(Some(String::new())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("timezone".to_string(), Value {
                                    data: ValueData::String(Some("CET-1CEST,M3.5.0,M10.5.0/3".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("supported_binding_and_modes".to_string(), Value {
                                    data: ValueData::String(Some("U".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("device_type".to_string(), Value {
                                    data: ValueData::String(Some("Mower CBT11".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("hardware_version".to_string(), Value {
                                    data: ValueData::String(Some("0.0.0".to_string())),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("software_version".to_string(), Value {
                                    data: ValueData::String(Some(
                                        "36.x_Com-App-T11-Posix-Xdebugnativesim_H586325_250214-174226".to_string()
                                    )),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("battery_status".to_string(), Value {
                                    data: Integer(Some(Unsigned(6))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("available_power_sources".to_string(), Value {
                                    data: ValueData::IntegerArray(IntegerArray::Signed(vec![Some(1)])),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("error_code".to_string(), Value {
                                    data: ValueData::IntegerArray(IntegerArray::Signed(vec![Some(0)])),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                            ])),
                        ])),
                    }),
                    ("connectivity_monitoring".to_string(), PayloadWithUrn
                    {
                        urn: Some("urn:oma:lwm2m:oma:4:1.2".to_string()),
                        data: Payload::MultiResourceValues(HashMap::from([
                            ("0".to_string(), HashMap::from([
                                ("network_bearer".to_string(), Value {
                                    data: Integer(Some(Unsigned(42))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("radio_signal_strength".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("link_quality".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("cell_id".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("smnc".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("smcc".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("signalsnr".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                                ("lac".to_string(), Value {
                                    data: Integer(Some(Unsigned(0))),
                                    time: Some(ts_to_system_time(1739981623)),
                                }),
                            ])),
                        ])),
                    }),
                ])),
                metadata: Some(OwnedMetadata{
                    source: Source::Source(lwm2mserver::SERVICE_NAME.to_string()),
                    sequence: Some(249),
                }),
                success: None,
            }]
        );
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn deserialize_lwm2mserver_devices_response() {
        const MSG: &str = "[{\
            \"payload\":{\
                \"301400000000004000000047\":{\
                    \"irrigation_control\":{\
                        \"_urn\":\"urn:oma:lwm2m:x:28152:0.2\",\
                        \"0\":{\
                            \"error\":{\"vi\":0,\"ts\":1742398480}\
                        }\
                    },\
                    \"actuator\":{\
                        \"_urn\":\"urn:oma:lwm2m:x:28180:0.4\",\
                        \"0\":{\
                            \"default_duration_seconds\":{\"vi\":0,\"ts\":1742398480},\
                            \"available\":{\"vb\":false,\"ts\":1742398480},\
                            \"error\":{\"vi\":0,\"ts\":1742398480},\
                            \"paused_until\":{\"vt\":0,\"ts\":1742398480},\
                            \"name\":{\"vs\":\"\",\"ts\":1742398480},\
                            \"master\":{\"vi\":0,\"ts\":1742398480},\
                            \"current\":{\"vi\":0,\"ts\":1742398480},\
                            \"state\":{\"vi\":0,\"ts\":1742398480}\
                        },\
                        \"1\":{\
                            \"default_duration_seconds\":{\"vi\":0,\"ts\":1742398480},\
                            \"available\":{\"vb\":false,\"ts\":1742398480},\
                            \"error\":{\"vi\":0,\"ts\":1742398480},\
                            \"paused_until\":{\"vt\":0,\"ts\":1742398480},\
                            \"name\":{\"vs\":\"\",\"ts\":1742398480},\
                            \"master\":{\"vi\":0,\"ts\":1742398480},\
                            \"current\":{\"vi\":0,\"ts\":1742398480},\
                            \"state\":{\"vi\":0,\"ts\":1742398480}\
                        }\
                    },\
                    \"sg_common\":{\
                        \"_urn\":\"urn:oma:lwm2m:x:28183:0.5\",\
                        \"0\":{\
                            \"name\":{\"vs\":\"\",\"ts\":1742398480},\
                            \"fatal_error_log\":{\"vo\":\"\",\"ts\":1742398480},\
                            \"reboot_reason\":{\"vi\":0,\"ts\":1742398480},\
                            \"sun_data\":{\"vo\":\"\",\"ts\":1742398480}\
                        }\
                    },\
                    \"master_channel\":{\
                        \"_urn\":\"urn:oma:lwm2m:x:30000:0.1\",\
                        \"0\":{\
                            \"available\":{\"vb\":false,\"ts\":1742398480},\
                            \"error\":{\"vi\":0,\"ts\":1742398480},\
                            \"active\":{\"vb\":false,\"ts\":1742398480},\
                            \"pressure_release_time\":{\"vi\":0,\"ts\":1742398480}\
                        }\
                    },\
                    \"schedule\":{\
                        \"_urn\":\"urn:oma:lwm2m:x:28181:0.2\",\
                        \"0\":{\
                            \"start_offset_from\":{\"vi\":0,\"ts\":1742398480},\
                            \"start_offset_seconds\":{\"vi\":0,\"ts\":1742398480},\
                            \"end_offset_from\":{\"vi\":0,\"ts\":1742398480},\
                            \"end_offset_seconds\":{\"vi\":0,\"ts\":1742398480},\
                            \"repetition_type\":{\"vi\":0,\"ts\":1742398480},\
                            \"repetition_value\":{\"vi\":0,\"ts\":1742398480},\
                            \"actuator\":{\"vi\":0,\"ts\":1742398480},\
                            \"pre_offset\":{\"vi\":0,\"ts\":1742398480}\
                        }\
                    },\
                    \"timeslot\":{\
                        \"_urn\":\"urn:oma:lwm2m:x:28182:0.2\",\
                        \"0\":{\
                            \"start\":{\"vt\":0,\"ts\":1742398480},\
                            \"adjusted_stop\":{\"vt\":0,\"ts\":1742398480},\
                            \"state\":{\"vi\":0,\"ts\":1742398480},\
                            \"initial_source\":{\"vi\":0,\"ts\":1742398480},\
                            \"adjustment_source\":{\"vi\":0,\"ts\":1742398480},\
                            \"pre_offset\":{\"vi\":0,\"ts\":1742398480},\
                            \"actuator\":{\"vi\":0,\"ts\":1742398480}\
                        },\
                        \"1\":{\
                            \"start\":{\"vt\":0,\"ts\":1742398480},\
                            \"adjusted_stop\":{\"vt\":0,\"ts\":1742398480},\
                            \"state\":{\"vi\":0,\"ts\":1742398480},\
                            \"initial_source\":{\"vi\":0,\"ts\":1742398480},\
                            \"adjustment_source\":{\"vi\":0,\"ts\":1742398480},\
                            \"pre_offset\":{\"vi\":0,\"ts\":1742398480},\
                            \"actuator\":{\"vi\":1,\"ts\":1742398480}\
                        }\
                    }\
                }\
            },\
            \"success\":true\
        }]";
        let messages = Message::<DevicesPayload>::try_vec_from_json_str(MSG);
        assert!(messages.is_ok(), "{}", messages.unwrap_err());
        assert_eq!(
            messages.unwrap(),
            vec![Message {
                operation: Method::None,
                entity: None,
                payload: DevicesPayload(HashMap::from([(
                    "301400000000004000000047".to_string(),
                    HashMap::from([
                        (
                            "irrigation_control".to_string(),
                            PayloadWithUrn {
                                urn: Some("urn:oma:lwm2m:x:28152:0.2".to_string()),
                                data: Payload::MultiResourceValues(HashMap::from([(
                                    "0".to_string(),
                                    HashMap::from([(
                                        "error".to_string(),
                                        Value {
                                            data: Integer(Some(Unsigned(0))),
                                            time: Some(ts_to_system_time(1742398480))
                                        }
                                    )])
                                )])),
                            }
                        ),
                        (
                            "actuator".to_string(),
                            PayloadWithUrn {
                                urn: Some("urn:oma:lwm2m:x:28180:0.4".to_string()),
                                data: Payload::MultiResourceValues(HashMap::from([
                                    (
                                        "0".to_string(),
                                        HashMap::from([
                                            (
                                                "default_duration_seconds".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "available".to_string(),
                                                Value {
                                                    data: Boolean(Some(false)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "error".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "paused_until".to_string(),
                                                Value {
                                                    data: Time(ts_to_system_time(0)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "name".to_string(),
                                                Value {
                                                    data: ValueData::String(Some(String::new())),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "master".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "current".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "state".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                        ])
                                    ),
                                    (
                                        "1".to_string(),
                                        HashMap::from([
                                            (
                                                "default_duration_seconds".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "available".to_string(),
                                                Value {
                                                    data: Boolean(Some(false)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "error".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "paused_until".to_string(),
                                                Value {
                                                    data: Time(ts_to_system_time(0)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "name".to_string(),
                                                Value {
                                                    data: ValueData::String(Some(String::new())),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "master".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "current".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "state".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                        ])
                                    ),
                                ])),
                            }
                        ),
                        (
                            "sg_common".to_string(),
                            PayloadWithUrn {
                                urn: Some("urn:oma:lwm2m:x:28183:0.5".to_string()),
                                data: Payload::MultiResourceValues(HashMap::from([(
                                    "0".to_string(),
                                    HashMap::from([
                                        (
                                            "name".to_string(),
                                            Value {
                                                data: ValueData::String(Some(String::new())),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "fatal_error_log".to_string(),
                                            Value {
                                                data: Opaque(vec![]),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "reboot_reason".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "sun_data".to_string(),
                                            Value {
                                                data: Opaque(vec![]),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                    ])
                                )])),
                            }
                        ),
                        (
                            "master_channel".to_string(),
                            PayloadWithUrn {
                                urn: Some("urn:oma:lwm2m:x:30000:0.1".to_string()),
                                data: Payload::MultiResourceValues(HashMap::from([(
                                    "0".to_string(),
                                    HashMap::from([
                                        (
                                            "available".to_string(),
                                            Value {
                                                data: Boolean(Some(false)),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "error".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "active".to_string(),
                                            Value {
                                                data: Boolean(Some(false)),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "pressure_release_time".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                    ])
                                )])),
                            }
                        ),
                        (
                            "schedule".to_string(),
                            PayloadWithUrn {
                                urn: Some("urn:oma:lwm2m:x:28181:0.2".to_string()),
                                data: Payload::MultiResourceValues(HashMap::from([(
                                    "0".to_string(),
                                    HashMap::from([
                                        (
                                            "start_offset_from".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "start_offset_seconds".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "end_offset_from".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "end_offset_seconds".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "repetition_type".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "repetition_value".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "actuator".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                        (
                                            "pre_offset".to_string(),
                                            Value {
                                                data: Integer(Some(Unsigned(0))),
                                                time: Some(ts_to_system_time(1742398480))
                                            }
                                        ),
                                    ])
                                )])),
                            }
                        ),
                        (
                            "timeslot".to_string(),
                            PayloadWithUrn {
                                urn: Some("urn:oma:lwm2m:x:28182:0.2".to_string()),
                                data: Payload::MultiResourceValues(HashMap::from([
                                    (
                                        "0".to_string(),
                                        HashMap::from([
                                            (
                                                "start".to_string(),
                                                Value {
                                                    data: Time(ts_to_system_time(0)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "adjusted_stop".to_string(),
                                                Value {
                                                    data: Time(ts_to_system_time(0)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "state".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "initial_source".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "adjustment_source".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "pre_offset".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "actuator".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                        ])
                                    ),
                                    (
                                        "1".to_string(),
                                        HashMap::from([
                                            (
                                                "start".to_string(),
                                                Value {
                                                    data: Time(ts_to_system_time(0)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "adjusted_stop".to_string(),
                                                Value {
                                                    data: Time(ts_to_system_time(0)),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "state".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "initial_source".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "adjustment_source".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "pre_offset".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(0))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                            (
                                                "actuator".to_string(),
                                                Value {
                                                    data: Integer(Some(Unsigned(1))),
                                                    time: Some(ts_to_system_time(1742398480))
                                                }
                                            ),
                                        ])
                                    ),
                                ])),
                            }
                        ),
                    ])
                )])),
                metadata: None,
                success: Some(true)
            }]
        );
    }

    #[test]
    fn ipc_message_parsing() {
        let file = File::open("test/ipc_messages.txt").unwrap();
        let reader = io::BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let message = Message::<ObjectsPayload>::try_from_json_str(line);
                assert!(
                    message.is_ok(),
                    "cannot parse: {}\n{}",
                    message.unwrap_err(),
                    line
                );
            }
        }
    }
}
