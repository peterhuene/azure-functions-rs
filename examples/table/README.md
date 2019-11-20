# Example Table Azure Functions

This project is an example of simple table-related Azure Functions.

## Example function implementations

An example function that creates a row in an Azure Storage using an output table binding:

```rust
use azure_functions::{
    bindings::{HttpRequest, Table},
    func,
};
use serde_json::from_slice;

#[func]
#[binding(name = "output1", table_name = "{table}")]
pub fn create_row(
    #[binding(route = "create/{table}/{partition}/{row}")] req: HttpRequest,
) -> ((), Table) {
    let mut table = Table::new();
    {
        let row = table.add_row(
            req.route_params.get("partition").unwrap(),
            req.route_params.get("row").unwrap(),
        );

        row.insert(
            "body".to_string(),
            from_slice(req.body.as_bytes()).expect("expected JSON body"),
        );
    }
    ((), table)
}
```

An example function that reads a row using an input table binding:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse, Table},
    func,
};

#[func]
pub fn read_row(
    #[binding(route = "read/{table}/{partition}/{row}")] _req: HttpRequest,
    #[binding(
        table_name = "{table}",
        partition_key = "{partition}",
        row_key = "{row}"
    )]
    table: Table,
) -> HttpResponse {
    table.into()
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage`
setting must be set to a connection string that the Azure Functions Host will use for
the default storage connection.

Add a setting for `AzureWebJobsStorage` into `local.settings.json`:

```
$ func settings add AzureWebJobsStorage <storage_connection_string>
```

You may encrypt `local.settings.json`, if desired:

```
$ func settings encrypt
```

Finally, start the Azure Functions application:

```
$ cargo func run
```

# Invoking the functions

## Invoke the `create_row` function

To create a row in a table named `test` with partition key `partition1` and row key `row1`,
use curl to invoke the `create_row` function:

```
$ curl -d "hello world!" http://localhost:8080/api/create/test/partition1/row1 -v
```

With any luck, this should return a `204 No Content` response.

## Invoke the `read_row` function

To read a row from a table named `test` with partition key `partition1` and row key `row1`:

```
$ curl http://localhost:8080/api/read/test/partition1/row1
```

With any luck, the entity should be printed by `curl`.
