use rusoto_dynamodb::{PutItemError, PutItemInput};

use crate::{Attributes, DynamoDb, DynamoError, Table};

/// Trait that declares a type can be built into a put item request
pub trait Put<D>: Table {
    /// The builder type that performs the put item request
    type Builder;
    /// Create the put item builder
    fn put(self, client: D) -> Self::Builder;
}

impl<D, T: Table + Into<Attributes>> Put<D> for T {
    type Builder = Expr<D>;
    fn put(self, client: D) -> Self::Builder {
        let input = PutItemInput {
            table_name: T::table_name(),
            item: self.into(),
            ..PutItemInput::default()
        };

        Expr::new(client, input)
    }
}

/// Final output of a put item builder chain
pub struct Expr<D> {
    client: D,
    input: PutItemInput,
}

impl<D> Expr<D> {
    /// Create a new `Expr`
    pub const fn new(client: D, input: PutItemInput) -> Self {
        Self { client, input }
    }
}

impl<D> Expr<D>
where
    D: DynamoDb + Send,
    for<'a> &'a D: Send,
{
    /// Execute the put item request
    ///
    /// # Errors
    /// Will error if the dynamodb request fails
    pub async fn execute(self) -> Result<(), DynamoError<PutItemError>> {
        let Self { client, input } = self;
        client.put_item(input).await?;
        Ok(())
    }
}
