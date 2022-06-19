use nitroglycerin::serde::{Deserialize, Serialize};
use nitroglycerin::{dynamodb::DynamoDbClient, DynamoDb, Key, Table};
use rusoto_core::Region;

#[derive(Debug, Serialize, Deserialize, Key)]
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
    let client = DynamoDbClient::new(Region::default());
    let time = client.get::<TimeTable>().id("foo").unwrap().time(&5u32).unwrap().execute().await.unwrap();

    println!("{:?}", time);
}
