use std::{convert::TryFrom, marker::PhantomData};

use rusoto_dynamodb::{GetItemError, GetItemInput};

use crate::{convert::IntoAttributeValue, client::DynamoDb, AttributeError, Attributes, DynamoError};

pub fn new_input<K: IntoAttributeValue>(table_name: String, key_name: &str, key_value: K) -> GetItemInput {
    GetItemInput {
        table_name,
        key: <_>::into_iter([(key_name.to_owned(), key_value.into_av())]).collect(),
        ..GetItemInput::default()
    }
}

pub struct GetExpr<D, Table> {
    client: D,
    input: GetItemInput,
    _phantom: PhantomData<Table>,
}

impl<D, T> GetExpr<D, T> {
    pub fn new(client: D, input: GetItemInput) -> Self {
        Self { client, input, _phantom: PhantomData }
    }

    pub fn consistent_read(mut self) -> Self {
        self.input.consistent_read = Some(true);
        self
    }
}

impl<D: DynamoDb, T> GetExpr<D, T>
where
    T: TryFrom<Attributes, Error = AttributeError>,
{
    pub async fn execute(self) -> Result<Option<T>, DynamoError<GetItemError>> {
        let output = self.client.get_item(self.input).await?;
        Ok(output.item.map(T::try_from).transpose()?)
    }
}
