use std::convert::TryFrom;

use dynomite::{
    dynamodb::{DynamoDb, QueryError, QueryInput},
    Attribute, Attributes,
};

use crate::{
    DynamoError,
    query::{self, QueryBuilderSort, QueryExpr},
    Query,
};

pub struct FooIndex {
    id: String, // partition key
    time: i64,  // sort key
}

impl TryFrom<Attributes> for FooIndex {
    type Error = dynomite::AttributeError;
    fn try_from(mut value: Attributes) -> Result<Self, Self::Error> {
        Ok(FooIndex {
            id: String::from_attr(value.remove("id").ok_or_else(|| dynomite::AttributeError::MissingField { name: "id".to_owned() })?)?,
            time: i64::from_attr(value.remove("time").ok_or_else(|| dynomite::AttributeError::MissingField { name: "time".to_owned() })?)?,
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

        let mut input = query::new_input("Foo", "id", id);
        input.index_name = Some("FooIndex".to_owned());

        FooIndexQueryBuilderPrimary { client, input }
    }
}

pub struct FooIndexQueryBuilderPrimary<D> {
    client: D,
    input: QueryInput,
}

impl<D: DynamoDb> FooIndexQueryBuilderPrimary<D> {
    pub fn consistent_read(mut self) -> QueryExpr<D, FooIndex> {
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
