#![warn(clippy::pedantic)]

// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use lwm2m::{
    bnw_consumer::{self, BnwConsumer},
    lwm2mserver, DevicesPayload, Entity, EntityKind, Message, Method, ObjectsPayload,
    OwnedMetadata, Source,
};
use tokio::sync::mpsc;

const DEVICE_IDS_TO_INCLUDE: &[&str] = &["301400000000004000000047"];

type Sender = mpsc::Sender<(String, String)>;

async fn handle_message(message: &Message<ObjectsPayload>, tx: Sender) {
    use lwm2m::{Payload, PayloadWithUrn, ValueData};
    match message {
        Message {
            operation: Method::Update,
            entity:
                Some(Entity {
                    path,
                    kind: EntityKind::Gateway { service },
                }),
            payload:
                ObjectsPayload::Single(PayloadWithUrn {
                    data: Payload::Values(values),
                    ..
                }),
            ..
        } if service == lwm2mserver::SERVICE_NAME => {
            if let Some(ValueData::String(Some(identifier))) =
                values.get("identifier").map(|v| &v.data)
            {
                if let Ok(Some(includable_device_id)) =
                    path.strip_prefix("includable_device/").map(|p| p.to_str())
                {
                    log::info!("Includable device id: {includable_device_id:?}");
                    tx.send((includable_device_id.to_string(), identifier.clone()))
                        .await
                        .unwrap();
                } else {
                    log::warn!("Unhandled message: {message:?}");
                }
            } else {
                log::warn!("Identifier not found in payload: {message:?}");
            }
        }
        Message {
            operation: Method::Update,
            entity:
                Some(Entity {
                    path,
                    kind: EntityKind::Device { device },
                }),
            payload,
            metadata:
                Some(OwnedMetadata {
                    source: Source::Source(source),
                    sequence: _,
                }),
            success: None,
        } if source == lwm2mserver::SERVICE_NAME && path.is_relative() => {
            let mut device_path = std::path::PathBuf::from(device);
            device_path.push(path);
            log::info!("Value update: {device_path:?} {payload:?}");
        }
        _ => {
            log::warn!("Unhandled message: {message:?}");
        }
    }
}

async fn get_device_infos(
    consumer: &mut BnwConsumer,
) -> Result<DevicesPayload, bnw_consumer::Error> {
    consumer
        .request(&Message {
            operation: Method::Read,
            entity: Some(Entity {
                path: std::path::PathBuf::from("devices"),
                kind: EntityKind::Gateway {
                    service: lwm2mserver::SERVICE_NAME.to_string(),
                },
            }),
            payload: (),
            metadata: None,
            success: None,
        })
        .await
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    log::info!("Starting event listener");

    let (tx, mut rx) = mpsc::channel(5);

    let mut consumer = BnwConsumer::new(
        lwm2mserver::URL_PREFIX,
        lwm2mserver::SERVICE_NAME,
        move |frame| {
            let message: Message<ObjectsPayload> =
                serde_json::from_str(&frame).expect("Failed to deserialize message from frame");
            let tx = tx.clone();
            Box::pin(async move {
                handle_message(&message, tx.clone()).await;
            })
        },
    )
    .await?;

    let device_infos = get_device_infos(&mut consumer).await?;
    log::info!("Device Infos: {:?}", device_infos);

    while let Some((includable_device_id, identifier)) = rx.recv().await {
        if DEVICE_IDS_TO_INCLUDE.contains(&identifier.as_str()) {
            log::info!("Start including device {identifier} at id {includable_device_id}");
            let res = consumer.include(&includable_device_id).await;
            log::info!("Inclusion result: {res:?}");
        } else {
            log::info!("Not including device {identifier} at id {includable_device_id}");
        }
    }
    Ok(())
}
