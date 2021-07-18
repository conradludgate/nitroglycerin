use nitroglycerin::Table;
#[derive(Table)]
#[nitro(table_name = "Foo")]
pub struct FooTable {}

fn main() {
    println!("Hello, world!");
}
