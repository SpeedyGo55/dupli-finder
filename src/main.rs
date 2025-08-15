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
                // if is_hidden(&path.to_str().unwrap()) && hidden {
                //     files.extend(find_files(
                //         vec![path.to_string_lossy().into_owned()],
                //         true,
                //         hidden,
                //         follow_symlinks,
                //         min_size,
                //     ));
                // } else if !is_hidden(&path.to_str().unwrap()) {
                //     files.extend(find_files(
                //         vec![path.to_string_lossy().into_owned()],
                //         true,
                //         hidden,
                //         follow_symlinks,
                //         min_size,
                //     ));
                // }

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
    files
}

fn find_duplicates(files: Vec<String>) -> Vec<String> {
    let mut duplicates = Vec::new();
    let mut files = files.to_vec();
    files.sort();
    let mut current_file = files.first().unwrap().clone();
    let mut current_hash = sha256::digest(&std::fs::read(&current_file).unwrap());
    for file in files.iter().skip(1) {
        let hash = sha256::digest(&std::fs::read(file).unwrap());
        if hash == current_hash {
            duplicates.push(file.clone());
        } else {
            current_file = file.clone();
            current_hash = hash;
        }
    }
    duplicates
}

fn list_duplicates(files: Vec<String>) {
    let duplicates = find_duplicates(files);
    println!("Duplicates found: {}", duplicates.len());
    for file in duplicates {
        println!("{}", file);
    }
}

fn is_hidden(file: &str) -> bool {
    file.starts_with('.')
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

    if args.replace {
        println!("Replacing duplicates with hard links");
    }

    let duplicates = find_duplicates(find_files(
        args.folders,
        args.recurse,
        args.hidden,
        args.follow_symlinks,
        args.min_size,
    ));
    println!("Duplicates found: {}", duplicates.len());

    if args.replace {
        println!("Replacing duplicates with hard links");
    }
}
