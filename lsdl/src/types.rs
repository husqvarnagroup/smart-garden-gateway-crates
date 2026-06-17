// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

//! This file defines types that are used inside payloads and part of the
//! Lemonbeat-specification but were not provided in a machine-readable form.

use anyhow::anyhow;
use anyhow::Context as _;
use num_traits::cast::FromPrimitive as _;

/// The device description types as described in Table 2.17 (2.2.6.2).
///
/// Settable types are marked with **rw**.
///
/// As with [Property], we are using a different name to reduce confusion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum PropertyId {
    /// Manufacturer type ID
    Type = 1,
    /// The manufacturer of the device.
    /// See Table 2.20(`Manufacturer`) for a list of manufacturers.
    Manufacturer = 2,
    /// Serialized Global Trade Item Number, which is a unique device number
    Sgtin = 3,
    /// The MAC address of the device, which is a unique device
    MacAddress = 4,
    /// The version of the hardware
    HardwareVersion = 5,
    /// The version of the bootloader
    BootloaderVersion = 6,
    /// The version of the stack
    StackVersion = 7,
    /// The version of the application
    ApplicationVersion = 8,
    /// The communication protocol that the device uses.
    /// See Table 2.18(`Protocol`) for a list of protocols.
    Protocol = 9,
    /// Manufacturer product ID
    Product = 10,
    /// Boolean value that indicates if the device is included.
    /// **rw**
    Included = 11,
    /// The name of the device.
    /// **rw**
    Name = 12,
    /// Device communication mode. See Table 2.19(`RadioMode`)
    RadioMode = 13,
    /// The time in milliseconds that the device is offline between
    /// consecutive RX-active periods. Currently set to 333 ms.
    /// **rw**
    WakeupInterval = 14,
    /// The offset in milliseconds needed for the calculation of the
    /// RX-active period. Not needed in the current version.
    WakeupOffset = 15,
    /// The radio channel that the device listens on for wake-up frames.
    /// **rw**
    WakeupChannel = 16,
    /// The current channel map having 4 channels defined as bit.
    /// **rw**
    ChannelMap = 17,
    /// The time it takes to scan a single channel
    ChannelScanTime = 18,
    /// IPv6 address
    Ipv6Address = 19,
    /// The time the device should stay awake. Only valid in the Partner Service.
    /// **rw**
    WakeupNow = 20,
    /// The diversity mode used in the device: 0 = off, 1 = on
    DiversityMode = 21,
    /// The TX power of the transceiver in dBm.
    /// **rw**
    TxPower = 27,
    /// Uptime (time since last reset) in seconds, since stack 1.6.0
    Uptime = 49,
}

/// The different protocols as described in Table 2.18 (2.2.6.2.1)
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum Protocol {
    Lemonbeat = 1,
    WiFi = 2,
    Ethernet = 3,
}

/// The Radio Mode describes how a device communicates over the air.
///
/// The default value for this is [RadioMode::AlwaysOnline].
/// Therefore, this information can be omitted in a device_description_report tag.
/// Lemonbeat specification: Table 2.19 (2.2.6.2.2)
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum RadioMode {
    AlwaysOnline = 0,
    /// only UDP is supported
    WakeOnRadio = 1,
    /// only UDP is supported
    WakeOnEvent = 2,
    TxOnly = 3,
    RxOnly = 4,
}

/// The different manufacturers as defined in Table 2.20 (2.2.6.2.3)
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum Manufacturer {
    /// RWE
    Rwe = 1,
    /// Seluxit
    Seluxit = 2,
    /// Gardena
    Gardena = 3,
    /// Lemonbeat
    Lemonbeat = 4,
    /// Alko
    Alko = 5,
    /// bit.b
    BitB = 6,
    /// innogy Metering
    InnogyMetering = 7,
    /// Pikkerton
    Pikkerton = 8,
}

#[derive(Debug, thiserror::Error)]
pub enum PropertyError {
    #[error("`{0}` is not a number")]
    NotANumber(u32),
    #[error("`{0}` is not a string")]
    NotAString(u32),
    #[error("`{0}` is not hex")]
    NotHex(u32),
    #[error("`{0}` is not valid hex: {1}")]
    FromHex(u32, hex::FromHexError),
}

