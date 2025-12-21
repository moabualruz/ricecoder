//! Profile management for keybind configurations

use std::time::SystemTime;

use crate::error::ProfileError;
use crate::models::Keybind;
use std::collections::HashMap;

/// Represents a keybind profile
#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub keybinds: Vec<Keybind>,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
}

// Manual serialization for Profile to handle SystemTime
impl serde::Serialize for Profile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Profile", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("keybinds", &self.keybinds)?;
        state.end()
    }
}

// Manual deserialization for Profile
impl<'de> serde::Deserialize<'de> for Profile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Name,
            Keybinds,
        }

        struct ProfileVisitor;

        impl<'de> Visitor<'de> for ProfileVisitor {
            type Value = Profile;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Profile")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Profile, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut keybinds = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Keybinds => {
                            if keybinds.is_some() {
                                return Err(de::Error::duplicate_field("keybinds"));
                            }
                            keybinds = Some(map.next_value()?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let keybinds = keybinds.ok_or_else(|| de::Error::missing_field("keybinds"))?;
                let now = SystemTime::now();

                Ok(Profile {
                    name,
                    keybinds,
                    created_at: now,
                    modified_at: now,
                })
            }
        }

        deserializer.deserialize_struct("Profile", &["name", "keybinds"], ProfileVisitor)
    }
}

impl Profile {
    /// Create a new profile
    pub fn new(name: impl Into<String>, keybinds: Vec<Keybind>) -> Self {
        let now = SystemTime::now();
        Profile {
            name: name.into(),
            keybinds,
            created_at: now,
            modified_at: now,
        }
    }
}

/// Manages keybind profiles
pub struct ProfileManager {
    profiles: HashMap<String, Profile>,
    active_profile: Option<String>,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new() -> Self {
        ProfileManager {
            profiles: HashMap::new(),
            active_profile: None,
        }
    }

    /// Create a new profile
    pub fn create_profile(
        &mut self,
        name: impl Into<String>,
        keybinds: Vec<Keybind>,
    ) -> Result<(), ProfileError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ProfileError::InvalidProfileName(
                "Profile name cannot be empty".to_string(),
            ));
        }

        if self.profiles.contains_key(&name) {
            return Err(ProfileError::ProfileAlreadyExists(name));
        }

        let profile = Profile::new(name.clone(), keybinds);
        self.profiles.insert(name, profile);

        // Set as active if it's the first profile
        if self.active_profile.is_none() {
            self.active_profile = Some(self.profiles.keys().next().unwrap().clone());
        }

        Ok(())
    }

    /// Select a profile as active
    pub fn select_profile(&mut self, name: &str) -> Result<(), ProfileError> {
        if !self.profiles.contains_key(name) {
            return Err(ProfileError::ProfileNotFound(name.to_string()));
        }

        self.active_profile = Some(name.to_string());
        Ok(())
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) -> Result<(), ProfileError> {
        if let Some(active) = &self.active_profile {
            if active == name {
                return Err(ProfileError::CannotDeleteActiveProfile(name.to_string()));
            }
        }

        if self.profiles.remove(name).is_none() {
            return Err(ProfileError::ProfileNotFound(name.to_string()));
        }

        Ok(())
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Vec<&Profile> {
        self.profiles.values().collect()
    }

    /// Get the active profile
    pub fn get_active_profile(&self) -> Result<&Profile, ProfileError> {
        let name = self
            .active_profile
            .as_ref()
            .ok_or(ProfileError::NoActiveProfile)?;

        self.profiles
            .get(name)
            .ok_or_else(|| ProfileError::ProfileNotFound(name.clone()))
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Get the active profile name
    pub fn active_profile_name(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }

    /// Update a profile's keybinds
    pub fn update_profile(
        &mut self,
        name: &str,
        keybinds: Vec<Keybind>,
    ) -> Result<(), ProfileError> {
        let profile = self
            .profiles
            .get_mut(name)
            .ok_or_else(|| ProfileError::ProfileNotFound(name.to_string()))?;

        profile.keybinds = keybinds;
        profile.modified_at = SystemTime::now();

        Ok(())
    }

    /// Get number of profiles
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Check if profile manager is empty
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}
