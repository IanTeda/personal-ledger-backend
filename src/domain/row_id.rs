//! # RowID Domain Type
//!
//! This module defines `RowID`, a domain type for unique row identifiers based on UUID v7.
//! RowIDs are sortable by creation time and provide type-safe identifier handling
//! throughout the Personal Ledger backend.
//!
//! ## Features
//!
//! - **Time-ordered**: Uses UUID v7 for chronological ordering
//! - **Type-safe**: Prevents mixing different ID types
//! - **Serializable**: Supports JSON serialization/deserialization
//! - **Sortable**: Built-in sorting and comparison operations
//! - **Mock support**: Test utilities for generating predictable IDs

/// A unique row identifier based on UUID v7.
///
/// `RowID` provides a type-safe wrapper around UUID v7, ensuring that row identifiers
/// are sortable by creation time and cannot be accidentally mixed with other types.
/// All RowIDs created with `new()` will be naturally ordered by creation time.
///
/// # Examples
///
/// ```rust
/// use personal_ledger_backend::domain::RowID;
///
/// // Create a new RowID
/// let id = RowID::new();
///
/// // Parse from string
/// let id_from_str = RowID::from_string("550e8400-e29b-41d4-a716-446655440000")?;
///
/// // Convert to UUID
/// let uuid = id.into_uuid();
/// ```
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct RowID(uuid::Uuid);

impl TryFrom<uuid::Uuid> for RowID {
    type Error = RowIDError;

    /// Create a RowID from an existing UUID.
    ///
    /// # Errors
    ///
    /// Returns `RowIDError::InvalidVersion` if the UUID is not version 7.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let uuid = uuid::Uuid::now_v7();
    /// let row_id = RowID::try_from(uuid)?;
    /// # Ok::<(), personal_ledger_backend::domain::RowIDError>(())
    /// ```
    fn try_from(uuid: uuid::Uuid) -> Result<Self, Self::Error> {
        if uuid.get_version_num() != 7 {
            return Err(RowIDError::InvalidVersion(uuid.get_version_num() as u8));
        }
        Ok(RowID(uuid))
    }
}

impl std::str::FromStr for RowID {
    type Err = RowIDError;
    
    /// Parse a RowID from a UUID string.
    ///
    /// # Errors
    ///
    /// Returns `RowIDError::InvalidUuid` if the string is not a valid UUID format,
    /// or `RowIDError::InvalidVersion` if the UUID is not version 7.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id = RowID::from_str("01800000-0000-7000-8000-000000000000")?;
    /// # Ok::<(), personal_ledger_backend::domain::RowIDError>(())
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::parse_str(s).map_err(RowIDError::from)?;
        RowID::try_from(uuid)
    }
}

impl std::fmt::Display for RowID {
    /// Format the RowID as a standard UUID string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialOrd for RowID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RowID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}



impl RowID {
    /// Create a new RowID using UUID v7 with current timestamp.
    ///
    /// UUID v7 ensures that RowIDs created in sequence will be naturally
    /// ordered by creation time, making them suitable for chronological sorting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id1 = RowID::new();
    /// let id2 = RowID::new();
    /// assert!(id1 < id2); // id1 was created before id2
    /// ```
    pub fn new() -> Self {
        let row_id = uuid::Uuid::now_v7();
        Self(row_id)
    }

    /// Convert the RowID into its underlying UUID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id = RowID::new();
    /// let uuid = id.into_uuid();
    /// ```
    pub fn into_uuid(self) -> uuid::Uuid {
        self.0
    }

    /// Get a reference to the underlying UUID without consuming the RowID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id = RowID::new();
    /// let uuid_ref = id.as_uuid();
    /// ```
    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }

    /// Sort a mutable slice of RowIDs in ascending (chronological) order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let mut ids = vec![id3, id1, id2];
    /// RowID::sort_ascending(&mut ids);
    /// // ids is now [id1, id2, id3] (chronological order)
    /// ```
    pub fn sort_ascending(ids: &mut [RowID]) {
        ids.sort();
    }

