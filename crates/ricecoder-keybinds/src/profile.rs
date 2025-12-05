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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_profile() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        assert!(manager.create_profile("default", keybinds).is_ok());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_create_duplicate_profile() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds.clone()).unwrap();
        assert!(manager.create_profile("default", keybinds).is_err());
    }

    #[test]
    fn test_select_profile() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds.clone()).unwrap();
        manager.create_profile("vim", keybinds).unwrap();

        assert!(manager.select_profile("vim").is_ok());
        assert_eq!(manager.active_profile_name(), Some("vim"));
    }

    #[test]
    fn test_delete_profile() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds.clone()).unwrap();
        manager.create_profile("vim", keybinds).unwrap();

        manager.select_profile("vim").unwrap();
        assert!(manager.delete_profile("default").is_ok());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_cannot_delete_active_profile() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds).unwrap();
        assert!(manager.delete_profile("default").is_err());
    }

    #[test]
    fn test_get_active_profile() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds).unwrap();
        let profile = manager.get_active_profile().unwrap();
        assert_eq!(profile.name, "default");
    }

    #[test]
    fn test_list_profiles() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds.clone()).unwrap();
        manager.create_profile("vim", keybinds).unwrap();

        let profiles = manager.list_profiles();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_update_profile() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds).unwrap();

        let new_keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        assert!(manager.update_profile("default", new_keybinds).is_ok());
        let profile = manager.get_profile("default").unwrap();
        assert_eq!(profile.keybinds.len(), 2);
    }

    #[test]
    fn test_profile_metadata_creation_time() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        let before_creation = SystemTime::now();
        manager.create_profile("default", keybinds).unwrap();
        let after_creation = SystemTime::now();

        let profile = manager.get_profile("default").unwrap();

        // Verify created_at is set and within expected time range
        assert!(profile.created_at >= before_creation);
        assert!(profile.created_at <= after_creation);

        // Verify modified_at is also set to creation time
        assert!(profile.modified_at >= before_creation);
        assert!(profile.modified_at <= after_creation);
    }

    #[test]
    fn test_profile_metadata_modification_time() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds).unwrap();
        let profile_after_creation = manager.get_profile("default").unwrap();
        let created_at = profile_after_creation.created_at;
        let modified_at_after_creation = profile_after_creation.modified_at;

        // Wait a bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Update the profile
        let new_keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        let before_update = SystemTime::now();
        manager.update_profile("default", new_keybinds).unwrap();
        let after_update = SystemTime::now();

        let profile_after_update = manager.get_profile("default").unwrap();

        // Verify created_at hasn't changed
        assert_eq!(profile_after_update.created_at, created_at);

        // Verify modified_at has been updated
        assert!(profile_after_update.modified_at >= before_update);
        assert!(profile_after_update.modified_at <= after_update);
        assert!(profile_after_update.modified_at > modified_at_after_creation);
    }

    #[test]
    fn test_profile_metadata_preserved_in_name_and_keybinds() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        manager.create_profile("my_profile", keybinds.clone()).unwrap();

        let profile = manager.get_profile("my_profile").unwrap();

        // Verify profile name is stored
        assert_eq!(profile.name, "my_profile");

        // Verify keybinds are stored
        assert_eq!(profile.keybinds.len(), 2);
        assert_eq!(profile.keybinds[0].action_id, "editor.save");
        assert_eq!(profile.keybinds[1].action_id, "editor.undo");
    }

    #[test]
    fn test_prevent_deletion_of_active_profile_with_metadata() {
        let mut manager = ProfileManager::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        manager.create_profile("default", keybinds).unwrap();

        // Verify profile has metadata
        let profile = manager.get_profile("default").unwrap();
        assert!(profile.created_at <= SystemTime::now());
        assert!(profile.modified_at <= SystemTime::now());

        // Store metadata before mutable borrow
        let created_at = profile.created_at;
        let modified_at = profile.modified_at;

        // Try to delete active profile - should fail
        assert!(manager.delete_profile("default").is_err());

        // Verify profile still exists with same metadata
        let profile_after_failed_delete = manager.get_profile("default").unwrap();
        assert_eq!(profile_after_failed_delete.created_at, created_at);
        assert_eq!(profile_after_failed_delete.modified_at, modified_at);
    }

    #[test]
    fn test_profile_metadata_across_multiple_profiles() {
        let mut manager = ProfileManager::new();
        let keybinds1 = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let keybinds2 = vec![Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo")];

        let before_profile1 = SystemTime::now();
        manager.create_profile("profile1", keybinds1).unwrap();
        let after_profile1 = SystemTime::now();

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));

        let before_profile2 = SystemTime::now();
        manager.create_profile("profile2", keybinds2).unwrap();
        let after_profile2 = SystemTime::now();

        let profile1 = manager.get_profile("profile1").unwrap();
        let profile2 = manager.get_profile("profile2").unwrap();

        // Verify profile1 metadata
        assert!(profile1.created_at >= before_profile1);
        assert!(profile1.created_at <= after_profile1);

        // Verify profile2 metadata
        assert!(profile2.created_at >= before_profile2);
        assert!(profile2.created_at <= after_profile2);

        // Verify profile2 was created after profile1
        assert!(profile2.created_at >= profile1.created_at);
    }
}
