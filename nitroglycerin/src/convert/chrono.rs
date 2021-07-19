use std::time::SystemTime;

use chrono::{DateTime, FixedOffset, Local, Utc};
use rusoto_dynamodb::AttributeValue;

use super::{FromAttributeValue, IntoAttributeValue};
use crate::AttributeError;

impl FromAttributeValue for DateTime<Utc> {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        let s = String::try_from_av(av)?;
        DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).map_err(|e| AttributeError::ParseError(Box::new(e)))
    }
}

impl IntoAttributeValue for DateTime<Utc> {
    fn into_av(self) -> AttributeValue {
        self.to_rfc3339().into_av()
    }
}

impl FromAttributeValue for DateTime<Local> {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        let s = String::try_from_av(av)?;
        DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Local)).map_err(|e| AttributeError::ParseError(Box::new(e)))
    }
}

impl IntoAttributeValue for DateTime<Local> {
    fn into_av(self) -> AttributeValue {
        self.to_rfc3339().into_av()
    }
}

impl FromAttributeValue for DateTime<FixedOffset> {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        let s = String::try_from_av(av)?;
        DateTime::parse_from_rfc3339(&s).map_err(|e| AttributeError::ParseError(Box::new(e)))
    }
}

impl IntoAttributeValue for DateTime<FixedOffset> {
    fn into_av(self) -> AttributeValue {
        self.to_rfc3339().into_av()
    }
}

impl FromAttributeValue for SystemTime {
    fn try_from_av(av: AttributeValue) -> Result<Self, AttributeError> {
        let s = String::try_from_av(av)?;
        DateTime::parse_from_rfc3339(&s).map(SystemTime::from).map_err(|e| AttributeError::ParseError(Box::new(e)))
    }
}

impl IntoAttributeValue for SystemTime {
    fn into_av(self) -> AttributeValue {
        DateTime::<Utc>::from(self).into_av()
    }
}
