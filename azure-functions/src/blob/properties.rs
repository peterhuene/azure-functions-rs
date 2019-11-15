use chrono::{DateTime, Utc};
use serde::{de::Error, Deserialize, Deserializer};

/// Represents the type of blob.
#[derive(Debug)]
pub enum BlobType {
    /// The blob type is not specified.
    Unspecified,
    /// A page blob.
    PageBlob,
    /// A block blob.
    BlockBlob,
    /// An append blob.
    AppendBlob,
}

impl Default for BlobType {
    fn default() -> Self {
        Self::Unspecified
    }
}

/// Represents the duration of the blob's lease.
#[derive(Debug)]
pub enum LeaseDuration {
    /// The lease duration is not specified.
    Unspecified,
    /// The lease duration is finite.
    Fixed,
    /// The lease duration is infinite.
    Infinite,
}

impl Default for LeaseDuration {
    fn default() -> Self {
        Self::Unspecified
    }
}

/// Represents the lease state of the blob.
#[derive(Debug)]
pub enum LeaseState {
    /// Not specified.
    Unspecified,
    /// The lease is in the Available state.
    Available,
    /// The lease is in the Leased state.
    Leased,
    /// The lease is in the Expired state.
    Expired,
    /// The lease is in the Breaking state.
    Breaking,
    /// The lease is in the Broken state.
    Broken,
}

impl Default for LeaseState {
    fn default() -> Self {
        Self::Unspecified
    }
}

/// Represents the status of the blob's lease.
#[derive(Debug)]
pub enum LeaseStatus {
    /// The lease status is not specified.
    Unspecified,
    /// The resource is locked.
    Locked,
    /// The resource is unlocked.
    Unlocked,
}

impl Default for LeaseStatus {
    fn default() -> Self {
        Self::Unspecified
    }
}

/// Represents the tier of the page blob.
#[derive(Debug)]
pub enum PremiumPageBlobTier {
    /// The tier is not recognized.
    Unknown,
    /// P4 tier.
    P4,
    /// P6 tier.
    P6,
    /// P10 tier.
    P10,
    /// P20 tier.
    P20,
    /// P30 tier.
    P30,
    /// P40 tier.
    P40,
    /// P50 tier.
    P50,
    /// P60 tier.
    P60,
}

