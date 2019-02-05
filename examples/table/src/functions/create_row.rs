use azure_functions::{
    bindings::{HttpRequest, Table},
    func,
};
use serde_json::Value;

#[func]
#[binding(
    name = "req",
    auth_level = "anonymous",
    route = "create/{table}/{partition}/{row}"
)]
#[binding(name = "output1", table_name = "{table}")]
pub fn create_row(req: &HttpRequest) -> ((), Table) {
    let mut table = Table::new();
    {
        let row = table.add_row(
            req.route_params().get("partition").unwrap(),
            req.route_params().get("row").unwrap(),
        );

        row.insert(
            "body".to_string(),
            Value::String(req.body().as_str().unwrap().to_owned()),
        );
    }
    ((), table)
}
