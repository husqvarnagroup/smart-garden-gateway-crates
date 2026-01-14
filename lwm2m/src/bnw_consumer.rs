// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use crate::{Entity, EntityKind, Message, Method, ObjectsPayload, Payloads};
use anyhow::Context;
use sg_ipc::ReqService;
use sg_ipc::SubService;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

const EVENT_URL_SUFFIX: &str = "-event.ipc";
const COMMAND_URL_SUFFIX: &str = "-command.ipc";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unsuccessful response: {0}")]
    UnsuccessfulResponse(String),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub struct BnwConsumer {
    pub service_name: String,
    _sub_service: SubService,
    req_service: ReqService,
}

impl BnwConsumer {
    pub async fn new<F>(url_prefix: &str, service_name: &str, callback: F) -> Result<Self, Error>
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + Clone + 'static,
    {
        let event_url = url_prefix.to_string() + EVENT_URL_SUFFIX;
        let mut sub_service = SubService::new(&event_url);
        sub_service.start(callback).await?;

        let command_url = url_prefix.to_string() + COMMAND_URL_SUFFIX;
        let req_service = ReqService::new(&command_url).await?;

        Ok(Self {
            service_name: service_name.to_string(),
            _sub_service: sub_service,
            req_service,
        })
    }

    fn include_message(service_name: &str, includable_device_id: &str) -> Message<()> {
        Message {
            operation: Method::Execute,
            entity: Some(Entity {
                path: PathBuf::from(format!("includable_device/{includable_device_id}/include")),
                kind: EntityKind::Gateway {
                    service: service_name.to_string(),
                },
            }),
            payload: (),
            metadata: None,
            success: None,
        }
    }

    pub async fn request<T: Payloads, S: Payloads + Sync>(
        &mut self,
        request: &Message<S>,
    ) -> Result<T, Error> {
        let request_data = serde_json::to_string(request)?;
        log::trace!("Sending command request: {request_data}");
        let response = self.req_service.send(request_data).await?;
        log::trace!("Received command response: {response}");
        let Message {
            operation,
            entity,
            payload,
            metadata,
            success,
        } = serde_json::de::from_str::<Message<T>>(&response)
            .context("Failed to deserialize response")?;
        if !success.unwrap_or_default() {
            return Err(Error::UnsuccessfulResponse(
                payload.get_single_string().unwrap_or_default().to_string(),
            ));
        }
        if !operation.is_none() {
            log::warn!(
                "Operation field in response expected to be None, but was {:?}",
                operation
            );
        }
        if entity.is_some() {
            log::warn!(
                "Entity field in response expected to be None, but was {:?}",
                operation
            );
        }
        if metadata.is_some() {
            log::warn!(
                "Metadata field in response expected to be None, but was {:?}",
                operation
            );
        }
        Ok(payload)
    }

    pub async fn include(&mut self, includable_device_id: &str) -> Result<(), Error> {
        let response_payload: ObjectsPayload = self
            .request(&Self::include_message(
                &self.service_name,
                includable_device_id,
            ))
            .await?;
        if !response_payload.is_none() {
            log::warn!(
                "Response payload expected to be None, but was {:?}",
                response_payload
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwm2mserver;

    #[test]
    fn serialize_include_message() {
        let msg = BnwConsumer::include_message(lwm2mserver::SERVICE_NAME, "42");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(
            json,
            "{\
                \"op\":\"execute\",\
                \"entity\":{\"path\":\"includable_device/42/include\",\"service\":\"lwm2mserver\"}\
            }"
        );
    }
}
