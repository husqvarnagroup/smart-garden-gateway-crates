// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

/// State according to LWM2M standard (urn:oma:lwm2m:oma:5).
#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum FirmwareUpdateState {
    Idle = 0,
    Downloading = 1,
    DownloadComplete = 2,
    Updating = 3,
}

impl Default for FirmwareUpdateState {
    #[inline]
    fn default() -> FirmwareUpdateState {
        FirmwareUpdateState::Idle
    }
}

/// Update result according to LWM2M standard (urn:oma:lwm2m:oma:5).
#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    serde::Deserialize,
    serde::Serialize,
)]
#[allow(dead_code)]
pub enum FirmwareUpdateResult {
    Initial = 0,
    Success = 1,
    NoFlash = 2,
    NoRam = 3,
    ConnectionLost = 4,
    IntegrityCheckFail = 5,
    UnsupportedPackageType = 6,
    InvalidUri = 7,
    UpdateFailed = 8,
    UnsupportedProtocol = 9,
}

impl Default for FirmwareUpdateResult {
    #[inline]
    fn default() -> FirmwareUpdateResult {
        FirmwareUpdateResult::Initial
    }
}

/// Data download status according to GARDENA spec (urn:oma:lwm2m:x:28174).
#[derive(Debug, Eq, PartialEq, num_derive::ToPrimitive)]
#[allow(dead_code)]
pub enum DataDownloadStatus {
    Initial = 0,
    Uploading = 1,
    Activating = 2,
    Activated = 3,
    UploadFailed = 4,
    ActivationFailed = 5,
}

impl Default for DataDownloadStatus {
    #[inline]
    fn default() -> DataDownloadStatus {
        DataDownloadStatus::Initial
    }
}
