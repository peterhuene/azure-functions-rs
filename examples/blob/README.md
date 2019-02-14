# Example Blob Triggered Azure Functions

This project is an example of simple blob-related Azure Functions.

## Example function implementations

An example blob-triggered Azure Function that runs when a new blob is created 
in the `watching` Azure Storage blob container.

```rust
use azure_functions::{bindings::BlobTrigger, func};

#[func]
#[binding(name = "trigger", path = "watching/{name}")]
pub fn blob_watcher(trigger: &BlobTrigger) {
    log::info!(
        "A blob was created at '{}' with contents: {:?}.",
        trigger.path,
        trigger.blob
    );
}
```

An example HTTP-triggered Azure Function that creates a new blob at the specified path:

```rust
use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
    http::Status,
};

#[func]
#[binding(name = "req", route = "create/blob/{container}/{name}")]
#[binding(name = "output1", path = "{container}/{name}")]
pub fn create_blob(req: &HttpRequest) -> (HttpResponse, Blob) {
    (
        HttpResponse::build()
            .status(Status::Created)
            .body("blob has been created.")
            .into(),
        req.body().as_bytes().into(),
    )
}
```

An example HTTP-triggered Azure Function that copies the specified blob:

```rust
use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "_req", route = "copy/blob/{container}/{name}")]
#[binding(name = "blob", path = "{container}/{name}")]
#[binding(name = "output1", path = "{container}/{name}.copy")]
pub fn copy_blob(_req: &HttpRequest, blob: &Blob) -> (HttpResponse, Blob) {
    ("blob has been copied.".into(), blob.clone())
}
```

An HTTP-triggered function that responds with the contents of a blob:

```rust
use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "_req", route = "print/blob/{container}/{path}")]
#[binding(name = "blob", path = "{container}/{path}")]
pub fn print_blob(_req: &HttpRequest, blob: &Blob) -> HttpResponse {
    blob.as_bytes().into()
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage` environment
variable must be set to a connection string that the Azure Functions Host will use for the default
storage connection.

To run with the `AzureWebJobsStorage` environment variable set:

```bash
$ AzureWebJobsStorage="<insert connection string here>" cargo func run
```

_Note: the syntax above works on macOS and Linux; on Windows, set the environment variables before running `cargo func run`._

# Invoking the functions

## Invoke the `create_blob` function

To create a blob called `hello` in the `test` container, use curl to invoke the `create_blob` function:

```
curl -d "hello world" http://localhost:8080/api/create/blob/test/hello
```

A message should print that the blob has been created.

With any luck, you should now see a `hello` blob in the `test` container with the contents `hello world`.

## Invoke the `copy_blob` function

To copy a blob called `hello` in the `test` container, use curl to invoke the `copy_blob` function:

```
curl http://localhost:8080/api/copy/blob/test/hello
```

A message should print that the blob was copied.

With any luck, you should now see a `hello.copy` blob in the `test` container with the contents `hello world`.

## Invoke the `print_blob` function

To print a blob called `hello` in the `test` container, use curl to invoke the `print_blob` function:

```
curl http://localhost:8080/api/print/blob/test/hello
```

With any luck, you should see `hello world` printed in your terminal.

## Invoke the `blob_watcher` function

To log a message when a blob is created, use curl to invoke the `create_blob` function to trigger the `blob_watcher` function:

```
curl -d "hello world" http://localhost:8080/api/create/blob/watching/hello
```

A message should be printed saying the blob was created.

With any luck, something like the following should be logged by the Azure Functions Host:

```
info: Function.blob_watcher[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executing 'Functions.blob_watcher' (Reason='New blob detected: watching/hello', Id=38848d35-01cc-4854-a3cb-3e0fb74b6704)
info: Worker.Rust.97626f24-4bbf-4895-a9d0-4362ea1e9498[0]
      A blob was created at 'watching/hello' with contents: Some("hello world")
info: Function.blob_watcher[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executed 'Functions.blob_watcher' (Succeeded, Id=38848d35-01cc-4854-a3cb-3e0fb74b6704)
```
