use rusoto_dynamodb::{AttributeValue, GetItemError, GetItemInput};
use serde::{de::DeserializeOwned};

use crate::{DynamoError, Table, client::DynamoDb, from_av, key};

/// Trait that declares a type can be built into a get item request
pub trait Get<'d, D: ?Sized>: Table {
    /// The builder type that performs the get item request
    type Builder;

    /// Create the get builder
    fn get(client: &'d D) -> Self::Builder;
}

impl<'d, D: 'd + ?Sized, K: key::Builder<'d, D, GetItemInput>> Get<'d, D> for K {
    type Builder = K::Builder;
    fn get(client: &'d D) -> Self::Builder {
        K::key(client)
    }
}

impl From<key::Key> for GetItemInput {
    fn from(k: key::Key) -> Self {
        let key::Key { table_name, key } = k;
        Self { key, table_name, ..Self::default() }
    }
}

impl<'d, D: 'd + ?Sized, T> key::Expr<'d, D, GetItemInput, T> {
    /// Enable consistent read for the get item request
    #[must_use]
    pub const fn consistent_read(mut self) -> Self {
        self.input.consistent_read = Some(true);
        self
    }
}

impl<'d, D: 'd + ?Sized, T> key::Expr<'d, D, GetItemInput, T>
where
    D: DynamoDb,
    &'d D: Send,
    T: DeserializeOwned + Send,
{
    /// Execute the get item request
    ///
    /// # Errors
    /// Will error if the dynamodb request fails or if the result could not be parsed
    pub async fn execute(self) -> Result<Option<T>, DynamoError<GetItemError>> {
        let Self { client, input, _phantom } = self;
        let output = client.get_item(input).await?;
        Ok(output.item.map(|item| AttributeValue{
            m: Some(item),
            ..AttributeValue::default()
        }).map(from_av).transpose()?)
    }
}
