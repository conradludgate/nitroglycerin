use rusoto_dynamodb::AttributeValue;
use serde::{
    de::{self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor},
    Deserialize,
};
use thiserror::Error;

use crate::Attributes;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("expected string type")]
    ExpectedStr,

    #[error("missing field")]
    MissingField,

    #[error("expected array")]
    ExpectedArray,

    #[error("error parsing int {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("error parsing float {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

struct Deserializer {
    input: AttributeValue,
}

impl Deserializer {
    const fn from_av(input: AttributeValue) -> Self {
        Self { input }
    }
}

/// Deserialises a type from it's [`AttributeValue`] form
///
/// # Errors
///
/// This function will return an error if it fails to deserialise the schema
pub fn from_av<T>(s: AttributeValue) -> Result<T>
where
    for<'de> T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_av(s);
    T::deserialize(&mut deserializer)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.l.is_some() || self.input.bs.is_some() || self.input.ns.is_some() || self.input.ss.is_some() {
            self.deserialize_seq(visitor)
        } else if self.input.b.is_some() {
            self.deserialize_bytes(visitor)
        } else if self.input.bool.is_some() {
            self.deserialize_bool(visitor)
        } else if self.input.m.is_some() {
            self.deserialize_map(visitor)
        } else if self.input.n.is_some() {
            self.deserialize_i64(visitor)
        } else if self.input.null.is_some() {
            self.deserialize_unit(visitor)
        } else if self.input.s.is_some() {
            self.deserialize_str(visitor)
        } else {
            Err(Error::MissingField)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.input.bool.take().ok_or(Error::MissingField)?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.input.n.take().ok_or(Error::MissingField)?.parse()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let s = self.input.s.take().ok_or(Error::MissingField)?;
        if s.len() == 1 {
            visitor.visit_char(s.chars().next().unwrap())
        } else {
            Err(de::Error::custom("string is bigger than a single char"))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.input.s.take().ok_or(Error::MissingField)?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(&self.input.b.take().ok_or(Error::MissingField)?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input.null.take() {
            Some(true) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input.null.take() {
            Some(true) => visitor.visit_none(),
            _ => Err(de::Error::custom("expected null")),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Some(bs) = self.input.bs.take() {
            visitor.visit_seq(List(bs.into_iter().map(|b| AttributeValue {
                b: Some(b),
                ..AttributeValue::default()
            })))
        } else if let Some(l) = self.input.l.take() {
            visitor.visit_seq(List(l.into_iter()))
        } else if let Some(ns) = self.input.ns.take() {
            visitor.visit_seq(List(ns.into_iter().map(|n| AttributeValue {
                n: Some(n),
                ..AttributeValue::default()
            })))
        } else if let Some(ss) = self.input.ss.take() {
            visitor.visit_seq(List(ss.into_iter().map(|s| AttributeValue {
                s: Some(s),
                ..AttributeValue::default()
            })))
        } else {
            Err(Error::ExpectedArray)
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Map::new(self.input.m.take().ok_or(Error::MissingField)?))
    }

    fn deserialize_struct<V>(self, _name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Some(s) = self.input.s.take() {
            visitor.visit_enum(s.into_deserializer())
        } else if let Some(m) = self.input.m.take() {
            visitor.visit_enum(Enum::new(m)?)
        } else {
            Err(Error::MissingField)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct List<I>(I);

impl<'de, I: Iterator<Item = AttributeValue>> SeqAccess<'de> for List<I> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(av) => {
                let mut deserializer = Deserializer::from_av(av);
                seed.deserialize(&mut deserializer).map(Some)
            }
            None => Ok(None),
        }
    }
}

struct Map<I> {
    iter: I,
    value: Option<AttributeValue>,
}

impl<I> Map<I> {
    fn new(iter: impl IntoIterator<IntoIter = I>) -> Self {
        Self { iter: iter.into_iter(), value: None }
    }
}

impl<'de, I: Iterator<Item = (String, AttributeValue)>> MapAccess<'de> for Map<I> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let av = AttributeValue {
                    s: Some(key),
                    ..AttributeValue::default()
                };
                let mut deserializer = Deserializer::from_av(av);
                seed.deserialize(&mut deserializer).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => {
                let mut deserializer = Deserializer::from_av(value);
                seed.deserialize(&mut deserializer)
            }
            None => Err(Error::MissingField),
        }
    }
}
struct Enum {
    key: String,
    value: AttributeValue,
}

