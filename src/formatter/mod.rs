use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub indent_size: usize,
    pub max_line_length: usize,
    pub use_tabs: bool,
    pub blank_line_after_fn: bool,
    pub space_around_operators: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_line_length: 120,
            use_tabs: false,
            blank_line_after_fn: true,
            space_around_operators: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FormatResult {
    Formatted(String),
    AlreadyFormatted,
    NeedsFormatting,
    Error(String),
}

pub struct Formatter {
    config: FormatterConfig,
    indent_stack: Vec<usize>,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
            indent_stack: vec![0],
        }
    }

    pub fn with_config(config: FormatterConfig) -> Self {
        Self {
            config,
            indent_stack: vec![0],
        }
    }

    pub fn format_source(&mut self, source: &str) -> Result<String, String> {
        let lines: Vec<&str> = source.lines().collect();
        let mut result: Vec<String> = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim_end();
            let trimmed = line.trim_start();

            if trimmed.is_empty() {
                if !result.is_empty() && !result.last().unwrap().is_empty() {
                    result.push(String::new());
                }
                i += 1;
                continue;
            }

            let prev_ends_with_brace = result
                .last()
                .map(|l| l.trim_end().ends_with('}'))
                .unwrap_or(false);
            let prev_line_ends_with_brace_or_is_fn = if !result.is_empty() {
                let prev = result.last().unwrap().trim();
                prev.ends_with('}')
            } else {
                false
            };

            let (processed_line, indent_change) = self.process_line(trimmed, prev_ends_with_brace);

            if self.config.blank_line_after_fn
                && prev_line_ends_with_brace_or_is_fn
                && !processed_line.is_empty()
                && !result.is_empty()
                && !result.last().unwrap().is_empty() {
                    result.push(String::new());
                }

            if let Some(change) = indent_change {
                if change > 0 {
                    let current = *self.indent_stack.last().unwrap();
                    self.indent_stack.push(current + change as usize);
                } else {
                    for _ in 0..change.abs() {
                        if self.indent_stack.len() > 1 {
                            self.indent_stack.pop();
                        }
                    }
                }
            }

            let indent_level = *self.indent_stack.last().unwrap();
            let indent_str = self.get_indent(indent_level);
            result.push(format!("{}{}", indent_str, processed_line));

            i += 1;
        }

        let joined = result.join("\n");
        Ok(self.normalize_whitespace(&joined))
    }

    fn process_line(&self, line: &str, _prev_ends_with_brace: bool) -> (String, Option<isize>) {
        let body = line.trim_start();
        let opens_block = body.contains('{');
        let closes_block = body.contains('}');

        let normalized_body = self.normalize_spaces(body);
        let indent_change = if opens_block && !closes_block {
            Some(1)
        } else if closes_block && !opens_block {
            Some(-1)
        } else {
            None
        };

        (normalized_body, indent_change)
    }

    fn normalize_spaces(&self, line: &str) -> String {
        let mut result = line.to_string();

        result = result.replace("->", " -> ");

        let re_multi_space = regex::Regex::new(r"\s{2,}").unwrap();
        result = re_multi_space.replace_all(&result, " ").to_string();

        result = self.normalize_params(&result);

        result = self.normalize_operators(&result);

        result = result.replace("( ", "(");
        result = result.replace(" )", ")");
        result = result.replace("{ ", "{");
        result = result.replace(" }", "}");

        result.trim().to_string()
    }

    fn normalize_operators(&self, line: &str) -> String {
        let mut result = line.to_string();
        let operators = [
            "+", "-", "*", "/", "%", "=", "==", "!=", "<", ">", "<=", ">=",
        ];

        for op in operators {
            let with_spaces = format!(" {} ", op);
            let without_spaces = format!("{}{}", op, op);
            result = result.replace(&without_spaces, &with_spaces);

            let pattern = regex::Regex::new(&format!(r"(\w)({})", regex::escape(op))).unwrap();
            result = pattern
                .replace_all(&result, &format!("$1{}", with_spaces))
                .to_string();

            let pattern = regex::Regex::new(&format!(r"({})(\w)", regex::escape(op),)).unwrap();
            result = pattern
                .replace_all(&result, &format!("{} $1$3", with_spaces))
                .to_string();
        }

        result
    }

    fn normalize_params(&self, s: &str) -> String {
        let re = regex::Regex::new(r"\(([^)]*)\)").unwrap();

        re.replace_all(s, |caps: &regex::Captures| {
            let inner = caps.get(1).map_or("", |m| m.as_str());
            let normalized = self.normalize_param_inner(inner);
            format!("({})", normalized)
        })
        .to_string()
    }

    fn normalize_param_inner(&self, inner: &str) -> String {
        let trimmed = inner.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let parts: Vec<&str> = trimmed.split(',').collect();
        let normalized_parts: Vec<String> = parts
            .iter()
            .map(|part| {
                let p = part.trim();
                if p.is_empty() {
                    String::new()
                } else {
                    let re_spaces = regex::Regex::new(r"\s+").unwrap();
                    let normalized = re_spaces.replace_all(p, " ").to_string();
                    if normalized.contains(':') && !normalized.contains(": ") {
                        let normalized = normalized.replace(":", ": ");
                        normalized.trim().to_string()
                    } else {
                        normalized.trim().to_string()
                    }
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        normalized_parts.join(", ")
    }

    fn normalize_parens(&self, line: &str) -> String {
        let mut result = String::new();
        let mut in_parens = false;
        let chars: Vec<char> = line.chars().collect();

        for i in 0..chars.len() {
            let c = chars[i];

            match c {
                '(' | '[' => {
                    in_parens = true;
                    result.push(c);
                    if i + 1 < chars.len() && chars[i + 1] == ' ' {
                        continue;
                    }
                }
                ')' | ']' => {
                    in_parens = false;
                    if result.ends_with(' ') {
                        result.pop();
                    }
                    result.push(c);
                    if i + 1 < chars.len() && chars[i + 1] == ' ' {
                        continue;
                    }
                }
                ',' if in_parens => {
                    result.push(',');
                    if i + 1 < chars.len() && chars[i + 1] != ' ' {
                        result.push(' ');
                    }
                }
                ' ' if i > 0 && i < chars.len() - 1 => {
                    let prev = chars[i - 1];
                    let next = chars[i + 1];
                    if prev == ' ' || next == ' ' {
                        continue;
                    }
                    if prev == '('
                        || prev == '['
                        || prev == ','
                        || next == ')'
                        || next == ']'
                        || next == ','
                    {
                        continue;
                    }
                    result.push(c);
                }
                _ => result.push(c),
            }
        }

        result.replace("  ", " ")
    }

    fn get_indent(&self, level: usize) -> String {
        if self.config.use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * self.config.indent_size)
        }
    }

    fn normalize_whitespace(&self, code: &str) -> String {
        let mut result = String::new();
        let mut blank_line_pending = false;

        for line in code.lines() {
            let trimmed = line.trim_end();

            if trimmed.is_empty() {
                blank_line_pending = true;
            } else {
                if blank_line_pending && !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(trimmed);
                result.push('\n');
                blank_line_pending = false;
            }
        }

        result.trim_end().to_string()
    }

    /// Check if code is already properly formatted
    pub fn is_formatted(&mut self, source: &str) -> bool {
        match self.format_source(source) {
            Ok(formatted) => source.trim() == formatted.trim(),
            Err(_) => false,
        }
    }

    /// Format a file and return result
    pub fn format_file(
        &mut self,
        file_path: &Path,
    ) -> Result<FormatResult, Box<dyn std::error::Error>> {
        let source = fs::read_to_string(file_path)?;

        if self.is_formatted(&source) {
            return Ok(FormatResult::AlreadyFormatted);
        }

        let formatted = self
            .format_source(&source)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(FormatResult::Formatted(formatted))
    }

    /// Format a file and write result
    pub fn format_file_in_place(
        &mut self,
        file_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match self.format_file(file_path)? {
            FormatResult::Formatted(formatted) => {
                fs::write(file_path, formatted)?;
                Ok(true)
            }
            FormatResult::AlreadyFormatted => Ok(false),
            FormatResult::NeedsFormatting => Ok(true),
            FormatResult::Error(e) => Err(Box::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            ))),
        }
    }

    /// Format all Bend files in a directory recursively
    pub fn format_directory(
        &mut self,
        dir_path: &Path,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut formatted_files = Vec::new();

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                formatted_files.extend(self.format_directory(&path)?);
            } else if path.extension().is_some_and(|ext| ext == "bend") {
                if let FormatResult::Formatted(_) = self.format_file(&path)? {
                    formatted_files.push(path.display().to_string());
                }
            }
        }

        Ok(formatted_files)
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let mut formatter = Formatter::new();

        let unformatted = r#"fn test(  x:i32,  y:i32  ) -> i32 {
  return   x   +   y;
  }"#;

        let result = formatter.format_source(unformatted);
        assert!(result.is_ok());

        if let Ok(formatted) = result {
            println!("Formatted: {:?}", formatted);
            println!(
                "Contains fn pattern: {}",
                formatted.contains("fn test(x: i32, y: i32) -> i32")
            );
            println!(
                "Contains return pattern: {}",
                formatted.contains("return x + y")
            );
            assert!(formatted.contains("fn test(x: i32, y: i32) -> i32"));
            assert!(formatted.contains("return x + y"));
        }
    }

    #[test]
    fn test_already_formatted() {
        let mut formatter = Formatter::new();

        let formatted = r#"fn test(x: i32) -> i32 {
    return x;
}"#;

        assert!(formatter.is_formatted(formatted));
    }

    #[test]
    fn test_indentation() {
        let mut formatter = Formatter::new();

        let unformatted = r#"fn test() -> i32 {
 return 1;
 }"#;

        let result = formatter.format_source(unformatted).unwrap();
        assert!(result.contains("    return 1"));
    }

    #[test]
    fn test_multiple_blank_lines() {
        let mut formatter = Formatter::new();

        let unformatted = r#"fn test() -> i32 {



    return 1;
 }"#;

        let result = formatter.format_source(unformatted).unwrap();
        assert_eq!(result.matches("\n\n").count(), 1);
    }
}
