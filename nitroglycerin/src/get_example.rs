use std::convert::TryFrom;

use dynomite::{dynamodb::GetItemInput, Attribute, Attributes};

use crate::{
    get::{self, GetExpr},
    Get,
};

struct Foo {
    id: String, // partition key
    time: i64,  // sort key
}

impl TryFrom<Attributes> for Foo {
    type Error = dynomite::AttributeError;
    fn try_from(mut value: Attributes) -> Result<Self, Self::Error> {
        Ok(Foo {
            id: String::from_attr(value.remove("id").ok_or_else(|| dynomite::AttributeError::MissingField { name: "id".to_owned() })?)?,
            time: i64::from_attr(value.remove("time").ok_or_else(|| dynomite::AttributeError::MissingField { name: "time".to_owned() })?)?,
        })
    }
}

impl<D> Get<D> for Foo {
    type Builder = FooGetBuilder<D>;
    fn get(client: D) -> Self::Builder {
        FooGetBuilder { client }
    }
}

struct FooGetBuilder<D> {
    client: D,
}

impl<D> FooGetBuilder<D> {
    fn id(self, id: String) -> FooGetBuilderPartition<D> {
        let Self { client } = self;

        let input = get::new_input("Foo", "id", id);

        FooGetBuilderPartition::new(client, input)
    }
}

struct FooGetBuilderPartition<D> {
    client: D,
    input: GetItemInput,
}

impl<D> FooGetBuilderPartition<D> {
    fn new(client: D, input: GetItemInput) -> Self {
        Self { client, input }
    }

    fn time(self, time: i64) -> GetExpr<D, Foo> {
        let sort = time;
        let Self { client, mut input } = self;

        input.key.insert("time".to_owned(), sort.into_attr());

        GetExpr::new(client, input)
    }
}
