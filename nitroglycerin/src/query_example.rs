use std::convert::TryFrom;

use rusoto_dynamodb::{QueryError, QueryInput};

use crate::{AttributeError, Attributes, DynamoError, Query, client::DynamoDb, convert::{FromAttributeValue, extract}, query::{self, QueryBuilderSort, QueryExpr}};

pub struct FooIndex {
    id: String, // partition key
    time: i64,  // sort key
}

impl TryFrom<Attributes> for FooIndex {
    type Error = AttributeError;
    fn try_from(mut value: Attributes) -> Result<Self, Self::Error> {
        Ok(FooIndex {
            id: extract(&mut value, "id")?,
            time: extract(&mut value, "time")?,
        })
    }
}

impl<D> Query<D> for FooIndex {
    type Builder = FooIndexQueryBuilder<D>;
    fn query(client: D) -> Self::Builder {
        Self::Builder { client }
    }
}

pub struct FooIndexQueryBuilder<D> {
    client: D,
}

impl<D> FooIndexQueryBuilder<D> {
    pub fn id(self, id: String) -> FooIndexQueryBuilderPrimary<D> {
        let Self { client } = self;

        let mut input = query::new_input("Foo".into(), "id", id);
        input.index_name = Some("FooIndex".to_owned());

        FooIndexQueryBuilderPrimary { client, input }
    }
}

pub struct FooIndexQueryBuilderPrimary<D> {
    client: D,
    input: QueryInput,
}

impl<D: DynamoDb> FooIndexQueryBuilderPrimary<D> {
    pub fn consistent_read(self) -> QueryExpr<D, FooIndex> {
        let Self { client, input } = self;
        QueryExpr::new(client, input).consistent_read()
    }

    pub async fn execute(self) -> Result<Vec<FooIndex>, DynamoError<QueryError>> {
        let Self { client, input } = self;
        QueryExpr::new(client, input).execute().await
    }

    pub fn time(self) -> QueryBuilderSort<D, i32, FooIndex> {
        let Self { client, input } = self;
        QueryBuilderSort::new(client, input, "time")
    }
}
