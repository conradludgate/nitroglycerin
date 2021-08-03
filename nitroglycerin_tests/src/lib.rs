#![cfg(test)]

#[macro_use]
extern crate async_trait;

use mockall::{predicate::*};
use nitroglycerin::{Attributes, DynamoDb, Key, Query, Table};
use rusoto_dynamodb::*;

mod mock;
use mock::MockDynamoDbClient;

#[derive(Debug, PartialEq, Key, Query)]
struct ExampleTable1 {
    #[nitro(partition_key, rename = "id")]
    pub partition: String,

    #[nitro(sort_key, rename = "range")]
    pub sort: i32,

    pub extra_values: Vec<String>,
}

impl Table for ExampleTable1 {
    fn table_name() -> String {
        "ExampleTable1Name".into()
    }
}

#[derive(Debug, PartialEq, Key, Query)]
struct ExampleTable2 {
    #[nitro(partition_key, rename = "id")]
    pub partition: String,

    pub extra_values: Vec<String>,
}

impl Table for ExampleTable2 {
    fn table_name() -> String {
        "ExampleTable2Name".into()
    }
}

macro_rules! av {
    ($t:ident: $v:expr) => {
        AttributeValue {
            $t: Some($v.into()),
            ..Default::default()
        }
    };
}

macro_rules! m {
    ($( $k:expr => $v:expr, )*) => {
        <_>::into_iter([
            $(
                ($k.to_string(), $v.into()),
            )*
        ]).collect()
    };
}

#[tokio::test]
async fn test_get() {
    let mut client = MockDynamoDbClient::new();
    client
        .expect_get_item()
        .with(eq(GetItemInput {
            key: m!(
                "id" => av!(s: "foo"),
                "range" => av!(n: "42"),
            ),
            table_name: "ExampleTable1Name".into(),
            ..Default::default()
        }))
        .returning(|_| {
            Ok(GetItemOutput {
                item: Some(m!(
                    "id" => av!(s: "foo"),
                    "range" => av!(n: "42"),
                    "extra_values" => av!(
                        l: vec![
                            av!(s: "foo"),
                            av!(s: "bar"),
                        ]
                    ),
                )),
                ..Default::default()
            })
        });

    let output = client.get::<ExampleTable1>().partition("foo").sort(42i32).execute().await.unwrap();
    assert_eq!(
        output,
        Some(ExampleTable1 {
            partition: "foo".into(),
            sort: 42,
            extra_values: vec!["foo".into(), "bar".into()],
        })
    );
}

#[tokio::test]
async fn test_get_no_sort() {
    let mut client = MockDynamoDbClient::new();
    client
        .expect_get_item()
        .with(eq(GetItemInput {
            key: m!(
                "id" => av!(s: "foo"),
            ),
            table_name: "ExampleTable2Name".into(),
            ..Default::default()
        }))
        .returning(|_| {
            Ok(GetItemOutput {
                item: Some(m!(
                    "id" => av!(s: "foo"),
                    "extra_values" => av!(
                        l: vec![
                            av!(s: "foo"),
                            av!(s: "bar"),
                        ]
                    ),
                )),
                ..Default::default()
            })
        });

    let output = client.get::<ExampleTable2>().partition("foo").execute().await.unwrap();
    assert_eq!(
        output,
        Some(ExampleTable2 {
            partition: "foo".into(),
            extra_values: vec!["foo".into(), "bar".into()],
        })
    );
}

