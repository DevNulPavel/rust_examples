//! (De)serialization for [`UmbraString`]

use std::{fmt, marker::PhantomData};

use crate::{
    heap::{ThinAsBytes, ThinDrop},
    UmbraString,
};

struct Visitor<B>(PhantomData<B>);

impl<'v, B> serde::de::Visitor<'v> for Visitor<B>
where
    B: ThinDrop + From<Vec<u8>> + for<'b> From<&'b [u8]>,
{
    type Value = UmbraString<B>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        UmbraString::try_from(s).map_err(serde::de::Error::custom)
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        UmbraString::try_from(s).map_err(serde::de::Error::custom)
    }
}

impl<B> serde::Serialize for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self)
    }
}

impl<'de, B> serde::Deserialize<'de> for UmbraString<B>
where
    B: ThinDrop + From<Vec<u8>> + for<'b> From<&'b [u8]>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(Visitor(PhantomData))
    }
}