impl Default for PremiumPageBlobTier {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Represents the rehydration status for a blob that is currently archived.
#[derive(Debug)]
pub enum RehydrationStatus {
    /// The rehydration status is unknown.
    Unknown,
    /// The blob is being rehydrated to hot storage.
    PendingToHot,
    /// The blob is being rehydrated to cool storage.
    PendingToCool,
}

impl Default for RehydrationStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Represents the standard tier of the block blob.
#[derive(Debug)]
pub enum StandardBlobTier {
    /// The tier is not recognized
    Unknown,
    /// Hot storage tier.
    Hot,
    /// Cool storage tier.
    Cool,
    /// Archive storage tier.
    Archive,
}

impl Default for StandardBlobTier {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Represents the properties of an Azure Storage blob.
#[derive(Default, Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Properties {
    /// The number of committed blocks, if the blob is an append blob.
    pub append_blob_committed_block_count: Option<i32>,
    /// The value indicating if the tier of the blob has been inferred.
    pub blob_tier_inferred: Option<bool>,
    /// The time for when the tier of the blob was last-modified.
    pub blob_tier_last_modified_time: Option<DateTime<Utc>>,
    /// The type of the blob.
    #[serde(deserialize_with = "deserialize_blob_type")]
    pub blob_type: BlobType,
    /// The cache-control value stored for the blob.
    pub cache_control: Option<String>,
    /// The content-disposition value stored for the blob.
    pub content_disposition: Option<String>,
    /// The content-encoding value stored for the blob.
    pub content_encoding: Option<String>,
    /// The content-language value stored for the blob.
    pub content_language: Option<String>,
    /// The content-MD5 value stored for the blob.
    #[serde(rename = "ContentMD5")]
    pub content_md5: Option<String>,
    /// The content-type value stored for the blob.
    pub content_type: Option<String>,
    /// The creation time for the blob
    pub created: Option<DateTime<Utc>>,
    /// The deletion time for the blob, if it was deleted.
    pub deleted_time: Option<DateTime<Utc>>,
    /// The blob's ETag value.
    #[serde(rename = "ETag")]
    pub etag: Option<String>,
    /// The value indicating whether or not this blob is an incremental copy.
    pub is_incremental_copy: bool,
    /// The blob's server-side encryption state.
    pub is_server_encrypted: bool,
    /// The last-modified time for the blob.
    pub last_modified: Option<DateTime<Utc>>,
    /// The blob's lease duration.
    #[serde(deserialize_with = "deserialize_lease_duration")]
    pub lease_duration: LeaseDuration,
    /// The blob's lease state.
    #[serde(deserialize_with = "deserialize_lease_state")]
    pub lease_state: LeaseState,
    /// The blob's lease status.
    #[serde(deserialize_with = "deserialize_lease_status")]
    pub lease_status: LeaseStatus,
    /// The size of the blob, in bytes.
    pub length: i64,
    /// The blob's current sequence number, if the blob is a page blob.
    pub page_blob_sequence_number: Option<i64>,
    /// The value indicating the tier of the premium page blob, if the blob is a page blob.
    #[serde(deserialize_with = "deserialize_page_blob_tier")]
    pub premium_page_blob_tier: Option<PremiumPageBlobTier>,
    #[serde(deserialize_with = "deserialize_rehydration_status")]
    /// The value indicating that the blob is being rehdrated and the tier of the blob once the rehydration from archive has completed.
    pub rehydration_status: Option<RehydrationStatus>,
    /// The number of remaining days before the blob is permenantly deleted, if the blob is soft-deleted.
    pub remaining_days_before_permanent_delete: Option<i32>,
    /// The value indicating the tier of the block blob.
    #[serde(deserialize_with = "deserialize_standard_blob_tier")]
    pub standard_blob_tier: Option<StandardBlobTier>,
}

fn deserialize_blob_type<'a, D>(deserializer: D) -> Result<BlobType, D::Error>
where
    D: Deserializer<'a>,
{
    match u32::deserialize(deserializer)? {
        0 => Ok(BlobType::Unspecified),
        1 => Ok(BlobType::PageBlob),
        2 => Ok(BlobType::BlockBlob),
        3 => Ok(BlobType::AppendBlob),
        _ => Err(Error::custom("unexpected blob type")),
    }
}

fn deserialize_lease_duration<'a, D>(deserializer: D) -> Result<LeaseDuration, D::Error>
where
    D: Deserializer<'a>,
{
    match u32::deserialize(deserializer)? {
        0 => Ok(LeaseDuration::Unspecified),
        1 => Ok(LeaseDuration::Fixed),
        2 => Ok(LeaseDuration::Infinite),
        _ => Err(Error::custom("unexpected lease duration")),
    }
}

fn deserialize_lease_state<'a, D>(deserializer: D) -> Result<LeaseState, D::Error>
where
    D: Deserializer<'a>,
{
    match u32::deserialize(deserializer)? {
        0 => Ok(LeaseState::Unspecified),
        1 => Ok(LeaseState::Available),
        2 => Ok(LeaseState::Leased),
        3 => Ok(LeaseState::Expired),
        4 => Ok(LeaseState::Breaking),
        5 => Ok(LeaseState::Broken),
        _ => Err(Error::custom("unexpected lease state")),
    }
}

fn deserialize_lease_status<'a, D>(deserializer: D) -> Result<LeaseStatus, D::Error>
where
    D: Deserializer<'a>,
{
    match u32::deserialize(deserializer)? {
        0 => Ok(LeaseStatus::Unspecified),
        1 => Ok(LeaseStatus::Locked),
        2 => Ok(LeaseStatus::Unlocked),
        _ => Err(Error::custom("unexpected lease status")),
    }
}

fn deserialize_page_blob_tier<'a, D>(
    deserializer: D,
) -> Result<Option<PremiumPageBlobTier>, D::Error>
where
    D: Deserializer<'a>,
{
    match Option::<u32>::deserialize(deserializer)? {
        Some(x) => match x {
            0 => Ok(Some(PremiumPageBlobTier::Unknown)),
            1 => Ok(Some(PremiumPageBlobTier::P4)),
            2 => Ok(Some(PremiumPageBlobTier::P6)),
            3 => Ok(Some(PremiumPageBlobTier::P10)),
            4 => Ok(Some(PremiumPageBlobTier::P20)),
            5 => Ok(Some(PremiumPageBlobTier::P30)),
            6 => Ok(Some(PremiumPageBlobTier::P40)),
            7 => Ok(Some(PremiumPageBlobTier::P50)),
            8 => Ok(Some(PremiumPageBlobTier::P60)),
            _ => Err(Error::custom("unexpected page blob tier")),
        },
        None => Ok(None),
    }
}

fn deserialize_rehydration_status<'a, D>(
    deserializer: D,
) -> Result<Option<RehydrationStatus>, D::Error>
where
    D: Deserializer<'a>,
{
    match Option::<u32>::deserialize(deserializer)? {
        Some(x) => match x {
            0 => Ok(Some(RehydrationStatus::Unknown)),
            1 => Ok(Some(RehydrationStatus::PendingToHot)),
            2 => Ok(Some(RehydrationStatus::PendingToCool)),
            _ => Err(Error::custom("unexpected rehydration status")),
        },
        None => Ok(None),
    }
}

