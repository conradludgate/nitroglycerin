use rusoto_dynamodb::{PutItemError, PutItemInput};

use crate::{Attributes, DynamoDb, DynamoError, Table};

/// Trait that declares a type can be built into a put item request
pub trait Put<'d, D: 'd + ?Sized>: Table {
    /// The builder type that performs the put item request
    type Builder;
    /// Create the put item builder
    fn put(self, client: &'d D) -> Self::Builder;
}

impl<'d, D: 'd + ?Sized, T: Table + Into<Attributes>> Put<'d, D> for T {
    type Builder = Expr<'d, D>;
    fn put(self, client: &'d D) -> Self::Builder {
        let input = PutItemInput {
            table_name: T::table_name(),
            item: self.into(),
            ..PutItemInput::default()
        };

        Expr::new(client, input)
    }
}

/// Final output of a put item builder chain
pub struct Expr<'d, D: 'd + ?Sized> {
    client: &'d D,
    input: PutItemInput,
}

impl<'d, D: 'd + ?Sized> Expr<'d, D> {
    /// Create a new `Expr`
    pub const fn new(client: &'d D, input: PutItemInput) -> Self {
        Self { client, input }
    }
}

impl<'d, D: 'd + ?Sized> Expr<'d, D>
where
    D: DynamoDb,
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
