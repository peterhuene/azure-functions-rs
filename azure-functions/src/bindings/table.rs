use crate::http::Body;
use crate::rpc::protocol;
use serde_json::{from_str, json, Map, Value};
use std::fmt;

/// Represents an Azure Storage table input or output binding.
///
/// The following binding attributes are supported:
///
/// | Name            | Description                                                                                                                        |
/// |-----------------|------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`          | The name of the parameter being bound.                                                                                             |
/// | `table_name`    | The name of the table.                                                                                                             |
/// | `partition_key` | The partition key of the table entity to read or write.                                                                            |
/// | `row_key`       | The row key of the table entity to read or write.                                                                                  |
/// | `filter`        | The maximum number of entities to read (optional; input only).                                                                     |
/// | `take`          | The OData filter expression of entities to read (optional; input only).                                                            |
/// | `connection`    | The name of an app setting that contains the Storage connection string to use for this binding. Defaults to `AzureWebJobsStorage`. |
///
/// # Examples
///
/// Read a table storage row based on a key posted to the `example` queue:
///
/// ```rust
/// use azure_functions::bindings::{QueueTrigger, Table};
/// use azure_functions::func;
/// use log::warn;
///
/// #[func]
/// #[binding(name = "trigger", queue_name = "example")]
/// #[binding(name = "table", table_name = "MyTable", partition_key = "MyPartition", row_key = "{queueTrigger}")]
/// pub fn log_row(trigger: QueueTrigger, table: Table) {
///     warn!("Row: {:?}", table.rows().nth(0));
/// }
/// ```
/// Run an Azure Storage table query based on a HTTP request:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, Table};
/// use azure_functions::func;
/// use log::warn;
///
/// #[func]
/// #[binding(name = "table", table_name = "MyTable", filter = "{filter}")]
/// pub fn log_rows(req: HttpRequest, table: Table) {
///     for row in table.rows() {
///         warn!("Row: {:?}", row);
///     }
/// }
#[derive(Default, Debug, Clone)]
pub struct Table(Value);

/// Represents the data of an Azure Storage table row.
pub type Row = Map<String, Value>;

impl Table {
    /// Creates a new table binding.
    ///
    /// The new table binding can be used for output.
    pub fn new() -> Table {
        Table(Value::Array(Vec::new()))
    }

    /// Gets whether or not the table binding is empty (no rows).
    pub fn is_empty(&self) -> bool {
        self.0.as_array().unwrap().is_empty()
    }

    /// Gets the current length of the rows stored in the table binding.
    pub fn len(&self) -> usize {
        self.0.as_array().unwrap().len()
    }

    /// Gets the iterator over the rows stored in the table binding.
    ///
    /// For input bindings, this will be the rows returned from either a single entity lookup
    /// or a filter query.
    ///
    /// For output bindings, this will be the rows that have been added to the table binding.
    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.0
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_object().unwrap())
    }

    /// Adds a new row to the table binding with the specified partition and row keys.
    pub fn add_row(&mut self, partition_key: &str, row_key: &str) -> &mut Row {
        let array = self.0.as_array_mut().unwrap();

        array.push(json!({
            "PartitionKey": partition_key,
            "RowKey": row_key
        }));

        array.last_mut().unwrap().as_object_mut().unwrap()
    }

    /// Adds a row as a value to the table.
    pub fn add_row_value(&mut self, value: Value) {
        let array = self.0.as_array_mut().unwrap();

        array.push(value);
    }

    /// Gets the table as a JSON value.
    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[doc(hidden)]
impl From<protocol::TypedData> for Table {
    fn from(data: protocol::TypedData) -> Self {
        if data.has_json() {
            let mut rows: Value =
                from_str(data.get_json()).expect("expected valid JSON data for table binding");

            if rows.is_object() {
                rows = Value::Array(vec![rows]);
            }

            if !rows.is_array() {
                panic!("expected an object or array for table binding data");
            }

            Table(rows)
        } else {
            Table::new()
        }
    }
}

impl Into<Value> for Table {
    fn into(self) -> Value {
        self.0
    }
}

impl<'a> Into<Body<'a>> for Table {
    fn into(self) -> Body<'a> {
        self.0.into()
    }
}

