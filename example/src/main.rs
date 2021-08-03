use nitroglycerin::{dynamodb::DynamoDbClient, DynamoDb, Key, Query, Table};
use nitroglycerin::serde::{Serialize, Deserialize, de::DeserializeOwned};
use rusoto_core::Region;

#[derive(Debug, Serialize, Deserialize, Query)]
pub struct TimeTable {
    #[nitro(partition_key)]
    id: String,

    // #[nitro(rename = "timestamp")]
    #[nitro(sort_key)]
    time: u32,
}

impl Table for TimeTable {
    fn table_name() -> String {
        "Time".to_string()
    }
}

#[tokio::main]
async fn main() {
    // let client = DynamoDbClient::new(Region::default());
    // let time = client.get::<TimeTable<String>>().id("foo").time(5u32).execute().await.unwrap();

    // println!("{:?}", time);
}
