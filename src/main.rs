use clap::{Arg, Command};
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const MAX_BOOKMARKS: usize = 36;
const BOOKMARK_FILE: &str = ".local/changeDirectory";
const HISTORY_FILE: &str = ".local/changeDirectoryHistory";
const TARGET_FILE: &str = ".local/share/changedir.target";

fn debug_print(verbose: bool, message: &str) {
    if verbose {
        eprintln!("{} {}", "[DEBUG]".bright_blue().bold(), message.bright_black());
    }
}

fn get_bookmark_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(BOOKMARK_FILE)
}

fn get_history_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(HISTORY_FILE)
}

fn get_target_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(TARGET_FILE)
}

fn delete_target_file(verbose: bool) -> io::Result<()> {
    let path = get_target_path();
    if path.exists() {
        debug_print(verbose, &format!("Deleting existing target file: {}", path.display()));
        fs::remove_file(&path)?;
    }
    Ok(())
}

fn write_target_file(path: &PathBuf, verbose: bool) -> io::Result<()> {
    let target_path = get_target_path();
    debug_print(verbose, &format!("Writing target directory to: {}", target_path.display()));
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&target_path, path.to_string_lossy().as_bytes())?;
    debug_print(verbose, "Target file written successfully");
    Ok(())
}

fn load_bookmarks(verbose: bool) -> Vec<PathBuf> {
    let path = get_bookmark_path();
    debug_print(verbose, &format!("Loading bookmarks from: {}", path.display()));
    
    if !path.exists() {
        debug_print(verbose, "Bookmark file does not exist");
        return Vec::new();
    }

    let bookmarks: Vec<PathBuf> = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        })
        .collect();
    
    debug_print(verbose, &format!("Loaded {} bookmarks", bookmarks.len()));
    bookmarks
}

fn save_bookmarks(bookmarks: &[PathBuf], verbose: bool) -> io::Result<()> {
    let path = get_bookmark_path();
    debug_print(verbose, &format!("Saving {} bookmarks to: {}", bookmarks.len(), path.display()));
    
    if let Some(parent) = path.parent() {
        debug_print(verbose, &format!("Creating parent directory: {}", parent.display()));
        fs::create_dir_all(parent)?;
    }

    let content = bookmarks
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&path, content)?;
    debug_print(verbose, "Bookmarks saved successfully");
    Ok(())
}

fn load_history(verbose: bool) -> Vec<PathBuf> {
    let path = get_history_path();
    debug_print(verbose, &format!("Loading history from: {}", path.display()));
    
    if !path.exists() {
        debug_print(verbose, "History file does not exist");
        return Vec::new();
    }

    let history: Vec<PathBuf> = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        })
        .collect();
    
    debug_print(verbose, &format!("Loaded {} history entries", history.len()));
    history
}

fn save_history(history: &[PathBuf], verbose: bool) -> io::Result<()> {
    let path = get_history_path();
    debug_print(verbose, &format!("Saving {} history entries to: {}", history.len(), path.display()));
    
    if let Some(parent) = path.parent() {
        debug_print(verbose, &format!("Creating parent directory: {}", parent.display()));
        fs::create_dir_all(parent)?;
    }

    let content = history
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&path, content)?;
    debug_print(verbose, "History saved successfully");
    Ok(())
}

fn add_to_history(path: PathBuf, verbose: bool) -> io::Result<()> {
    debug_print(verbose, &format!("Adding to history: {}", path.display()));
    let mut history = load_history(verbose);
    
    // Remove if already exists (to avoid duplicates)
    let initial_len = history.len();
    history.retain(|p| p != &path);
    if history.len() < initial_len {
        debug_print(verbose, "Removed duplicate entry from history");
    }
    
    // Add to front
    history.insert(0, path.clone());
    debug_print(verbose, &format!("Added {} to history", path.display()));
    
    // Keep only last 10 entries
    if history.len() > 10 {
        let removed = history.len() - 10;
        history.truncate(10);
        debug_print(verbose, &format!("Truncated history, removed {} old entries", removed));
    }
    
    save_history(&history, verbose)
}