/// Extended implementation for properties
///
/// Since we don't declare the struct in this module and [Property] is just a
/// typedef we have to use a trait to extend functionality.
/// The alternative would be to wrap [crate::xsd::common::infoType]
/// in our own struct but that seems less convenient.
pub trait PropertyEx {
    /// return ID of this property
    ///
    /// returns [None] if it can't be represented by [PropertyId].
    fn id<T: num_traits::cast::FromPrimitive>(&self) -> Option<T>;

    fn number(&self) -> Result<u64, PropertyError>;
    fn str(&self) -> Result<&str, PropertyError>;

    /// return value if this is a hex property
    ///
    /// The returned value is a copy and not a reference because lemonbeat stores
    /// this as a hex-string.
    fn hex(&self) -> Result<Vec<u8>, PropertyError>;
}

/// Lemonbeat device description property
///
/// The specification calls this `infoType`.
/// Following the specification for naming here would make for very long and
/// confusing names so we use one that actually describes the meaning.
pub type Property = crate::xsd::common::infoType;

impl Property {
    pub fn new_number<T: num_traits::cast::ToPrimitive>(
        id: T,
        value: u64,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            type_id: id.to_u32().context("can't convert property id to u32")?,
            number: Some(value),
            string: None,
            hex: None,
        })
    }

    pub fn new_string<T: num_traits::cast::ToPrimitive>(
        id: T,
        value: String,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            type_id: id.to_u32().context("can't convert property id to u32")?,
            number: None,
            string: Some(value),
            hex: None,
        })
    }

    pub fn new_hex<T: num_traits::cast::ToPrimitive>(
        id: T,
        value: &[u8],
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            type_id: id.to_u32().context("can't convert property id to u32")?,
            number: None,
            string: None,
            hex: Some(hex::encode_upper(value)),
        })
    }
}

impl PropertyEx for Property {
    fn id<T: num_traits::cast::FromPrimitive>(&self) -> Option<T> {
        T::from_u32(self.type_id)
    }

    fn number(&self) -> Result<u64, PropertyError> {
        self.number.ok_or(PropertyError::NotANumber(self.type_id))
    }

    fn str(&self) -> Result<&str, PropertyError> {
        self.string
            .as_deref()
            .ok_or(PropertyError::NotAString(self.type_id))
    }

    fn hex(&self) -> Result<Vec<u8>, PropertyError> {
        self.hex
            .as_ref()
            .ok_or(PropertyError::NotHex(self.type_id))
            .map_or_else(Err, |s| {
                hex::decode(s).map_err(|e| PropertyError::FromHex(self.type_id, e))
            })
    }
}

/// The valid options for the `memory_id` attributes in the
/// `memory_information` tag from Table 2.14.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum MemoryId {
    Value = 1,
    PartnerInformation = 2,
    ActionItems = 3,
    Calculation = 4,
    Timer = 5,
    Calendar = 6,
    Statemachine = 7,
    StatemachineTransactions = 8,
}

/// The valid options for the `type_id` attribute in the Value Description as
/// listed in Table 2.25 (2.2.7.1.1).
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum ValueType {
    Temperature = 1,
    Luminance = 2,
    Power = 3,
    Electricity = 4,
    Humidity = 5,
    Velocity = 6,
    Direction = 7,
    Atmospheric = 8,
    Barometric = 9,
    #[serde(rename = "Solar Radiation")]
    SolarRadiation = 10,
    #[serde(rename = "Dew Point")]
    DewPoint = 11,
    #[serde(rename = "Rain Rate")]
    RainRate = 12,
    #[serde(rename = "Tide Level")]
    TideLevel = 13,
    #[serde(rename = "On/Off")]
    OnOff = 14,
    #[serde(rename = "Awake State")]
    AwakeState = 15,
    Event = 16,
    #[serde(rename = "General Purpose")]
    GeneralPurpose = 17,
    Counter = 18,
    Energy = 19,
    Level = 20,
    CO2 = 21,
    #[serde(rename = "Airflow")]
    AirFlow = 22,
    #[serde(rename = "Tankcapacity")]
    TankCapacity = 23,
    Distance = 24,
    #[serde(rename = "Climatecontrol")]
    ClimateControl = 25,
    Program = 26,
    #[serde(rename = "Fanspeed")]
    FanSpeed = 27,
    #[serde(rename = "Error Code")]
    ErrorCode = 28,
    #[serde(rename = "Operation Mode")]
    OperationMode = 29,
    Louvre = 30,
    Mode = 31,
    Time = 32,
    #[serde(rename = "Duty Cycle")]
    DutyCycle = 33,
    Voltage = 34,
    Current = 35,
    Frequency = 36,
    Battery = 37,
    #[serde(rename = "Timezone Offset")]
    TimezoneOffset = 38,
    Year = 39,
    Month = 40,
    #[serde(rename = "Day Of Month")]
    DayOfMonth = 41,
    Weekday = 42,
    Hour = 43,
    Minute = 44,
}

