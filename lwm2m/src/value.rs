// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

use crate::Error;
use anyhow::{anyhow, Context};
use base64::Engine;
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

fn serialize_systemtime_stamp<S>(
    t: &std::time::SystemTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(
        t.duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?
            .as_secs(),
    )
}

fn deserialize_systemtime_stamp<'de, D>(deserializer: D) -> Result<std::time::SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds_since_epoch = u64::deserialize(deserializer)?;

    // special case required as long as gateway builds use libc with 32-bit time_t
    let seconds_since_epoch = seconds_since_epoch.min(i32::MAX as u64);

    std::time::SystemTime::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(seconds_since_epoch))
        .ok_or_else(|| serde::de::Error::custom("can't convert to SystemTime"))
}

#[allow(unknown_lints)] // TODO: remove after clippy has been upgraded to 1.83.0
#[allow(clippy::ref_option)]
fn serialize_opt_systemtime_stamp<S>(
    t: &Option<std::time::SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(t) = t {
        serializer.serialize_u64(
            t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_err(serde::ser::Error::custom)?
                .as_secs(),
        )
    } else {
        serializer.serialize_none()
    }
}

pub fn deserialize_opt_systemtime_stamp<'de, D>(
    deserializer: D,
) -> Result<Option<std::time::SystemTime>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u64>::deserialize(deserializer)? {
        None => Ok(None),
        Some(n) => Ok(Some(
            std::time::SystemTime::UNIX_EPOCH
                .checked_add(std::time::Duration::from_secs(n))
                .context("time is too large")
                .map_err(serde::de::Error::custom)?,
        )),
    }
}

fn serialize_opt_systemtime_stamp_array<S>(
    arr: &[Option<std::time::SystemTime>],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(arr.len()))?;

    for value in arr {
        seq.serialize_element(&match value {
            Some(t) => Some(
                t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .map_err(serde::ser::Error::custom)?
                    .as_secs(),
            ),
            None => None,
        })?;
    }

    seq.end()
}

pub fn deserialize_opt_systemtime_stamp_array<'de, D>(
    deserializer: D,
) -> Result<Vec<Option<std::time::SystemTime>>, D::Error>
where
    D: Deserializer<'de>,
{
    let arr = Vec::<Option<u64>>::deserialize(deserializer)?;

    let mut res = Vec::with_capacity(arr.len());
    for t in arr {
        res.push(match t {
            Some(t) => Some(
                std::time::SystemTime::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_secs(t))
                    .context("time is too large")
                    .map_err(serde::de::Error::custom)?,
            ),
            None => None,
        });
    }

    Ok(res)
}

fn serialize_base64<S>(v: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::prelude::BASE64_STANDARD.encode(v))
}

pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    base64::prelude::BASE64_STANDARD
        .decode(s)
        .map_err(serde::de::Error::custom)
}

fn serialize_base64_array<S>(arr: &[Option<Vec<u8>>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(arr.len()))?;

    for value in arr {
        seq.serialize_element(
            &value
                .as_ref()
                .map(|v| base64::prelude::BASE64_STANDARD.encode(v)),
        )?;
    }

    seq.end()
}

pub fn deserialize_base64_array<'de, D>(deserializer: D) -> Result<Vec<Option<Vec<u8>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let arr = Vec::<Option<String>>::deserialize(deserializer)?;

    let mut res = Vec::with_capacity(arr.len());
    for s in arr {
        res.push(match s {
            Some(s) => Some(
                base64::prelude::BASE64_STANDARD
                    .decode(s)
                    .map_err(serde::de::Error::custom)?,
            ),
            None => None,
        });
    }

    Ok(res)
}

// NOTE: remove/improve serde derives. We currently don't need object links
//       so it has no priority.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectLink(String);

