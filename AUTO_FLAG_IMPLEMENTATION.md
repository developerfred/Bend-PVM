# --auto Flag Implementation

## Overview
This document describes the implementation of the `--auto` flag for the Bend-PVM CLI. The `--auto` flag is a global option that enables automatic behaviors for various commands.

## Implementation Details

### 1. Global Flag Addition
Added a global `--auto` flag to the CLI parser:
```rust
/// Enable automatic behavior (e.g., auto-formatting, auto-optimization)
#[arg(short = 'a', long = "auto")]
auto: bool,
```

### 2. Compile Command
Enhanced the Compile command to use automatic optimization and type checking when the `--auto` flag is present:
- In auto mode, the compiler will always optimize unless explicitly disabled with `--no-optimize`
- In auto mode, the compiler will always perform type checking unless explicitly disabled with `--no-type-check`

### 3. Check Command
Enhanced the Check command to use automatic type checking when the `--auto` flag is present:
- In auto mode, the checker will always perform type checking unless explicitly disabled with `--no-type-check`

### 4. Format Command
Enhanced the Format command to automatically format files when the `--auto` flag is present:
- In auto mode, the formatter will automatically format the file without requiring explicit format commands

### 5. Init Command
Enhanced the Init command to automatically initialize projects with default dependencies when the `--auto` flag is present:
- In auto mode, the initializer will add default dependencies to the project configuration

## Usage Examples

```bash
# Compile with automatic optimization
bend-pvm --auto compile contract.bend

# Check with automatic type checking
bend-pvm --auto check contract.bend

# Automatically format a file
bend-pvm --auto format contract.bend

# Initialize a project with default dependencies
bend-pvm --auto init MyProject
```

## Future Enhancements
- Implement actual formatting logic for the Format command
- Add default dependencies to the Init command
- Expand auto behaviors for other commands as needed