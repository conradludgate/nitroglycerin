mod client;
pub mod convert;
pub mod get;
mod get_example;
pub mod query;
mod query_example;

use std::{collections::HashMap, convert::TryFrom, error::Error};

pub use client::DynamoDb;
pub use nitroglycerin_derive::Table;
pub use rusoto_dynamodb as dynamodb;
use thiserror::Error;

pub trait Get<D>: TryFrom<Attributes, Error = AttributeError> {
    type Builder;
    fn get(client: D) -> Self::Builder;
}

pub trait Query<D>: TryFrom<Attributes, Error = AttributeError> {
    type Builder;
    fn query(client: D) -> Self::Builder;
}
#[derive(Debug, Error)]
pub enum DynamoError<E: Error + 'static> {
    #[error("could not parse dynamo attributes: {0}")]
    ParseError(#[from] AttributeError),
    #[error("could not connect to dynamo: {0}")]
    Rusoto(#[from] rusoto_core::RusotoError<E>),
}

pub type Attributes = HashMap<String, rusoto_dynamodb::AttributeValue>;

#[derive(Debug, Error)]
pub enum AttributeError {
    #[error("missing field {0}")]
    MissingField(String),

    #[error("incorrect type")]
    IncorrectType,

    #[error("could not parse value: {0}")]
    ParseError(#[from] Box<dyn Error>),
}