/// Possible interaction with value
///
/// The lemonbeat spec calls this `mode` in Table 2.22.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum Permission {
    /// Read Only (from the device)
    #[serde(rename = "r")]
    ReadOnly = 1,
    /// Read/Write (to and from the device)
    #[serde(rename = "rw")]
    ReadWrite = 2,
    /// Write Only (to the device)
    #[serde(rename = "w")]
    WriteOnly = 3,
}

/// The configuration mode from Table 3.68 (3.2.17.1)
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum ConfigurationMode {
    /// Rollback any configuration changes
    Rollback = 0,
    /// Save any configuration changes and reset the state machine states
    SaveAndReset = 1,
    /// Save any configuration changes and do not change the state machine states
    SaveAndPreserve = 2,
    /// Clear the configuration and set the default configuration
    SetDefault = 3,
    /// Clear the configuration
    Clear = 4,
}

/// Status type from Table 3.52
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusType {
    PublicKey = 1,
    MemoryInformation = 2,
    DeviceDescription = 3,
    ValueDescription = 4,
    Value = 5,
    PartnerInformation = 6,
    Action = 7,
    Calculation = 8,
    Timer = 9,
    Calendar = 10,
    StateMachine = 11,
    FirmwareUpdate = 12,
    Configuration = 13,
    Exi = 100,
    System = 101,
    Application = 200,
}

/// Status level from Table 3.53
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusLevel {
    /// No status reports will be sent
    Disabled = 0,
    /// Only important status information will be sent
    Important = 1,
    /// An error
    Error = 2,
    /// A warning
    Warning = 3,
    /// Very verbose information
    Info = 4,
    /// Debug information
    Debug = 5,
}

/// device description code from Table 3.54
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeDeviceDescription {
    /// Trying to set a read-only value
    WrongId = 11,
    /// Trying to set a channel map that does not contain 4 channels
    WrongSizeOfChannelMap = 12,
    /// Trying to set a channel map without the required synchronization channels
    MissingSynchronizationChannels = 13,
}

/// value description code from Table 3.55
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeValueDescription {
    /// Trying to get a value description with an invalid ID
    GetWrongId = 1,
    /// Trying to add a virtual value description with an invalid ID
    SetWrongId = 2,
    /// Trying to delete a virtual value with an invalid ID
    DeleteWrongId = 3,
    /// Trying to add a virtual value with a type that is not supported
    NotSupported = 11,
    /// Trying to add a virtual value with an invalid step
    InvalidStep = 12,
}

/// value code from Table 3.56
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeValue {
    /// Trying to get a value with an invalid ID
    GetWrongId = 1,
    /// Trying to set a value with an invalid ID
    SetWrongId = 2,
    /// Trying to use a value with an invalid ID
    CheckWrongId = 11,
    /// The value is invalid
    ValueInvalid = 12,
    /// Trying to set a value with a wrong data type
    WrongDataType = 13,
    /// The step is not supported
    InvalidStep = 14,
    /// Trying to read a write-only value.
    CannotReadWriteonly = 15,
    /// Trying to write a read-only value.
    CannotWriteReadonly = 16,
}

/// partner information code from Table 3.57
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodePartnerInformation {
    /// Trying to get a partner that is not configured
    GetWrongId = 1,
    /// Trying to set a partner with a wrong ID
    SetWrongId = 2,
    /// Trying to delete a partner that is not configured
    DeleteWrongId = 3,
    /// The reference partner is not configured
    WrongId = 11,
    /// Sending a message to a partner but there was no reply
    FailedToSendToPartner = 12,
    /// Trying to set a partner with an unsupported info type
    SetWrongType = 13,
    /// Trying to add a group to a group
    GroupInGriupNotAllowed = 14,
    /// Trying to add more partners to a group than allowed
    TooManyPartnersInGroup = 15,
    /// Trying to add a partner with an address of an existing partner
    PartnerAlreadyExists = 16,

    // those were not part of the spec and were extracted from `nexus_status.h`
    AddressWrongLength = 17,
    ChannelMapWrongLength = 18,
    RadioSettingsWrongLength = 19,
}