fn list_bookmarks(verbose: bool) -> io::Result<()> {
    debug_print(verbose, "Listing bookmarks and history");
    let bookmarks = load_bookmarks(verbose);
    let history = load_history(verbose);
    
    // Filter out history entries that are already in bookmarks
    let filtered_history: Vec<PathBuf> = history
        .iter()
        .filter(|hist_dir| !bookmarks.contains(hist_dir))
        .cloned()
        .collect();
    
    let total_items = bookmarks.len() + filtered_history.len();
    
    if total_items == 0 {
        println!("{}", "No bookmarked directories.".yellow());
        return Ok(());
    }

    // List bookmarks
    if !bookmarks.is_empty() {
        debug_print(verbose, &format!("Displaying {} bookmarks", bookmarks.len()));
        for (i, bookmark) in bookmarks.iter().enumerate() {
            let prefix = get_prefix_char(i);
            println!("{} {}", 
                format!("[{}]", prefix).bright_cyan().bold(),
                bookmark.display().to_string().bright_white()
            );
        }
    }

    // List history directories with continuous numbering (excluding duplicates)
    if !filtered_history.is_empty() {
        // Add blank line between bookmarks and history if both exist
        if !bookmarks.is_empty() {
            println!();
        }
        
        debug_print(verbose, &format!("Displaying {} history entries (after filtering duplicates)", filtered_history.len()));
        let start_index = bookmarks.len();
        for (i, hist_dir) in filtered_history.iter().enumerate() {
            let index = start_index + i;
            // Only show if within the 36-item limit (0-9, a-z)
            if index < 36 {
                let prefix = get_prefix_char(index);
                println!("{} {}", 
                    format!("[{}]", prefix).bright_cyan().bold(),
                    hist_dir.display().to_string().bright_white()
                );
            }
        }
    }
    
    Ok(())
}

fn get_prefix_char(index: usize) -> char {
    if index < 10 {
        (b'0' + index as u8) as char
    } else if index < 36 {
        (b'a' + (index - 10) as u8) as char
    } else {
        '?'
    }
}

fn get_index_from_char(ch: char) -> Option<usize> {
    match ch {
        '0'..='9' => Some(ch as usize - '0' as usize),
        'a'..='z' => Some(10 + (ch as usize - 'a' as usize)),
        _ => None,
    }
}

fn bookmark_current(verbose: bool) -> io::Result<()> {
    let current_dir = std::env::current_dir()?;
    debug_print(verbose, &format!("Bookmarking current directory: {}", current_dir.display()));
    let mut bookmarks = load_bookmarks(verbose);

    if bookmarks.iter().any(|b| b == &current_dir) {
        debug_print(verbose, "Directory already bookmarked");
        eprintln!("{}", "Current directory is already bookmarked.".yellow());
        return Ok(());
    }

    debug_print(verbose, &format!("Current bookmark count: {}", bookmarks.len()));
    if bookmarks.len() >= MAX_BOOKMARKS {
        eprintln!("{}", format!("Error: Maximum of {} bookmarks reached. Remove a bookmark first.", MAX_BOOKMARKS).red().bold());
        std::process::exit(1);
    }

    bookmarks.push(current_dir.clone());
    save_bookmarks(&bookmarks, verbose)?;
    println!("{}", format!("Bookmarked: {}", current_dir.display()).green());
    Ok(())
}

