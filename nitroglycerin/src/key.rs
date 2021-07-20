use std::marker::PhantomData;

use crate::{convert::IntoAttributeValue, Attributes, Table};

/// Key type that can be built into `GetItem` requests
pub struct Key {
    pub(crate) table_name: String,
    pub(crate) key: Attributes,
}

impl Key {
    /// create a [`Key`] using the table and partition key
    pub fn new<T: Table, K: IntoAttributeValue>(key_name: &str, key_value: K) -> Self {
        Self {
            table_name: T::table_name(),
            key: <_>::into_iter([(key_name.to_owned(), key_value.into_av())]).collect(),
        }
    }

    /// Insert a new name/value pair into the key
    pub fn insert(&mut self, key_name: impl Into<String>, key_value: impl IntoAttributeValue) {
        self.key.insert(key_name.into(), key_value.into_av());
    }
}

/// Trait that declares a type can be built into a request key
pub trait Builder<D, R: From<Key>>: Table {
    /// The builder type that performs the get item request
    type Builder;

    /// Create the key builder
    fn key(client: D) -> Self::Builder;
}

/// Final output of a key builder chain
pub struct Expr<D, Input, Table> {
    pub(crate) client: D,
    pub(crate) input: Input,
    pub(crate) _phantom: PhantomData<Table>,
}

impl<D, Input, Table> Expr<D, Input, Table>
where
    Input: From<Key>,
{
    /// Create a new `Expr`
    pub fn new(client: D, key: Key) -> Self {
        let input = key.into();
        Self { client, input, _phantom: PhantomData }
    }
}
