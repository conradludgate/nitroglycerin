use rusoto_dynamodb::{AttributeValue, DeleteItemError, DeleteItemInput};
use serde::de::DeserializeOwned;

use crate::{AttributeError, DynamoDb, DynamoError, Table, de::from_av, key};

/// Trait that declares a type can be built into a delete item request
pub trait Delete<'d, D: 'd + ?Sized>: Table {
    /// The builder type that performs the delete item request
    type Builder;
    /// Create the delete item builder
    fn delete(client: &'d D) -> Self::Builder;
}

impl<'d, D: 'd + ?Sized, K: key::Builder<'d, D, DeleteItemInput>> Delete<'d, D> for K {
    type Builder = K::Builder;
    fn delete(client: &'d D) -> Self::Builder {
        K::key(client)
    }
}

impl From<key::Key> for DeleteItemInput {
    fn from(k: key::Key) -> Self {
        let key::Key { table_name, key } = k;
        Self { key, table_name, ..Self::default() }
    }
}

impl<'d, D: 'd + ?Sized, T> key::Expr<'d, D, DeleteItemInput, T>
where
    D: DynamoDb,
    &'d D: Send,
    T: Send,
{
    /// Execute the delete item request
    ///
    /// # Errors
    /// Will error if the dynamodb request fails
    pub async fn execute(self) -> Result<(), DynamoError<DeleteItemError>> {
        let Self { client, input, marker: _phantom } = self;
        client.delete_item(input).await?;
        Ok(())
    }
}

impl<'d, D: 'd + ?Sized, T> key::Expr<'d, D, DeleteItemInput, T> {
    /// Execute the delete item request
    ///
    /// # Errors
    /// Will error if the dynamodb request fails
    #[must_use]
    pub fn return_all_old(self) -> key::Expr<'d, D, ReturnAllOld, T> {
        let Self { client, mut input, marker } = self;
        input.return_values = Some("ALL_OLD".to_string());
        key::Expr {
            client,
            input: ReturnAllOld { input },
            marker,
        }
    }
}

/// Input which indicates that the delete request will return
/// all the old values
pub struct ReturnAllOld {
    input: DeleteItemInput,
}

impl<'d, D: 'd + ?Sized, T> key::Expr<'d, D, ReturnAllOld, T>
where
    D: DynamoDb,
    &'d D: Send,
    T: DeserializeOwned + Send,
{
    /// Execute the delete item request returning the contents of the deleted item
    ///
    /// # Errors
    /// Will error if the dynamodb request fails
    pub async fn execute(self) -> Result<T, DynamoError<DeleteItemError>> {
        let Self { client, input, marker: _phantom } = self;
        let output = client.delete_item(input.input).await?;
        let item = output.attributes.ok_or(AttributeError::MissingAttributes)?;
        Ok(from_av(AttributeValue{
            m: Some(item),
            ..AttributeValue::default()
        })?)
    }
}