/// action code from Table 3.58
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeAction {
    /// Trying to get an action that is not configured
    GetWrongId = 1,
    /// Trying to set an action with an invalid ID
    SetWrongId = 2,
    /// Trying to delete an action that is not configured
    DeleteWrongId = 3,
    /// Trying to execute an action that is not configured
    ExecuteWrongId = 11,
    /// Trying to enqueue action, but queue is full
    ExecuteQueueOverflow = 12,

    // those were not part of the spec and were extracted from `nexus_status.h`
    TransportModeNotSupported = 13,
}

/// calculation code from Table 3.59
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeCalculation {
    /// Trying to get an calculation that is not configured
    GetWrongId = 1,
    /// Trying to set an calculation with an invalid ID
    SetWrongId = 2,
    /// Trying to delete an calculation that is not configured
    DeleteWrongId = 3,
    /// Trying to evaluate a calculation that is not configured
    CheckWrongId = 11,
}

/// timer code from Table 3.60
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeTimer {
    /// Trying to get a timer that is not configured
    GetWrongId = 1,
    /// Trying to set a timer with a wrong ID
    SetWrongId = 2,
    /// Trying to delete a timer that is not configured
    DeleteWrongId = 3,
    /// Trying to start a timer that is not configured
    StartWrongId = 11,
    /// Trying to stop a timer that is not configured
    StopWrongId = 12,
}

/// calendar code from Table 3.61
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeCalendar {
    /// Trying to get a calendar task that is not configured
    GetWrongId = 1,
    /// Trying to set a calendar task with a wrong ID
    SetWrongId = 2,
    /// Trying to delete a calendar task that is not configured
    DeleteWrongId = 3,
}

/// state machine code from Table 3.62
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeStateMachine {
    /// Trying to get a state machine with an invalid ID
    GetWrongId = 1,
    /// Trying to set a state machine with an invalid ID
    SetWrongId = 2,
    /// Trying to delete a state machine with an invalid ID
    DeleteWrongId = 3,
    /// Trying to get a state machine state with an invalid ID
    GetWrongStateId = 4,
    /// Trying to set a state machine state with an invalid ID
    SetWrongStateId = 5,
    /// Trying to delete a state machine state with an invalid ID
    DeleteWrongStateId = 6,
    /// Trying to get a state machine state with invalid ID
    CheckWrongId = 11,
    /// The execution of the state machine was been running too long
    RunningTooLong = 12,
}

/// firmware update code from Table 3.63
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeFirmwareUpdate {
    /// Failed to upgrade firmware from boot loader
    FailedToUpgrade = 11,
}

/// configuration code from Table 3.64
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeConfiguration {
    /// The started configuration timed out and the configuration is rolled back
    Timeout = 11,
    /// The configuration saved on the device is invalid and needs to be validated
    Invalid = 12,
    /// The configuration has started on the device and the statemachine is stopped
    Started = 13,
}

/// EXI status code (not part of the spec)
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeExi {
    // those were not part of the spec and were extracted from `nexus_status.h`
    BufferOverflow = 1,
}

/// system code from Table 3.65
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum StatusCodeSystem {
    /// Failed to synchronize with the NTP Server
    NoNtp = 11,
    /// Device was woken up and is now awake
    Awake = 12,
    /// The device has detected a problem with the dataflash
    HardwareFailDataflash = 20,

    // those were not part of the spec and were extracted from `nexus_status.h`
    FactoryResetPending = 13,
    GatewayBusy = 30,
}

#[derive(Clone, Debug)]
pub enum StatusCode {
    DeviceDescription(StatusCodeDeviceDescription),
    ValueDescription(StatusCodeValueDescription),
    Value(StatusCodeValue),
    PartnerInformation(StatusCodePartnerInformation),
    Action(StatusCodeAction),
    Calculation(StatusCodeCalculation),
    Timer(StatusCodeTimer),
    Calendar(StatusCodeCalendar),
    StateMachine(StatusCodeStateMachine),
    FirmwareUpdate(StatusCodeFirmwareUpdate),
    Configuration(StatusCodeConfiguration),
    Exi(StatusCodeExi),
    System(StatusCodeSystem),

