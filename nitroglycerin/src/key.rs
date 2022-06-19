use std::marker::PhantomData;

use serde::Serialize;

use crate::{ser, Attributes, Table};

/// Key type that can be built into `GetItem` requests
pub struct Key {
    pub(crate) table_name: String,
    pub(crate) key: Attributes,
}

impl Key {
    /// create a [`Key`] using the table and partition key
    ///
    /// # Errors
    /// Returns an error if the `key_value` cannot serialise into the attribute values
    pub fn new<T: Table, K: Serialize + ?Sized>(key_name: &str, key_value: &K) -> Result<Self, ser::Error> {
        Ok(Self {
            table_name: T::table_name(),
            key: std::iter::once((key_name.to_owned(), ser::to_av(key_value)?)).collect(),
        })
    }

    /// Insert a new name/value pair into the key
    ///
    /// # Errors
    /// Returns an error if the `key_value` cannot serialise into the attribute values
    pub fn insert(&mut self, key_name: impl Into<String>, key_value: &(impl Serialize + ?Sized)) -> Result<(), ser::Error> {
        self.key.insert(key_name.into(), ser::to_av(key_value)?);
        Ok(())
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
    pub(crate) marker: PhantomData<Table>,
}

impl<'d, D: 'd + ?Sized, Input, Table> Expr<'d, D, Input, Table>
where
    Input: From<Key>,
{
    /// Create a new `Expr`
    pub fn new(client: &'d D, key: Key) -> Self {
        let input = key.into();
        Self { client, input, marker: PhantomData }
    }
}
