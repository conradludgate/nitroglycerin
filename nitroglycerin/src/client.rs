use crate::{Get, Query};

pub trait DynamoDb: dynomite::dynamodb::DynamoDb + Clone {
    fn get<T: Get<Self>>(&self) -> T::Builder {
        T::get(self.clone())
    }
    fn query<T: Query<Self>>(&self) -> T::Builder {
        T::query(self.clone())
    }
}

impl<D: dynomite::dynamodb::DynamoDb + Clone> DynamoDb for D {}
