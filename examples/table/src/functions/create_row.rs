use azure_functions::{
    bindings::{HttpRequest, Table},
    func,
};
use serde_json::from_slice;

#[func]
#[binding(name = "output1", table_name = "{table}")]
pub fn create_row(
    #[binding(route = "create/{table}/{partition}/{row}")] req: HttpRequest,
) -> ((), Table) {
    let mut table = Table::new();
    {
        let row = table.add_row(
            req.route_params.get("partition").unwrap(),
            req.route_params.get("row").unwrap(),
        );

        row.insert(
            "body".to_string(),
            from_slice(req.body.as_bytes()).expect("expected JSON body"),
        );
    }
    ((), table)
}
