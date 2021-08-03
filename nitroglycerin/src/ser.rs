use bytes::Bytes;
use rusoto_dynamodb::AttributeValue;
use serde::{
    ser::{self, Error, SerializeMap},
    Serialize,
};
use thiserror::Error;

use crate::Attributes;
pub struct Serializer<'a> {
    output: &'a mut AttributeValue,
}

#[derive(Debug, Error)]
pub enum SerError {
    #[error("{0}")]
    Message(String),

    #[error("expected string type")]
    ExpectedStr,
}

impl ser::Error for SerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerError::Message(msg.to_string())
    }
}

type Result<T, E = SerError> = std::result::Result<T, E>;

pub fn to_av<T>(value: &T) -> Result<AttributeValue>
where
    T: Serialize,
{
    let mut output = AttributeValue::default();
    let serializer = Serializer { output: &mut output };
    value.serialize(serializer)?;
    Ok(output)
}

impl<'a> ser::Serializer for Serializer<'a> {
    type Ok = ();
    type Error = SerError;

    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = SeqSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = MapSerializer<'a>;
    type SerializeStructVariant = MapSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.bool = Some(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output.n = Some(v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.n = Some(v.to_string());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.n = Some(v.to_string());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output.s = Some(v.to_owned());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output.b = Some(Bytes::copy_from_slice(v));
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.output.null = Some(true);
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.m.get_or_insert_with(Attributes::new).insert(variant.to_owned(), to_av(&value)?);
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer {
            output: self.output.l.get_or_insert_with(|| Vec::with_capacity(len.unwrap_or_default())),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(SeqSerializer {
            output: self.output.l.get_or_insert_with(|| Vec::with_capacity(len)),
        })
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant> {
        let m = self.output.m.get_or_insert_with(Attributes::new);
        let av = m.entry(variant.to_owned()).or_default();

        Ok(SeqSerializer {
            output: av.l.get_or_insert_with(|| Vec::with_capacity(len)),
        })
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer {
            output: self.output.m.get_or_insert_with(|| Attributes::with_capacity(len.unwrap_or_default())),
            key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(MapSerializer {
            output: self.output.m.get_or_insert_with(|| Attributes::with_capacity(len)),
            key: None,
        })
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant> {
        let m = self.output.m.get_or_insert_with(Attributes::new);
        let av = m.entry(variant.to_owned()).or_default();
        Self { output: av }.serialize_struct(variant, len)
    }
}

pub struct SeqSerializer<'a> {
    output: &'a mut Vec<AttributeValue>,
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.push(to_av(&value)?);
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.push(to_av(&value)?);
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.push(to_av(&value)?);
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for SeqSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.push(to_av(&value)?);
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub struct MapSerializer<'a> {
    output: &'a mut Attributes,
    key: Option<String>,
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let s = StrSerializer { output: &mut self.key };
        key.serialize(s)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.key.take().ok_or_else(|| SerError::custom("missing key"))?;
        self.output.insert(key, to_av(&value)?);
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for MapSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_entry(key, value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for MapSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_entry(key, value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

struct StrSerializer<'a> {
    output: &'a mut Option<String>,
}

impl<'a> ser::Serializer for StrSerializer<'a> {
    type Ok = ();

    type Error = SerError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_u8(self, _v: u8) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_u16(self, _v: u16) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_u32(self, _v: u32) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_u64(self, _v: u64) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        *self.output = Some(v.to_owned());
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_none(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<()> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(SerError::ExpectedStr)
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
        Err(SerError::ExpectedStr)
    }
}

impl<'a> ser::SerializeSeq for StrSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn end(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }
}

impl<'a> ser::SerializeTuple for StrSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn end(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }
}

impl<'a> ser::SerializeTupleStruct for StrSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn end(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }
}

impl<'a> ser::SerializeTupleVariant for StrSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn end(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }
}

impl<'a> ser::SerializeMap for StrSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn end(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }
}

impl<'a> ser::SerializeStruct for StrSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn end(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }
}

impl<'a> ser::SerializeStructVariant for StrSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(SerError::ExpectedStr)
    }

    fn end(self) -> Result<()> {
        Err(SerError::ExpectedStr)
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use rusoto_dynamodb::AttributeValue;

    use super::to_av;

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test { int: 1, seq: vec!["a", "b"] };
        let expected = AttributeValue {
            m: Some(
                <_>::into_iter([
                    ("int".to_owned(), AttributeValue {
                        n: Some("1".to_owned()),
                        ..AttributeValue::default()
                    }),
                    ("seq".to_owned(), AttributeValue {
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
                    }),
                ])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        assert_eq!(to_av(&test).unwrap(), expected);
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let u = E::Unit;
        let expected = AttributeValue {
            s: Some("Unit".to_owned()),
            ..AttributeValue::default()
        };
        assert_eq!(to_av(&u).unwrap(), expected);

        let n = E::Newtype(1);
        let expected = AttributeValue {
            m: Some(
                <_>::into_iter([("Newtype".to_owned(), AttributeValue {
                    n: Some("1".to_owned()),
                    ..AttributeValue::default()
                })])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        assert_eq!(to_av(&n).unwrap(), expected);

        let t = E::Tuple(1, 2);
        let expected = AttributeValue {
            m: Some(
                <_>::into_iter([("Tuple".to_owned(), AttributeValue {
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
                })])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        assert_eq!(to_av(&t).unwrap(), expected);

        let s = E::Struct { a: 1 };
        let expected = AttributeValue {
            m: Some(
                <_>::into_iter([("Struct".to_owned(), AttributeValue {
                    m: Some(
                        <_>::into_iter([("a".to_owned(), AttributeValue {
                            n: Some("1".to_owned()),
                            ..AttributeValue::default()
                        })])
                        .collect(),
                    ),
                    ..AttributeValue::default()
                })])
                .collect(),
            ),
            ..AttributeValue::default()
        };
        assert_eq!(to_av(&s).unwrap(), expected);
    }
}
