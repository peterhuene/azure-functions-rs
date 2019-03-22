use crate::bindings::Blob;
use crate::blob::Properties;
use crate::rpc::protocol;
use crate::util::convert_from;
use serde_json::from_str;
use std::collections::HashMap;

const PATH_KEY: &str = "BlobTrigger";
const URI_KEY: &str = "Uri";
const PROPERTIES_KEY: &str = "Properties";
const METADATA_KEY: &str = "Metadata";

/// Represents an Azure Storage blob trigger binding.
///
/// # Examples
///
/// A function that runs when a blob is created in the `test` container:
///
/// ```rust
/// use azure_functions::bindings::BlobTrigger;
/// use azure_functions::func;
/// use log::info;
///
/// #[func]
/// #[binding(name = "trigger", path = "example/")]
/// pub fn print_blob(trigger: BlobTrigger) {
///     info!("Blob (as string): {}", trigger.blob.as_str().unwrap());
/// }
/// ```
#[derive(Debug)]
pub struct BlobTrigger {
    /// The blob that triggered the function.
    pub blob: Blob,
    /// The path of the blob.
    pub path: String,
    /// The URI of the blob.
    pub uri: String,
    /// The properties of the blob.
    pub properties: Properties,
    /// The metadata of the blob.
    pub metadata: HashMap<String, String>,
}

