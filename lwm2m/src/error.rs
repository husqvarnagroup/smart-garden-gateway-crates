// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("unsupported optional resource")]
    UnsupportedOptionalResource,

    #[error("object-instance doesn't support partial-write")]
    UnsupportedPartialWrite,
}
