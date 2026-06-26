// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use crate::{Error, Method, ObjectType, Payload, Request, Value, ValueData};
use anyhow::anyhow;
use async_trait::async_trait;

/// Generic object type that IPC can interact with.
#[async_trait]
pub trait Object: Send + Sync {
    /// Return URN for this object.
    fn urn(&self) -> &'static str;

    /// Read resource instance from given object instance.
    async fn read_resource(
        &self,
        object_instance: usize,
        resource_id: usize,
        resource_instance: usize,
    ) -> Result<Value, Error>;

    async fn write_resource(
        &mut self,
        object_instance: usize,
        resource_id: usize,
        resource_instance: usize,
        value: Value,
    ) -> Result<(), Error>;

    async fn exec(
        &mut self,
        object_instance: usize,
        resource_id: usize,
        resource_instance: usize,
        args: Option<Vec<String>>,
    ) -> Result<(), Error>;

    /// Returns the lwm2m resource id.
    fn parse_resource_name(&self, name: &str) -> Result<usize, Error>;

    /// Returns the lwm2m resource name.
    fn get_resource_name(&self, resource_id: usize) -> Result<&str, Error>;

    /// Returns a bitmask with the supported operations for the given resource.
    fn supported_resource_operations(&self, resource_id: usize) -> Result<usize, Error>;

    /// Returns true if this resource supports multiple instances.
    fn is_array_resource(&self, resource_id: usize) -> Result<bool, Error>;

    async fn handle_resource_instance(
        &self,
        object_instance: usize,
        resource_instances: &[(usize, usize)],
        resource_id_needle: usize,
    ) -> Result<Value, anyhow::Error> {
        let mut res: Option<Value> = None;

        for (resource_id, resource_instance) in resource_instances {
            if (self.supported_resource_operations(*resource_id)? & lwm2m_types::OP_READ) == 0 {
                continue;
            }
            if *resource_id != resource_id_needle {
                continue;
            }

            let val = self
                .read_resource(object_instance, *resource_id, *resource_instance)
                .await?;

            if let Some(val_arr) = &mut res {
                if !self.is_array_resource(*resource_id)? {
                    return Err(anyhow!("multiple non-array resource instances"));
                }

                val_arr.add_to_array(*resource_instance, val)?;
            } else if self.is_array_resource(*resource_id)? {
                res = Some(val.into_array(*resource_instance)?);
            } else {
                if *resource_instance != 0 {
                    return Err(anyhow!(
                        "non-zero resource-instance `{resource_id}` with non-array resource"
                    ));
                }

                res = Some(val);
            }
        }

        res.ok_or_else(|| anyhow!("no resource of type {resource_id_needle}"))
    }

    async fn handle_resources<'a>(
        &'a self,
        object_instance: usize,
        resource_instances: &[(usize, usize)],
    ) -> Result<std::collections::HashMap<&'a str, Value>, Error> {
        let mut res = std::collections::HashMap::<&str, Value>::new();

        for (resource_id, resource_instance) in resource_instances {
            if (self.supported_resource_operations(*resource_id)? & lwm2m_types::OP_READ) == 0 {
                continue;
            }

            let val = self
                .read_resource(object_instance, *resource_id, *resource_instance)
                .await?;

            let resource_name = self.get_resource_name(*resource_id)?;
            if let Some(val_arr) = res.get_mut(resource_name) {
                if !self.is_array_resource(*resource_id)? {
                    return Err(Error::Anyhow(anyhow!(
                        "multiple non-array resource instances"
                    )));
                }

                val_arr.add_to_array(*resource_instance, val)?;
            } else if self.is_array_resource(*resource_id)? {
                res.insert(resource_name, val.into_array(*resource_instance)?);
            } else {
                if *resource_instance != 0 {
                    return Err(Error::Anyhow(anyhow!(
                        "non-zero resource-instance `{resource_id}` with non-array resource"
                    )));
                }

                res.insert(resource_name, val);
            }
        }

        Ok(res)
    }

    async fn handle_partial_write(
        &mut self,
        _object_instance: usize,
        _values: std::collections::HashMap<String, Value>,
    ) -> Result<(), Error> {
        Err(Error::UnsupportedPartialWrite)
    }
}

