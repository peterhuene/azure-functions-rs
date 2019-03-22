# Example Cosmos DB Azure Functions

This project is an example of using Cosmos DB with Azure Functions for Rust.

## Example function implementations

An example HTTP-triggered Azure Function that outputs a Cosmos DB document:

```rust
use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};
use serde_json::json;

#[func]
#[binding(name = "req", route = "create/{id}")]
#[binding(
    name = "output1",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    create_collection = true
)]
pub fn create_document(req: HttpRequest) -> (HttpResponse, CosmosDbDocument {
    (
        "Document was created.".into(),
        json!({
            "id": req.route_params().get("id").unwrap(),
            "name": req.query_params().get("name").map_or("stranger", |x| x)
        })
        .into(),
    )
}
```

An example Cosmos DB triggered Azure Function that will log warnings for each new Cosmos DB document inserted or updated to a collection:

```rust
use azure_functions::{bindings::CosmosDbTrigger, func};
use log::warn;

#[func]
#[binding(
    name = "trigger",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents"
)]
pub fn log_documents(trigger: CosmosDbTrigger) {
    for document in trigger.documents {
        warn!("{}", document);
    }
}
```

An example HTTP-triggered Azure Function that will read a Cosmos DB document and return it as a HTTP response:

```rust
use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "req", route = "read/{id}")]
#[binding(
    name = "documents",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    id = "{id}",
    partition_key = "{id}",
)]
pub fn read_document(req: HttpRequest, document: CosmosDbDocument) -> HttpResponse {
    if document.is_null() {
        format!(
            "Document with id '{}' does not exist.",
            req.route_params().get("id").unwrap()
        )
        .into()
    } else {
        document.into()
    }
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage`
setting must be set to a connection string that the Azure Functions Host will use for 
the default storage connection.

Start by creating a known script root for the Azure Functions application:

```
$ cargo run -- init --script-root /tmp/myroot && cd /tmp/myroot
```

Next, add a setting for `AzureWebJobsStorage`:

```
$ func settings add AzureWebJobsStorage <storage_connection_string>
```

Additionally, this example uses a connection setting named `connection` for the Cosmos DB connection string, so add that setting:

```
$ func settings add connection <cosmos_db_connection_string>
```

This example expects an `exampledb` database with a collection named `documents` to exist for the Cosmos DB triggered function to monitor.  Use the Cosmos DB Data Explorer to create a database named `exampledb` that contains a collection named `documents` before running the example.

Finally, change back to the example directory and start the Azure Functions application:

```
$ cargo func run --script-root /tmp/myroot
```

# Invoking the functions

## Invoke the `create_document` function

This function is designed to trigger the `log_documents` function by creating a Cosmos DB document in the collection being monitored.

Simply use `curl` to invoke the `create_document` function with the desired document identifier:

```
$ curl http://localhost:8080/api/create/<id>
```

Where `<id>` is the document identifier.

With any luck, something like the following should be logged by the Azure Functions Host as a warning:

```
[3/12/19 7:14:53 AM] {"_etag":"\"12005511-0000-0000-0000-5c875c6a0000\"","_lsn":10,"_metadata":{},"_rid":"JeJJAIEVMHMFAAAAAAAAAA==","_self":"dbs/JeJJAA==/colls/JeJJAIEVMHM=/docs/JeJJAIEVMHMFAAAAAAAAAA==/","_ts":1552374890,"id":"test","name":"stranger"}
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
