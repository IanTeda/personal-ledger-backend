
/// Input for creating a new category.
///
/// Contains all the fields needed to create a category, with sensible defaults
/// for optional fields and automatic generation of timestamps and ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCategory {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub category_type: domain::CategoryTypes,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}


impl NewCategory {
    /// Create a NewCategory from the minimal required fields.
    ///
    /// Validates the category type against allowed values.
    pub fn new(code: String, name: String, category_type: String) -> Result<Self, String> {
        // Validate and parse category type
        let parsed_category_type = domain::CategoryTypes::from_str(&category_type)
            .map_err(|e| format!("Invalid category type '{}': {}", category_type, e))?;

        Ok(Self {
            code,
            name,
            category_type: parsed_category_type,
            description: None,
            slug: None,
            color: None,
            icon: None,
            is_active: Some(true),
        })
    }

    /// Set optional description.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set optional slug.
    pub fn with_slug(mut self, slug: String) -> Self {
        self.slug = Some(slug);
        self
    }

    /// Set optional color (validates hex format).
    pub fn with_color(mut self, color: String) -> Result<Self, String> {
        if color.len() == 7 && color.starts_with('#') {
            self.color = Some(color);
            Ok(self)
        } else {
            Err("Color must be in hex format (#RRGGBB)".to_string())
        }
    }

    /// Set optional icon.
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set active status.
    pub fn with_active_status(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }
}