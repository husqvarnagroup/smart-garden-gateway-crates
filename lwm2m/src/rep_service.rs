// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use crate::Request;
use anyhow::Context as _;

/// Start Rep0 service.
///
/// An internally spawned task will handle IPC/LWM2M requests on the given
/// service socket.
pub fn start_repservice<Fut, E>(
    service: sg_ipc::RepService,
    name: String,
    callback: impl Fn(Request) -> Fut + Clone + Send + Sync + 'static,
) -> Result<(), anyhow::Error>
where
    Fut: std::future::Future<Output = Result<serde_json::Value, E>> + Send,
    E: std::fmt::Display + Into<anyhow::Error> + Send + Sync,
{
    struct Context {
        sequence: u64,
        name: String,
    }

    let context = std::sync::Arc::new(std::sync::Mutex::new(Context { sequence: 0, name }));

    service.start(move | msg| {
        let context = context.clone();
        let callback = callback.clone();
        async move {
            let messages: Vec<Request> = match serde_json::from_str(&msg.clone()) {
                Ok(v) => v,
                Err(e) => {
                    let strmessage = match std::str::from_utf8(msg.as_bytes()) {
                        Ok(v) => std::borrow::Cow::Borrowed(v),
                        Err(_) => std::borrow::Cow::Owned(hex::encode(msg.clone())),
                    };
                    tracing::error!(
                        ipcmsg = &*strmessage,
                        "Can't parse IPC message as json: {:?}",
                        e
                    );

                    // need a minimal reply otherwise IPC will send request over and over again
                    let response = serde_json::json!({
                        "success": false,
                        "payload": {
                            "vs": "could not parse request"
                        },
                        "metadata": {
                            "error_source": context.lock().unwrap().name,
                        }
                    });

                    return Ok::<String, anyhow::Error>(response.to_string());
                }
            };

            if messages.iter().any(Request::is_big) {
                tracing::debug!("Request: too big to log");
            } else {
                tracing::debug!("Request: {:#?}", messages);
            }

            let mut results = Vec::new();
            let sequence = {
                let mut context = context.lock().unwrap();
                let sequence = context.sequence;
                context.sequence = context.sequence.wrapping_add(1);
                sequence
            };
            for msg in messages {
                let entity = msg.entity.clone();
                results.push(match callback(msg).await {
                    Ok(payload) => serde_json::json!({
                        "success": true,
                        "entity": entity,
                        "payload": payload,
                        "metadata": {
                            "source": &context.lock().unwrap().name,
                            "sequence": sequence,
                        },
                    }),
                    Err(e) => serde_json::json!({
                        "success": false,
                        "entity": entity,
                        "metadata": {
                            "source": &context.lock().unwrap().name,
                            "sequence": sequence,
                            "error_reason": e.into().chain().map(ToString::to_string).collect::<Vec<String>>()
                        },
                    }),
                });
            }

            let response =
                serde_json::to_string(&results).context("can't convert json response to string")?;
            tracing::debug!("Response: {}", response);

            Ok(response)
        }
    })
}
