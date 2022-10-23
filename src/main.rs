use std::{env, fs, path, process};

const FOLDER_NAME: &str = "node_modules";

fn main() {
    let files = find_node_modules().unwrap();

    if files.is_empty() {
        println!(
            "No node_modules directories found inside {}.",
            env::current_dir().unwrap().display()
        );
        process::exit(0);
    }

    println!("Deleting the following node_modules:");
    for file in files.iter() {
        println!("- {}", file.display());

        fs::remove_dir_all(file).expect("Failed to remove folder");
    }
}

fn find_files<P: AsRef<path::Path>>(path: P) -> Result<Vec<path::PathBuf>, std::io::Error> {
    let mut paths: Vec<path::PathBuf> = Vec::new();
    let dir_contents = fs::read_dir(&path)?;

    for dir_content in dir_contents {
        let dir_content = dir_content?;
        let current_path = dir_content.path();

        if current_path.is_dir() {
            let found_paths = find_files(&current_path)?;
            paths.extend_from_slice(&found_paths);
        }

        paths.push(current_path);
    }

    Ok(paths)
}

fn find_node_modules() -> std::io::Result<Vec<path::PathBuf>> {
    let cwd = env::current_dir().unwrap();

    let files = find_files(&cwd)?;

    let filtered_paths = files
        .into_iter()
        .filter(|file| {
            return file.is_dir() && file.to_str().unwrap().contains(FOLDER_NAME);
        })
        .collect();

    Ok(filtered_paths)
}