/// Endpoint that IPC can interact with.
#[async_trait]
pub trait Endpoint: Send + Sync {
    /// Get handle for given object type.
    fn get_object<'a>(&'a mut self, ty: ObjectType) -> Result<Box<dyn Object + 'a>, Error>;

    /// Get list of object instances.
    fn object_list(&self) -> Vec<(ObjectType, usize)>;

    /// Get list of object instances for given object type.
    ///
    /// (resource-id, num-instances)
    fn resource_instance_list(&self, ty: ObjectType) -> Vec<(usize, usize)>;

    /// Return a type that an be used to serialize the requested object.
    async fn serializable_object<'a>(
        &'a mut self,
        ty: ObjectType,
        id: usize,
    ) -> Result<serde_json::Value, Error> {
        let resource_instances = self.resource_instance_list(ty);
        let object = self.get_object(ty)?;
        Ok(serde_json::json!(
            object.handle_resources(id, &resource_instances).await?
        ))
    }

    /// serialize the whole endpoint into a json value
    ///
    /// NOTE: usually we'd want this to be unrelated to json but since we're
    ///       still using json values internally we can't really provide that
    ///       right now.
    async fn json_endpoint<'a>(&'a mut self) -> Result<serde_json::Value, Error> {
        let mut res_object_types = std::collections::HashMap::<
            &str,
            std::collections::HashMap<String, serde_json::Value>,
        >::new();

        for (object_type, object_instance) in self.object_list() {
            let resource_instances = self.resource_instance_list(object_type);
            let object = self.get_object(object_type)?;
            let val = serde_json::json!(
                object
                    .handle_resources(object_instance, &resource_instances)
                    .await?
            );

            let object_type_str = object_type.as_str();
            if let Some(res_object_type) = res_object_types.get_mut(object_type_str) {
                res_object_type.insert(object_instance.to_string(), val);
            } else {
                let mut res_object_type = std::collections::HashMap::new();
                res_object_type.insert("_urn".to_string(), object.urn().into());
                res_object_type.insert(object_instance.to_string(), val);

                res_object_types.insert(object_type_str, res_object_type);
            }
        }

        Ok(serde_json::json!(res_object_types))
    }

    /// handle IPC request to this endpoint
    ///
    /// this will eventually run callbacks of the supported resources
    #[allow(clippy::too_many_lines)]
    async fn handle_request(&mut self, request: Request) -> Result<serde_json::Value, Error> {
        let mut path = request.entity.path.iter();
        let object_type = if let Some(s) = path.next() {
            let t: ObjectType = s
                .to_str()
                .ok_or_else(|| anyhow!("object-type is not utf8"))?
                .parse()?;
            Some(t)
        } else {
            None
        };
        let object_instance = if let Some(s) = path.next() {
            let n: usize = s
                .to_str()
                .ok_or_else(|| anyhow!("object-instance is not utf8"))?
                .parse()
                .map_err(|e| anyhow::Error::new(e).context("can't parse object instance id"))?;

            Some(n)
        } else {
            None
        };
        let resource_name = if let Some(s) = path.next() {
            Some(
                s.to_str()
                    .ok_or_else(|| anyhow!("resource-type is not utf8"))?,
            )
        } else {
            None
        };
        let resource_instance = if let Some(s) = path.next() {
            let n: usize = s
                .to_str()
                .ok_or_else(|| anyhow!("resource-instance is not utf8"))?
                .parse()
                .map_err(|e| anyhow::Error::new(e).context("can't parse resource instance id"))?;

            Some(n)
        } else {
            // NOTE: this will NOT work for multi instance resources,
            //       will have to correct it soon
            Some(0)
        };
        if path.next().is_some() {
            return Err(Error::Anyhow(anyhow!(
                "path has too many segments: {path:?}"
            )));
        }

        Ok(if let Some(object_type) = object_type {
            if let Some(object_instance) = object_instance {
                if let Some(resource_name) = resource_name {
                    // access a single resource instance
                    if let Some(resource_instance) = resource_instance {
                        let mut object = self.get_object(object_type)?;
                        let resource_id = object.parse_resource_name(resource_name)?;

                        match &request.op {
                            Method::Read => {
                                let val = object
                                    .read_resource(object_instance, resource_id, resource_instance)
                                    .await?;
                                serde_json::json!(val)
                            }
                            Method::Write => match request.payload {
                                Some(Payload::Value(v)) => serde_json::json!(
                                    object
                                        .write_resource(
                                            object_instance,
                                            resource_id,
                                            resource_instance,
                                            v
                                        )
                                        .await?
                                ),
                                Some(_) => {
                                    return Err(Error::Anyhow(anyhow!(
                                        "multiple-values for resource-instance writ"
                                    )))
                                }
                                None => {
                                    return Err(Error::Anyhow(anyhow!("write: no value provided")))
                                }
                            },
                            Method::Execute => {
                                let args = match request.payload {
                                    Some(Payload::Value(payload)) => match payload.data {
                                        ValueData::StringArray(arr) => {
                                            Some(arr.into_iter().flatten().collect())
                                        }
                                        _ => None,
                                    },
                                    Some(Payload::Values(_)) => {
                                        return Err(anyhow!(
                                            "execute doesn't support multiple values"
                                        )
                                        .into())
                                    }
                                    _ => None,
                                };
                                serde_json::json!(
                                    object
                                        .exec(object_instance, resource_id, resource_instance, args)
                                        .await?
                                )
                            }
                            other => {
                                return Err(Error::Anyhow(anyhow!(
                                    "unsupported method: {other:?}"
                                )));
                            }
                        }
                    }
                    // list all resource instances of this resource-type
                    else {
                        let resource_instances = self.resource_instance_list(object_type);
                        let object = self.get_object(object_type)?;
                        let resource_id = object.parse_resource_name(resource_name)?;

                        match &request.op {
                            Method::Read => serde_json::json!(
                                object
                                    .handle_resource_instance(
                                        object_instance,
                                        &resource_instances,
                                        resource_id,
                                    )
                                    .await?
                            ),
                            other => {
                                return Err(Error::Anyhow(anyhow!(
                                    "unsupported method: {other:?}"
                                )));
                            }
                        }
                    }
                }
                // list all resources(+all instances) of this object instance
                else {
                    let resource_instances = self.resource_instance_list(object_type);
                    let mut object = self.get_object(object_type)?;

                    match &request.op {
                        Method::Read => serde_json::json!(
                            object
                                .handle_resources(object_instance, &resource_instances)
                                .await?
                        ),
                        // NOTE: We assume `Write (Partial Update)`
                        //       If write-replace is needed in future, we should
                        //       probably add that to the `Method` enum.
                        Method::Write => match request.payload {
                            Some(Payload::Values(values)) => serde_json::json!(
                                object.handle_partial_write(object_instance, values).await?
                            ),
                            Some(Payload::Value(_)) => {
                                return Err(Error::Anyhow(anyhow!(
                                    "can't partial-write single value"
                                )))
                            }
                            Some(Payload::MultiResourceValues(_)) => todo!(),
                            None => {
                                return Err(Error::Anyhow(anyhow!(
                                    "can't partial-write without payload"
                                )))
                            }
                        },
                        other => {
                            return Err(Error::Anyhow(anyhow!("unsupported method: {other:?}")));
                        }
                    }
                }
            }
            // list all objects of the current type
            else {
                let resource_instances = self.resource_instance_list(object_type);
                let object_list = self.object_list();
                let object = self.get_object(object_type)?;

                match &request.op {
                    Method::Read => {
                        let mut res = std::collections::HashMap::new();
                        res.insert("_urn".to_string(), serde_json::json!(object.urn()));

                        for (object_type_item, object_instance) in object_list {
                            if object_type_item == object_type {
                                continue;
                            }

                            res.insert(
                                object_instance.to_string(),
                                serde_json::json!(
                                    object
                                        .handle_resources(object_instance, &resource_instances)
                                        .await?
                                ),
                            );
                        }

                        serde_json::json!(res)
                    }
                    other => return Err(Error::Anyhow(anyhow!("unsupported method: {other:?}"))),
                }
            }
        }
        // list all objects of all types
        else {
            match &request.op {
                Method::Read => self.json_endpoint().await?,
                other => return Err(Error::Anyhow(anyhow!("unsupported method: {other:?}"))),
            }
        })
    }
}
