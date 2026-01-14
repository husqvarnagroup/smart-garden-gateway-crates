// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use crate::{Entity, EntityKind, Error, Event, Metadata, Method};
use anyhow::anyhow;
use anyhow::Context as _;
use rand::Rng as _;

struct PubServiceInner {
    sequence: u64,

    /// map between includable device ids used by IPC and device IP addresses
    ///
    /// we actually have to map both ways but since we expect this list to be
    /// rather small we just accept having to iterate the whole map for one
    /// direction.
    ///
    /// There's the `bimap` crate which I don't want to use for these reasons:
    /// - it seems relatively small and unpopular and could thus go
    ///   unmaintained at any time
    /// - it seems very inefficient since all it does is maintain two
    ///   hashmaps kept in sync. I'm sure there is a better way to do this.
    ///
    /// In addition to all that we use a btreemap so it'll be faster to find a
    /// free ID in case the list is very full.
    includable_map: std::collections::BTreeMap<u16, std::net::IpAddr>,

    source: String,
}

impl PubServiceInner {
    fn sequence(&mut self) -> u64 {
        let ret = self.sequence;
        self.sequence += 1;
        ret
    }

    /// this randomly selects an ID within an inclusive range and allocates it.
    ///
    /// `unchecked` refers to not checking if the ID already exists in the map.
    fn includable_range_unchecked(
        &mut self,
        rng: &mut rand::rngs::ThreadRng,
        address: std::net::IpAddr,
        start: u16,
        end: u16,
    ) -> u16 {
        match end - start {
            // we expect our callers to never do that
            0 => unreachable!(),
            // a gap of 1, we have our id
            1 => {
                let new_id = start;
                self.includable_map.insert(new_id, address);
                new_id
            }
            // a gap of more than one, randomly chose our id
            _ => {
                let new_id: u16 = rng.gen_range(start..=end);
                self.includable_map.insert(new_id, address);
                new_id
            }
        }
    }

    /// Allocate an includable ID with proper randomness
    ///
    /// The code is a little bit more complex because it tries to avoid
    /// consuming too much CPU time in case there aren't many IDs left. One
    /// might be inclined to say that that's unlikely to happen or that
    /// brute-forcing random numbers is probably gonna yield a free channel
    /// within a reasonable amount of time, but the solution here isn't complex
    /// enough to not do it.
    fn includable_id(&mut self, address: std::net::IpAddr) -> Result<u16, anyhow::Error> {
        // we have an id already
        if let Some((&id, _)) = self.includable_map.iter().find(|(_k, &v)| v == address) {
            return Ok(id);
        }

        // no space left
        let u16max: usize = u16::MAX as usize;
        if self.includable_map.len() == u16max {
            return Err(anyhow!("no includable device IDs left"));
        }

        // calculate the odds that we get a used id on the first attempt.
        // The odds are `used-ids / total-ids`.
        // Then you can divide both sides by `used-ids` to reduce it to the
        // `1 in X` format.
        //
        // The following code is just the simplified version of all those steps.
        // The odds will be `1 : odds` for a used id.
        let odds = match self.includable_map.len() {
            0 => u16max,
            _ => u16max / self.includable_map.len(),
        };

        // try to randomly find a free slot
        // we only try that `odds` times because it's somewhat unlikely to hit
        // `odds` used IDs in a row when the odds are high.
        // When the odds are low we don't try all that often and will fall back
        // to the iterative method of finding the lowest free ID.
        let mut rng = rand::thread_rng();
        for _ in 0..odds {
            let id: u16 = rng.gen();

            if let std::collections::btree_map::Entry::Vacant(e) = self.includable_map.entry(id) {
                e.insert(address);
                return Ok(id);
            }
        }

        // Iterate through the whole map until we found our free ID.
        // While this is really slow by comparison, this is okay because:
        // - we're using u16. u16::MAX iterations shouldn't take too long
        // - it's very unlikely for the list to become anywhere even close to
        //   being full given how realistic it would be to have over 65000
        //   unincluded devices in reach.
        let mut prev_id = 0;
        for id in self.includable_map.keys() {
            match id - prev_id {
                // 0 can only happen for id `0`. 1 is the previous id + 1.
                0 | 1 => (),
                // we found a gap
                _ => {
                    let maxid = *id - 1;
                    return Ok(self.includable_range_unchecked(
                        &mut rng,
                        address,
                        prev_id + 1,
                        maxid,
                    ));
                }
            }

            prev_id = *id;
        }

        if prev_id == u16::MAX {
            // PANIC: this should never happen because we checked the length of
            //        the map at the beginning of this function
            unreachable!()
        }

        Ok(self.includable_range_unchecked(&mut rng, address, prev_id + 1, u16::MAX))
    }

