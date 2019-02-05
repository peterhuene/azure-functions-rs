use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse, Table},
    func,
};
use serde_json::Value;

#[func]
#[binding(
    name = "_req",
    auth_level = "anonymous",
    route = "read/{table}/{partition}/{row}"
)]
#[binding(
    name = "table",
    table_name = "{table}",
    partition_key = "{partition}",
    row_key = "{row}"
)]
#[binding(
    name="output1", // TODO: remove this workaround binding
    path="unused"
)]
pub fn read_row(_req: &HttpRequest, table: &Table) -> (HttpResponse, Option<Blob>) {
    (table.as_value().get(0).unwrap_or(&Value::Null).into(), None)
}
