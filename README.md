# nitroglycerin
Type safe abstractions over dynamodb

[![nitroglycerin crate](https://img.shields.io/crates/v/nitroglycerin?style=flat-square)](https://crates.io/crates/nitroglycerin)
[![nitroglycerin docs](https://img.shields.io/docsrs/nitroglycerin?style=flat-square)](https://docs.rs/nitroglycerin/latest/nitroglycerin/)

```rust
use nitroglycerin::{Attributes, Key, Query, Table, DynamoDb, dynamodb::DynamoDbClient};
use rusoto_core::Region;

#[derive(Debug, PartialEq, Attributes, Key, Query)]
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

let new_employee = Employee {
    id: "emp_1234".into(),
    name: "Conrad".into(),
    joined: 1626900000,
    left: None,
};
// Put the new employee item into the db
client.put(new_employee).execute().await?;

let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
    .name("John".to_string()) // query the db for all employees named "John"
    .execute().await?;

let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
    .name("John".to_string()) // query the db for all employees named "John"
    .joined().between(1626649200, 1626735600) // and who joined between 2021-07-19 and 2021-07-20
    .execute().await?;
```
