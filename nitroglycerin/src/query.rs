use std::{marker::PhantomData, ops::RangeInclusive};

use rusoto_dynamodb::{AttributeValue, DynamoDb, QueryError, QueryInput};
use serde::{de::DeserializeOwned, Serialize};

use crate::{from_av, to_av, DynamoError, Table, TableIndex};

/// create a [`QueryInput`] using the table and partition key
pub fn new_input<I: TableIndex, K: Serialize>(key_name: &str, key_value: &K) -> QueryInput {
    QueryInput {
        table_name: I::Table::table_name(),
        index_name: I::index_name(),
        key_condition_expression: Some("#0 = :0".to_string()),
        expression_attribute_names: Some(<_>::into_iter([("#0".to_owned(), key_name.to_owned())]).collect()),
        expression_attribute_values: Some(<_>::into_iter([(":0".to_owned(), to_av(key_value).unwrap())]).collect()),
        ..QueryInput::default()
    }
}

/// Trait that declares a type can be built into a query request
pub trait Query<'d, D: 'd + ?Sized>: TableIndex {
    /// The builder type that performs the query request
    type Builder;

    /// Create the query builder
    fn query(client: &'d D) -> Self::Builder;
}

/// Sort key expression builder
pub struct BuilderSort<'d, D: 'd + ?Sized, SortKey, Index> {
    client: &'d D,
    input: QueryInput,
    _phantom: PhantomData<(SortKey, Index)>,
}

impl<'d, D: 'd + ?Sized, S, I> BuilderSort<'d, D, S, I> {
    /// Create a new `BuilderSort`
    pub fn new(client: &'d D, mut input: QueryInput, sort_key: &str) -> Self {
        input.expression_attribute_names.as_mut().map(|n| n.insert("#1".to_owned(), sort_key.to_owned()));
        Self { client, input, _phantom: PhantomData }
    }
}

impl<'d, D: 'd + ?Sized, S, I> BuilderSort<'d, D, S, I>
where
    S: Serialize,
{
    fn push_expr(&mut self, f: &str) {
        if let Some(s) = self.input.key_condition_expression.as_mut() {
            *s = format!("{} {}", *s, f);
        }
    }
    fn push_value(&mut self, key: &str, sort: &S) {
        if let Some(v) = self.input.expression_attribute_values.as_mut() {
            v.insert(key.to_owned(), to_av(sort).unwrap());
        }
    }
    fn build(self) -> Expr<'d, D, I> {
        let Self { client, input, _phantom } = self;
        Expr::new(client, input)
    }

    /// Query for sort key equal
    pub fn equal(mut self, sort: &S) -> Expr<'d, D, I> {
        self.push_expr("AND #1 = :1");
        self.push_value(":1", sort);
        self.build()
    }

    /// Query for sort key less than
    pub fn less_than(mut self, sort: &S) -> Expr<'d, D, I> {
        self.push_expr("AND #1 < :1");
        self.push_value(":1", sort);
        self.build()
    }

    /// Query for sort key less than or equal
    pub fn less_than_or_equal(mut self, sort: &S) -> Expr<'d, D, I> {
        self.push_expr("AND #1 <= :1");
        self.push_value(":1", sort);
        self.build()
    }

    /// Query for sort key greater than
    pub fn greater_than(mut self, sort: &S) -> Expr<'d, D, I> {
        self.push_expr("AND #1 > :1");
        self.push_value(":1", sort);
        self.build()
    }

    /// Query for sort key greater than or equal
    pub fn greater_than_or_equal(mut self, sort: &S) -> Expr<'d, D, I> {
        self.push_expr("AND #1 >= :1");
        self.push_value(":1", sort);
        self.build()
    }

    /// Query for sort key between
    pub fn between(mut self, sort: RangeInclusive<&S>) -> Expr<'d, D, I> {
        let (sort1, sort2) = sort.into_inner();

        self.push_expr("AND #1 BETWEEN :1 AND :2");
        self.push_value(":1", sort1);
        self.push_value(":2", sort2);
        self.build()
    }

    /// Query for sort key beginning with
    pub fn begins_with(mut self, sort: &S) -> Expr<'d, D, I> {
        self.push_expr("AND begins_with(#1, :1)");
        self.push_value(":1", sort);
        self.build()
    }
}

/// Final output of a query builder chain
pub struct Expr<'d, D: 'd + ?Sized, Index> {
    client: &'d D,
    input: QueryInput,
    _phantom: PhantomData<Index>,
}

impl<'d, D: 'd + ?Sized, I> Expr<'d, D, I> {
    /// Create a new `Expr`
    pub const fn new(client: &'d D, input: QueryInput) -> Self {
        Self { client, input, _phantom: PhantomData }
    }

    /// Enable consistent read for the query request
    #[must_use]
    pub const fn consistent_read(mut self) -> Self {
        self.input.consistent_read = Some(true);
        self
    }
}

impl<'d, D: 'd + ?Sized, I> Expr<'d, D, I>
where
    D: DynamoDb,
    &'d D: Send,
    I: DeserializeOwned + Send,
{
    /// Execute the query request
    ///
    /// # Errors
    /// Will error if the dynamodb request fails or the resulting items could not be parsed
    pub async fn execute(self) -> Result<Vec<I>, DynamoError<QueryError>> {
        let output = self.client.query(self.input).await?;
        let items = output.items.unwrap_or_else(Vec::new).into_iter();
        Ok(items
            .map(|item| AttributeValue {
                m: Some(item),
                ..AttributeValue::default()
            })
            .map(from_av)
            .collect::<Result<_, _>>()?)
    }
}
