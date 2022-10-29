use anyhow::{Context, Result};
use std::{env, path, process};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let cwd = env::current_dir().context("Failed to read current working directory")?;

    let files = find_node_modules(&cwd).unwrap();

    if files.is_empty() {
        println!(
            "No node_modules directories found inside {}.",
            env::current_dir().unwrap().display()
        );
        process::exit(0);
    }

    println!("Deleting the following node_modules:");
    for file in files.into_iter() {
        println!("- {}", file.display());
        tokio::task::spawn(async move {
            if let Err(e) = fs::remove_dir_all(&file).await {
                println!("Failed ro remove directory at {}, reason: {}", &file.display(), &e.to_string());
            };
       });
    }


    Ok(())
}

fn find_node_modules<P: AsRef<path::Path>>(path: P) -> Result<Vec<path::PathBuf>> {
    let mut paths: Vec<path::PathBuf> = Vec::new();
    let dir_contents = std::fs::read_dir(&path).with_context(|| format!("Failed to read directory at {}", path.as_ref().display()))?;

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