fn forget_current(verbose: bool) -> io::Result<()> {
    let current_dir = std::env::current_dir()?;
    debug_print(verbose, &format!("Forgetting current directory: {}", current_dir.display()));
    let mut bookmarks = load_bookmarks(verbose);

    let initial_len = bookmarks.len();
    bookmarks.retain(|b| b != &current_dir);

    if bookmarks.len() < initial_len {
        debug_print(verbose, "Directory was bookmarked, removing it");
        save_bookmarks(&bookmarks, verbose)?;
        println!("{}", format!("Removed bookmark: {}", current_dir.display()).green());
    } else {
        debug_print(verbose, "Directory was not bookmarked");
    }
    Ok(())
}

fn forget_all(verbose: bool) -> io::Result<()> {
    let path = get_bookmark_path();
    debug_print(verbose, &format!("Forgetting all bookmarks, file: {}", path.display()));
    
    if path.exists() {
        debug_print(verbose, "Removing bookmark file");
        fs::remove_file(&path)?;
        println!("{}", "All bookmarks removed.".green());
    } else {
        debug_print(verbose, "Bookmark file does not exist");
        println!("{}", "No bookmarks to remove.".yellow());
    }
    Ok(())
}

fn choose_directory_interactive(verbose: bool) -> io::Result<()> {
    debug_print(verbose, "Interactive directory selection");
    let bookmarks = load_bookmarks(verbose);
    let history = load_history(verbose);
    
    // Filter out history entries that are already in bookmarks
    let filtered_history: Vec<PathBuf> = history
        .iter()
        .filter(|hist_dir| !bookmarks.contains(hist_dir))
        .cloned()
        .collect();
    
    let total_items = bookmarks.len() + filtered_history.len();
    
    if total_items == 0 {
        eprintln!("{}", "No bookmarked directories.".yellow());
        std::process::exit(1);
    }

    // Display bookmarks (to stdout for proper color display)
    if !bookmarks.is_empty() {
        debug_print(verbose, &format!("Displaying {} bookmarks for selection", bookmarks.len()));
        for (i, bookmark) in bookmarks.iter().enumerate() {
            let prefix = get_prefix_char(i);
            println!("{} {}", 
                format!("[{}]", prefix).bright_cyan().bold(),
                bookmark.display().to_string().bright_white()
            );
        }
    }

    // Display history directories with continuous numbering (to stdout for proper color display, excluding duplicates)
    if !filtered_history.is_empty() {
        // Add blank line between bookmarks and history if both exist
        if !bookmarks.is_empty() {
            println!();
        }
        
        debug_print(verbose, &format!("Displaying {} history entries for selection (after filtering duplicates)", filtered_history.len()));
        let start_index = bookmarks.len();
        for (i, hist_dir) in filtered_history.iter().enumerate() {
            let index = start_index + i;
            // Only show if within the 36-item limit (0-9, a-z)
            if index < 36 {
                let prefix = get_prefix_char(index);
                println!("{} {}", 
                    format!("[{}]", prefix).bright_cyan().bold(),
                    hist_dir.display().to_string().bright_white()
                );
            }
        }
    }
    io::stdout().flush()?;

    // Prompt on stdout (same stream as list for consistency)
    print!("{}", "Select directory (0-9, a-z): ".bright_yellow());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    debug_print(verbose, &format!("User input: '{}'", input.trim()));
    
    let ch = input.trim().chars().next();
    if let Some(ch) = ch {
        if let Some(index) = get_index_from_char(ch) {
            debug_print(verbose, &format!("Parsed index: {}", index));
            // Check if index is in bookmarks
            if index < bookmarks.len() {
                let selected = &bookmarks[index];
                debug_print(verbose, &format!("Selected directory: {}", selected.display()));
                add_to_history(selected.clone(), verbose)?;
                write_target_file(selected, verbose)?;
                return Ok(());
            } 
            // Check if index is in filtered history (accounting for bookmark offset)
            else if index < total_items && index < 36 {
                let history_index = index - bookmarks.len();
                if history_index < filtered_history.len() {
                    let selected = &filtered_history[history_index];
                    debug_print(verbose, &format!("Selected directory: {}", selected.display()));
                    add_to_history(selected.clone(), verbose)?;
                    write_target_file(selected, verbose)?;
                    return Ok(());
                }
            } else {
                debug_print(verbose, &format!("Index {} out of range (max: {})", index, total_items.min(36)));
            }
        } else {
            debug_print(verbose, &format!("Invalid character: '{}'", ch));
        }
    }
    
    eprintln!("{}", "Invalid selection.".red());
    std::process::exit(1);
}

