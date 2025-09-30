

/// Partial updates for an existing category.
///
/// Uses `Option<Option<T>>` for nullable fields to distinguish between
/// "don't change" (None) and "set to null" (Some(None)).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub description: Option<Option<String>>, // Some(Some(v)) -> set, Some(None) -> clear, None -> ignore
    pub slug: Option<Option<String>>,
    pub category_type: Option<String>,
    pub color: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub is_active: Option<bool>,
}
