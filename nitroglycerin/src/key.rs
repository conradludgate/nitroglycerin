use std::marker::PhantomData;

use serde::Serialize;

use crate::{to_av, Attributes, Table};

/// Key type that can be built into `GetItem` requests
pub struct Key {
    pub(crate) table_name: String,
    pub(crate) key: Attributes,
}

impl Key {
    /// create a [`Key`] using the table and partition key
    pub fn new<T: Table, K: Serialize>(key_name: &str, key_value: &K) -> Self {
        Self {
            table_name: T::table_name(),
            key: <_>::into_iter([(key_name.to_owned(), to_av(key_value).unwrap())]).collect(),
        }
    }

    /// Insert a new name/value pair into the key
    pub fn insert(&mut self, key_name: impl Into<String>, key_value: &impl Serialize) {
        self.key.insert(key_name.into(), to_av(key_value).unwrap());
    }
}

/// Trait that declares a type can be built into a request key
pub trait Builder<'d, D: 'd + ?Sized, R: From<Key>>: Table {
    /// The builder type that performs the get item request
    type Builder;

    /// Create the key builder
    fn key(client: &'d D) -> Self::Builder;
}

/// Final output of a key builder chain
pub struct Expr<'d, D: 'd + ?Sized, Input, Table> {
    pub(crate) client: &'d D,
    pub(crate) input: Input,
    pub(crate) _phantom: PhantomData<Table>,
}

impl<'d, D: 'd + ?Sized, Input, Table> Expr<'d, D, Input, Table>
where
    Input: From<Key>,
{
    /// Create a new `Expr`
    pub fn new(client: &'d D, key: Key) -> Self {
        let input = key.into();
        Self { client, input, _phantom: PhantomData }
    }
}
