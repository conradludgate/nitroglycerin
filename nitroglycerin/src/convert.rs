use std::borrow::Cow;

use rusoto_dynamodb::AttributeValue;

use crate::{AttributeError, Attributes};

pub fn extract<T: FromAttributeValue>(map: &mut Attributes, key: &str) -> Result<T, AttributeError> {
    T::try_from_av(map.remove(key).ok_or_else(|| AttributeError::MissingField(key.to_owned()))?)
}

pub trait FromAttributeValue: Sized {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError>;
}

pub trait IntoAttributeValue: Sized {
    fn into_av(self) -> AttributeValue;
}

impl FromAttributeValue for String {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        av.s.ok_or(AttributeError::IncorrectType)
    }
}

impl IntoAttributeValue for String {
    fn into_av(self) -> AttributeValue {
        AttributeValue {
            s: Some(self),
            ..AttributeValue::default()
        }
    }
}

impl<T> FromAttributeValue for Cow<'_, T>
where
    T: ToOwned,
    T::Owned: FromAttributeValue,
{
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        T::Owned::try_from_av(av).map(Cow::Owned)
    }
}

impl<T> IntoAttributeValue for Cow<'_, T>
where
    T: ToOwned,
    T::Owned: IntoAttributeValue,
{
    fn into_av(self) -> AttributeValue {
        self.into_owned().into_av()
    }
}

impl<T> FromAttributeValue for Vec<T>
where
    T: FromAttributeValue,
{
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        let l = av.l.ok_or(AttributeError::IncorrectType)?;
        l.into_iter().map(T::try_from_av).collect()
    }
}

impl<T> IntoAttributeValue for Vec<T>
where
    T: IntoAttributeValue,
{
    fn into_av(self) -> AttributeValue {
        AttributeValue {
            l: Some(self.into_iter().map(T::into_av).collect()),
            ..AttributeValue::default()
        }
    }
}

impl<T> FromAttributeValue for Option<T>
where
    T: FromAttributeValue,
{
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        match av.null {
            Some(true) => Ok(None),
            _ => Ok(Some(T::try_from_av(av)?)),
        }
    }
}

impl<T> IntoAttributeValue for Option<T>
where
    T: IntoAttributeValue,
{
    fn into_av(self) -> AttributeValue {
        self.map(T::into_av).unwrap_or_else(|| AttributeValue {
            null: Some(true),
            ..AttributeValue::default()
        })
    }
}

macro_rules! convert_num {
    ($n:ident) => {
        impl FromAttributeValue for $n {
            fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
                let n = av.n.ok_or(AttributeError::IncorrectType)?;
                match n.parse() {
                    Ok(n) => Ok(n),
                    Err(e) => Err(AttributeError::ParseError(Box::new(e))),
                }
            }
        }

        impl IntoAttributeValue for $n {
            fn into_av(self) -> AttributeValue {
                AttributeValue {
                    n: Some(self.to_string()),
                    ..AttributeValue::default()
                }
            }
        }
    };
}

convert_num!(isize);
convert_num!(i128);
convert_num!(i64);
convert_num!(i32);
convert_num!(i16);
convert_num!(i8);
convert_num!(usize);
convert_num!(u128);
convert_num!(u64);
convert_num!(u32);
convert_num!(u16);
convert_num!(u8);
convert_num!(f64);
convert_num!(f32);
