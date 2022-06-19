//! High level dynamodb crate
//!
//! ```ignore
//! use nitroglycerin::{Attributes, Key, Query, Table, DynamoDb, dynamodb::DynamoDbClient};
//! use rusoto_core::Region;
//!
//! #[derive(Debug, PartialEq, Attributes, Key, Query)]
//! struct Employee {
//!     #[nitro(partition_key)]
//!     id: String,
//!     #[nitro(rename = "firstName")]
//!     name: String,
//!     joined: i64,
//!     left: Option<i64>,
//! }
//!
//! impl Table for Employee {
//!     fn table_name() -> String {
//!         "Employees".to_string()
//!     }
//! }
//!
//! #[derive(Debug, PartialEq, Attributes, Query)]
//! struct EmployeeNameIndex {
//!     #[nitro(partition_key, rename = "firstName")]
//!     name: String,
//!     #[nitro(sort_key)]
//!     joined: i64,
//! }
//!
//! impl IndexTable for EmployeeNameIndex {
//!     type Table = Employees;
//!     fn index_name() -> Option<String> {
//!         Some("EmployeeNamesIndex".to_string())
//!     }
//! }
//!
//! let client = DynamoDbClient::new(Region::default());
//!
//! let employee: Option<Employee> = client.get::<Employee>()
//!    .id("emp_1") // get the employee with id "emp_1"
//!    .execute().await?;
//!
//! let new_employee = Employee {
//!    id: "emp_1234".into(),
//!    name: "Conrad".into(),
//!    joined: 1626900000,
//!    left: None,
//! };
//! // Put the new employee item into the db
//! client.put(new_employee).execute().await?;
//!
//! let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
//!    .name("John") // query the db for all employees named "John"
//!    .execute().await?;
//!
//! let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
//!    .name("John") // query the db for all employees named "John"
//!    .joined().between(1626649200, 1626735600) // and who joined between 2021-07-19 and 2021-07-20
//!    .execute().await?;
//! ```

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_const_for_fn)]
// #![warn(missing_docs)]

mod client;

/// module covering conversions to and from dynamodb attribute values
// pub mod convert;
/// collection of functions and types used to make get item requests
mod get;
/// collection of functions and types used to make key requests
pub mod key;
/// collection of functions and types used to make put item requests
pub mod put;
/// collection of functions and types used to make delete item requests
pub mod delete;
/// collection of functions and types used to make query requests
pub mod query;

pub mod ser;
pub mod de;

// pub use ser::{to_av, to_av_map, SerError};
// pub use de::{from_av, Error};
pub use serde;

use std::{collections::HashMap, error::Error};

pub use client::DynamoDb;
pub use nitroglycerin_derive::{Key, Query};
pub use rusoto_dynamodb as dynamodb;
use thiserror::Error;

/// Trait indicating that a type is a dynamodb table
pub trait Table {
    /// get the table name
    fn table_name() -> String;
}
/// Trait indicating that a type is a dynamodb index
pub trait TableIndex {
    /// The dynamodb table this index belongs to
    type Table: Table;
    /// get the index name
    fn index_name() -> Option<String>;
}
impl<T: Table> TableIndex for T {
    type Table = Self;
    fn index_name() -> Option<String> {
        None
    }
}

/// Error returned by dynamodb requests
#[derive(Debug, Error)]
pub enum DynamoError<E: Error + 'static> {
    /// Error originated from an attribute value parse error
    #[error("could not deserialize dynamo attributes: {0}")]
    DeError(#[from] de::Error),
    /// Error originated from an attribute value convert error
    #[error("could not serialize dynamo attributes: {0}")]
    SerError(#[from] ser::Error),
    /// Error originated from an attribute value parse error
    #[error("{0}")]
    AttributeError(#[from] AttributeError),
    /// Error originated from a dynamodb request error
    #[error("could not connect to dynamo: {0}")]
    Rusoto(#[from] rusoto_core::RusotoError<E>),
}

/// Convenient type for a attribute value map
pub type Attributes = HashMap<String, rusoto_dynamodb::AttributeValue>;

/// Error returned when parsing attribute values
#[derive(Debug, Error)]
pub enum AttributeError {
    /// Error occurs when no item is returned by dynamodb
    #[error("no item returned by dynamodb")]
    MissingAttributes,
}