// struct Enum<'a, 'de: 'a> {
//     de: &'a mut Deserializer<'de>,
// }

impl Enum {
    fn new(map: Attributes) -> Result<Self> {
        let mut iter = map.into_iter();
        let (key, value) = iter.next().ok_or_else(|| Error::Message("no values in map for enum".into()))?;
        if iter.next().is_some() {
            return Err(Error::Message("too many values in map for enum".into()));
        }
        Ok(Self { key, value })
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de> EnumAccess<'de> for Enum {
    type Error = Error;
    type Variant = Variant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let key = AttributeValue {
            s: Some(self.key),
            ..AttributeValue::default()
        };
        let mut deserializer = Deserializer::from_av(key);
        let val = seed.deserialize(&mut deserializer)?;

        Ok((val, Variant(self.value)))
    }
}
struct Variant(AttributeValue);

impl<'de> VariantAccess<'de> for Variant {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::MissingField)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        let mut deserializer = Deserializer::from_av(self.0);
        seed.deserialize(&mut deserializer)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut deserializer = Deserializer::from_av(self.0);
        de::Deserializer::deserialize_seq(&mut deserializer, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut deserializer = Deserializer::from_av(self.0);
        de::Deserializer::deserialize_map(&mut deserializer, visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use rusoto_dynamodb::AttributeValue;
    use serde::Deserialize;

    use super::from_av;

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
        }

        let j = AttributeValue {
            m: Some(
                <_>::into_iter([
                    (
                        "int".to_owned(),
                        AttributeValue {
                            n: Some("1".to_owned()),
                            ..AttributeValue::default()
                        },
                    ),
                    (
                        "seq".to_owned(),
                        AttributeValue {
                            l: Some(vec![
                                AttributeValue {
                                    s: Some("a".to_owned()),
                                    ..AttributeValue::default()
                                },
                                AttributeValue {
                                    s: Some("b".to_owned()),
                                    ..AttributeValue::default()
                                },
                            ]),
                            ..AttributeValue::default()
                        },
                    ),
                ])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        let expected = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
        };
        assert_eq!(expected, from_av(j).unwrap());
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let expected = E::Unit;
        let u = AttributeValue {
            s: Some("Unit".to_owned()),
            ..AttributeValue::default()
        };
        assert_eq!(expected, from_av(u).unwrap());

        let expected = E::Newtype(1);
        let n = AttributeValue {
            m: Some(
                <_>::into_iter([(
                    "Newtype".to_owned(),
                    AttributeValue {
                        n: Some("1".to_owned()),
                        ..AttributeValue::default()
                    },
                )])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        assert_eq!(expected, from_av(n).unwrap());

        let expected = E::Tuple(1, 2);
        let t = AttributeValue {
            m: Some(
                <_>::into_iter([(
                    "Tuple".to_owned(),
                    AttributeValue {
                        l: Some(vec![
                            AttributeValue {
                                n: Some("1".to_owned()),
                                ..AttributeValue::default()
                            },
                            AttributeValue {
                                n: Some("2".to_owned()),
                                ..AttributeValue::default()
                            },
                        ]),
                        ..AttributeValue::default()
                    },
                )])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        assert_eq!(expected, from_av(t).unwrap());

        let expected = E::Struct { a: 1 };
        let s = AttributeValue {
            m: Some(
                <_>::into_iter([(
                    "Struct".to_owned(),
                    AttributeValue {
                        m: Some(
                            <_>::into_iter([(
                                "a".to_owned(),
                                AttributeValue {
                                    n: Some("1".to_owned()),
                                    ..AttributeValue::default()
                                },
                            )])
                            .collect(),
                        ),
                        ..AttributeValue::default()
                    },
                )])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        assert_eq!(expected, from_av(s).unwrap());
    }
}
