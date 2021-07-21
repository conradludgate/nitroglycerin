use crate::{get::Get, put::Put, query::Query};

/// Extension trait providing high level implementations of dynamodb requests
pub trait DynamoDb: rusoto_dynamodb::DynamoDb {
    /// Perform a get item request
    fn get<'d, T: Get<'d, Self>>(&'d self) -> T::Builder {
        T::get(self)
    }
    /// Perform a query request
    fn query<'d, T: Query<'d, Self>>(&'d self) -> T::Builder {
        T::query(self)
    }
    /// Perform a put item request
    fn put<'d, T: Put<'d, Self>>(&'d self, t: T) -> T::Builder {
        t.put(self)
    }
}

impl<D: rusoto_dynamodb::DynamoDb> DynamoDb for D {}
