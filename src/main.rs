use std::collections::HashMap;
use clap::Parser;

/// Find duplicate files in a directory tree.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// folders to search
    #[arg(value_name = "PATH", required = true)]
    folders: Vec<String>,
    /// recurse into subdirectories
    #[arg(short, long, default_value_t = false)]
    recurse: bool,
    /// Minimum file size in bytes
    #[arg(short, long, default_value_t = 0)]
    min_size: u64,
    /// Number of worker threads
    #[arg(short, long, default_value_t = 4)]
    threads: usize,
    /// Follow symlinks
    #[arg(long, default_value_t = false)]
    follow_symlinks: bool,
    /// Include hidden files
    #[arg(long, default_value_t = false)]
    hidden: bool,
    /// Output as JSON
    #[arg(long, default_value_t = false)]
    json: bool,
    /// Automatically delete duplicates (DANGEROUS)
    #[arg(long, default_value_t = false)]
    auto_delete: bool,
    /// Replace duplicates with a hard link to the original file
    #[arg(long, default_value_t = false)]
    replace: bool,
}

fn find_files(
    folders: Vec<String>,
    recurse: bool,
    hidden: bool,
    follow_symlinks: bool,
    min_size: u64,
) -> Vec<String> {
    let mut files = Vec::new();
    for folder in folders {
        let mut entries = std::fs::read_dir(folder.clone()).unwrap();
        while let Some(entry) = entries.next() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_symlink() && follow_symlinks {
                if let Ok(target) = std::fs::read_link(&path) {
                    if target.is_file() {
                        files.push(target.to_string_lossy().into_owned());
                    }
                }
            } else if path.is_file() && path.metadata().unwrap().len() >= min_size {
                if is_hidden(&path.to_str().unwrap()) && hidden {
                    files.push(path.to_string_lossy().into_owned());
                } else if !is_hidden(&path.to_str().unwrap()) {
                    files.push(path.to_string_lossy().into_owned());
                }
            } else if path.is_dir() && recurse {
                if is_hidden(&path.to_str().unwrap()) && hidden {
                    files.extend(find_files(
                        vec![path.to_string_lossy().into_owned()],
                        true,
                        hidden,
                        follow_symlinks,
                        min_size,
                    ));
                } else if !is_hidden(&path.to_str().unwrap()) {
                    files.extend(find_files(
                        vec![path.to_string_lossy().into_owned()],
                        true,
                        hidden,
                        follow_symlinks,
                        min_size,
                    ));
                }
            }
        }
    }
    files
}

fn find_duplicates(files: Vec<String>) -> HashMap<String, Vec<String>> {
    let mut file_map: HashMap<String, String> = HashMap::new();
    let mut duplicates: HashMap<String, Vec<String>> = HashMap::new();

    for file in files {
        let content = std::fs::read(file.clone()).unwrap();
        let hash = format!("{:x}", md5::compute(content));

        if let Some(existing) = file_map.get(&hash) {
            duplicates.entry(existing.clone()).or_default().push(file);
        } else {
            file_map.insert(hash, file.clone());
        }
    }

    let mut duplicate_files = Vec::new();
    for (original, dupes) in duplicates.clone() {
        duplicate_files.push(original);
        duplicate_files.extend(dupes);
    }
    duplicates
}

fn list_duplicates(files: HashMap<String, Vec<String>>) {
    for (original, duplicates) in files {
        println!("Original: {}", original);
        for dup in duplicates {
            println!("  Duplicate: {}", dup);
        }
    }
}

fn is_hidden(file: &str) -> bool {
    file.split("/")
        .last().unwrap()
        .split("\\")
        .last().unwrap()
        .starts_with(".")
}

fn main() {
    let args = Args::parse();

    if args.follow_symlinks {
        println!("Following symlinks");
    }

    if args.hidden {
        println!("Including hidden files");
    }

    if args.json {
        println!("Outputting as JSON");
    }

    if args.auto_delete {
        println!("Automatically deleting duplicates");
    }

    if args.recurse {
        println!("Recursing into subdirectories");
    }

    let duplicates = find_duplicates(find_files(
        args.folders,
        args.recurse,
        args.hidden,
        args.follow_symlinks,
        args.min_size,
    ));

    list_duplicates(duplicates);

    if args.replace {
        println!("Replacing duplicates with hard links");
    }
}
