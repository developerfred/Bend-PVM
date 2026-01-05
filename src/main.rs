#![allow(dead_code)]
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use bend_pvm::debugger::{DebugInfo, Debugger};
use bend_pvm::formatter::Formatter;
use bend_pvm::{compile, generate_riscv_from_source, CompilerOptions};

#[derive(Parser, Debug)]
#[command(name = "bend-pvm")]
#[command(author = "Codingsh <codingsh@pm.me>")]
#[command(version = bend_pvm::version())]
#[command(about = "Bend Programming Language with PolkaVM Integration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable automatic behavior (e.g., auto-formatting, auto-optimization)
    #[arg(short = 'a', long = "auto")]
    auto: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compile a Bend source file
    Compile {
        /// Bend source file
        #[arg(required = true)]
        file: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Disable optimizations
        #[arg(short = 'O', long)]
        no_optimize: bool,

        /// Generate debug information
        #[arg(short, long)]
        debug: bool,

        /// Disable type checking
        #[arg(short = 'T', long)]
        no_type_check: bool,

        /// Output assembly
        #[arg(short, long)]
        assembly: bool,

        /// Disable metadata generation
        #[arg(short = 'M', long)]
        no_metadata: bool,

        /// Disable ABI generation
        #[arg(short = 'A', long)]
        no_abi: bool,
    },

    /// Check a Bend source file for errors
    Check {
        /// Bend source file
        #[arg(required = true)]
        file: PathBuf,

        /// Disable type checking
        #[arg(short = 'T', long)]
        no_type_check: bool,
    },

    /// Run a Bend source file
    Run {
        /// Bend source file
        #[arg(required = true)]
        file: PathBuf,

        /// Disable optimizations
        #[arg(short = 'O', long)]
        no_optimize: bool,

        /// Step through instructions
        #[arg(short, long)]
        step: bool,

        /// Set initial breakpoint at line
        #[arg(short, long)]
        breakpoint: Option<usize>,
    },

    /// Format a Bend source file
    Format {
        /// Bend source file
        #[arg(required = true)]
        file: PathBuf,

        /// Output file (defaults to overwriting the input file)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Check if the file is formatted correctly
        #[arg(short, long)]
        check: bool,
    },

    /// Initialize a new Bend project
    Init {
        /// Project name
        #[arg(required = true)]
        name: String,

        /// Project directory (defaults to a new directory with the project name)
        #[arg(short, long)]
        directory: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            file,
            output,
            no_optimize,
            debug,
            no_type_check,
            assembly,
            no_metadata,
            no_abi,
        } => {
            // Handle auto flag behavior
            let optimize = if cli.auto { !no_optimize } else { !no_optimize };

            let type_check = if cli.auto {
                !no_type_check
            } else {
                !no_type_check
            };

            // Determine output path if not specified
            let output = output.or_else(|| {
                file.file_stem().map(|stem| {
                    let mut output = PathBuf::from(stem);
                    output.set_extension("bin");
                    output
                })
            });

            // Set compiler options
            let options = CompilerOptions {
                output,
                optimize,
                debug,
                type_check,
                assembly,
                metadata: !no_metadata,
                abi: !no_abi,
                security_scan: true,
                static_analysis: true,
                fuzz_testing: false,
                security_level: 2,
            };

            // Compile file
            compile(&file, options)?;

            println!("Compilation successful.");
        }

        Commands::Check {
            file,
            no_type_check,
        } => {
            // Handle auto flag behavior
            let type_check = if cli.auto {
                !no_type_check
            } else {
                !no_type_check
            };

            // Set compiler options for checking
            let options = CompilerOptions {
                output: None,
                optimize: false,
                debug: false,
                type_check,
                assembly: false,
                metadata: false,
                abi: false,
                security_scan: true,
                static_analysis: true,
                fuzz_testing: false,
                security_level: 2,
            };

            // Check file
            compile(&file, options)?;

            println!("No errors found.");
        }

        Commands::Run {
            file,
            no_optimize,
            step,
            breakpoint,
        } => {
            // Read source file
            let source = std::fs::read_to_string(&file)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            // Generate RISC-V instructions
            let optimize = !no_optimize;
            let instructions = generate_riscv_from_source(&source, optimize)
                .map_err(|e| format!("Failed to generate code: {}", e))?;

            println!("Generated {} RISC-V instructions", instructions.len());

            if instructions.is_empty() {
                println!("No instructions generated. Make sure the source file contains a valid main function.");
                return Ok(());
            }

            // Create debug info (basic)
            let debug_info = DebugInfo {
                source_path: file.clone(),
                source_code: source.clone(),
                line_to_instruction: std::collections::HashMap::new(),
                instruction_to_line: std::collections::HashMap::new(),
                locals: std::collections::HashMap::new(),
                functions: std::collections::HashMap::new(),
            };

            // Create context with default values
            let context = bend_pvm::runtime::env::ExecutionContext::new_default();

            // Create debugger
            let mut debugger = Debugger::new(debug_info, instructions, context);

            // Set breakpoint if specified
            if let Some(line) = breakpoint {
                use bend_pvm::debugger::Breakpoint;
                debugger
                    .add_breakpoint(Breakpoint::Line(line))
                    .map_err(|e| format!("Failed to set breakpoint: {}", e))?;
                println!("Breakpoint set at line {}", line);
            }

            // Set event handler to print state
            debugger.set_event_handler(|event| match event {
                bend_pvm::debugger::DebuggerEvent::Started => {
                    println!("Program started");
                }
                bend_pvm::debugger::DebuggerEvent::Stepped => {
                    println!("Stepped");
                }
                bend_pvm::debugger::DebuggerEvent::Continued => {
                    println!("Continuing...");
                }
                bend_pvm::debugger::DebuggerEvent::Finished => {
                    println!("Program finished successfully");
                }
                bend_pvm::debugger::DebuggerEvent::Breakpoint(bp) => {
                    println!("Breakpoint reached: {:?}", bp);
                }
                bend_pvm::debugger::DebuggerEvent::Crashed(msg) => {
                    println!("Program crashed: {}", msg);
                }
            });

            if step {
                // Step through instructions
                println!("Starting stepped execution...");
                loop {
                    match debugger.step() {
                        Ok(()) => {
                            // Check if program has finished
                            if debugger.state().execution_state
                                == bend_pvm::debugger::state::ExecutionState::Stopped
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Execution error: {}", e);
                            break;
                        }
                    }
                }
            } else {
                // Run to completion or breakpoint
                println!("Running program...");
                match debugger.run() {
                    Ok(()) => {
                        println!("Execution completed");
                    }
                    Err(e) => {
                        return Err(format!("Execution failed: {}", e).into());
                    }
                }
            }

            println!("Execution finished.");
        }

        Commands::Format {
            file,
            output,
            check,
        } => {
            let mut formatter = Formatter::new();

            if check {
                // Check if file is formatted
                let source = std::fs::read_to_string(&file)
                    .map_err(|e| format!("Failed to read file: {}", e))?;

                if formatter.is_formatted(&source) {
                    println!("File is already formatted.");
                } else {
                    println!("File needs formatting.");
                    return Err("File is not formatted".into());
                }
            } else {
                // Format the file
                let output_path = output.as_ref().unwrap_or(&file);

                if file == *output_path {
                    // Format in-place
                    match formatter.format_file_in_place(&file) {
                        Ok(modified) => {
                            if modified {
                                println!("Formatted: {}", file.display());
                            } else {
                                println!("File is already formatted: {}", file.display());
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to format file: {}", e);
                            return Err(e);
                        }
                    }
                } else {
                    // Format to different output file
                    match formatter.format_file(&file) {
                        Ok(bend_pvm::formatter::FormatResult::Formatted(formatted)) => {
                            std::fs::write(output_path, formatted)
                                .map_err(|e| format!("Failed to write output: {}", e))?;
                            println!("Formatted: {} -> {}", file.display(), output_path.display());
                        }
                        Ok(bend_pvm::formatter::FormatResult::AlreadyFormatted) => {
                            println!("File is already formatted: {}", file.display());
                            // Copy file to output if different
                            if file != *output_path {
                                std::fs::copy(&file, output_path)
                                    .map_err(|e| format!("Failed to copy file: {}", e))?;
                                println!("Copied: {} -> {}", file.display(), output_path.display());
                            }
                        }
                        Ok(bend_pvm::formatter::FormatResult::NeedsFormatting) => {
                            eprintln!("File needs formatting but was not processed");
                            return Err("Formatting error".into());
                        }
                        Ok(bend_pvm::formatter::FormatResult::Error(e)) => {
                            eprintln!("Failed to format file: {}", e);
                            return Err(e.into());
                        }
                        Err(e) => {
                            eprintln!("Failed to format file: {}", e);
                            return Err(e);
                        }
                    }
                }
            }
        }

        Commands::Init { name, directory } => {
            // Determine project directory
            let project_dir = directory.unwrap_or_else(|| PathBuf::from(&name));

            // Create project directory
            std::fs::create_dir_all(&project_dir)?;

            // Create project structure
            create_project_structure(&project_dir, &name)?;

            if cli.auto {
                // In auto mode, also initialize with default dependencies
                println!(
                    "Auto-initializing project '{}' with default dependencies.",
                    name
                );
                // TODO: Add default dependencies to bend.toml
            }

            println!("Project '{}' initialized in {:?}.", name, project_dir);
        }
    }

    Ok(())
}

