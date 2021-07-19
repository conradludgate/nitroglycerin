use std::{convert::TryFrom, marker::PhantomData};

use rusoto_dynamodb::{GetItemError, GetItemInput};

use crate::{client::DynamoDb, convert::IntoAttributeValue, AttributeError, Attributes, DynamoError, Table};

/// create a [`GetItemInput`] using the table and partition key
pub fn new_input<T: Table, K: IntoAttributeValue>(key_name: &str, key_value: K) -> GetItemInput {
    GetItemInput {
        table_name: T::table_name(),
        key: <_>::into_iter([(key_name.to_owned(), key_value.into_av())]).collect(),
        ..GetItemInput::default()
    }
}

/// Trait that declares a type can be built into a get item request
pub trait Get<D>: Table {
    /// The builder type that performs the get item request
    type Builder;

    /// Create the get item builder
    fn get(client: D) -> Self::Builder;
}

/// Final output of a get item builder chain
pub struct Expr<D, Table> {
    client: D,
    input: GetItemInput,
    _phantom: PhantomData<Table>,
}

impl<D, T> Expr<D, T> {
    /// Create a new `Expr`
    pub const fn new(client: D, input: GetItemInput) -> Self {
        Self { client, input, _phantom: PhantomData }
    }

    /// Enable consistent read for the get item request
    pub const fn consistent_read(mut self) -> Self {
        self.input.consistent_read = Some(true);
        self
    }
}

impl<D, T> Expr<D, T>
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