fn choose_directory_by_letter(letter: &str, verbose: bool) -> io::Result<()> {
    debug_print(verbose, &format!("Choosing directory by letter: '{}'", letter));
    let bookmarks = load_bookmarks(verbose);
    let history = load_history(verbose);
    
    // Filter out history entries that are already in bookmarks
    let filtered_history: Vec<PathBuf> = history
        .iter()
        .filter(|hist_dir| !bookmarks.contains(hist_dir))
        .cloned()
        .collect();
    
    let total_items = bookmarks.len() + filtered_history.len();
    
    if total_items == 0 {
        eprintln!("{}", "No bookmarked directories.".yellow());
        std::process::exit(1);
    }

    let ch = letter.chars().next();
    if let Some(ch) = ch {
        if let Some(index) = get_index_from_char(ch) {
            debug_print(verbose, &format!("Parsed index: {}", index));
            // Check if index is in bookmarks
            if index < bookmarks.len() {
                let selected = &bookmarks[index];
                debug_print(verbose, &format!("Selected directory: {}", selected.display()));
                add_to_history(selected.clone(), verbose)?;
                write_target_file(selected, verbose)?;
                return Ok(());
            } 
            // Check if index is in filtered history (accounting for bookmark offset)
            else if index < total_items && index < 36 {
                let history_index = index - bookmarks.len();
                if history_index < filtered_history.len() {
                    let selected = &filtered_history[history_index];
                    debug_print(verbose, &format!("Selected directory: {}", selected.display()));
                    add_to_history(selected.clone(), verbose)?;
                    write_target_file(selected, verbose)?;
                    return Ok(());
                }
            } else {
                debug_print(verbose, &format!("Index {} out of range (max: {})", index, total_items.min(36)));
            }
        } else {
            debug_print(verbose, &format!("Invalid character: '{}'", ch));
        }
    }
    
    eprintln!("{}", format!("Invalid directory letter: {}", letter).red());
    std::process::exit(1);
}

fn change_to_previous(verbose: bool) -> io::Result<()> {
    debug_print(verbose, "Changing to previous directory");
    let history = load_history(verbose);
    
    if history.is_empty() {
        eprintln!("{}", "No directory history.".yellow());
        std::process::exit(1);
    }

    // Get the first entry (most recent)
    let previous = &history[0];
    debug_print(verbose, &format!("Previous directory: {}", previous.display()));
    
    if !previous.exists() {
        debug_print(verbose, "Previous directory no longer exists");
        eprintln!("{}", format!("Previous directory no longer exists: {}", previous.display()).red());
        std::process::exit(1);
    }

    write_target_file(previous, verbose)?;
    Ok(())
}

fn change_up_one_level(verbose: bool) -> io::Result<()> {
    let current = std::env::current_dir()?;
    debug_print(verbose, &format!("Current directory: {}", current.display()));
    
    if let Some(parent) = current.parent() {
        let parent_path = parent.to_path_buf();
        debug_print(verbose, &format!("Parent directory: {}", parent_path.display()));
        add_to_history(parent_path.clone(), verbose)?;
        write_target_file(&parent_path, verbose)?;
        Ok(())
    } else {
        debug_print(verbose, "Already at root directory");
        eprintln!("{}", "Already at root directory.".yellow());
        std::process::exit(1)
    }
}

