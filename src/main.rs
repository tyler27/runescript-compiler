extern crate core;

use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::parser::{Parser, Script, AstKind};
use crate::evaluator::Evaluator;
use crate::config::Config;
use std::fs;
use std::path::PathBuf;
use clap::{Parser as ClapParser, Subcommand};

mod error;
mod lexer;
mod parser;
mod token;
mod evaluator;
mod analysis;
mod config;

#[derive(ClapParser)]
#[command(author, version, about = "RuneScript Compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a RuneScript file with arguments
    Run {
        /// Name of the script to run (without .rs2 extension)
        script_name: String,
        /// Arguments to pass to the script
        args: Vec<i32>,
    },
    /// Analyze the 2004Scape codebase
    #[command(name = "2004")]
    Analyze2004,
    /// Update the RuneScript Compiler to the latest version
    Update,
    /// Manage RuneScript configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Edit the RC file for the current environment
    Edit,
    /// Show the current RC file contents
    Show,
    /// Initialize a new RC file with defaults
    Init,
    /// List all environment variables and aliases
    List,
}

fn get_rs2_files(config: &Config) -> Result<Vec<PathBuf>, CompilerError> {
    let scripts_path = &config.scripts_dir;

    if !scripts_path.exists() {
        return Err(CompilerError::FileNotFound(format!(
            "Scripts directory not found: {}\n\nTo fix this:\n1. Create the directory\n2. Add your .rs2 files there\n3. Or set RSC_SCRIPTS_DIR in your RC file (rsc config edit)",
            scripts_path.display()
        )));
    }

    if !scripts_path.is_dir() {
        return Err(CompilerError::FileNotFound(format!(
            "Expected {} to be a directory",
            scripts_path.display()
        )));
    }

    let mut found_scripts: Vec<PathBuf> = Vec::new();
    let files = fs::read_dir(scripts_path).map_err(|e| {
        CompilerError::FileNotFound(format!(
            "Cannot access scripts directory: {}\nError: {}",
            scripts_path.display(), e
        ))
    })?;

    for entry in files {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs2") {
                found_scripts.push(path);
            }
        }
    }

    if found_scripts.is_empty() {
        return Err(CompilerError::FileNotFound(format!(
            "No .rs2 files found in: {}\n\nTo fix this:\n1. Add your RuneScript (.rs2) files to this directory\n2. Or set RSC_SCRIPTS_DIR in your RC file (rsc config edit)\n3. Example script path: {}/example.rs2",
            scripts_path.display(),
            scripts_path.display()
        )));
    }

    Ok(found_scripts)
}

fn process_rs2_file(path_buf: &PathBuf) -> Result<Script, CompilerError> {
    let source_code = fs::read_to_string(path_buf)
        .map_err(|e| CompilerError::IO(e))?;
    
    let tokens = Lexer::new(&source_code, path_buf)
        .tokenize()
        .map_err(|e| CompilerError::LexingError(e))?;
        
    let mut parser = Parser::new(tokens, path_buf);
    parser.parse()
        .map_err(|e| CompilerError::Syntax(e))
}

