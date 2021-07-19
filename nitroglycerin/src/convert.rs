use std::borrow::Cow;

use rusoto_dynamodb::AttributeValue;

use crate::{AttributeError, Attributes};

#[cfg(feature = "uuid")]
mod uuid;

#[cfg(feature = "chrono")]
mod chrono;

/// Remove and parse a value from an Attribute `HashMap`
///
/// # Errors
/// Will return an error if the key is missing from map or of the value could not be parsed
pub fn extract<T: FromAttributeValue>(map: &mut Attributes, key: &str) -> Result<T, AttributeError> {
    T::try_from_av(map.remove(key).ok_or_else(|| AttributeError::MissingField(key.to_owned()))?)
}

/// Trait for types that can be created from `AttribueValues`
pub trait FromAttributeValue: Sized {
    /// try convert the attribute value into Self
    ///
    /// # Errors
    /// Will return an error value could not be parsed into `Self`
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError>;
}

/// Trait for types that can be converted into `AttribueValues`
pub trait IntoAttributeValue: Sized {
    /// convert self into an attribute value
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
        self.map_or_else(
            || AttributeValue {
                null: Some(true),
                ..AttributeValue::default()
            },
            T::into_av,
        )
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
