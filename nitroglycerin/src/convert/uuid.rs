use rusoto_dynamodb::AttributeValue;
use uuid::Uuid;

use crate::AttributeError;

use super::{FromAttributeValue, IntoAttributeValue};


impl FromAttributeValue for Uuid {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        let s = String::try_from_av(av)?;
        Uuid::parse_str(&s).map_err(|e| AttributeError::ParseError(Box::new(e)))
    }
}

impl IntoAttributeValue for Uuid {
    fn into_av(self) -> AttributeValue {
        self.to_string().into_av()
    }
}
