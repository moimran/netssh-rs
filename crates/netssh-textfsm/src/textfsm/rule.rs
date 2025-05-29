use crate::textfsm::errors::{Result, TextFSMError};
use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    /// Cache for compiled regex patterns to improve performance
    static ref REGEX_CACHE: Mutex<HashMap<String, Regex>> = Mutex::new(HashMap::new());
}

/// Get or create a cached regex pattern
fn get_cached_regex(pattern: &str) -> Result<Regex> {
    let mut cache = REGEX_CACHE.lock().unwrap();

    if let Some(regex) = cache.get(pattern) {
        // Clone the regex (fancy-regex Regex implements Clone)
        return Ok(regex.clone());
    }

    // Compile new regex
    let regex = Regex::new(pattern).map_err(|e| {
        TextFSMError::TemplateError(format!(
            "Invalid regular expression: '{}'. Error: {}",
            pattern, e
        ))
    })?;

    // Store in cache
    cache.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}

#[derive(Debug, Clone)]
pub struct TextFSMRule {
    pub match_pattern: String,
    pub regex: String,
    pub regex_obj: Regex,
    pub line_op: String,
    pub record_op: String,
    pub new_state: String,
    pub line_num: usize,
}

impl TextFSMRule {
    pub fn new(line: &str, line_num: usize, var_map: &HashMap<String, String>) -> Result<Self> {
        let line = line.trim();
        if line.is_empty() {
            return Err(TextFSMError::TemplateError(format!(
                "Null data in FSMRule. Line: {}",
                line_num
            )));
        }

        let mut rule = Self {
            match_pattern: String::new(),
            regex: String::new(),
            regex_obj: Regex::new("").unwrap(), // Will be replaced
            line_op: String::new(),
            record_op: String::new(),
            new_state: String::new(),
            line_num,
        };

        // Check for '->' action
        if let Some(arrow_pos) = line.find(" ->") {
            rule.match_pattern = line[..arrow_pos].to_string();
            let action_part = &line[arrow_pos + 3..].trim();
            rule.parse_action(action_part)?;
        } else {
            rule.match_pattern = line.to_string();
        }

        // Replace ${varname} entries
        rule.regex = rule.match_pattern.clone();
        for (var_name, var_template) in var_map {
            let placeholder = format!("${{{}}}", var_name);
            rule.regex = rule.regex.replace(&placeholder, var_template);
        }

        // Convert escaped angle brackets to literal angle brackets
        // In TextFSM templates, \< and \> are meant to be literal < and > characters
        rule.regex = rule.regex.replace(r"\<", "<");
        rule.regex = rule.regex.replace(r"\>", ">");

        // Compile regex using cache
        rule.regex_obj = get_cached_regex(&rule.regex)
            .map_err(|e| TextFSMError::TemplateError(format!("{}. Line: {}", e, line_num)))?;

        Ok(rule)
    }

    fn parse_action(&mut self, action: &str) -> Result<()> {
        let parts: Vec<&str> = action.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let mut idx = 0;

        // Parse line and record operations
        if let Some(operation) = parts.get(idx) {
            if operation.contains('.') {
                // Line.Record format
                let op_parts: Vec<&str> = operation.split('.').collect();
                if op_parts.len() == 2 {
                    self.line_op = op_parts[0].to_string();
                    self.record_op = op_parts[1].to_string();
                }
            } else if self.is_line_op(operation) {
                self.line_op = operation.to_string();
            } else if self.is_record_op(operation) {
                self.record_op = operation.to_string();
            } else {
                // Must be a state name
                self.new_state = operation.to_string();
                return Ok(());
            }
            idx += 1;
        }

        // Parse new state
        if let Some(state) = parts.get(idx) {
            self.new_state = state.to_string();
        }

        // Validate combinations
        if self.line_op == "Continue" && !self.new_state.is_empty() {
            return Err(TextFSMError::TemplateError(format!(
                "Action '{}' with new state {} specified. Line: {}",
                self.line_op, self.new_state, self.line_num
            )));
        }

        Ok(())
    }

    fn is_line_op(&self, op: &str) -> bool {
        matches!(op, "Continue" | "Next" | "Error")
    }

    fn is_record_op(&self, op: &str) -> bool {
        matches!(op, "Clear" | "Clearall" | "Record" | "NoRecord")
    }

    pub fn check_match<'a>(&self, line: &'a str) -> Option<fancy_regex::Captures<'a>> {
        self.regex_obj.captures(line).ok().flatten()
    }
}
