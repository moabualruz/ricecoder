//! ARIA-like properties and roles for accessibility
//!
//! Provides semantic roles, live region properties, and screen reader descriptions
//! based on WAI-ARIA specifications for improved accessibility.

/// ARIA live property values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AriaLive {
    Off,       // No announcements
    Polite,    // Announce when user is idle
    Assertive, // Announce immediately
}

/// ARIA relevant property values
#[derive(Debug, Clone, PartialEq)]
pub enum AriaRelevant {
    Additions, // Only additions
    Removals,  // Only removals
    Text,      // Text changes
    All,       // All changes
}

/// ARIA-like semantic roles for UI elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AriaRole {
    Banner,
    Navigation,
    Main,
    Complementary,
    ContentInfo,
    Search,
    Form,
    Button,
    Link,
    Heading,
    List,
    ListItem,
    Status,
    Alert,
    Dialog,
    Menu,
    MenuItem,
    Tab,
    TabPanel,
    ProgressBar,
    TextBox,
    CheckBox,
    Radio,
    Slider,
    SpinButton,
}

/// ARIA-like properties for accessibility
#[derive(Debug, Clone)]
pub struct AriaProperties {
    pub role: Option<AriaRole>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub expanded: Option<bool>,
    pub selected: Option<bool>,
    pub checked: Option<bool>,
    pub disabled: Option<bool>,
    pub required: Option<bool>,
    pub invalid: Option<bool>,
    pub level: Option<u32>, // For headings
    pub value_text: Option<String>,
    pub value_min: Option<f64>,
    pub value_max: Option<f64>,
    pub value_now: Option<f64>,
}

impl Default for AriaProperties {
    fn default() -> Self {
        Self {
            role: None,
            label: None,
            description: None,
            expanded: None,
            selected: None,
            checked: None,
            disabled: None,
            required: None,
            invalid: None,
            level: None,
            value_text: None,
            value_min: None,
            value_max: None,
            value_now: None,
        }
    }
}

impl AriaProperties {
    /// Create a new AriaProperties with a specific role
    pub fn with_role(role: AriaRole) -> Self {
        Self {
            role: Some(role),
            ..Default::default()
        }
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Generate screen reader description
    pub fn screen_reader_description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(role) = &self.role {
            parts.push(format!("{:?}", role).to_lowercase().replace('_', " "));
        }

        if let Some(label) = &self.label {
            parts.push(label.clone());
        }

        if let Some(description) = &self.description {
            parts.push(description.clone());
        }

        if let Some(expanded) = self.expanded {
            parts.push(if expanded {
                "expanded".to_string()
            } else {
                "collapsed".to_string()
            });
        }

        if let Some(selected) = self.selected {
            if selected {
                parts.push("selected".to_string());
            }
        }

        if let Some(checked) = self.checked {
            parts.push(if checked {
                "checked".to_string()
            } else {
                "unchecked".to_string()
            });
        }

        if let Some(disabled) = self.disabled {
            if disabled {
                parts.push("disabled".to_string());
            }
        }

        if let Some(required) = self.required {
            if required {
                parts.push("required".to_string());
            }
        }

        if let Some(invalid) = self.invalid {
            if invalid {
                parts.push("invalid".to_string());
            }
        }

        if let Some(level) = self.level {
            parts.push(format!("level {}", level));
        }

        if let Some(value_text) = &self.value_text {
            parts.push(value_text.clone());
        } else if let Some(value_now) = self.value_now {
            if let (Some(min), Some(max)) = (self.value_min, self.value_max) {
                parts.push(format!("{:.0}%", ((value_now - min) / (max - min)) * 100.0));
            } else {
                parts.push(format!("{:.0}", value_now));
            }
        }

        parts.join(", ")
    }
}
