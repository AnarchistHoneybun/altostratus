use clap::{Parser, ArgAction};

#[derive(Parser)]
#[command(name = "altostratus")]
#[command(about = "Visualize 3D point files in the terminal!", long_about = None)]
#[command(version)]
pub struct Args {
    /// Point cloud file to visualize
    #[arg(value_name = "FILE")]
    pub file: Option<String>,
    
    /// Load multiple point cloud files
    #[arg(short = 'f', long = "files", value_name = "FILES", action = ArgAction::Append)]
    pub files: Vec<String>,
    
    /// Show detailed help information
    #[arg(long = "help-detailed", hide = true)]
    pub detailed_help: bool,
}

pub enum ParseResult {
    ShowUsage,
    ShowDetailedHelp,
    LoadFiles(Vec<String>),
}

pub fn parse_arguments() -> ParseResult {
    // Handle no arguments case
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        return ParseResult::ShowUsage;
    }
    
    // Check for help flags manually to show our custom help
    if args.len() == 2 && (args[1] == "-h" || args[1] == "--help") {
        return ParseResult::ShowDetailedHelp;
    }
    
    let args = Args::parse();
    
    // Collect all files from both positional and flag arguments
    let mut all_files = Vec::new();
    
    if let Some(file) = args.file {
        all_files.push(file);
    }
    
    all_files.extend(args.files);
    
    if all_files.is_empty() {
        ParseResult::ShowUsage
    } else {
        ParseResult::LoadFiles(all_files)
    }
}

pub fn print_usage() {
    println!("Usage: altostratus [FILE] | altostratus -f [FILES...]");
    println!("       altostratus --help | -h for detailed help");
    println!();
    println!("Examples:");
    println!("  altostratus points.txt              # Load single file");
    println!("  altostratus -f file1.txt file2.txt  # Load multiple files");
}

pub fn print_detailed_help() {
    const HELP_MSG: &str = "\
\x1b[1mAltostratus\x1b[0m: Visualize 3D point files in the terminal!

\x1b[1mUsage\x1b[0m:
    \"altostratus <filepath.txt>\": Interactively view the provided point file.
    \"altostratus -f <file1.txt> <file2.txt> ...\": Load multiple point files.
    \"altostratus --help\", \"altostratus -h\": Show this help message.
    \"altostratus\": Show usage examples.

\x1b[1mFile Format\x1b[0m:
    Each line should contain three space-separated coordinates: x y z

\x1b[1mControls\x1b[0m:
    Scroll down to zoom out, scroll up to zoom in.
    Click and drag the mouse to rotate around the data.
    Click and drag the mouse while holding [ctrl] to pan.
    Press [/] to enter command mode and load new datasets.
    Press [Ctrl+C] to exit.
";
    
    print!("{}", HELP_MSG);
}