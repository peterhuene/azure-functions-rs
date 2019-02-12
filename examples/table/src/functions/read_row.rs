use azure_functions::{
    bindings::{HttpRequest, HttpResponse, Table},
    func,
};
use serde_json::Value;

#[func]
#[binding(name = "_req", route = "read/{table}/{partition}/{row}")]
#[binding(
    name = "table",
    table_name = "{table}",
    partition_key = "{partition}",
    row_key = "{row}"
)]
pub fn read_row(_req: &HttpRequest, table: &Table) -> HttpResponse {
    table.as_value().get(0).unwrap_or(&Value::Null).into()
}
