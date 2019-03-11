# Example HTTP Azure Function

This project is an example of a simple HTTP-triggered Azure Function.

## Example function implementations

An example HTTP-triggered Azure Function:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func, Context,
};

#[func]
pub fn greet(context: Context, req: HttpRequest) -> HttpResponse {
    log::info!("Context: {:?}, Request: {:?}", context, req);

    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    )
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
