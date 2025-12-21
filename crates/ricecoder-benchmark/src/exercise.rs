//! Exercise representation and loading

use crate::error::BenchmarkError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseConfig {
    pub files: ExerciseFiles,
    #[serde(default)]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseFiles {
    pub solution: Vec<String>,
    pub test: Vec<String>,
    #[serde(default)]
    pub example: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Exercise {
    pub name: String,
    pub language: String,
    pub path: PathBuf,
    pub config: ExerciseConfig,
    pub instructions: String,
    pub introduction: Option<String>,
    pub instructions_append: Option<String>,
}

impl Exercise {
    pub fn load_from_path(path: &Path) -> Result<Self, BenchmarkError> {
        let config_path = path.join(".meta/config.json");
        let config: ExerciseConfig = serde_json::from_reader(std::fs::File::open(&config_path)?)?;

        let instructions_path = path.join(".docs/instructions.md");
        let instructions = std::fs::read_to_string(&instructions_path)?;

        let introduction_path = path.join(".docs/introduction.md");
        let introduction = if introduction_path.exists() {
            Some(std::fs::read_to_string(&introduction_path)?)
        } else {
            None
        };

        let instructions_append_path = path.join(".docs/instructions.append.md");
        let instructions_append = if instructions_append_path.exists() {
            Some(std::fs::read_to_string(&instructions_append_path)?)
        } else {
            None
        };

        let language = path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Exercise {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            language,
            path: path.to_path_buf(),
            config,
            instructions,
            introduction,
            instructions_append,
        })
    }

    pub fn get_solution_files(&self) -> Vec<PathBuf> {
        self.config
            .files
            .solution
            .iter()
            .map(|f| self.path.join(f))
            .collect()
    }

    pub fn get_test_files(&self) -> Vec<PathBuf> {
        self.config
            .files
            .test
            .iter()
            .map(|f| self.path.join(f))
            .collect()
    }

    pub fn get_ignore_files(&self) -> HashSet<String> {
        let mut ignore = HashSet::new();

        // Add test and example files
        ignore.extend(self.config.files.test.iter().cloned());
        ignore.extend(self.config.files.example.iter().cloned());

        // Add common ignore files
        ignore.insert("CMakeLists.txt".to_string());
        ignore.insert("Cargo.toml".to_string());

        // Add .meta and .docs directories
        ignore.insert(".meta".to_string());
        ignore.insert(".docs".to_string());

        ignore
    }

    pub fn get_full_instructions(&self) -> String {
        let mut instructions = String::new();

        if let Some(intro) = &self.introduction {
            instructions.push_str(intro);
        }

        instructions.push_str(&self.instructions);

        if let Some(append) = &self.instructions_append {
            instructions.push_str(append);
        }

        // Add file list
        let solution_files: Vec<_> = self
            .config
            .files
            .solution
            .iter()
            .filter(|f| !self.get_ignore_files().contains(*f))
            .collect();
        let file_list = solution_files
            .iter()
            .map(|f| {
                Path::new(f)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            })
            .collect::<Vec<_>>()
            .join(" ");

        instructions.push_str(&format!(
            "\n\nThe solution should be implemented in the following files: {}\n",
            file_list
        ));

        instructions
    }
}
