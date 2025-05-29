use crate::textfsm::errors::{Result, TextFSMError};
use crate::textfsm::rule::TextFSMRule;
use crate::textfsm::value::TextFSMValue;
use fancy_regex::Regex;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::Read;

lazy_static! {
    /// Pre-compiled common regex patterns for better performance
    static ref COMMENT_REGEX: Regex = Regex::new(r"^\s*#").unwrap();
}

#[derive(Debug)]
pub struct TextFSM {
    pub values: Vec<TextFSMValue>,
    pub states: HashMap<String, Vec<TextFSMRule>>,
    pub state_list: Vec<String>,
    pub value_map: HashMap<String, String>,
    pub current_state: String,
    pub result: Vec<Vec<String>>,
    line_num: usize,
}

impl TextFSM {
    pub fn new<R: Read>(mut template: R) -> Result<Self> {
        let mut content = String::new();
        template.read_to_string(&mut content)?;

        let mut fsm = Self {
            values: Vec::new(),
            states: HashMap::new(),
            state_list: Vec::new(),
            value_map: HashMap::new(),
            current_state: "Start".to_string(),
            result: Vec::new(),
            line_num: 0,
        };

        fsm.parse_template(&content)?;
        fsm.reset();
        Ok(fsm)
    }

    fn parse_template(&mut self, content: &str) -> Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        let mut line_idx = 0;

        // Parse values section
        while line_idx < lines.len() {
            self.line_num = line_idx + 1;
            let line = lines[line_idx].trim();

            if line.is_empty() {
                line_idx += 1;
                break; // End of values section
            }

            if COMMENT_REGEX.is_match(line).unwrap_or(false) {
                line_idx += 1;
                continue; // Skip comments
            }

            if line.starts_with("Value ") {
                let mut value = TextFSMValue::new();
                value.parse(line)?;

                // Check for duplicate value names
                if self.values.iter().any(|v| v.name == value.name) {
                    return Err(TextFSMError::TemplateError(format!(
                        "Duplicate declarations for Value '{}'. Line: {}",
                        value.name, self.line_num
                    )));
                }

                self.value_map
                    .insert(value.name.clone(), value.template.clone());
                self.values.push(value);
            } else if self.values.is_empty() {
                return Err(TextFSMError::TemplateError(
                    "No Value definitions found".to_string(),
                ));
            } else {
                return Err(TextFSMError::TemplateError(format!(
                    "Expected blank line after last Value entry. Line: {}",
                    self.line_num
                )));
            }

            line_idx += 1;
        }

        // Parse states section
        while line_idx < lines.len() {
            if let Some(_state_name) = self.parse_state(&lines, &mut line_idx)? {
                // State was parsed successfully
            } else {
                break; // No more states
            }
        }

