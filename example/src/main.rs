use nitroglycerin::{Table, dynamodb::DynamoDbClient, DynamoDb};
use rusoto_core::Region;
#[derive(Debug, Table)]
#[nitro(table_name = "Foo")]
pub struct FooTable {
    #[nitro(partition_key)]
    id: String,
    #[nitro(rename = "sortKey")]
    // #[nitro(sort_key)]
    sort: u32,
}

#[tokio::main]
async fn main() {
    let client = DynamoDbClient::new(Region::default());
    let foo = client.get::<FooTable>().id("foo".to_owned()).sort(5).execute().await.unwrap();

    println!("{:?}", foo);
}
