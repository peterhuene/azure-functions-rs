# Example HTTP Azure Function

This project is an example of a simple HTTP-triggered Azure Function.

## Example function implementations

An example HTTP-triggered Azure Function:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn greet(req: HttpRequest) -> HttpResponse {
    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    )
    .into()
}
```

An async version of the `greet` function when the example is built with a nightly compiler and the `unstable` feature enabled:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};
use futures::future::ready;

#[func]
pub async fn greet_async(req: HttpRequest) -> HttpResponse {
    // Use ready().await to simply demonstrate the async/await feature
    ready(format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    ))
    .await
    .into()
}
```

An example HTTP-triggered Azure Function using JSON for request and response:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
    http::Status,
};
use serde::{Deserialize, Serialize};
use serde_json::to_value;

#[derive(Deserialize)]
struct Request {
    name: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

#[func]
pub fn greet_with_json(req: HttpRequest) -> HttpResponse {
    if let Ok(request) = req.body().as_json::<Request>() {
        let response = Response {
            message: format!("Hello from Rust, {}!", request.name),
        };
        return to_value(response).unwrap().into();
    }

    HttpResponse::build()
        .status(Status::BadRequest)
        .body("Invalid JSON request.")
        .into()
}
```

# Running the example locally

Run the example application with `cargo func run`:

```bash
$ cargo func run
```

To run the example with support for async functions when using a nightly compiler:

```bash
$ cargo func run -- --features unstable
```

# Invoking the functions

## Invoke the `greet` function

The easiest way to invoke the function is to use `curl`:

```
$ curl localhost:8080/api/greet\?name=Peter
```

With any luck, you should see the following output:

```
Hello from Rust, Peter!
```

## Invoke the `greet_async` function

With support for async functions enabled, invoke the function with `curl`:

```
$ curl localhost:8080/api/greet_async\?name=Peter
```

With any luck, you should see the following output:

```
Hello from Rust, Peter!
```

## Invoke the `greet_with_json` function

The easiest way to invoke the function is to use `curl`:

```
$ curl --header "Content-Type: application/json" -d '{"name": "Peter"}' http://localhost:8080/api/greet_with_json
```

With any luck, you should see the following output:

```json
{
  "message": "Hello from Rust, Peter!"
}
```
