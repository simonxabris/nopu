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
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir().context("Failed to read current working directory")?;

    let folders = find_node_modules(&cwd).unwrap();

    match &cli.commands {
        Some(Commands::List) => {
            list_folders(&folders);
        }
        None => {
            delete_files(folders.clone()).await;
        }
    };

    Ok(())
}

fn list_folders(folders: &Vec<path::PathBuf>) {
    println!("Found the following node_modules folders:");

    let mut sum_of_file_sizes: u64 = 0;
    for folder in folders {
        let metadata = folder.metadata().unwrap();

        sum_of_file_sizes += metadata.len();

        println!(
            "- {}. size: {}",
            folder.display(),
            format_file_size(metadata.len())
        )
    }

    println!(
        "Total size of node_modules: {}",
        format_file_size(sum_of_file_sizes)
    )
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

    println!("Freed up {}", format_file_size(accumulated_size));
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
        } else if current_path.is_dir() {
            let found_paths = find_node_modules(&current_path)?;
            paths.extend_from_slice(&found_paths);
        }
    }

    Ok(paths)
}

fn format_file_size(size: u64) -> String {
    let units = ["Bytes", "KB", "MB", "GB"];
    let mut size_f64 = size as f64;

    let mut unit_index = 0;
    while size_f64 >= 1024.0 && unit_index < units.len() - 1 {
        size_f64 /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size_f64, units[unit_index])
}
