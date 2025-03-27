use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Write};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "bend-fmt")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "Bend code formatter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Format a file
    Format {
        /// File to format
        #[arg(required = true)]
        file: PathBuf,

        /// Output file (defaults to overwriting the input file)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Check if the file is formatted correctly without modifying it
        #[arg(short, long)]
        check: bool,

        /// Indent using tabs instead of spaces
        #[arg(short, long)]
        tabs: bool,

        /// Number of spaces for indentation
        #[arg(short, long, default_value_t = 4)]
        indent: usize,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Format { file, output, check, tabs, indent } => {
            let content = if file.as_os_str() == "-" {
                // Read from stdin
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                // Read from file
                fs::read_to_string(&file)?
            };

            // Format the content
            let formatted = format_content(&content, tabs, indent)?;

            if check {
                // Check if the file is already formatted correctly
                if content == formatted {
                    println!("{} is already formatted correctly.", file.display());
                    return Ok(());
                } else {
                    eprintln!("{} would be reformatted.", file.display());
                    return Err(io::Error::new(io::ErrorKind::Other, "File is not formatted correctly"));
                }
            }

            // Write the formatted content
            if let Some(output_file) = output {
                // Write to output file
                fs::write(&output_file, formatted)?;
                println!("Formatted {} and wrote to {}.", file.display(), output_file.display());
            } else if file.as_os_str() == "-" {
                // Write to stdout
                io::stdout().write_all(formatted.as_bytes())?;
            } else {
                // Overwrite input file
                fs::write(&file, formatted)?;
                println!("Formatted {}.", file.display());
            }
        }
    }

    Ok(())
}

/// Format Bend code content
fn format_content(content: &str, use_tabs: bool, indent_size: usize) -> io::Result<String> {
    // This is a simplified formatter that just handles basic indentation
    // A real formatter would parse the code and format it more comprehensively
    
    let mut formatted = String::new();
    let mut indent_level = 0;
    let mut in_string = false;
    let mut in_comment = false;
    let mut in_multiline_comment = false;
    
    let indent_str = if use_tabs {
        "\t".to_string()
    } else {
        " ".repeat(indent_size)
    };
    
    let lines = content.lines().collect::<Vec<_>>();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip empty lines
        if trimmed.is_empty() {
            formatted.push_str("\n");
            continue;
        }
        
        // Handle comments
        if trimmed.starts_with("#") && !in_string {
            formatted.push_str(line);
            formatted.push_str("\n");
            continue;
        }
        
        // Check for multiline comment start
        if trimmed.starts_with("#{") && !in_string && !in_multiline_comment {
            in_multiline_comment = true;
            formatted.push_str(line);
            formatted.push_str("\n");
            continue;
        }
        
        // Check for multiline comment end
        if trimmed.contains("}#") && in_multiline_comment {
            in_multiline_comment = false;
            formatted.push_str(line);
            formatted.push_str("\n");
            continue;
        }
        
        // Inside multiline comment
        if in_multiline_comment {
            formatted.push_str(line);
            formatted.push_str("\n");
            continue;
        }
        
        // Handle indentation for next line
        if trimmed.ends_with(":") {
            indent_level += 1;
        }
        
        // Reduce indentation for closing blocks
        if trimmed == "else:" || trimmed.starts_with("elif ") || trimmed.starts_with("case ") {
            indent_level = indent_level.saturating_sub(1);
        }
        
        // Add indentation
        for _ in 0..indent_level {
            formatted.push_str(&indent_str);
        }
        
        // Add the line content
        formatted.push_str(trimmed);
        formatted.push_str("\n");
        
        // Check for block end
        if i + 1 < lines.len() {
            let next_line = lines[i + 1].trim();
            if !next_line.is_empty() && !next_line.starts_with("#") && 
               !trimmed.ends_with(":") && !trimmed.ends_with(",") &&
               (next_line.starts_with("else:") || next_line.starts_with("elif ") || 
                next_line.starts_with("case ")) {
                indent_level = indent_level.saturating_sub(1);
            }
        }
    }
    
    // Remove trailing newline
    if formatted.ends_with("\n") {
        formatted.truncate(formatted.len() - 1);
    }
    
    Ok(formatted)
}