fn deserialize_standard_blob_tier<'a, D>(
    deserializer: D,
) -> Result<Option<StandardBlobTier>, D::Error>
where
    D: Deserializer<'a>,
{
    match Option::<u32>::deserialize(deserializer)? {
        Some(x) => match x {
            0 => Ok(Some(StandardBlobTier::Unknown)),
            1 => Ok(Some(StandardBlobTier::Hot)),
            2 => Ok(Some(StandardBlobTier::Cool)),
            3 => Ok(Some(StandardBlobTier::Archive)),
            _ => Err(Error::custom("unexpected blob tier")),
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::matches;
    use serde_json::{from_value, json};

    #[test]
    fn it_deserializes_from_json() {
        const CACHE_CONTROL: &'static str = "test-cache-control";
        const CONTENT_DISPOSITION: &'static str = "test-content-disposition";
        const CONTENT_ENCODING: &'static str = "test-content-encoding";
        const CONTENT_LANGUAGE: &'static str = "test-content-language";
        const CONTENT_LENGTH: u32 = 12345;
        const CONTENT_MD5: &'static str = "test-md5-hash";
        const CONTENT_TYPE: &'static str = "test-content-type";
        const ETAG: &'static str = "test-etag";
        const IS_SERVER_ENCRYPTED: bool = false;
        const IS_INCREMENTAL_COPY: bool = true;
        const BLOB_TIER_INFERRED: bool = true;
        const APPEND_BLOCK_COUNT: i32 = 101;
        const PAGE_BLOCK_SEQ_NUM: i64 = 54321;

        let now = Utc::now();

        let value = json!({
            "CacheControl": CACHE_CONTROL,
            "ContentDisposition": CONTENT_DISPOSITION,
            "ContentEncoding": CONTENT_ENCODING,
            "ContentLanguage": CONTENT_LANGUAGE,
            "Length": CONTENT_LENGTH,
            "ContentMD5": CONTENT_MD5,
            "ContentType": CONTENT_TYPE,
            "ETag": ETAG,
            "LastModified": now.to_rfc3339(),
            "BlobType": 1,
            "LeaseStatus": 1,
            "LeaseState": 0,
            "LeaseDuration": 2,
            "PageBlobSequenceNumber": PAGE_BLOCK_SEQ_NUM,
            "AppendBlobCommittedBlockCount": APPEND_BLOCK_COUNT,
            "IsServerEncrypted": IS_SERVER_ENCRYPTED,
            "IsIncrementalCopy": IS_INCREMENTAL_COPY,
            "StandardBlobTier": 1,
            "RehydrationStatus": 2,
            "PremiumPageBlobTier": 1,
            "BlobTierInferred": BLOB_TIER_INFERRED,
            "BlobTierLastModifiedTime": now.to_rfc3339()
        });

        let properties: Properties = from_value(value).unwrap();

        assert_eq!(
            *properties
                .append_blob_committed_block_count
                .as_ref()
                .unwrap(),
            APPEND_BLOCK_COUNT
        );
        assert_eq!(
            *properties.blob_tier_inferred.as_ref().unwrap(),
            BLOB_TIER_INFERRED
        );
        assert_eq!(
            properties
                .blob_tier_last_modified_time
                .as_ref()
                .unwrap()
                .to_rfc3339(),
            now.to_rfc3339()
        );
        assert!(matches!(properties.blob_type, BlobType::PageBlob));
        assert_eq!(properties.cache_control.as_ref().unwrap(), CACHE_CONTROL);
        assert_eq!(
            properties.content_disposition.as_ref().unwrap(),
            CONTENT_DISPOSITION
        );
        assert_eq!(
            properties.content_encoding.as_ref().unwrap(),
            CONTENT_ENCODING
        );
        assert_eq!(
            properties.content_language.as_ref().unwrap(),
            CONTENT_LANGUAGE
        );
        assert_eq!(properties.content_md5.as_ref().unwrap(), CONTENT_MD5);
        assert_eq!(properties.content_type.as_ref().unwrap(), CONTENT_TYPE);
        assert!(properties.created.is_none());
        assert!(properties.deleted_time.is_none());
        assert_eq!(properties.etag.as_ref().unwrap(), ETAG);
        assert_eq!(properties.is_incremental_copy, IS_INCREMENTAL_COPY);
        assert_eq!(properties.is_server_encrypted, IS_SERVER_ENCRYPTED);
        assert_eq!(
            properties.last_modified.as_ref().unwrap().to_rfc3339(),
            now.to_rfc3339()
        );
        assert!(matches!(properties.lease_duration, LeaseDuration::Infinite));
        assert!(matches!(properties.lease_state, LeaseState::Unspecified));
        assert!(matches!(properties.lease_status, LeaseStatus::Locked));
        assert_eq!(properties.length, CONTENT_LENGTH as i64);
        assert_eq!(
            *properties.page_blob_sequence_number.as_ref().unwrap(),
            PAGE_BLOCK_SEQ_NUM
        );
        assert!(matches!(
            properties.premium_page_blob_tier.as_ref().unwrap(),
            PremiumPageBlobTier::P4
        ));
        assert!(matches!(
            properties.rehydration_status.as_ref().unwrap(),
            RehydrationStatus::PendingToCool
        ));
        assert!(properties.remaining_days_before_permanent_delete.is_none());
        assert!(matches!(
            properties.standard_blob_tier.as_ref().unwrap(),
            StandardBlobTier::Hot
        ));
    }
}
