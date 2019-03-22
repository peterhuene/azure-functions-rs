# Example Event Grid Triggered Azure Function

This project is an example of a simple Event Grid triggered Azure Function.

## Example function implementations

An example Event Grid triggered Azure Function that runs when a new event is posted:

```rust
use azure_functions::{bindings::EventGridEvent, func};

#[func]
pub fn log_event(event: EventGridEvent) {
    log::warn!("Event Data: {}", event.data);
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage` setting must be set to a connection string that the Azure Functions Host will use for the default
storage connection.

Start by creating a known script root for the Azure Functions application:

```
$ cargo run -- init --script-root /tmp/myroot && cd /tmp/myroot
```

Next, add a setting for `AzureWebJobsStorage`:

```
$ func settings add AzureWebJobsStorage <storage_connection_string>
```

Finally, change back to the example directory and start the Azure Functions application:

```
$ cargo func run --script-root /tmp/myroot
```

# Invoking the functions

## Invoke the `log_event` function

Unfortunately, it's not easy to invoke an Event Grid triggered Azure Function locally because Event Grid will not be able to deliver the events to your local Azure Functions application.

However, it is possible to manually post data to the webhook that will invoke the function.

Start by creating a file called `events.json` with the following contents (from [the Event Grid Schema documentation](https://docs.microsoft.com/en-us/azure/event-grid/event-schema)):

```json
[
  {
    "topic": "/subscriptions/{subscription-id}/resourceGroups/Storage/providers/Microsoft.Storage/storageAccounts/xstoretestaccount",
    "subject": "/blobServices/default/containers/oc2d2817345i200097container/blobs/oc2d2817345i20002296blob",
    "eventType": "Microsoft.Storage.BlobCreated",
    "eventTime": "2017-06-26T18:41:00.9584103Z",
    "id": "831e1650-001e-001b-66ab-eeb76e069631",
    "data": {
      "api": "PutBlockList",
      "clientRequestId": "6d79dbfb-0e37-4fc4-981f-442c9ca65760",
      "requestId": "831e1650-001e-001b-66ab-eeb76e000000",
      "eTag": "0x8D4BCC2E4835CD0",
      "contentType": "application/octet-stream",
      "contentLength": 524288,
      "blobType": "BlockBlob",
      "url": "https://oc2d2817345i60006.blob.core.windows.net/oc2d2817345i200097container/oc2d2817345i20002296blob",
      "sequencer": "00000000000004420000000000028963",
      "storageDiagnostics": {
        "batchId": "b68529f3-68cd-4744-baa4-3c0498ec19f0"
      }
    },
    "dataVersion": "",
    "metadataVersion": "1"
  }
]
```

Next, use `curl` to POST this data to the Event Grid webhook:

```
$ curl -H "aeg-event-type: Notification" -H "Content-Type: application/json" -X POST --data @events.json http://localhost:8080/runtime/webhooks/eventgrid\?functionName=log_event
```

With any luck, something like the following should be logged by the Azure Functions Host:

```
Event Data: {"api":"PutBlockList","blobType":"BlockBlob","clientRequestId":"6d79dbfb-0e37-4fc4-981f-442c9ca65760","contentLength":524288,"contentType":"application/octet-stream","eTag":"0x8D4BCC2E4835CD0","requestId":"831e1650-001e-001b-66ab-eeb76e000000","sequencer":"00000000000004420000000000028963","storageDiagnostics":{"batchId":"b68529f3-68cd-4744-baa4-3c0498ec19f0"},"url":"https://oc2d2817345i60006.blob.core.windows.net/oc2d2817345i200097container/oc2d2817345i20002296blob"}
```
