use nitroglycerin::{dynamodb::DynamoDbClient, DynamoDb, Table};
use rusoto_core::Region;

#[derive(Debug, Table)]
#[nitro(table_name = "Foo")]
pub struct FooTable<ID: Clone> {
    #[nitro(partition_key)]
    id: ID,

    #[nitro(rename = "timestamp")]
    #[nitro(sort_key)]
    time: u32,
}

// #[derive(Debug)]
// pub struct FooTable<ID> {
//     id: ID,
//     time: u32,
// }

#[tokio::main]
async fn main() {
    let client = DynamoDbClient::new(Region::default());
    let foo = client.get::<FooTable<String>>().id("foo".to_owned()).time(5).execute().await.unwrap();

    println!("{:?}", foo);
}
