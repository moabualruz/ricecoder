use std::path::{Path, PathBuf};
use serde_json::Value;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Assistant to install (opencode, claude, codex, factory-droid)
    pub assistant: String,

    /// Dry run without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Force installation without prompts
    #[arg(long)]
    pub force: bool,

    /// Override the config root (defaults to user home)
    #[arg(long, value_name = "PATH")]
    pub config_root: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Assistant to uninstall (opencode, claude, codex, factory-droid)
    pub assistant: String,

    /// Dry run without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Force uninstallation without prompts
    #[arg(long)]
    pub force: bool,

    /// Override the config root (defaults to user home)
    #[arg(long, value_name = "PATH")]
    pub config_root: Option<PathBuf>,
}

pub fn edit_json_file(file_path: &Path, json_path: &str, value: Value) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut config: Value = if file_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(file_path)?)?
    } else {
        Value::Object(serde_json::Map::new())
    };
    set_json_path(&mut config, json_path, value)?;
    std::fs::write(file_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

pub fn remove_from_json_file(file_path: &Path, json_path: &str) -> Result<()> {
    let mut config: Value = if file_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(file_path)?)?
    } else {
        return Ok(());
    };
    remove_json_path(&mut config, json_path)?;
    std::fs::write(file_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

pub fn set_json_path(config: &mut Value, json_path: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = json_path.split('.').collect();
    let mut current = config;
    for &part in &parts[..parts.len() - 1] {
        if !current.is_object() {
            *current = Value::Object(serde_json::Map::new());
        }
        let obj = current.as_object_mut().unwrap();
        obj.entry(part).or_insert(Value::Object(serde_json::Map::new()));
        current = obj.get_mut(part).unwrap();
    }
    if let Some(last) = parts.last() {
        if !current.is_object() {
            *current = Value::Object(serde_json::Map::new());
        }
        if let Some(obj) = current.as_object_mut() {
            obj.insert(last.to_string(), value);
        }
    }
    Ok(())
}

pub fn remove_json_path(config: &mut Value, json_path: &str) -> Result<()> {
    let parts: Vec<&str> = json_path.split('.').collect();
    let mut current = config;
    for &part in &parts[..parts.len() - 1] {
        if !current.is_object() {
            return Ok(());
        }
        let obj = current.as_object_mut().unwrap();
        if !obj.contains_key(part) {
            return Ok(());
        }
        current = obj.get_mut(part).unwrap();
    }
    if let Some(last) = parts.last() {
        if let Some(obj) = current.as_object_mut() {
            obj.remove(*last);
        }
    }
    Ok(())
}

fn resolve_config_root(root: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = root {
        return Ok(root.clone());
    }
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home directory not found"))
}

pub async fn run_install(args: InstallArgs) -> Result<()> {
    if args.dry_run {
        println!("Dry run: would install {} with version pinning", args.assistant);
        return Ok(());
    }
    if !args.force {
        println!("Installation requires --force to proceed.");
        return Ok(());
    }
    let config_root = resolve_config_root(&args.config_root)?;
    println!("Installing {}...", args.assistant);
    match args.assistant.as_str() {
        "opencode" => {
            let config_file = config_root.join(".config/opencode/opencode.json");
            let value = serde_json::json!({
                "type": "local",
                "command": ["ricegrep", "mcp"],
                "enabled": true
            });
            edit_json_file(&config_file, "mcp.ricegrep", value)?;
            println!("Updated config at {}", config_file.display());
            println!("Installation complete for {}", args.assistant);
            return Ok(());
        }
        "claude" => {
            let config_file = config_root.join(".claude.json");
            let value = serde_json::json!({
                "command": "ricegrep",
                "args": ["mcp"]
            });
            edit_json_file(&config_file, "projects.*.mcpServers.ricegrep", value)?;
            println!("Updated config at {}", config_file.display());
            println!("Installation complete for {}", args.assistant);
            return Ok(());
        }
        "codex" => {
            let config_file = config_root.join(".codex/config.toml");
            let content = "\n[mcp.servers.ricegrep]\ncommand = \"ricegrep mcp\"\n";
            let existing = if config_file.exists() {
                std::fs::read_to_string(&config_file)?
            } else {
                String::new()
            };
            if !existing.contains("[mcp.servers.ricegrep]") {
                let new_content = existing + content;
                std::fs::write(&config_file, new_content)?;
                println!("Updated config at {}", config_file.display());
            } else {
                println!("RiceGrep already configured in {}", config_file.display());
            }
            println!("Installation complete for {}", args.assistant);
            return Ok(());
        }
        "factory-droid" => {
            let config_file = config_root.join(".factory/settings.json");
            let value = serde_json::json!({
                "command": ["ricegrep", "mcp"]
            });
            edit_json_file(&config_file, "mcp.ricegrep", value)?;
            println!("Updated config at {}", config_file.display());
            println!("Installation complete for {}", args.assistant);
            return Ok(());
        }
        _ => {
            println!("Unknown assistant: {}", args.assistant);
            return Ok(());
        }
    }
}

pub async fn run_uninstall(args: UninstallArgs) -> Result<()> {
    if args.dry_run {
        println!("Dry run: would uninstall {}", args.assistant);
        return Ok(());
    }
    if !args.force {
        println!("Uninstallation requires --force to proceed.");
        return Ok(());
    }
    let config_root = resolve_config_root(&args.config_root)?;
    println!("Uninstalling {}...", args.assistant);
    match args.assistant.as_str() {
        "opencode" => {
            let config_file = config_root.join(".config/opencode/opencode.json");
            remove_from_json_file(&config_file, "mcp.ricegrep")?;
            println!("Updated config at {}", config_file.display());
            println!("Uninstallation complete for {}", args.assistant);
            return Ok(());
        }
        "claude" => {
            let config_file = config_root.join(".claude.json");
            remove_from_json_file(&config_file, "projects.*.mcpServers.ricegrep")?;
            println!("Updated config at {}", config_file.display());
            println!("Uninstallation complete for {}", args.assistant);
            return Ok(());
        }
        "codex" => {
            let config_file = config_root.join(".codex/config.toml");
            if config_file.exists() {
                let content = std::fs::read_to_string(&config_file)?;
                let lines: Vec<&str> = content.lines().collect();
                let new_lines: Vec<String> = lines
                    .into_iter()
                    .filter(|line| {
                        !line.contains("[mcp.servers.ricegrep]")
                            && !line.contains("command = \"ricegrep mcp\"")
                    })
                    .map(|s| s.to_string())
                    .collect();
                std::fs::write(&config_file, new_lines.join("\n"))?;
                println!("Updated config at {}", config_file.display());
            }
            println!("Uninstallation complete for {}", args.assistant);
            return Ok(());
        }
        "factory-droid" => {
            let config_file = config_root.join(".factory/settings.json");
            remove_from_json_file(&config_file, "mcp.ricegrep")?;
            println!("Updated config at {}", config_file.display());
            println!("Uninstallation complete for {}", args.assistant);
            return Ok(());
        }
        _ => {
            println!("Unknown assistant: {}", args.assistant);
            return Ok(());
        }
    }
}
