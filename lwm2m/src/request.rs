// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use crate::{Request, Response};
use anyhow::anyhow;
use anyhow::Context as _;

/// Perform a client request on the given service socket.
///
/// The protocol allows sending multiple LWM2M requests in one IPC message.
/// This is supposed to make things more efficient or ensure a certain order.
pub async fn make_requests<T: for<'a> serde::Deserialize<'a> + std::fmt::Debug>(
    service: &mut sg_ipc::ReqService,
    requests: &[Request],
) -> Result<Vec<Response<T>>, anyhow::Error> {
    let data = serde_json::to_string(requests)?;

    let msg = service.send(data).await.context("can't send request")?;

    let result = serde_json::from_str::<Vec<Response<T>>>(&msg);
    if result.is_err() {
        tracing::debug!("Failed to parse JSON: {msg:?}");
    }
    result.context("can't parse json")
}

/// Perform a multiple client requests on the given service socket.
///
/// The protocol allows sending multiple LWM2M requests in one IPC message.
/// This is supposed to make things more efficient or ensure a certain order.
#[allow(clippy::module_name_repetitions)]
pub async fn make_request<T: for<'a> serde::Deserialize<'a> + std::fmt::Debug>(
    service: &mut sg_ipc::ReqService,
    requests: &Request,
) -> Result<Response<T>, anyhow::Error> {
    let mut responses = make_requests(service, std::slice::from_ref(requests)).await?;
    if responses.len() != 1 {
        return Err(anyhow!(
            "received {} responses to a single request: {:#?}",
            responses.len(),
            responses
        ));
    }

    Ok(responses.remove(0))
}