fn list_subdirectories(verbose: bool) -> io::Result<()> {
    let current = std::env::current_dir()?;
    debug_print(verbose, &format!("Listing subdirectories of: {}", current.display()));
    
    let mut subdirs: Vec<PathBuf> = fs::read_dir(&current)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    debug_print(verbose, &format!("Found {} subdirectories", subdirs.len()));

    if subdirs.is_empty() {
        eprintln!("{}", "No subdirectories found.".yellow());
        std::process::exit(1);
    }

    subdirs.sort();
    debug_print(verbose, "Sorted subdirectories");

    for (i, subdir) in subdirs.iter().enumerate() {
        if i >= 36 {
            break;
        }
        let prefix = get_prefix_char(i);
        let dir_name = subdir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");
        println!("{} {}", 
            format!("[{}]", prefix).bright_cyan().bold(),
            dir_name.bright_white()
        );
    }

    print!("{}", "Select directory (0-9, a-z): ".bright_yellow());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    debug_print(verbose, &format!("User input: '{}'", input.trim()));
    
    let ch = input.trim().chars().next();
    if let Some(ch) = ch {
        if let Some(index) = get_index_from_char(ch) {
            debug_print(verbose, &format!("Parsed index: {}", index));
            if index < subdirs.len() && index < 36 {
                let selected = &subdirs[index];
                debug_print(verbose, &format!("Selected directory: {}", selected.display()));
                add_to_history(selected.clone(), verbose)?;
                write_target_file(selected, verbose)?;
                return Ok(());
            } else {
                debug_print(verbose, &format!("Index {} out of range (max: {})", index, subdirs.len().min(36)));
            }
        } else {
            debug_print(verbose, &format!("Invalid character: '{}'", ch));
        }
    }
    
    eprintln!("{}", "Invalid selection.".red());
    std::process::exit(1);
}