#[doc(hidden)]
impl Into<protocol::TypedData> for Table {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();
        data.set_json(self.0.to_string());
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn it_constructs_an_empty_table() {
        let table = Table::new();
        assert_eq!(table.len(), 0);
        assert_eq!(table.rows().count(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn it_is_not_empty_when_rows_are_present() {
        let mut table = Table::new();
        table.add_row("partition1", "row1");
        assert!(!table.is_empty());
    }

    #[test]
    fn it_has_a_length_equal_to_number_of_rows() {
        let mut table = Table::new();
        assert_eq!(table.len(), 0);
        table.add_row("partition1", "row1");
        table.add_row("partition2", "row2");
        table.add_row("partition3", "row3");
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn it_iterates_rows() {
        let mut table = Table::new();
        assert_eq!(table.len(), 0);
        table.add_row("partition1", "row1");
        table.add_row("partition2", "row2");
        table.add_row("partition3", "row3");
        assert_eq!(table.len(), 3);

        for (i, row) in table.rows().enumerate() {
            assert_eq!(
                row.get("PartitionKey").unwrap().as_str().unwrap(),
                format!("partition{}", i + 1)
            );
            assert_eq!(
                row.get("RowKey").unwrap().as_str().unwrap(),
                format!("row{}", i + 1)
            );
        }
    }

    #[test]
    fn it_adds_row_value() {
        let mut table = Table::new();
        assert_eq!(table.len(), 0);
        table.add_row_value(json!({
            "PartitionKey": "partition1",
            "RowKey": "row1",
            "data": "hello world"
        }));

        assert_eq!(
            table.to_string(),
            r#"[{"PartitionKey":"partition1","RowKey":"row1","data":"hello world"}]"#
        );
    }

    #[test]
    fn it_casts_to_value_reference() {
        let mut table = Table::new();
        table.add_row("partition1", "row1");

        assert_eq!(
            table.as_value().to_string(),
            r#"[{"PartitionKey":"partition1","RowKey":"row1"}]"#
        );
    }

    #[test]
    fn it_displays_as_a_string() {
        let mut table = Table::new();
        {
            let row = table.add_row("partition1", "row1");
            row.insert("data".to_string(), Value::String("value".to_string()));
        }
        let mut s = String::new();
        write!(s, "{}", table).unwrap();

        assert_eq!(
            s,
            r#"[{"PartitionKey":"partition1","RowKey":"row1","data":"value"}]"#
        );
    }

    #[test]
    fn it_converts_from_typed_data() {
        const TABLE: &'static str =
            r#"[{"PartitionKey":"partition1","RowKey":"row1","data":"value"}]"#;

        let mut data = protocol::TypedData::new();
        data.set_json(TABLE.to_string());

        let table: Table = data.into();
        assert_eq!(table.len(), 1);
        assert_eq!(table.to_string(), TABLE);

        let mut data = protocol::TypedData::new();
        data.set_string("".to_string());

        let table: Table = data.into();
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn it_converts_to_json() {
        let mut table = Table::new();
        table.add_row("partition1", "row1");

        let value: Value = table.into();

        assert_eq!(
            value.to_string(),
            r#"[{"PartitionKey":"partition1","RowKey":"row1"}]"#
        );
    }

    #[test]
    fn it_converts_to_body() {
        const TABLE: &'static str =
            r#"[{"PartitionKey":"partition1","RowKey":"row1","data":"value"}]"#;

        let mut data = protocol::TypedData::new();
        data.set_json(TABLE.to_string());

        let table: Table = data.into();
        let body: Body = table.into();
        assert_eq!(
            body.as_str().unwrap(),
            r#"[{"PartitionKey":"partition1","RowKey":"row1","data":"value"}]"#
        );
    }

    #[test]
    fn it_converts_to_typed_data() {
        let mut table = Table::new();
        {
            let row = table.add_row("partition1", "row1");
            row.insert("data".to_string(), Value::String("value".to_string()));
        }
        let data: protocol::TypedData = table.into();
        assert!(data.has_json());
        assert_eq!(
            data.get_json(),
            r#"[{"PartitionKey":"partition1","RowKey":"row1","data":"value"}]"#
        );
    }
}
