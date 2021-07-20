use std::{convert::TryFrom};

use rusoto_dynamodb::{GetItemError, GetItemInput};

use crate::{client::DynamoDb, key, AttributeError, Attributes, DynamoError};

impl From<key::Key> for GetItemInput {
    fn from(k: key::Key) -> Self {
        let key::Key { table_name, key } = k;
        GetItemInput {
            table_name,
            key,
            ..GetItemInput::default()
        }
    }
}

impl<D, T> key::Expr<D, GetItemInput, T> {
    /// Enable consistent read for the get item request
    pub const fn consistent_read(mut self) -> Self {
        self.input.consistent_read = Some(true);
        self
    }
}

impl<D, T> key::Expr<D, GetItemInput, T>
where
    D: DynamoDb + Send,
    for<'a> &'a D: Send,
    T: TryFrom<Attributes, Error = AttributeError> + Send,
{
    /// Execute the get item request
    ///
    /// # Errors
    /// Will error if the dynamodb request fails or if the result could not be parsed
    pub async fn execute(self) -> Result<Option<T>, DynamoError<GetItemError>> {
        let Self { client, input, _phantom } = self;
        let output = client.get_item(input).await?;
        Ok(output.item.map(T::try_from).transpose()?)
    }
}
