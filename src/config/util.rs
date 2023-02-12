// pub(super) use display_from_str::{
//     deserialize as deserialize_from_str, serialize as serialize_display,
// };

pub(super) mod display_from_str {
    use std::{fmt::Display, marker::PhantomData, str::FromStr};

    use serde::{de::Visitor, Deserializer, Serializer};

    struct DeserializeFromStrVisitor<T>(PhantomData<T>);
    impl<'de, T: FromStr> Visitor<'de> for DeserializeFromStrVisitor<T> {
        type Value = T;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a str")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            v.parse()
                .map_err(|_| E::custom(format!("Failed to parse str")))
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
    {
        deserializer.deserialize_str(DeserializeFromStrVisitor::<T>(PhantomData::default()))
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        serializer.serialize_str(&format!("{}", value))
    }
}

#[allow(unused)]
pub(super) mod display_from_str_option {
    use std::{fmt::Display, marker::PhantomData, str::FromStr};

    use serde::{de::Visitor, Deserializer, Serializer};

    struct DeserializeFromStrOptionVisitor<T>(PhantomData<T>);
    impl<'de, T: FromStr> Visitor<'de> for DeserializeFromStrOptionVisitor<T> {
        type Value = Option<T>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a str")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            v.parse()
                .map(|v| Some(v))
                .map_err(|_| E::custom(format!("Failed to parse str")))
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(Self(PhantomData::default()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
    {
        deserializer
            .deserialize_option(DeserializeFromStrOptionVisitor::<T>(PhantomData::default()))
    }

    pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        match value {
            Some(value) => serializer.serialize_some(&format!("{}", value)),
            None => serializer.serialize_none(),
        }
    }
}
