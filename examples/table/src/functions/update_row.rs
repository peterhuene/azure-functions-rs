use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};
#[cfg(feature = "azure-sdk")]
use azure_sdk_storage_core::client::Client;
#[cfg(feature = "azure-sdk")]
use azure_sdk_storage_table::{table::TableService, TableEntry};

#[cfg(not(feature = "azure-sdk"))]
#[func]
#[binding(name = "_req", route = "update/{table}/{partition}/{row}")]
pub async fn update_row(_req: HttpRequest) -> HttpResponse {
    "azure-sdk feature was not enabled".into()
}

#[cfg(feature = "azure-sdk")]
#[func]
#[binding(name = "req", route = "update/{table}/{partition}/{row}")]
pub async fn update_row(req: HttpRequest) -> HttpResponse {
    use serde_json::{from_slice, Value};

    let body: Value = from_slice(req.body.as_bytes()).expect("expected JSON body");
    let ts = get_table_service();
    match ts
        .update_entry(
            req.route_params.get("table").unwrap(),
            &TableEntry {
                partition_key: req.route_params.get("partition").unwrap().into(),
                row_key: req.route_params.get("row").unwrap().into(),
                etag: None,
                payload: { body },
            },
        )
        .await
    {
        Ok(_) => "Updated entity.".into(),
        Err(err) => format!("Failed when trying to update entity: {}", err).into(),
    }
}

#[cfg(feature = "azure-sdk")]
fn get_table_service() -> TableService {
    use regex::Regex;
    use std::env::var;

    let connection_string = var("AzureWebJobsStorage").unwrap();
    let re = Regex::new(r"AccountName=(?P<name>\S*)?;AccountKey=(?P<key>\S+);").unwrap();
    let connection_string_matches = re.captures_iter(connection_string.as_str()).nth(0).unwrap();
    let account = connection_string_matches.name("name").unwrap().as_str();
    let key = connection_string_matches.name("key").unwrap().as_str();
    let client = Client::new(account, key).unwrap();
    TableService::new(client)
}