    fn remove_includable_device(&mut self, address: std::net::IpAddr) -> Result<(), anyhow::Error> {
        let (&id, _) = self
            .includable_map
            .iter()
            .find(|(_k, &v)| v == address)
            .ok_or_else(|| anyhow!("device with address=`{}` not found", address))?;

        self.includable_map.remove(&id);

        Ok(())
    }

    fn address_from_includable_id(&self, id: u16) -> Option<std::net::IpAddr> {
        self.includable_map.get(&id).copied()
    }
}

/// A LWM2M Pub0 service
///
/// This service sends LWM2M events.
/// Cloning it results in another handle to the same underlying service.
#[derive(Clone)]
pub struct PubService {
    ipc_pub_service: sg_ipc::PubService,
    inner: std::sync::Arc<std::sync::Mutex<PubServiceInner>>,
}

impl PubService {
    pub fn new(ipc_pub_service: sg_ipc::PubService, source: String) -> Self {
        Self {
            ipc_pub_service,
            inner: std::sync::Arc::new(std::sync::Mutex::new(PubServiceInner {
                sequence: 0,
                includable_map: std::collections::BTreeMap::new(),
                source,
            })),
        }
    }

    pub fn publish_includable_device(
        &mut self,
        address: std::net::IpAddr,
        payload: &serde_json::Value,
        op: Method,
    ) -> Result<(), anyhow::Error> {
        let source = self.inner.lock().unwrap().source.clone().into();
        let sequence = self.inner.lock().unwrap().sequence();
        let service = self.inner.lock().unwrap().source.clone();
        let includable_id = self.inner.lock().unwrap().includable_id(address)?;
        let msg = serde_json::to_string(&serde_json::json!([Event {
            op,
            entity: Entity {
                path: format!("includable_device/{includable_id}").into(),
                kind: EntityKind::Gateway { service },
            },
            payload: Some(std::borrow::Cow::Borrowed(payload)),
            metadata: Some(Metadata { source, sequence }),
        }]))?;
        log::trace!("Publish includable device: {}", msg);

        self.ipc_pub_service.publish(&msg)?;
        Ok(())
    }

    /// free up includable-device ID
    ///
    /// this does not publish a delete event
    pub fn remove_includable_device(
        &mut self,
        address: std::net::IpAddr,
    ) -> Result<(), anyhow::Error> {
        self.inner.lock().unwrap().remove_includable_device(address)
    }

    pub fn address_from_includable_id(&self, id: u16) -> Option<std::net::IpAddr> {
        self.inner.lock().unwrap().address_from_includable_id(id)
    }

    /// Sends an IPC deletion event for this device.
    pub fn publish_device_deletion(&mut self, device: String) -> Result<(), Error> {
        let source = self.inner.lock().unwrap().source.clone().into();
        let sequence = self.inner.lock().unwrap().sequence();
        let msg = serde_json::to_string(&serde_json::json!([Event {
            op: Method::Delete,
            entity: Entity {
                path: "".into(),
                kind: EntityKind::Device { device },
            },
            payload: None,
            metadata: Some(Metadata { source, sequence }),
        }]))
        .context("can't create json string for deletion event")?;

        log::trace!("Publish device-deletion: {}", msg);
        self.ipc_pub_service
            .publish(&msg)
            .context("can't publish deletion event to IPC")?;

        Ok(())
    }

    /// Publishes an update of the item given by `path`
    pub fn publish_update(
        &mut self,
        device: String,
        path: String,
        payload: serde_json::Value,
    ) -> Result<(), Error> {
        let source = self.inner.lock().unwrap().source.clone().into();
        let sequence = self.inner.lock().unwrap().sequence();
        let msg = serde_json::to_string(&serde_json::json!([Event {
            op: Method::Update,
            entity: Entity {
                path: path.into(),
                kind: EntityKind::Device { device },
            },
            payload: Some(std::borrow::Cow::Owned(payload)),
            metadata: Some(Metadata { source, sequence }),
        }]))
        .context("can't create json string for endpoint event")?;

        log::trace!("Publish update: {}", msg);
        if let Err(e) = self.ipc_pub_service.publish(&msg) {
            log::error!("can't publish update event to IPC: {}", e);
        }

        Ok(())
    }
}