        self.validate_fsm()?;
        Ok(())
    }

    fn parse_state(&mut self, lines: &[&str], line_idx: &mut usize) -> Result<Option<String>> {
        // Skip empty lines and comments
        while *line_idx < lines.len() {
            self.line_num = *line_idx + 1;
            let line = lines[*line_idx].trim();

            if !line.is_empty() && !COMMENT_REGEX.is_match(line).unwrap_or(false) {
                // Found state name
                let state_name = line.to_string();

                // Validate state name
                if state_name.len() > 48
                    || !state_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    return Err(TextFSMError::TemplateError(format!(
                        "Invalid state name: '{}'. Line: {}",
                        state_name, self.line_num
                    )));
                }

                if self.states.contains_key(&state_name) {
                    return Err(TextFSMError::TemplateError(format!(
                        "Duplicate state name: '{}'. Line: {}",
                        state_name, self.line_num
                    )));
                }

                self.states.insert(state_name.clone(), Vec::new());
                self.state_list.push(state_name.clone());
                *line_idx += 1;

                // Parse rules for this state
                self.parse_state_rules(&state_name, lines, line_idx)?;

                return Ok(Some(state_name));
            }
            *line_idx += 1;
        }

        Ok(None)
    }

    fn parse_state_rules(
        &mut self,
        state_name: &str,
        lines: &[&str],
        line_idx: &mut usize,
    ) -> Result<()> {
        while *line_idx < lines.len() {
            self.line_num = *line_idx + 1;
            let line = lines[*line_idx];

            if line.trim().is_empty() {
                *line_idx += 1;
                break; // End of state rules
            }

            if COMMENT_REGEX.is_match(line).unwrap_or(false) {
                *line_idx += 1;
                continue; // Skip comments
            }

            // Rules must start with whitespace and ^
            if !line.starts_with("  ^") && !line.starts_with(" ^") && !line.starts_with("\t^") {
                return Err(TextFSMError::TemplateError(format!(
                    "Missing white space or carat ('^') before rule. Line: {}",
                    self.line_num
                )));
            }

            let rule = TextFSMRule::new(line, self.line_num, &self.value_map)?;
            self.states.get_mut(state_name).unwrap().push(rule);

            *line_idx += 1;
        }

        Ok(())
    }

    fn validate_fsm(&self) -> Result<()> {
        // Must have 'Start' state
        if !self.states.contains_key("Start") {
            return Err(TextFSMError::TemplateError(
                "Missing state 'Start'".to_string(),
            ));
        }

        // Validate state transitions
        for (state_name, rules) in &self.states {
            for rule in rules {
                if rule.line_op == "Error" {
                    continue;
                }

                if !rule.new_state.is_empty()
                    && rule.new_state != "End"
                    && rule.new_state != "EOF"
                    && !self.states.contains_key(&rule.new_state)
                {
                    return Err(TextFSMError::TemplateError(format!(
                        "State '{}' not found, referenced in state '{}'",
                        rule.new_state, state_name
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.current_state = "Start".to_string();
        self.result.clear();
        self.clear_all_record();
    }

    pub fn header(&self) -> Vec<String> {
        self.values.iter().map(|v| v.header()).collect()
    }

    pub fn parse_text(&mut self, text: &str) -> Result<Vec<Vec<String>>> {
        let lines: Vec<&str> = text.lines().collect();

        for line in lines {
            self.check_line(line)?;
            if self.current_state == "End" || self.current_state == "EOF" {
                break;
            }
        }

        // Implicit EOF performs Next.Record operation
        if self.current_state != "End" && !self.states.contains_key("EOF") {
            self.append_record()?;
        }

        Ok(self.result.clone())
    }

    pub fn parse_text_to_dicts(&mut self, text: &str) -> Result<Vec<IndexMap<String, JsonValue>>> {
        let result_lists = self.parse_text(text)?;
        let header = self.header();
        let mut result_dicts = Vec::new();

        for row in result_lists {
            let mut dict = IndexMap::new();
            // Insert fields in the order they appear in the template to match Python behavior
            for (i, key) in header.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    // Check if this value is from a List option
                    let json_value = if let Some(textfsm_value) = self.values.get(i) {
                        if textfsm_value.has_option_str("List") {
                            // Parse the JSON string back to a proper JSON array
                            match serde_json::from_str::<JsonValue>(value) {
                                Ok(parsed) => parsed,
                                Err(_) => JsonValue::String(value.clone()), // Fallback to string if parsing fails
                            }
                        } else {
                            JsonValue::String(value.clone())
                        }
                    } else {
                        JsonValue::String(value.clone())
                    };

                    dict.insert(key.clone(), json_value);
                }
            }
            result_dicts.push(dict);
        }

        Ok(result_dicts)
    }

    fn check_line(&mut self, line: &str) -> Result<()> {
        let state_rules = self.states.get(&self.current_state).unwrap().clone();

        for rule in &state_rules {
            if let Some(captures) = rule.check_match(line) {
                // Assign captured values and collect fillup operations
                let mut fillup_operations = Vec::new();
                for value in &mut self.values {
                    if let Some(matched) = captures.name(&value.name) {
                        value.assign_var(matched.as_str())?;

                        // Collect Fillup values for later processing
                        if value.has_option_str("Fillup") && !matched.as_str().is_empty() {
                            fillup_operations
                                .push((value.name.clone(), matched.as_str().to_string()));
                        }
                    }
                }

                // Apply fillup operations
                for (name, val) in fillup_operations {
                    self.fillup_value(&name, &val)?;
                }

                // Process operations
                if self.process_operations(&rule, line)? {
                    // State transition
                    if !rule.new_state.is_empty() {
                        if rule.new_state != "End" && rule.new_state != "EOF" {
                            self.current_state = rule.new_state.clone();
                        } else {
                            self.current_state = rule.new_state.clone();
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    fn process_operations(&mut self, rule: &TextFSMRule, line: &str) -> Result<bool> {
        // Process record operations first
        match rule.record_op.as_str() {
            "Record" => self.append_record()?,
            "Clear" => self.clear_record()?,
            "Clearall" => self.clear_all_record(),
            _ => {} // NoRecord or empty
        }

        // Process line operations
        match rule.line_op.as_str() {
            "Error" => {
                let error_msg = if !rule.new_state.is_empty() {
                    format!(
                        "Error: {}. Rule Line: {}. Input Line: {}",
                        rule.new_state, rule.line_num, line
                    )
                } else {
                    format!(
                        "State Error raised. Rule Line: {}. Input Line: {}",
                        rule.line_num, line
                    )
                };
                return Err(TextFSMError::FSMError(error_msg));
            }
            "Continue" => return Ok(false), // Continue with current line
            _ => {}                         // Next or empty (default)
        }

        Ok(true) // Move to next line
    }

    fn append_record(&mut self) -> Result<()> {
        if self.values.is_empty() {
            return Ok(());
        }

        let mut record = Vec::new();
        let mut skip_record = false;

        for value in &self.values {
            match value.on_save_record() {
                Ok(val) => record.push(val),
                Err(TextFSMError::FSMError(msg)) if msg == "SkipRecord" => {
                    skip_record = true;
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        if skip_record {
            self.clear_record()?;
            return Ok(());
        }

        // Check if record is all empty
        if record.iter().all(|v| v.is_empty() || v == "[]") {
            return Ok(());
        }

        // Replace empty strings with ""
        for item in &mut record {
            if item.is_empty() {
                *item = String::new();
            }
        }

        self.result.push(record);
        self.clear_record()?;
        Ok(())
    }

    fn clear_record(&mut self) -> Result<()> {
        for value in &mut self.values {
            value.clear_var()?;
        }
        Ok(())
    }

    fn clear_all_record(&mut self) {
        for value in &mut self.values {
            let _ = value.clear_all_var();
        }
    }

    fn fillup_value(&mut self, value_name: &str, value: &str) -> Result<()> {
        // Find the index of the value in the values list
        let value_idx = self.values.iter().position(|v| v.name == value_name);
        if let Some(idx) = value_idx {
            // Go up the result list from the end until we see a filled value
            for record in self.result.iter_mut().rev() {
                if record.len() > idx && !record[idx].is_empty() {
                    // Stop when a record has this column already filled
                    break;
                }
                // Otherwise set the column value
                if record.len() > idx {
                    record[idx] = value.to_string();
                }
            }
        }
        Ok(())
    }
}
