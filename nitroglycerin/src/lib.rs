mod client;
pub mod get;
mod get_example;
pub mod query;
mod query_example;

use std::{convert::TryFrom, error::Error};

pub use dynomite;
use dynomite::Attributes;
pub use nitroglycerin_derive::Table;
use thiserror::Error;

pub trait Get<D>: TryFrom<Attributes, Error = dynomite::AttributeError> {
    type Builder;
    fn get(client: D) -> Self::Builder;
}

pub trait Query<D>: TryFrom<Attributes, Error = dynomite::AttributeError> {
    type Builder;
    fn query(client: D) -> Self::Builder;
}
#[derive(Debug, Error)]
pub enum DynamoError<E: Error + 'static> {
    #[error("could not parse dynamo attributes: {0}")]
    ParseError(#[from] dynomite::AttributeError),
    #[error("could not connect to dynamo: {0}")]
    Rusoto(#[from] rusoto_core::RusotoError<E>),
}

// pub trait Index: TryFrom<Attributes, Error = dynomite::AttributeError> {
//     type Table: Table;
//     const INDEX_NAME: &'static str;
// }

// pub trait Get: Into<dynomite::Attributes> {
//     type Table: Table;
// }

// pub trait Query: Into<Condition> {
//     type Response: TryFrom<Attributes, Error = dynomite::AttributeError>;

//     fn partition_key() -> String;
//     fn sort_key() -> Option<String> {
//         None
//     }

//     fn table_name() -> String;
//     fn index_name() -> Option<String> {
//         None
//     }

//     fn into_query_input(self) -> QueryInput {
//         let table_name = Self::table_name();
//         let index_name = Self::index_name();
//         let Condition {
//             expr,
//             values,
//             names,
//         } = self.into();

//         QueryInput {
//             table_name,
//             index_name,
//             key_condition_expression: Some(expr),
//             expression_attribute_names: Some(
//                 names
//                     .into_iter()
//                     .enumerate()
//                     .map(|(i, name)| (format!("#{}", i), name))
//                     .collect(),
//             ),
//             expression_attribute_values: Some(
//                 values
//                     .into_iter()
//                     .enumerate()
//                     .map(|(i, value)| (format!(":{}", i), value))
//                     .collect(),
//             ),
//             ..QueryInput::default()
//         }
//     }
// }

// // pub struct Condition {
// //     names: Vec<String>,
// //     values: Vec<AttributeValue>,
// //     expr: String,
// // }

// pub struct QueryCondition<P, S> {
//     pub partition_key: String,
//     pub partition_value: P,

//     pub sort_key: Option<String>,
//     pub sort_condition: Option<SortCondition<S>>,
// }

// pub enum SortCondition<T> {
//     Equal(T),
//     LessThan(T),
//     LessThanOrEqual(T),
//     GreaterThan(T),
//     GreaterThanOrEqual(T),
//     Between(T, T),
//     BeginsWith(T),
// }

// // impl QueryExpr {
// //     fn enrich(self, names: &mut Vec<String>, values: &mut Vec<AttributeValue>) -> String {
// //         match self {
// //             QueryExpr::Equal(name, value) => {
// //                 let i = names.len();
// //                 let j = values.len();

// //                 names.push(name);
// //                 values.push(value);

// //                 format!("#{} = :{}", i, j)
// //             }
// //             QueryExpr::And(lhs, rhs) => {
// //                 let lhs = lhs.enrich(names, values);
// //                 let rhs = rhs.enrich(names, values);

// //                 format!("({}) AND ({})", lhs, rhs)
// //             }
// //         }
// //     }
// // }
