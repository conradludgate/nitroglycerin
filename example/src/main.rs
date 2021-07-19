use nitroglycerin::{dynamodb::DynamoDbClient, Attributes, DynamoDb, Get, Query, Table};
use rusoto_core::Region;

#[derive(Debug, Get, Query, Attributes)]
pub struct FooTable<ID: Clone> {
    #[nitro(partition_key)]
    id: ID,

    #[nitro(rename = "timestamp")]
    #[nitro(sort_key)]
    time: u32,
}

impl<ID: Clone> Table for FooTable<ID> {
    fn table_name() -> String {
        "Foo".to_string()
    }
}

#[tokio::main]
async fn main() {
    let client = DynamoDbClient::new(Region::default());
    let foo = client.get::<FooTable<String>>().id("foo".to_owned()).time(5).execute().await.unwrap();

    println!("{:?}", foo);
}