fn run_script(script_name: &str, args: &[i32], config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Load and register all scripts
    let mut evaluator = Evaluator::new();
    
    let mut found_script = false;
    let scripts = match get_rs2_files(config) {
        Ok(scripts) => scripts,
        Err(CompilerError::FileNotFound(msg)) => {
            println!("Error: {}", msg);
            println!("\nCurrent configuration:");
            println!("  Environment: {}", config.env_name);
            println!("  Scripts directory: {}", config.scripts_dir.display());
            println!("\nTo change the scripts directory:");
            println!("1. Edit your RC file: rsc config edit");
            println!("2. Add: export RSC_SCRIPTS_DIR=/path/to/your/scripts");
            return Ok(());
        }
        Err(e) => return Err(Box::new(e)),
    };

    // First pass to register scripts and check if target exists
    for path in &scripts {
        let script = process_rs2_file(path)?;
        for node in &script.body {
            if let AstKind::Trigger { name, .. } = node {
                if let AstKind::Identifier(script_name_found) = &**name {
                    evaluator.register_script(script_name_found.clone(), node.clone());
                    if script_name_found.to_lowercase() == script_name.to_lowercase() {
                        found_script = true;
                    }
                }
            }
        }
    }

    if !found_script {
        println!("Error: Script '{}' not found in {}", script_name, config.scripts_dir.display());
        println!("\nAvailable scripts:");
        for path in &scripts {
            if let Ok(script) = process_rs2_file(path) {
                if let Some(AstKind::Trigger { name, .. }) = script.body.get(0) {
                    if let AstKind::Identifier(name) = &**name {
                        println!("  {}", name);
                    }
                }
            }
        }
        return Ok(());
    }

    println!("Evaluating {} with args: {:?}", script_name, args);
    // Run the specified script
    let result = evaluator.eval_script(script_name, args);
    println!("{}", result);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        Commands::Run { script_name, args } => {
            run_script(&script_name, &args, &config)?;
        }
        Commands::Analyze2004 => {
            println!("Analyzing 2004Scape codebase...");
            let mut analyzer = analysis::ScriptAnalysis::new();
            match analyzer.analyze_repository() {
                Ok(_) => analyzer.print_analysis(),
                Err(e) => println!("Error analyzing 2004Scape codebase: {}", e),
            }
        }
        Commands::Update => {
            // Get the current directory
            let current_dir = std::env::current_dir()?;
            let install_script = if cfg!(windows) {
                "install.ps1"
            } else {
                "install.sh"
            };

            if !current_dir.join(install_script).exists() {
                println!("Error: Installation script not found. Please run this command from the RuneScript Compiler directory.");
                return Ok(());
            }

            println!("Updating RuneScript Compiler ({} environment)...", config.env_name);
            
            // Check if git is initialized and has a remote
            let has_git = std::process::Command::new("git")
                .args(["rev-parse", "--git-dir"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);

            let has_remote = if has_git {
                std::process::Command::new("git")
                    .args(["remote", "get-url", "origin"])
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            } else {
                false
            };

            // Only try to pull if we have a git repo with a remote
            if has_git && has_remote {
                println!("Pulling latest changes from git...");
                if let Ok(status) = std::process::Command::new("git")
                    .args(["pull", "origin", "main"])
                    .status()
                {
                    if !status.success() {
                        println!("Warning: Failed to pull latest changes. Continuing with local version...");
                    }
                }
            } else {
                println!("No git repository found or no remote configured. Using local version...");
            }

            // Run the installation script with environment variables
            println!("Rebuilding and reinstalling...");
            if cfg!(windows) {
                std::process::Command::new("powershell")
                    .args(["-ExecutionPolicy", "Bypass", "-File", install_script])
                    .env("RSC_ENV", &config.env_name)
                    .env("RSC_INSTALL_DIR", config.install_dir.to_str().unwrap())
                    .env("RSC_SCRIPTS_DIR", config.scripts_dir.to_str().unwrap())
                    .status()?;
            } else {
                std::process::Command::new("sh")
                    .arg(install_script)
                    .env("RSC_ENV", &config.env_name)
                    .env("RSC_INSTALL_DIR", config.install_dir.to_str().unwrap())
                    .env("RSC_SCRIPTS_DIR", config.scripts_dir.to_str().unwrap())
                    .status()?;
            }
            
            println!("Update complete!");
        }
        Commands::Config { command } => {
            match command {
                ConfigCommands::Edit => {
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
                        if cfg!(windows) {
                            String::from("notepad")
                        } else {
                            String::from("nano")
                        }
                    });
                    
                    let rc_path = Config::get_rc_path();
                    if !rc_path.exists() {
                        Config::load_rc_file()?;
                    }
                    
                    std::process::Command::new(editor)
                        .arg(rc_path)
                        .status()?;
                }
                ConfigCommands::Show => {
                    let contents = Config::load_rc_file()?;
                    println!("{}", contents);
                }
                ConfigCommands::Init => {
                    let rc_path = Config::get_rc_path();
                    if rc_path.exists() {
                        println!("RC file already exists at: {}", rc_path.display());
                        println!("Use 'rsc config edit' to modify it or remove the file to reinitialize.");
                    } else {
                        Config::load_rc_file()?;
                        println!("Initialized new RC file at: {}", rc_path.display());
                        println!("Use 'rsc config edit' to modify it.");
                    }
                }
                ConfigCommands::List => {
                    let contents = Config::load_rc_file()?;
                    let (aliases, env_vars) = Config::parse_rc_file(&contents);
                    
                    println!("Environment: {}", config.env_name);
                    println!("\nEnvironment Variables:");
                    for (key, value) in env_vars {
                        println!("  {}={}", key, value);
                    }
                    
                    println!("\nAliases:");
                    for alias in aliases {
                        println!("  {}", alias);
                    }
                }
            }
        }
    }

    Ok(())
}