    /// Sort a mutable slice of RowIDs in descending (reverse chronological) order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let mut ids = vec![id1, id3, id2];
    /// RowID::sort_descending(&mut ids);
    /// // ids is now [id3, id2, id1] (newest first)
    /// ```
    pub fn sort_descending(ids: &mut [RowID]) {
        ids.sort_by(|a, b| b.cmp(a));
    }

    /// Create a new Vec with RowIDs sorted in ascending (chronological) order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let ids = vec![id3, id1, id2];
    /// let sorted = RowID::sorted_ascending(ids);
    /// // sorted is [id1, id2, id3]
    /// ```
    pub fn sorted_ascending<I>(ids: I) -> Vec<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        let mut result: Vec<RowID> = ids.into_iter().collect();
        result.sort();
        result
    }

    /// Create a new Vec with RowIDs sorted in descending (reverse chronological) order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let ids = vec![id1, id3, id2];
    /// let sorted = RowID::sorted_descending(ids);
    /// // sorted is [id3, id2, id1] (newest first)
    /// ```
    pub fn sorted_descending<I>(ids: I) -> Vec<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        let mut result: Vec<RowID> = ids.into_iter().collect();
        result.sort_by(|a, b| b.cmp(a));
        result
    }

    /// Find the earliest (minimum) RowID from a collection.
    ///
    /// Returns `None` if the collection is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let ids = vec![id2, id1, id3];
    /// let earliest = RowID::min(ids).unwrap();
    /// assert_eq!(earliest, id1);
    /// ```
    pub fn min<I>(ids: I) -> Option<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        ids.into_iter().min()
    }

    /// Find the latest (maximum) RowID from a collection.
    ///
    /// Returns `None` if the collection is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let ids = vec![id2, id1, id3];
    /// let latest = RowID::max(ids).unwrap();
    /// assert_eq!(latest, id3);
    /// ```
    pub fn max<I>(ids: I) -> Option<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        ids.into_iter().max()
    }

    /// Check if this RowID was created before another RowID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id1 = RowID::new();
    /// // ... some time passes ...
    /// let id2 = RowID::new();
    ///
    /// assert!(id1.is_before(&id2));
    /// ```
    pub fn is_before(&self, other: &RowID) -> bool {
        self < other
    }

    /// Check if this RowID was created after another RowID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id1 = RowID::new();
    /// // ... some time passes ...
    /// let id2 = RowID::new();
    ///
    /// assert!(id2.is_after(&id1));
    /// ```
    pub fn is_after(&self, other: &RowID) -> bool {
        self > other
    }


    /// Create a mock RowID with a random timestamp for testing.
    ///
    /// This generates a RowID with a random creation time, useful for testing
    /// scenarios where you need predictable but varied identifiers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let mock_id = RowID::mock();
    /// assert_eq!(mock_id.as_uuid().get_version_num(), 7);
    /// ```
    #[cfg(test)]
    pub fn mock() -> Self {
        use chrono::{DateTime, Utc};
        use fake::faker::chrono::en::DateTimeAfter;
        use fake::Fake;

        // Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
        let random_datetime: DateTime<Utc> =
            DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

        // Convert datetime to a UUID timestamp
        let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
            uuid::NoContext,
            random_datetime.timestamp() as u64,
            random_datetime.timestamp_nanos_opt().unwrap() as u32,
        );

        // Generate Uuid V7
        let row_id = uuid::Uuid::new_v7(random_uuid_timestamp);

        Self(row_id)
    }

    /// Create a mock RowID with a specific timestamp for testing.
    ///
    /// This allows creating RowIDs with deterministic timestamps, useful for
    /// testing chronological ordering and time-based logic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::{DateTime, Utc};
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let timestamp = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
    /// let id = RowID::mock_from_datetime(timestamp);
    /// ```
    #[cfg(test)]
    pub fn mock_from_datetime(date_time: chrono::DateTime<chrono::Utc>) -> Self {
        // Convert datetime to a UUID timestamp
        let uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
            uuid::NoContext,
            date_time.timestamp() as u64,
            date_time.timestamp_nanos_opt().unwrap() as u32,
        );

        // Generate Uuid V7
        let row_id = uuid::Uuid::new_v7(uuid_timestamp);

        Self(row_id)
    }

    /// Create a RowID from an existing UUID.
    ///
    /// This is equivalent to the `From<uuid::Uuid>` implementation but provides
    /// a more explicit method name for clarity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uuid::Uuid;
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let uuid = Uuid::now_v7();
    /// let id = RowID::from_uuid(uuid);
    /// ```
    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        RowID(uuid)
    }

    /// Parse a RowID from a string representation of a UUID.
    ///
    /// This is a convenience method that wraps `str::parse()`.
    ///
    /// # Errors
    ///
    /// Returns `RowIDError::InvalidUuid` if the string is not a valid UUID format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id = RowID::from_string("550e8400-e29b-41d4-a716-446655440000")?;
    /// ```
    pub fn from_string(s: &str) -> Result<Self, RowIDError> {
        s.parse()
    }

    /// Convert an i64 to a RowID by embedding it in a UUID structure.
    ///
    /// This method places the i64 value in the last 8 bytes of a UUID,
    /// with the first 8 bytes being zero. This is useful for migrating
    /// from integer-based ID systems.
    ///
    /// # Errors
    ///
    /// Returns `RowIDError::TypeCast` if the provided i64 is negative.
    ///
    /// # Note
    ///
    /// The resulting UUID will not be a valid v7 UUID and will not have
    /// proper timestamp ordering properties.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let id = RowID::from_i64(12345)?;
    /// ```
    pub fn from_i64(id: i64) -> Result<Self, RowIDError> {
        if id < 0 {
            return Err(RowIDError::TypeCast("Negative IDs not allowed".to_string()));
        }
        // Convert i64 to UUID by treating it as a simple identifier
        // This is a simplified example - in practice you might want more sophisticated mapping
        let bytes = id.to_be_bytes();
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes[8..16].copy_from_slice(&bytes);
        let uuid = uuid::Uuid::from_bytes(uuid_bytes);
        Ok(RowID(uuid))
    }

    /// Extract an i64 value from a RowID that was created with `from_i64`.
    ///
    /// This method extracts the i64 value from the last 8 bytes of the UUID.
    /// Only use this with RowIDs that were originally created from i64 values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// let original_id = 12345i64;
    /// let row_id = RowID::from_i64(original_id)?;
    /// let extracted_id = row_id.to_i64();
    /// assert_eq!(original_id, extracted_id);
    /// ```
    pub fn to_i64(&self) -> i64 {
        let uuid_bytes = self.0.as_bytes();
        i64::from_be_bytes(uuid_bytes[8..16].try_into().unwrap())
    }
}