// TODO: unused?
// NOTE: remove/improve serde derives. We currently don't need object links
//       so it has no priority.
#[derive(Debug, Serialize, Deserialize)]
pub struct CoreLink(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Integer {
    Unsigned(u64),
    Signed(i64),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IntegerArray {
    Signed(Vec<Option<i64>>),
    Unsigned(Vec<Option<u64>>),
}

#[allow(clippy::module_name_repetitions)]
/// value-data as sent through IPC
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueData {
    #[serde(rename = "vs")]
    String(Option<String>),
    #[serde(rename = "vi")]
    Integer(Option<Integer>),
    #[serde(rename = "vf")]
    Float(Option<f64>),
    #[serde(rename = "vb")]
    Boolean(Option<bool>),
    #[serde(rename = "vo")]
    #[serde(serialize_with = "serialize_base64")]
    #[serde(deserialize_with = "deserialize_base64")]
    Opaque(Vec<u8>),
    #[serde(rename = "vt")]
    #[serde(serialize_with = "serialize_systemtime_stamp")]
    #[serde(deserialize_with = "deserialize_systemtime_stamp")]
    Time(std::time::SystemTime),
    #[serde(rename = "vl")]
    Objlnk(Option<ObjectLink>),

    #[serde(rename = "as")]
    StringArray(Vec<Option<String>>),
    #[serde(rename = "ai")]
    IntegerArray(IntegerArray),
    #[serde(rename = "af")]
    FloatArray(Vec<Option<f64>>),
    #[serde(rename = "ab")]
    BooleanArray(Vec<Option<bool>>),
    #[serde(rename = "ao")]
    #[serde(serialize_with = "serialize_base64_array")]
    #[serde(deserialize_with = "deserialize_base64_array")]
    OpaqueArray(Vec<Option<Vec<u8>>>),
    #[serde(rename = "at")]
    #[serde(serialize_with = "serialize_opt_systemtime_stamp_array")]
    #[serde(deserialize_with = "deserialize_opt_systemtime_stamp_array")]
    TimeArray(Vec<Option<std::time::SystemTime>>),
    #[serde(rename = "al")]
    ObjlnkArray(Vec<Option<ObjectLink>>),
}

impl std::convert::From<String> for ValueData {
    fn from(v: String) -> Self {
        Self::String(Some(v))
    }
}

impl std::convert::From<i64> for ValueData {
    fn from(v: i64) -> Self {
        Self::Integer(Some(Integer::Signed(v)))
    }
}

impl std::convert::From<u64> for ValueData {
    fn from(v: u64) -> Self {
        Self::Integer(Some(Integer::Unsigned(v)))
    }
}

impl std::convert::From<f64> for ValueData {
    fn from(v: f64) -> Self {
        Self::Float(Some(v))
    }
}

impl std::convert::From<bool> for ValueData {
    fn from(v: bool) -> Self {
        Self::Boolean(Some(v))
    }
}

impl std::convert::From<Vec<u8>> for ValueData {
    fn from(v: Vec<u8>) -> Self {
        Self::Opaque(v)
    }
}

impl std::convert::From<std::time::SystemTime> for ValueData {
    fn from(v: std::time::SystemTime) -> Self {
        Self::Time(v)
    }
}

impl std::convert::From<ObjectLink> for ValueData {
    fn from(v: ObjectLink) -> Self {
        Self::Objlnk(Some(v))
    }
}

impl std::convert::From<Vec<Option<String>>> for ValueData {
    fn from(v: Vec<Option<String>>) -> Self {
        Self::StringArray(v)
    }
}

impl std::convert::From<Vec<Option<i64>>> for ValueData {
    fn from(v: Vec<Option<i64>>) -> Self {
        Self::IntegerArray(IntegerArray::Signed(v))
    }
}

impl std::convert::From<Vec<Option<u64>>> for ValueData {
    fn from(v: Vec<Option<u64>>) -> Self {
        Self::IntegerArray(IntegerArray::Unsigned(v))
    }
}

impl std::convert::From<Vec<Option<f64>>> for ValueData {
    fn from(v: Vec<Option<f64>>) -> Self {
        Self::FloatArray(v)
    }
}

impl std::convert::From<Vec<Option<bool>>> for ValueData {
    fn from(v: Vec<Option<bool>>) -> Self {
        Self::BooleanArray(v)
    }
}

impl std::convert::From<Vec<Option<Vec<u8>>>> for ValueData {
    fn from(v: Vec<Option<Vec<u8>>>) -> Self {
        Self::OpaqueArray(v)
    }
}

impl std::convert::From<Vec<Option<std::time::SystemTime>>> for ValueData {
    fn from(v: Vec<Option<std::time::SystemTime>>) -> Self {
        Self::TimeArray(v)
    }
}

impl std::convert::From<Vec<Option<ObjectLink>>> for ValueData {
    fn from(v: Vec<Option<ObjectLink>>) -> Self {
        Self::ObjlnkArray(v)
    }
}

impl std::convert::TryFrom<ValueData> for String {
    type Error = Error;

    fn try_from(v: ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::String(Some(v)) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!(
                "failed to convert ValueData to String"
            ))),
        }
    }
}

impl<'a> std::convert::TryFrom<&'a ValueData> for &'a str {
    type Error = Error;

    fn try_from(v: &'a ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::String(Some(v)) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!(
                "failed to convert ValueData to &str"
            ))),
        }
    }
}

impl std::convert::TryFrom<ValueData> for i64 {
    type Error = Error;

