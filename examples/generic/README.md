# Example Cosmos DB Azure Functions

This project is an example of using generic trigger, input, and output bindings with Azure Functions for Rust.

This is a copy of the CosmosDB example, except it uses `GenericTrigger`, `GenericInput`, and `GenericOutput` bindings
in place of the actual CosmosDB bindings.

## Example function implementations

An example HTTP-triggered Azure Function that outputs a Cosmos DB document:

```rust
use azure_functions::{
    bindings::{GenericOutput, HttpRequest, HttpResponse},
    func,
};
use serde_json::json;

#[func]
#[binding(
    type = "cosmosDB",
    name = "output1",
    connectionStringSetting = "connection",
    databaseName = "exampledb",
    collectionName = "documents",
    createIfNotExists = true
)]
pub fn create_document(
    #[binding(route = "create/{id}")] mut req: HttpRequest,
) -> (HttpResponse, GenericOutput) {
    (
        "Document was created.".into(),
        json!({
            "id": req.route_params.remove("id").unwrap(),
            "name": req.query_params.remove("name").expect("expected a 'name' query parameter"),
        })
        .into(),
    )
}
```

An example Cosmos DB triggered Azure Function that will log informational messages for each new Cosmos DB document inserted or updated to a collection:

```rust
use azure_functions::{bindings::GenericTrigger, func, generic::Value};
use log::info;

#[func]
pub fn log_documents(
    #[binding(
        type = "cosmosDBTrigger",
        connectionStringSetting = "connection",
        databaseName = "exampledb",
        collectionName = "documents",
        createLeaseCollectionIfNotExists = true
    )]
    trigger: GenericTrigger,
) {
    match trigger.data {
        Value::Json(v) => {
            info!("{}", v);
        }
        _ => panic!("expected JSON for Cosmos DB trigger data"),
    }
}
```

An example HTTP-triggered Azure Function that will query for Cosmos DB documents and return them as a HTTP response:

```rust
use azure_functions::{
    bindings::{GenericInput, HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn query_documents(
    #[binding(route = "query/{name}")] _req: HttpRequest,
    #[binding(
        type = "cosmosDB",
        connectionStringSetting = "connection",
        databaseName = "exampledb",
        collectionName = "documents",
        sqlQuery = "select * from documents d where contains(d.name, {name})",
        createIfNotExists = true
    )]
    documents: GenericInput,
) -> HttpResponse {
    documents.into()
}
```

An example HTTP-triggered Azure Function that will read a Cosmos DB document and return it as a HTTP response:

```rust
use azure_functions::{
    bindings::{GenericInput, HttpRequest, HttpResponse},
    func,
    generic::Value,
};
use serde_json::from_str;

#[func]
pub fn read_document(
    #[binding(route = "read/{id}")] req: HttpRequest,
    #[binding(
        type = "cosmosDB",
        connectionStringSetting = "connection",
        databaseName = "exampledb",
        collectionName = "documents",
        id = "{id}",
        partitionKey = "{id}"
    )]
    document: GenericInput,
) -> HttpResponse {
    match document.data {
        Value::String(s) => {
            let v: serde_json::Value = from_str(&s).expect("expected JSON data");
            if v.is_null() {
                format!(
                    "Document with id '{}' does not exist.",
                    req.route_params.get("id").unwrap()
                )
                .into()
            } else {
                v.into()
            }
        }
        _ => panic!("expected string for CosmosDB document data"),
    }
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

Additionally, this example uses a connection setting named `connection` for the Cosmos DB connection string, so add that setting:

```
$ func settings add connection <cosmos_db_connection_string>
```

You may encrypt `local.settings.json`, if desired:

```
$ func settings encrypt
```

This example expects an `exampledb` database with a collection named `documents` to exist for the Cosmos DB triggered function to monitor.  Use the Cosmos DB Data Explorer to create a database named `exampledb` that contains a collection named `documents` before running the example.

Finally, start the Azure Functions application:

```
$ cargo func run
```

# Invoking the functions

## Invoke the `create_document` function

This function is designed to trigger the `log_documents` function by creating a Cosmos DB document in the collection being monitored.

Simply use `curl` to invoke the `create_document` function with the desired document identifier:

```
$ curl http://localhost:8080/api/create/<id>
```

Where `<id>` is the document identifier.

With any luck, something like the following should be logged by the Azure Functions Host as an informational message:

```
[3/12/19 7:14:53 AM] [{"_etag":"\"12005511-0000-0000-0000-5c875c6a0000\"","_lsn":10,"_metadata":{},"_rid":"JeJJAIEVMHMFAAAAAAAAAA==","_self":"dbs/JeJJAA==/colls/JeJJAIEVMHM=/docs/JeJJAIEVMHMFAAAAAAAAAA==/","_ts":1552374890,"id":"test","name":"stranger"}]
```

This was logged by the `log_documents` function when the Cosmos DB document was saved to the database.

## Invoke the `read_document` function

This function reads a Cosmos DB document with a given identifier and returns it in a HTTP response.

Simply use `curl` to invoke the `read_document` function with the desired document identifier:

```
$ curl http://localhost:8080/api/read/<id>
```

Where `<id>` is the document identifier used when calling `create_document` above.

With any luck, `curl` should output the JSON of the Cosmos DB document:

```json
{
  "_etag": "\"12005511-0000-0000-0000-5c875c6a0000\"",
  "_rid": "JeJJAIEVMHMFAAAAAAAAAA==",
  "_self": "dbs/JeJJAA==/colls/JeJJAIEVMHM=/docs/JeJJAIEVMHMFAAAAAAAAAA==/",
  "_ts": 1552374890,
  "id": "test",
  "name": "stranger"
}
```

## Invoke the `query_documents` function

This function queries documents based on a query for the `name` field.

Simply use `curl` to invoke the `query_documents` function with the name to query:

```
$ curl http://localhost:8080/api/query/stranger
```

With any luck, `curl` should output the JSON of any Cosmos DB documents with a `name` field containing "stranger":

```json
[
  {
    "_etag": "\"12005511-0000-0000-0000-5c875c6a0000\"",
    "_rid": "JeJJAIEVMHMFAAAAAAAAAA==",
    "_self": "dbs/JeJJAA==/colls/JeJJAIEVMHM=/docs/JeJJAIEVMHMFAAAAAAAAAA==/",
    "_ts": 1552374890,
    "id": "test",
    "name": "stranger"
  }
]
```
