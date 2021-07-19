use crate::{get::Get, put::Put, query::Query};

/// Extension trait providing high level implementations of dynamodb requests
pub trait DynamoDb: rusoto_dynamodb::DynamoDb + Clone {
    /// Perform a get item request
    fn get<T: Get<Self>>(&self) -> T::Builder {
        T::get(self.clone())
    }
    /// Perform a query request
    fn query<T: Query<Self>>(&self) -> T::Builder {
        T::query(self.clone())
    }
    /// Perform a put item request
    fn put<T: Put<Self>>(&self, t: T) -> T::Builder {
        t.put(self.clone())
    }
}

impl<D: rusoto_dynamodb::DynamoDb + Clone> DynamoDb for D {}
