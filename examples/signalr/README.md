# Example SignalR Functions

This project is an example of using the Azure SignalR Service with Azure Functions for Rust.

## Example function implementations

The functions for this example are ported from the [Simple Chat Sample](https://github.com/Azure/azure-functions-signalrservice-extension/tree/dev/samples/simple-chat) from the [Azure Functions Bindings for Azure SignalR Service Repository](https://github.com/Azure/azure-functions-signalrservice-extension).

To use the Azure SignalR service from Azure Functions, the SignalR client needs to discover the service URL and authentication token to use.  For this, we expose a `negotiate` function that returns the information provided to the function by the Azure SignalR service:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse, SignalRConnectionInfo},
    func,
};

#[func]
pub fn negotiate(
    #[binding(auth_level = "anonymous")] _req: HttpRequest,
    #[binding(
        hub_name = "simplechat",
        user_id = "{headers.x-ms-signalr-userid}",
        connection = "connection"
    )]
    info: SignalRConnectionInfo,
) -> HttpResponse {
    info.into()
}
```

The `x-ms-signalr-userid` header is coming from the sample code to convey the user to authenticate with the Azure SignalR Service.  The sample will prompt for a username when it starts.

The sample uses an Azure Function to send a message to the SignalR service:

```rust
use crate::serialization::ChatMessage;
use azure_functions::{
    bindings::{HttpRequest, SignalRMessage},
    func,
};
use serde_json::{from_slice, to_value};

#[func(name = "messages")]
#[binding(name = "$return", hub_name = "simplechat", connection = "connection")]
pub fn send_message(
    #[binding(auth_level = "anonymous", methods = "post")] req: HttpRequest,
) -> SignalRMessage {
    let message: ChatMessage =
        from_slice(req.body.as_bytes()).expect("failed to deserialize chat message");

    SignalRMessage {
        user_id: message.recipient.clone(),
        group_name: message.group_name.clone(),
        target: "newMessage".to_string(),
        arguments: vec![to_value(message).expect("failed to serialize chat message")],
    }
}
```

The function accepts a JSON body and returns a SignalR message that will be sent to a specific user, a specific group, or to all clients.  The `target` is the JavaScript function to invoke on the clients (e.g. `newMessage`) and the `arguments` are the JSON values to pass as arguments to the script.

In addition to sending messages, the sample uses two functions to manage group membership:

```rust
use crate::serialization::ChatMessage;
use azure_functions::{
    bindings::{HttpRequest, SignalRGroupAction},
    func,
    signalr::GroupAction,
};
use serde_json::from_slice;

#[func(name = "addToGroup")]
#[binding(name = "$return", hub_name = "simplechat", connection = "connection")]
pub fn add_to_group(
    #[binding(auth_level = "anonymous", methods = "post")] req: HttpRequest,
) -> SignalRGroupAction {
    let message: ChatMessage =
        from_slice(req.body.as_bytes()).expect("failed to deserialize chat message");

    SignalRGroupAction {
        user_id: message.recipient.unwrap(),
        group_name: message.group_name.unwrap(),
        action: GroupAction::Add,
    }
}
```

When the above function is invoked, the user gets added to the specified group.

```rust
use crate::serialization::ChatMessage;
use azure_functions::{
    bindings::{HttpRequest, SignalRGroupAction},
    func,
    signalr::GroupAction,
};
use serde_json::from_slice;

#[func(name = "removeFromGroup")]
#[binding(name = "$return", hub_name = "simplechat", connection = "connection")]
pub fn remove_from_group(
    #[binding(auth_level = "anonymous", methods = "post")] req: HttpRequest,
) -> SignalRGroupAction {
    let message: ChatMessage =
        from_slice(req.body.as_bytes()).expect("failed to deserialize chat message");

    SignalRGroupAction {
        user_id: message.recipient.unwrap(),
        group_name: message.group_name.unwrap(),
        action: GroupAction::Remove,
    }
}
```

When the above function is invoked, the user gets removed from the specified group.

# Running the example locally

This example uses a connection setting named `connection` for the Azure SignalR Service connection string, so add that setting to `local.settings.json`:

```
$ func settings add connection <signalr_connection_string>
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

To invoke the functions we'll use a HTML document that implements a simple chat client.  This file is available from the [Simple Chat Sample](https://github.com/Azure/azure-functions-signalrservice-extension/tree/dev/samples/simple-chat):

```
$ wget https://raw.githubusercontent.com/Azure/azure-functions-signalrservice-extension/c1e53f799ec444362c5db1dbd48aeb02a42016ee/samples/simple-chat/content/index.html
```

Open the downloaded `index.html` in your web browser.

When prompted for the Azure Function app base URL, use `http://localhost:8080`.

When prompted for a username, enter a display name you want for the chat client.

You may then send messages with the provided textbox.

Load `index.html` in additional tabs with different usernames to simulate multiple users participating in the chat.