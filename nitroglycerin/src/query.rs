use std::{convert::TryFrom, marker::PhantomData, ops::RangeInclusive};

use rusoto_dynamodb::{DynamoDb, QueryError, QueryInput};

use crate::{convert::IntoAttributeValue, AttributeError, Attributes, DynamoError};

pub fn new_input<K: IntoAttributeValue>(table_name: String, key_name: &str, key_value: K) -> QueryInput {
    QueryInput {
        table_name,
        key_condition_expression: Some("#0 = :0".to_string()),
        expression_attribute_names: Some(<_>::into_iter([("#0".to_owned(), key_name.to_owned())]).collect()),
        expression_attribute_values: Some(<_>::into_iter([(":0".to_owned(), key_value.into_av())]).collect()),
        ..QueryInput::default()
    }
}

pub struct QueryBuilderSort<D, SortKey, Index> {
    client: D,
    input: QueryInput,
    _phantom: PhantomData<(SortKey, Index)>,
}

impl<D, S, I> QueryBuilderSort<D, S, I> {
    pub fn new(client: D, mut input: QueryInput, sort_key: &str) -> Self {
        input.expression_attribute_names.as_mut().map(|n| n.insert("#1".to_owned(), sort_key.to_owned()));
        Self { client, input, _phantom: PhantomData }
    }
}

impl<D, S, I> QueryBuilderSort<D, S, I>
where
    S: IntoAttributeValue,
{
    fn push_expr(&mut self, f: &str) {
        self.input.key_condition_expression.as_mut().map(|s| *s = format!("{} {}", *s, f));
    }
    fn push_value(&mut self, key: &str, sort: S) {
        self.input.expression_attribute_values.as_mut().map(|v| v.insert(key.to_owned(), sort.into_av()));
    }
    fn build(self) -> QueryExpr<D, I> {
        let Self { client, input, _phantom } = self;
        QueryExpr::new(client, input)
    }

    pub fn equal(mut self, sort: S) -> QueryExpr<D, I> {
        self.push_expr("AND #1 = :1");
        self.push_value(":1", sort);
        self.build()
    }

    pub fn less_than(mut self, sort: S) -> QueryExpr<D, I> {
        self.push_expr("AND #1 < :1");
        self.push_value(":1", sort);
        self.build()
    }

    pub fn less_than_or_equal(mut self, sort: S) -> QueryExpr<D, I> {
        self.push_expr("AND #1 <= :1");
        self.push_value(":1", sort);
        self.build()
    }

    pub fn greater_than(mut self, sort: S) -> QueryExpr<D, I> {
        self.push_expr("AND #1 > :1");
        self.push_value(":1", sort);
        self.build()
    }

    pub fn greater_than_or_equal(mut self, sort: S) -> QueryExpr<D, I> {
        self.push_expr("AND #1 >= :1");
        self.push_value(":1", sort);
        self.build()
    }

    pub fn between(mut self, sort: RangeInclusive<S>) -> QueryExpr<D, I> {
        let (sort1, sort2) = sort.into_inner();

        self.push_expr("AND #1 BETWEEN :1 AND :2");
        self.push_value(":1", sort1);
        self.push_value(":2", sort2);
        self.build()
    }

    pub fn begins_with(mut self, sort: S) -> QueryExpr<D, I> {
        self.push_expr("AND begins_with(#1, :1)");
        self.push_value(":1", sort);
        self.build()
    }
}

pub struct QueryExpr<D, Index> {
    client: D,
    input: QueryInput,
    _phantom: PhantomData<Index>,
}

impl<D, I> QueryExpr<D, I> {
    pub fn new(client: D, input: QueryInput) -> Self {
        Self { client, input, _phantom: PhantomData }
    }

    pub fn consistent_read(mut self) -> Self {
        self.input.consistent_read = Some(true);
        self
    }
}

impl<D: DynamoDb, I> QueryExpr<D, I>
where
    I: TryFrom<Attributes, Error = AttributeError>,
{
    pub async fn execute(self) -> Result<Vec<I>, DynamoError<QueryError>> {
        let output = self.client.query(self.input).await?;
        let items = output.items.unwrap_or_else(Vec::new).into_iter();
        Ok(items.map(I::try_from).collect::<Result<_, _>>()?)
    }
}