/// Errors that can occur during RowID operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RowIDError {
    /// The provided string is not a valid UUID format.
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(String),
    
    /// The provided UUID is not version 7.
    #[error("UUID is not version 7: version {0}")]
    InvalidVersion(u8),

    /// Type casting operation failed.
    #[error("Type casting failed: {0}")]
    TypeCast(String),
}

impl From<uuid::Error> for RowIDError {
    /// Convert a UUID parsing error into a RowIDError.
    fn from(err: uuid::Error) -> Self {
        RowIDError::InvalidUuid(err.to_string())
    }
}

// SQLx database integration
impl sqlx::Type<sqlx::Any> for RowID {
    fn type_info() -> <sqlx::Any as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Any>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Any> for RowID {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Any as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.0.to_string();
        <String as sqlx::Encode<sqlx::Any>>::encode_by_ref(&s, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Any> for RowID {
    fn decode(
        value: <sqlx::Any as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Any>>::decode(value)?;
        let uuid = uuid::Uuid::parse_str(&s)
            .map_err(|e| format!("Invalid UUID in database: {}", e))?;
        Ok(Self(uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{str::FromStr, thread, time::Duration};

    #[test]
    fn test_rowid_new_creates_uuid_v7() {
        let row_id = RowID::new();
        // UUID v7 has version 7
        assert_eq!(row_id.0.get_version_num(), 7);
    }

    #[test]
    fn test_rowid_mock_creates_uuid_v7() {
        let row_id = RowID::mock();
        assert_eq!(row_id.0.get_version_num(), 7);
    }

    #[test]
    fn test_rowid_equality() {
        let row_id1 = RowID::new();
        let row_id2 = RowID(row_id1.0);
        assert_eq!(row_id1, row_id2);
    }

    #[test]
    fn test_rowid_serialization() {
        let row_id = RowID::new();
        let serialized = serde_json::to_string(&row_id).unwrap();
        let deserialized: RowID = serde_json::from_str(&serialized).unwrap();
        assert_eq!(row_id, deserialized);
    }

    #[test]
    fn test_from_uuid_for_rowid() {
        let uuid = uuid::Uuid::now_v7();
        let row_id = RowID::try_from(uuid).unwrap();
        assert_eq!(row_id.0, uuid);
    }

    #[test]
    fn test_from_str_for_rowid_valid() {
        let uuid = uuid::Uuid::now_v7();
        let uuid_str = uuid.to_string();
        let row_id = RowID::from_str(&uuid_str).unwrap();
        assert_eq!(row_id.0, uuid);
    }

    #[test]
    fn test_from_str_for_rowid_invalid() {
        let invalid_str = "not-a-uuid";
        let result = RowID::from_str(invalid_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, RowIDError::InvalidUuid(_)));
    }

    #[test]
    fn test_rowid_ordering() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
    }

    #[test]
    fn test_sort_ascending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let mut ids = vec![id3, id1, id2];
        RowID::sort_ascending(&mut ids);

        assert_eq!(ids, vec![id1, id2, id3]);
    }

    #[test]
    fn test_sort_descending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let mut ids = vec![id1, id3, id2];
        RowID::sort_descending(&mut ids);

        assert_eq!(ids, vec![id3, id2, id1]);
    }

    #[test]
    fn test_sorted_ascending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let sorted = RowID::sorted_ascending([id3, id1, id2]);

        assert_eq!(sorted, vec![id1, id2, id3]);
    }

    #[test]
    fn test_sorted_descending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let ids = vec![id1, id3, id2];
        let sorted = RowID::sorted_descending(ids);

        assert_eq!(sorted, vec![id3, id2, id1]);
    }

