use rusoto_dynamodb::AttributeValue;

use super::{FromAttributeValue, IntoAttributeValue};
use crate::AttributeError;

macro_rules! convert_wrapper {
    ($ty:ty: $T:ty) => {
        impl FromAttributeValue for $ty {
            fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
                Ok(Self::new(<$T>::try_from_av(av)?))
            }
        }

        impl IntoAttributeValue for $ty {
            fn into_av(self) -> AttributeValue {
                // I dislike wrappers that 'protect' inner values with no way to take ownership of them :sob:
                unsafe { std::mem::transmute::<_, $T>(self) }.into_av()
            }
        }
    };
}

convert_wrapper!(oauth2::AccessToken: String);
convert_wrapper!(oauth2::RefreshToken: String);
convert_wrapper!(oauth2::Scope: String);

impl FromAttributeValue for oauth2::basic::BasicTokenType {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        let s = String::try_from_av(av)?;
        serde_json::from_str(&s).map_err(|e| AttributeError::ParseError(Box::new(e)))
    }
}

impl IntoAttributeValue for oauth2::basic::BasicTokenType {
    fn into_av(self) -> AttributeValue {
        serde_json::to_string(&self).expect("could not serialize token type").into_av()
    }
}