fn find_directory_by_name(name: &str, verbose: bool) -> io::Result<()> {
    let current = std::env::current_dir()?;
    debug_print(verbose, &format!("Searching for directory: '{}'", name));
    debug_print(verbose, &format!("Current directory: {}", current.display()));
    
    // First, check bookmarks
    debug_print(verbose, "Searching in bookmarks");
    let bookmarks = load_bookmarks(verbose);
    for bookmark in bookmarks {
        if let Some(dir_name) = bookmark.file_name() {
            if dir_name.to_string_lossy() == name {
                debug_print(verbose, &format!("Found in bookmarks: {}", bookmark.display()));
                if bookmark.exists() {
                    add_to_history(bookmark.clone(), verbose)?;
                    write_target_file(&bookmark, verbose)?;
                    return Ok(());
                } else {
                    debug_print(verbose, "Bookmark exists but directory does not");
                }
            }
        }
    }
    
    // Then check subdirectories of current directory
    debug_print(verbose, "Searching in current directory subdirectories");
    if let Ok(entries) = fs::read_dir(&current) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name() {
                        if dir_name.to_string_lossy() == name {
                            debug_print(verbose, &format!("Found in subdirectories: {}", path.display()));
                            add_to_history(path.clone(), verbose)?;
                            write_target_file(&path, verbose)?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
    
    // Check parent directories recursively (limited depth)
    debug_print(verbose, "Searching in parent directories");
    let mut search_path = current.clone();
    for depth in 0..5 {
        if let Some(parent) = search_path.parent() {
            search_path = parent.to_path_buf();
            let candidate = search_path.join(name);
            debug_print(verbose, &format!("Checking at depth {}: {}", depth + 1, candidate.display()));
            if candidate.exists() && candidate.is_dir() {
                debug_print(verbose, &format!("Found in parent directories: {}", candidate.display()));
                add_to_history(candidate.clone(), verbose)?;
                write_target_file(&candidate, verbose)?;
                return Ok(());
            }
        } else {
            debug_print(verbose, "Reached root directory");
            break;
        }
    }
    
    debug_print(verbose, "Directory not found in any location");
    eprintln!("{}", format!("Directory not found: {}", name).red());
    std::process::exit(1);
}

fn print_current_directory(verbose: bool) {
    debug_print(verbose, "Printing current directory");
    match std::env::current_dir() {
        Ok(path) => {
            debug_print(verbose, &format!("Current directory: {}", path.display()));
            println!("{}", path.display().to_string().bright_white())
        },
        Err(e) => {
            debug_print(verbose, &format!("Error getting current directory: {}", e));
            eprintln!("{}", format!("Error getting current directory: {}", e).red());
            std::process::exit(1);
        }
    }
}

fn main() {
    // Delete target file on startup if it exists
    // Check for verbose flag early to pass to delete_target_file
    let early_verbose = std::env::args().any(|arg| arg == "-v" || arg == "--verbose");
    if let Err(e) = delete_target_file(early_verbose) {
        eprintln!("Warning: Could not delete target file: {}", e);
    }
    
    // Build the command definition
    let cmd = Command::new("changeDir")
        .about("Intelligent directory bookmarking and navigation")
        .arg(Arg::new("list")
            .short('l')
            .long("list")
            .action(clap::ArgAction::SetTrue)
            .help("List all bookmarked directories"))
        .arg(Arg::new("bookmark")
            .long("bookmark")
            .action(clap::ArgAction::SetTrue)
            .help("Bookmark the current directory"))
        .arg(Arg::new("forget")
            .short('f')
            .long("forget")
            .action(clap::ArgAction::SetTrue)
            .help("Forget the current directory if bookmarked"))
        .arg(Arg::new("forget-all")
            .short('F')
            .long("forget-all")
            .action(clap::ArgAction::SetTrue)
            .help("Forget all bookmarked directories"))
        .arg(Arg::new("choose")
            .short('c')
            .long("choose")
            .num_args(0..=1)
            .help("Choose a directory from bookmarks (with optional letter)"))
        .arg(Arg::new("back")
            .short('b')
            .long("back")
            .action(clap::ArgAction::SetTrue)
            .help("Change to the previous directory"))
        .arg(Arg::new("up")
            .short('u')
            .long("up")
            .action(clap::ArgAction::SetTrue)
            .help("Change up one directory level"))
        .arg(Arg::new("down")
            .short('d')
            .long("down")
            .action(clap::ArgAction::SetTrue)
            .help("List and select a subdirectory"))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .action(clap::ArgAction::SetTrue)
            .help("Enable verbose/debug output"))
        .arg(Arg::new("directory")
            .help("Directory name to change to")
            .index(1));

    // Check for -? help flag
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "-?") {
        cmd.clone().print_help().unwrap();
        std::process::exit(0);
    }

    let matches = cmd.get_matches();
    let verbose = matches.get_flag("verbose");

    if verbose {
        debug_print(verbose, "Verbose mode enabled");
        debug_print(verbose, &format!("Command arguments: {:?}", std::env::args().collect::<Vec<_>>()));
    }

    let result = if matches.get_flag("list") {
        list_bookmarks(verbose)
    } else if matches.get_flag("bookmark") {
        bookmark_current(verbose)
    } else if matches.get_flag("forget") {
        forget_current(verbose)
    } else if matches.get_flag("forget-all") {
        forget_all(verbose)
    } else if matches.contains_id("choose") {
        if let Some(letter) = matches.get_one::<String>("choose") {
            choose_directory_by_letter(letter, verbose)
        } else {
            choose_directory_interactive(verbose)
        }
    } else if matches.get_flag("back") {
        change_to_previous(verbose)
    } else if matches.get_flag("up") {
        change_up_one_level(verbose)
    } else if matches.get_flag("down") {
        list_subdirectories(verbose)
    } else if let Some(dir_name) = matches.get_one::<String>("directory") {
        find_directory_by_name(dir_name, verbose)
    } else {
        print_current_directory(verbose);
        Ok(())
    };

    if let Err(e) = result {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }
}