    #[test]
    fn test_min_max() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let ids = [id2, id1, id3];

        assert_eq!(RowID::min(ids.iter().cloned()).unwrap(), id1);
        assert_eq!(RowID::max(ids.iter().cloned()).unwrap(), id3);
    }

    #[test]
    fn test_min_max_empty() {
        let ids: Vec<RowID> = vec![];
        assert!(RowID::min(ids.iter().cloned()).is_none());
        assert!(RowID::max(ids.iter().cloned()).is_none());
    }

    #[test]
    fn test_is_before_after() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();

        assert!(id1.is_before(&id2));
        assert!(!id2.is_before(&id1));
        assert!(id2.is_after(&id1));
        assert!(!id1.is_after(&id2));
    }

    #[test]
    fn test_equality() {
        let id = RowID::new();
        let same_id = RowID(id.0);

        assert!(!(id.is_before(&same_id)));
        assert!(!(id.is_after(&same_id)));
        assert_eq!(id, same_id);
    }

    #[test]
    fn test_mock_ids_are_sortable() {
        let mut mock_ids = vec![RowID::mock(), RowID::mock(), RowID::mock()];
        
        // Should not panic when sorting mock IDs
        RowID::sort_ascending(&mut mock_ids);
        
        // All should be different (very high probability with UUIDs)
        assert_ne!(mock_ids[0], mock_ids[1]);
        assert_ne!(mock_ids[1], mock_ids[2]);
        assert_ne!(mock_ids[0], mock_ids[2]);
    }

    #[test]
    fn test_large_collection_sorting() {
        let mut ids: Vec<RowID> = (0..1000).map(|_| RowID::new()).collect();
        let original_ids = ids.clone();
        
        RowID::sort_descending(&mut ids);
        
        // Should be in reverse chronological order
        for i in 0..ids.len()-1 {
            assert!(ids[i] >= ids[i+1]);
        }
        
        // All original IDs should still be present
        let mut sorted_original = original_ids;
        sorted_original.sort();
        ids.sort();
        assert_eq!(ids, sorted_original);
    }

    #[test]
    fn test_from_string() {
        let uuid = uuid::Uuid::now_v7();
        let uuid_str = uuid.to_string();
        let row_id = RowID::from_string(&uuid_str).unwrap();
        assert_eq!(row_id.to_string(), uuid_str);

        let invalid_str = "not-a-uuid";
        let result = RowID::from_string(invalid_str);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RowIDError::InvalidUuid(_)));
    }

    #[test]
    fn test_from_i64() {
        let id = 12345i64;
        let row_id = RowID::from_i64(id).unwrap();
        // The UUID should contain the i64 in the last 8 bytes
        let uuid_bytes = row_id.0.as_bytes();
        let extracted_id = i64::from_be_bytes(uuid_bytes[8..16].try_into().unwrap());
        assert_eq!(extracted_id, id);

        let negative_id = -1i64;
        let result = RowID::from_i64(negative_id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RowIDError::TypeCast(_)));
    }

    #[test]
    fn test_to_i64() {
        let original_id = 98765i64;
        let row_id = RowID::from_i64(original_id).unwrap();
        let extracted_id = row_id.to_i64();
        assert_eq!(original_id, extracted_id);

        // Test with zero
        let zero_id = 0i64;
        let row_id = RowID::from_i64(zero_id).unwrap();
        assert_eq!(row_id.to_i64(), zero_id);

        // Test with maximum value
        let max_id = i64::MAX;
        let row_id = RowID::from_i64(max_id).unwrap();
        assert_eq!(row_id.to_i64(), max_id);
    }

    #[test]
    fn test_as_uuid() {
        let row_id = RowID::new();
        let uuid_ref = row_id.as_uuid();
        assert_eq!(*uuid_ref, row_id.0);
        assert_eq!(uuid_ref.get_version_num(), 7);
    }

    #[test]
    fn test_from_uuid_function() {
        let uuid = uuid::Uuid::now_v7();
        let row_id1 = RowID::from_uuid(uuid);
        let row_id2 = RowID::try_from(uuid).unwrap();
        assert_eq!(row_id1, row_id2);
    }

    #[test]
    fn test_mock_from_datetime() {
        use chrono::{DateTime, Utc};
        
        let timestamp1 = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let timestamp2 = DateTime::parse_from_rfc3339("2023-01-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let id1 = RowID::mock_from_datetime(timestamp1);
        let id2 = RowID::mock_from_datetime(timestamp2);

        assert!(id1 < id2);
        assert_eq!(id1.as_uuid().get_version_num(), 7);
        assert_eq!(id2.as_uuid().get_version_num(), 7);

        // UUIDs with same timestamp should have same timestamp portion but may differ in random bits
        let id3 = RowID::mock_from_datetime(timestamp1);
        // They should be roughly equal in time (both have same timestamp base)
        // but the random portions may differ, so we just verify ordering consistency
        assert!(id3 < id2); // id3 and id1 have earlier timestamp than id2
    }

    #[test]
    fn test_error_display() {
        let uuid_error = RowIDError::InvalidUuid("bad format".to_string());
        assert_eq!(format!("{}", uuid_error), "Invalid UUID format: bad format");
    }

    #[test]
    fn test_error_debug() {
        let error = RowIDError::InvalidUuid("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidUuid"));
        assert!(debug_str.contains("test"));
    }
}