fn create_project_structure(project_dir: &Path, name: &str) -> std::io::Result<()> {
    // Create main source file
    let main_file = project_dir.join("src").join("main.bend");
    std::fs::create_dir_all(main_file.parent().unwrap())?;

    std::fs::write(
        &main_file,
        format!(
            r#"
#{{{name}}}
# A smart contract written in Bend-PVM.

def main() -> u24:
    return 42
"#
        ),
    )?;

    // Create project configuration
    let config_file = project_dir.join("bend.toml");
    std::fs::write(
        &config_file,
        format!(
            r#"
[package]
name = "{name}"
version = "0.1.0"
authors = ["Your Name <your.email@example.com>"]

[dependencies]
# Add your dependencies here
"#
        ),
    )?;

    // Create README.md
    let readme_file = project_dir.join("README.md");
    std::fs::write(
        &readme_file,
        format!(
            r#"
# {name}

A smart contract written in Bend-PVM.

## Building

```
bend-pvm compile src/main.bend
```

## Testing

```
bend-pvm check src/main.bend
```
"#
        ),
    )?;

    // Create .gitignore
    let gitignore_file = project_dir.join(".gitignore");
    std::fs::write(
        &gitignore_file,
        r#"
# Build artifacts
*.bin
*.s
*.metadata.json
*.abi.json

# Editor files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db
"#,
    )?;

    Ok(())
}