    fn try_from(v: ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Integer(Some(Integer::Signed(v))) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!("failed to convert ValueData to i64"))),
        }
    }
}

impl std::convert::TryFrom<ValueData> for u64 {
    type Error = Error;

    fn try_from(v: ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Integer(Some(Integer::Unsigned(v))) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!("failed to convert ValueData to u64"))),
        }
    }
}

impl std::convert::TryFrom<&ValueData> for u64 {
    type Error = Error;

    fn try_from(v: &ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Integer(Some(Integer::Unsigned(v))) => Ok(*v),
            _ => Err(Error::Anyhow(anyhow!("failed to convert ValueData to u64"))),
        }
    }
}

impl std::convert::TryFrom<ValueData> for f64 {
    type Error = Error;

    fn try_from(v: ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Float(Some(v)) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!("failed to convert ValueData to f64"))),
        }
    }
}

impl std::convert::TryFrom<ValueData> for bool {
    type Error = Error;

    fn try_from(v: ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Boolean(Some(v)) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!(
                "failed to convert ValueData to bool"
            ))),
        }
    }
}

impl std::convert::TryFrom<ValueData> for Vec<u8> {
    type Error = Error;

    fn try_from(v: ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Opaque(v) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!(
                "failed to convert ValueData to Vec<u8>"
            ))),
        }
    }
}

impl<'a> std::convert::TryFrom<&'a ValueData> for &'a [u8] {
    type Error = Error;

    fn try_from(v: &'a ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Opaque(v) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!(
                "failed to convert ValueData to &[u8]"
            ))),
        }
    }
}

impl std::convert::TryFrom<ValueData> for std::time::SystemTime {
    type Error = Error;

    fn try_from(v: ValueData) -> Result<Self, Self::Error> {
        match v {
            ValueData::Time(v) => Ok(v),
            _ => Err(Error::Anyhow(anyhow!(
                "failed to convert ValueData to std::time::SystemTime"
            ))),
        }
    }
}

impl std::fmt::Display for ValueData {
    /// manual implementation to meet troubleshooters needs of no internal type information
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValueData::String(None)
            | ValueData::Integer(None)
            | ValueData::Float(None)
            | ValueData::Boolean(None) => f.write_str("<empty>"),
            ValueData::String(Some(v)) => std::fmt::Display::fmt(v, f),
            ValueData::Integer(Some(Integer::Unsigned(v))) => std::fmt::Display::fmt(v, f),
            ValueData::Integer(Some(Integer::Signed(v))) => std::fmt::Display::fmt(v, f),
            ValueData::Float(Some(v)) => std::fmt::Display::fmt(v, f),
            ValueData::Boolean(Some(v)) => std::fmt::Display::fmt(v, f),
            ValueData::Opaque(v) => write!(f, "0x{}", hex::encode(v)),
            ValueData::Time(v) => std::fmt::Debug::fmt(v, f),
            ValueData::Objlnk(v) => std::fmt::Debug::fmt(v, f),
            ValueData::StringArray(v) => std::fmt::Debug::fmt(v, f),
            ValueData::IntegerArray(v) => std::fmt::Debug::fmt(v, f),
            ValueData::FloatArray(v) => std::fmt::Debug::fmt(v, f),
            ValueData::BooleanArray(v) => std::fmt::Debug::fmt(v, f),
            ValueData::OpaqueArray(v) => std::fmt::Debug::fmt(v, f),
            ValueData::TimeArray(v) => std::fmt::Debug::fmt(v, f),
            ValueData::ObjlnkArray(v) => std::fmt::Debug::fmt(v, f),
        }
    }
}

/// value map as sent through IPC
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Value {
    #[serde(flatten)]
    pub data: ValueData,
    #[serde(default, rename = "ts")]
    #[serde(serialize_with = "serialize_opt_systemtime_stamp")]
    #[serde(deserialize_with = "deserialize_opt_systemtime_stamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<std::time::SystemTime>,
}

impl Value {
    pub fn new(data: ValueData, time: Option<std::time::SystemTime>) -> Self {
        Self { data, time }
    }

    pub fn is_big(&self) -> bool {
        match self {
            Value {
                data: ValueData::Opaque(binary),
                time: _,
            } => binary.len() > 100,
            _ => false,
        }
    }
}

impl<T: std::convert::Into<ValueData>> std::convert::From<TimedData<T>> for Value {
    fn from(v: TimedData<T>) -> Self {
        Self {
            data: v.0.into(),
            time: v.1,
        }
    }
}

