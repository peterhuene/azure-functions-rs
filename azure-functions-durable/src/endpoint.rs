use std::fmt::Write;
use url::Url;

/// Represents a Durable Functions HTTP API endpoint.
#[derive(Debug, Clone)]
pub struct Endpoint {
    base_uri: Url,
    task_hub: String,
    connection: String,
    code: String,
}

impl Endpoint {
    /// Create a new endpoint from a status query URL.
    pub fn new(status_query_url: Url) -> Self {
        let mut task_hub = None;
        let mut connection = None;
        let mut code = None;

        for (k, v) in status_query_url.query_pairs() {
            match k.to_ascii_lowercase().as_ref() {
                "taskhub" => task_hub = Some(v.into_owned()),
                "connection" => connection = Some(v.into_owned()),
                "code" => code = Some(v.into_owned()),
                _ => {}
            };
        }

        Self {
            base_uri: status_query_url,
            task_hub: task_hub.expect("expected a taskhub parameter"),
            connection: connection.expect("expected a connection parameter"),
            code: code.expect("expected a code parameter"),
        }
    }

    /// Gets the task hub associated with the endpoint.
    pub fn task_hub(&self) -> &str {
        &self.task_hub
    }

    /// Gets the "create new instance" URL from the endpoint.
    pub fn create_new_instance_url(&self, function_name: &str, instance_id: Option<&str>) -> Url {
        let mut url = self.base_uri.clone();

        let path = match instance_id {
            Some(id) => format!(
                "/runtime/webhooks/durabletask/orchestrators/{}/{}",
                function_name, id
            ),
            None => format!(
                "/runtime/webhooks/durabletask/orchestrators/{}",
                function_name
            ),
        };

        url.set_path(&path);

        url.query_pairs_mut()
            .clear()
            .append_pair("code", &self.code);

        url
    }

    /// Gets the "status query" URL.
    pub fn status_query_url(&self, instance_id: Option<&str>) -> Url {
        self.build_query_url(instance_id, None)
    }

    /// Gets the "purge history" URL.
    pub fn purge_history_url(&self, instance_id: Option<&str>) -> Url {
        self.build_query_url(instance_id, None)
    }

    /// Gets the "rewind history" URL.
    pub fn rewind_url(&self, instance_id: &str, reason: &str) -> Url {
        let mut url = self.build_query_url(Some(instance_id), Some("rewind"));
        url.query_pairs_mut().append_pair("reason", reason);
        url
    }

    /// Gets the "raise event" URL.
    pub fn raise_event_url(&self, instance_id: &str, event_name: &str) -> Url {
        self.build_query_url(
            Some(instance_id),
            Some(&format!("raiseEvent/{}", event_name)),
        )
    }

    /// Gets the "terminate instance" URL.
    pub fn terminate_url(&self, instance_id: &str, reason: &str) -> Url {
        let mut url = self.build_query_url(Some(instance_id), Some("terminate"));
        url.query_pairs_mut().append_pair("reason", reason);
        url
    }

    /// Gets the "signal entity" URL.
    pub fn signal_entity_url(&self, entity_type: &str, entity_key: &str, op: Option<&str>) -> Url {
        let mut url = self.base_uri.clone();

        url.set_path(&format!(
            "/runtime/webhooks/durabletask/entities/{}/{}",
            entity_type, entity_key
        ));

        url.query_pairs_mut()
            .clear()
            .append_pair("taskHub", &self.task_hub)
            .append_pair("connection", &self.connection)
            .append_pair("code", &self.code);

        if let Some(op) = op {
            url.query_pairs_mut().append_pair("op", op);
        }

        url
    }

    /// Gets the "query entity" URL.
    pub fn query_entity_url(&self, entity_type: &str, entity_key: &str) -> Url {
        self.signal_entity_url(entity_type, entity_key, None)
    }

    fn build_query_url(&self, instance_id: Option<&str>, action: Option<&str>) -> Url {
        let mut url = self.base_uri.clone();
        let mut path = "/runtime/webhooks/durabletask/instances".to_string();

        if let Some(id) = instance_id {
            write!(&mut path, "/{}", id).unwrap();
        }

        if let Some(action) = action {
            write!(&mut path, "/{}", action).unwrap();
        }

        url.set_path(&path);

        url.query_pairs_mut()
            .clear()
            .append_pair("taskHub", &self.task_hub)
            .append_pair("connection", &self.connection)
            .append_pair("code", &self.code);

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_parsing() {
        let endpoint = Endpoint::new(Url::parse("http://localhost:7071/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=myHub&connection=Storage&code=myCode").unwrap());
        assert_eq!(endpoint.code, "myCode");

        let rewind_result = "http://localhost:7071/runtime/webhooks/durabletask/instances/1234/rewind?taskHub=myHub&connection=Storage&code=myCode&reason=myReason";
        let rewind_url = endpoint.rewind_url("1234", "myReason");
        assert_eq!(rewind_url.to_string(), rewind_result);
    }

    #[test]
    #[should_panic]
    fn test_bad_endpoint() {
        Endpoint::new(
            Url::parse("http://localhost:7071/runtime/webhooks/durabletask/instances/INSTANC")
                .unwrap(),
        );
    }
}
