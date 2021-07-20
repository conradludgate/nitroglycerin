use nitroglycerin::{dynamodb::DynamoDbClient, Attributes, DynamoDb, Key, Query, Table};
use rusoto_core::Region;

#[derive(Debug, Attributes, Key, Query)]
pub struct TimeTable<ID: Clone> {
    #[nitro(partition_key)]
    id: ID,

    #[nitro(rename = "timestamp")]
    #[nitro(sort_key)]
    time: u32,
}

impl<ID: Clone> Table for TimeTable<ID> {
    fn table_name() -> String {
        "Time".to_string()
    }
}

#[tokio::main]
async fn main() {
    let client = DynamoDbClient::new(Region::default());
    let time = client.get::<TimeTable<String>>().id("foo").time(5u32).execute().await.unwrap();

    println!("{:?}", time);
}