#[tokio::test]
async fn test_query() {
    let mut client = MockDynamoDbClient::new();
    client
        .expect_query()
        .with(eq(QueryInput {
            key_condition_expression: Some("#0 = :0 AND #1 > :1".into()),
            expression_attribute_names: Some(m! {
                "#0" => "id",
                "#1" => "range",
            }),
            expression_attribute_values: Some(m! {
                ":0" => av!(s: "foo"),
                ":1" => av!(n: "42"),
            }),
            table_name: "ExampleTable1Name".into(),
            ..Default::default()
        }))
        .returning(|_| {
            Ok(QueryOutput {
                items: Some(vec![
                    m!(
                        "id" => av!(s: "foo"),
                        "range" => av!(n: "43"),
                        "extra_values" => av!(
                            l: vec![
                                av!(s: "foo"),
                                av!(s: "bar"),
                            ]
                        ),
                    ),
                    m!(
                        "id" => av!(s: "foo"),
                        "range" => av!(n: "44"),
                        "extra_values" => av!(
                            l: vec![
                                av!(s: "baz"),
                            ]
                        ),
                    ),
                ]),
                ..Default::default()
            })
        });

    let output = client.query::<ExampleTable1>().partition("foo").sort().greater_than(42i32).execute().await.unwrap();
    assert_eq!(output, vec![
        ExampleTable1 {
            partition: "foo".into(),
            sort: 43,
            extra_values: vec!["foo".into(), "bar".into()],
        },
        ExampleTable1 {
            partition: "foo".into(),
            sort: 44,
            extra_values: vec!["baz".into()],
        },
    ]);
}

#[tokio::test]
async fn test_query_optional_sort() {
    let mut client = MockDynamoDbClient::new();
    client
        .expect_query()
        .with(eq(QueryInput {
            key_condition_expression: Some("#0 = :0".into()),
            expression_attribute_names: Some(m! {
                "#0" => "id",
            }),
            expression_attribute_values: Some(m! {
                ":0" => av!(s: "foo"),
            }),
            table_name: "ExampleTable1Name".into(),
            ..Default::default()
        }))
        .returning(|_| {
            Ok(QueryOutput {
                items: Some(vec![
                    m!(
                        "id" => av!(s: "foo"),
                        "range" => av!(n: "42"),
                        "extra_values" => av!(
                            l: vec![
                                av!(s: "foo"),
                                av!(s: "bar"),
                            ]
                        ),
                    ),
                    m!(
                        "id" => av!(s: "foo"),
                        "range" => av!(n: "43"),
                        "extra_values" => av!(
                            l: vec![
                                av!(s: "foo"),
                                av!(s: "bar"),
                            ]
                        ),
                    ),
                    m!(
                        "id" => av!(s: "foo"),
                        "range" => av!(n: "44"),
                        "extra_values" => av!(
                            l: vec![
                                av!(s: "baz"),
                            ]
                        ),
                    ),
                ]),
                ..Default::default()
            })
        });

    let output = client.query::<ExampleTable1>().partition("foo").execute().await.unwrap();
    assert_eq!(output, vec![
        ExampleTable1 {
            partition: "foo".into(),
            sort: 42,
            extra_values: vec!["foo".into(), "bar".into()],
        },
        ExampleTable1 {
            partition: "foo".into(),
            sort: 43,
            extra_values: vec!["foo".into(), "bar".into()],
        },
        ExampleTable1 {
            partition: "foo".into(),
            sort: 44,
            extra_values: vec!["baz".into()],
        },
    ]);
}

#[tokio::test]
async fn test_query_no_sort() {
    let mut client = MockDynamoDbClient::new();
    client
        .expect_query()
        .with(eq(QueryInput {
            key_condition_expression: Some("#0 = :0".into()),
            expression_attribute_names: Some(m! {
                "#0" => "id",
            }),
            expression_attribute_values: Some(m! {
                ":0" => av!(s: "foo"),
            }),
            table_name: "ExampleTable2Name".into(),
            ..Default::default()
        }))
        .returning(|_| {
            Ok(QueryOutput {
                items: Some(vec![
                    m!(
                        "id" => av!(s: "foo"),
                        "extra_values" => av!(
                            l: vec![
                                av!(s: "foo"),
                                av!(s: "bar"),
                            ]
                        ),
                    ),
                ]),
                ..Default::default()
            })
        });

    let output = client.query::<ExampleTable2>().partition("foo").execute().await.unwrap();
    assert_eq!(output, vec![
        ExampleTable2 {
            partition: "foo".into(),
            extra_values: vec!["foo".into(), "bar".into()],
        },
    ]);
}