    // we don't have any values for those
    PublicKey(u32),
    MemoryInformation(u32),
    Application(u32),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RawStatus {
    pub type_id: u32,
    pub code: u32,
    pub level: u32,
    pub data: Option<String>,
}

impl From<crate::xsd::status::statusReportType> for RawStatus {
    fn from(report: crate::xsd::status::statusReportType) -> Self {
        Self {
            type_id: report.type_id,
            code: report.code,
            level: report.level,
            data: report.data,
        }
    }
}

impl RawStatus {
    pub fn level(&self) -> Option<StatusLevel> {
        StatusLevel::from_u32(self.level)
    }

    pub fn code(&self) -> anyhow::Result<StatusCode> {
        let code = match StatusType::from_u32(self.type_id)
            .ok_or_else(|| anyhow!("unknown type_id"))?
        {
            StatusType::PublicKey => StatusCode::PublicKey(self.code),
            StatusType::MemoryInformation => StatusCode::MemoryInformation(self.code),
            StatusType::DeviceDescription => StatusCode::DeviceDescription(
                StatusCodeDeviceDescription::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown device-description code"))?,
            ),
            StatusType::ValueDescription => StatusCode::ValueDescription(
                StatusCodeValueDescription::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown value-description code"))?,
            ),
            StatusType::Value => StatusCode::Value(
                StatusCodeValue::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown value code"))?,
            ),
            StatusType::PartnerInformation => StatusCode::PartnerInformation(
                StatusCodePartnerInformation::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown partner-information code"))?,
            ),
            StatusType::Action => StatusCode::Action(
                StatusCodeAction::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown action code"))?,
            ),
            StatusType::Calculation => StatusCode::Calculation(
                StatusCodeCalculation::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown calculation code"))?,
            ),
            StatusType::Timer => StatusCode::Timer(
                StatusCodeTimer::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown timer code"))?,
            ),
            StatusType::Calendar => StatusCode::Calendar(
                StatusCodeCalendar::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown calendar code"))?,
            ),
            StatusType::StateMachine => StatusCode::StateMachine(
                StatusCodeStateMachine::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown state-machine code"))?,
            ),
            StatusType::FirmwareUpdate => StatusCode::FirmwareUpdate(
                StatusCodeFirmwareUpdate::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown firmware-update code"))?,
            ),
            StatusType::Configuration => StatusCode::Configuration(
                StatusCodeConfiguration::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown configuration code"))?,
            ),
            StatusType::Exi => StatusCode::Exi(
                StatusCodeExi::from_u32(self.code).ok_or_else(|| anyhow!("unknown exi code"))?,
            ),
            StatusType::System => StatusCode::System(
                StatusCodeSystem::from_u32(self.code)
                    .ok_or_else(|| anyhow!("unknown system code"))?,
            ),
            StatusType::Application => StatusCode::Application(self.code),
        };
        Ok(code)
    }
}

/// The valid values for type_id in partner information as described in Table 3.30
#[derive(Clone, Copy, Debug, Eq, PartialEq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum PartnerPropertyId {
    /// The partners radio mode. See Table 3.19 or [RadioMode]
    RadioMode = 13,
    /// The offset in milliseconds indicating when the partner starts to wake up.
    /// Currently set to fixed 333 ms.
    WakeupInterval = 14,
    /// How often in milliseconds the partner will wake up.
    /// Not in use right now. For future purposes.
    WakeupOffset = 15,
    /// The radio channel that the partner listens on to see if it has to wake up
    WakeupChannel = 16,
    /// The current channel map
    ChannelMap = 17,
    /// The time is takes to scan a single channel
    ChannelScanTime = 18,
    /// The full IPv6 address of the partner
    Ipv6Address = 19,
    /// The time in milliseconds the partner should be awake
    WakeupNow = 20,
}

pub const FIRMWARE_UPDATE_STATUS_OK: u32 = 1;
pub const FIRMWARE_UPDATE_STATUS_NOT_OK: u32 = 2;
pub const FIRMWARE_UPDATE_TOO_BIG: u32 = 3;
pub const FIRMWARE_UPDATE_CHECKSUM_ERR: u32 = 4;
pub const FIRMWARE_UPDATE_WRONG_OFFSET: u32 = 6;
pub const FIRMWARE_UPDATE_BLOCKED_BY_APPLICATION: u32 = 10;

pub const CONFIGURATION_STATUS_IDLE: u32 = 0;
pub const CONFIGURATION_STATUS_INIT: u32 = 1;
pub const CONFIGURATION_STATUS_STARTED: u32 = 2;
