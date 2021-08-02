use rusoto_dynamodb::{DeleteItemError, DeleteItemInput};

use crate::{key, DynamoDb, DynamoError, Table};

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
        let Self { client, input, _phantom } = self;
        client.delete_item(input).await?;
        Ok(())
    }
}