impl BlobTrigger {
    #[doc(hidden)]
    pub fn new(
        data: protocol::TypedData,
        metadata: &mut HashMap<String, protocol::TypedData>,
    ) -> Self {
        BlobTrigger {
            blob: data.into(),
            path: metadata
                .get_mut(PATH_KEY)
                .map_or(String::new(), protocol::TypedData::take_string),
            uri: metadata.get(URI_KEY).map_or(String::new(), |x| {
                convert_from(x)
                    .unwrap_or_else(|| panic!("failed to read '{}' from metadata", URI_KEY))
            }),
            properties: metadata
                .get(PROPERTIES_KEY)
                .map_or(Default::default(), |x| {
                    from_str(x.get_json()).expect("failed to deserialize blob properties")
                }),
            metadata: metadata.get(METADATA_KEY).map_or(Default::default(), |x| {
                from_str(x.get_json()).expect("failed to deserialize blob metadata")
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blob::*;
    use chrono::Utc;
    use matches::matches;
    use serde_json::{json, to_string};

    #[test]
    fn it_constructs() {
        const BLOB: &'static str = "blob";
        const PATH: &'static str = "foo/bar";
        const URI: &'static str = "https://example.com/blob";
        const CACHE_CONTROL: &'static str = "cache-control";
        const CONTENT_DISPOSITION: &'static str = "content-disposition";
        const CONTENT_ENCODING: &'static str = "content-encoding";
        const CONTENT_LANGUAGE: &'static str = "content-language";
        const CONTENT_LENGTH: u32 = 1234;
        const CONTENT_MD5: &'static str = "abcdef";
        const CONTENT_TYPE: &'static str = "text/plain";
        const ETAG: &'static str = "12345";
        const IS_SERVER_ENCRYPTED: bool = true;
        const IS_INCREMENTAL_COPY: bool = false;
        const BLOB_TIER_INFERRED: bool = false;
        const USER_METADAT_KEY: &'static str = "key";
        const USER_METADATA_VALUE: &'static str = "value";

        let now = Utc::now();

        let properties = json!({
            "CacheControl": CACHE_CONTROL,
            "ContentDisposition": CONTENT_DISPOSITION,
            "ContentEncoding": CONTENT_ENCODING,
            "ContentLanguage": CONTENT_LANGUAGE,
            "Length": CONTENT_LENGTH,
            "ContentMD5": CONTENT_MD5,
            "ContentType": CONTENT_TYPE,
            "ETag": ETAG,
            "LastModified": now.to_rfc3339(),
            "BlobType": 2,
            "LeaseStatus": 2,
            "LeaseState": 1,
            "LeaseDuration": 0,
            "PageBlobSequenceNumber": null,
            "AppendBlobCommittedBlockCount": null,
            "IsServerEncrypted": IS_SERVER_ENCRYPTED,
            "IsIncrementalCopy": IS_INCREMENTAL_COPY,
            "StandardBlobTier": 0,
            "RehydrationStatus": null,
            "PremiumPageBlobTier": null,
            "BlobTierInferred": BLOB_TIER_INFERRED,
            "BlobTierLastModifiedTime": null
        });

        let mut data = protocol::TypedData::new();
        data.set_string(BLOB.to_string());

        let mut metadata = HashMap::new();

        let mut value = protocol::TypedData::new();
        value.set_string(PATH.to_string());
        metadata.insert(PATH_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_json("\"".to_string() + URI + "\"");
        metadata.insert(URI_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_json(properties.to_string());
        metadata.insert(PROPERTIES_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        let mut user_metadata = HashMap::new();
        user_metadata.insert(
            USER_METADAT_KEY.to_string(),
            USER_METADATA_VALUE.to_string(),
        );
        value.set_json(to_string(&user_metadata).unwrap());
        metadata.insert(METADATA_KEY.to_string(), value);

        let trigger = BlobTrigger::new(data, &mut metadata);
        assert_eq!(trigger.path, PATH);
        assert_eq!(trigger.uri, URI);

        assert!(trigger
            .properties
            .append_blob_committed_block_count
            .is_none());
        assert_eq!(
            *trigger.properties.blob_tier_inferred.as_ref().unwrap(),
            BLOB_TIER_INFERRED
        );
        assert!(trigger.properties.blob_tier_last_modified_time.is_none());
        assert!(matches!(trigger.properties.blob_type, BlobType::BlockBlob));
        assert_eq!(
            trigger.properties.cache_control.as_ref().unwrap(),
            CACHE_CONTROL
        );
        assert_eq!(
            trigger.properties.content_disposition.as_ref().unwrap(),
            CONTENT_DISPOSITION
        );
        assert_eq!(
            trigger.properties.content_encoding.as_ref().unwrap(),
            CONTENT_ENCODING
        );
        assert_eq!(
            trigger.properties.content_language.as_ref().unwrap(),
            CONTENT_LANGUAGE
        );
        assert_eq!(
            trigger.properties.content_md5.as_ref().unwrap(),
            CONTENT_MD5
        );
        assert_eq!(
            trigger.properties.content_type.as_ref().unwrap(),
            CONTENT_TYPE
        );
        assert!(trigger.properties.created.is_none());
        assert!(trigger.properties.deleted_time.is_none());
        assert_eq!(trigger.properties.etag.as_ref().unwrap(), ETAG);
        assert_eq!(trigger.properties.is_incremental_copy, IS_INCREMENTAL_COPY);
        assert_eq!(trigger.properties.is_server_encrypted, IS_SERVER_ENCRYPTED);
        assert_eq!(
            trigger
                .properties
                .last_modified
                .as_ref()
                .unwrap()
                .to_rfc3339(),
            now.to_rfc3339()
        );
        assert!(matches!(
            trigger.properties.lease_duration,
            LeaseDuration::Unspecified
        ));
        assert!(matches!(
            trigger.properties.lease_state,
            LeaseState::Available
        ));
        assert!(matches!(
            trigger.properties.lease_status,
            LeaseStatus::Unlocked
        ));
        assert_eq!(trigger.properties.length, CONTENT_LENGTH as i64);
        assert!(trigger.properties.page_blob_sequence_number.is_none());
        assert!(trigger.properties.premium_page_blob_tier.is_none());
        assert!(trigger.properties.rehydration_status.is_none());
        assert!(trigger
            .properties
            .remaining_days_before_permanent_delete
            .is_none());
        assert!(matches!(
            trigger.properties.standard_blob_tier.as_ref().unwrap(),
            StandardBlobTier::Unknown
        ));

        assert_eq!(trigger.metadata.len(), 1);
        assert_eq!(
            trigger.metadata.get(USER_METADAT_KEY).unwrap(),
            USER_METADATA_VALUE
        );
    }
}
