# nitroglycerin
Type safe abstractions over dynamodb (extending on dynomite)

```rust
use nitroglycerin::{Attributes, Get, Query, Table, DynamoDb, dynamodb::DynamoDbClient};
use rusoto_core::Region;

#[derive(Debug, PartialEq, Attributes, Get, Query)]
struct Employee {
    #[nitro(partition_key)]
    id: String,
    #[nitro(rename = "firstName")]
    name: String,
    joined: i64,
    left: Option<i64>,
}

impl Table for Employee {
    fn table_name() -> String {
        "Employees".to_string()
    }
}

#[derive(Debug, PartialEq, Attributes, Query)]
struct EmployeeNameIndex {
    #[nitro(partition_key, rename = "firstName")]
    name: String,
    #[nitro(sort_key)]
    joined: i64,
}

impl IndexTable for EmployeeNameIndex {
    type Table = Employees;
    fn index_name() -> Option<String> {
        Some("EmployeeNamesIndex".to_string())
    }
}

let client = DynamoDbClient::new(Region::default());

let employee: Option<Employee> = client.get::<Employee>()
    .id("emp_1") // get the employee with id "emp_1"
    .execute().await?;

let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
    .name("John".to_string()) // query all employees named "John"
    .joined().between(1626649200, 1626735600) // who joined between 2021-07-19 and 2021-07-20
    .execute().await?;

let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
    .name("John".to_string()) // query all employees named "John"
    .joined().between(1626649200, 1626735600) // who joined between 2021-07-19 and 2021-07-20
    .execute().await?;
```
