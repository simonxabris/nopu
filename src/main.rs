use anyhow::{Context, Result};
use clap::{command, Parser, Subcommand};
use std::{env, path, process};
use tokio::{fs, task::JoinSet};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    List
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir().context("Failed to read current working directory")?;

    let folders = find_node_modules(&cwd).unwrap();

    match cli.commands {
        Some(command) => match command {
            Commands::List => {
                list_folders(&folders);
            }
        },
        None => {
            delete_files(folders.clone()).await;
        }
    };

    Ok(())
}

fn list_folders(folders: &Vec<path::PathBuf>) {
    println!("Found the following node_modules folders:");

    for folder in folders {
        println!("- {}", folder.display())
    }
}

async fn delete_files(folders: Vec<path::PathBuf>) {
    if folders.is_empty() {
        println!(
            "No node_modules directories found inside {}.",
            env::current_dir().unwrap().display()
        );
        process::exit(0);
    }

    let mut set: JoinSet<u64> = JoinSet::new();

    println!("Deleting the following node_modules:");
    for file in folders.into_iter() {
        println!("- {}", file.display());
        set.spawn(async move {
            let file_size = file.metadata().expect("Failed to read file metadata").len();

            if let Err(e) = fs::remove_dir_all(&file).await {
                println!(
                    "Failed ro remove directory at {}, reason: {}",
                    &file.display(),
                    &e.to_string()
                );
            };

            file_size
        });
    }

    let mut accumulated_size = 0;

    while let Some(file_size) = set.join_next().await {
        accumulated_size += file_size.unwrap();
    }

    println!("Freed up {} GB", accumulated_size / 1024 / 1024 / 1024);
}

fn find_node_modules<P: AsRef<path::Path>>(path: P) -> Result<Vec<path::PathBuf>> {
    let mut paths: Vec<path::PathBuf> = Vec::new();
    let dir_contents = std::fs::read_dir(&path)
        .with_context(|| format!("Failed to read directory at {}", path.as_ref().display()))?;

    for dir_content in dir_contents {
        let dir_content = dir_content?;
        let current_path = dir_content.path();

        if current_path.is_dir() && current_path.to_str().unwrap().contains("node_modules") {
            paths.push(current_path);
            break;
        } else if current_path.is_dir() {
            let found_paths = find_node_modules(&current_path)?;
            paths.extend_from_slice(&found_paths);
        }
    }

    Ok(paths)
}
