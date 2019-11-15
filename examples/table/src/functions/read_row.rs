use azure_functions::{
    bindings::{HttpRequest, HttpResponse, Table},
    func,
};

#[func]
pub fn read_row(
    #[binding(route = "read/{table}/{partition}/{row}")] _req: HttpRequest,
    #[binding(
        table_name = "{table}",
        partition_key = "{partition}",
        row_key = "{row}"
    )]
    table: Table,
) -> HttpResponse {
    table.into()
}