fn vec_add_index<T>(vec: &mut Vec<Option<T>>, index: usize, value: T) -> anyhow::Result<()> {
    if let Some(element) = vec.get_mut(index) {
        if element.is_some() {
            return Err(anyhow!("value with index {} does already exist", index));
        }

        *element = Some(value);
    } else {
        while vec.len() < index {
            vec.push(None);
        }
        vec.push(Some(value));
    }

    Ok(())
}

fn vec_new_with_index<T>(index: usize, value: T) -> anyhow::Result<Vec<Option<T>>> {
    let mut vec = vec![];
    vec_add_index(&mut vec, index, value)?;
    Ok(vec)
}

impl Value {
    pub fn into_array(self, id: usize) -> anyhow::Result<Self> {
        Ok(Self {
            data: match self.data {
                ValueData::String(Some(v)) => ValueData::StringArray(vec_new_with_index(id, v)?),
                ValueData::Integer(Some(Integer::Signed(v))) => {
                    ValueData::IntegerArray(IntegerArray::Signed(vec_new_with_index(id, v)?))
                }
                ValueData::Integer(Some(Integer::Unsigned(v))) => {
                    ValueData::IntegerArray(IntegerArray::Unsigned(vec_new_with_index(id, v)?))
                }
                ValueData::Float(Some(v)) => ValueData::FloatArray(vec_new_with_index(id, v)?),
                ValueData::Boolean(Some(v)) => ValueData::BooleanArray(vec_new_with_index(id, v)?),
                ValueData::Opaque(v) => ValueData::OpaqueArray(vec_new_with_index(id, v)?),
                ValueData::Time(v) => ValueData::TimeArray(vec_new_with_index(id, v)?),
                ValueData::Objlnk(Some(v)) => ValueData::ObjlnkArray(vec_new_with_index(id, v)?),
                _ => return Err(anyhow!("can't convert array-value to array")),
            },
            time: self.time,
        })
    }

    pub fn add_to_array(&mut self, id: usize, other: Self) -> anyhow::Result<()> {
        match (&mut self.data, other.data) {
            (ValueData::StringArray(arr), ValueData::String(Some(v))) => vec_add_index(arr, id, v)?,
            (
                ValueData::IntegerArray(IntegerArray::Signed(arr)),
                ValueData::Integer(Some(Integer::Signed(v))),
            ) => vec_add_index(arr, id, v)?,
            (
                ValueData::IntegerArray(IntegerArray::Unsigned(arr)),
                ValueData::Integer(Some(Integer::Unsigned(v))),
            ) => vec_add_index(arr, id, v)?,
            (ValueData::FloatArray(arr), ValueData::Float(Some(v))) => vec_add_index(arr, id, v)?,
            (ValueData::BooleanArray(arr), ValueData::Boolean(Some(v))) => {
                vec_add_index(arr, id, v)?;
            }
            (ValueData::OpaqueArray(arr), ValueData::Opaque(v)) => vec_add_index(arr, id, v)?,
            (ValueData::ObjlnkArray(arr), ValueData::Objlnk(Some(v))) => vec_add_index(arr, id, v)?,
            (ValueData::TimeArray(arr), ValueData::Time(v)) => vec_add_index(arr, id, v)?,
            _ => return Err(anyhow!("can't add to array(different types or non-array)")),
        }

        if let Some(other_time) = other.time {
            if let Some(time) = self.time {
                // always use the oldest time
                if other_time < time {
                    self.time = Some(other_time);
                }
            } else {
                self.time = Some(other_time);
            }
        }

        Ok(())
    }
}

/// A tuple which holds both a value and a time.
pub type TimedData<T> = (T, Option<std::time::SystemTime>);

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::value::{Error as ValueError, I32Deserializer, U32Deserializer};
    use serde::de::IntoDeserializer;
    use std::time::{Duration, SystemTime};

    #[test]
    fn deserialize_systemtime_stamp_0() {
        let deserializer: U32Deserializer<ValueError> = 0u32.into_deserializer();
        assert_eq!(
            deserialize_systemtime_stamp(deserializer),
            Ok(SystemTime::UNIX_EPOCH)
        );
    }

    #[test]
    fn deserialize_systemtime_stamp_i32_max() {
        let deserializer: I32Deserializer<ValueError> = i32::MAX.into_deserializer();
        assert_eq!(
            deserialize_systemtime_stamp(deserializer),
            Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(i32::MAX as u64))
        );
    }

    #[test]
    fn deserialize_systemtime_stamp_u32_max() {
        // special case required as long as gateway builds use libc with 32-bit time_t
        let deserializer: U32Deserializer<ValueError> = u32::MAX.into_deserializer();
        assert_eq!(
            deserialize_systemtime_stamp(deserializer),
            Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(i32::MAX as u64))
        );
    }
}
