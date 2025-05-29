use crate::textfsm::errors::{Result, TextFSMError};
use crate::textfsm::options::{OptionType, ValueOption};
use fancy_regex::Regex;

#[derive(Debug, Clone)]
pub struct TextFSMValue {
    pub name: String,
    pub regex: String,
    pub template: String,
    pub value: String,
    pub options: Vec<ValueOption>,
    pub compiled_regex: Option<Regex>,
}

impl TextFSMValue {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            regex: String::new(),
            template: String::new(),
            value: String::new(),
            options: Vec::new(),
            compiled_regex: None,
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(TextFSMError::TemplateError(
                "Expect at least 3 tokens on line".to_string(),
            ));
        }

        if parts[0] != "Value" {
            return Err(TextFSMError::TemplateError(
                "Line must start with 'Value'".to_string(),
            ));
        }

        let mut idx = 1;

        // Check if there are options
        // If the current part doesn't start with '(' and the next part also doesn't start with '(',
        // then the current part contains options
        if idx < parts.len() && !parts[idx].starts_with('(') {
            // Look ahead to see if the next part starts with '(' (which would be the regex)
            let has_options = idx + 1 < parts.len() && !parts[idx + 1].starts_with('(');

            if has_options {
                // Parse options
                let options_str = parts[idx];
                for option in options_str.split(',') {
                    self.add_option(option)?;
                }
                idx += 1;
            }
        }

        if idx >= parts.len() {
            return Err(TextFSMError::TemplateError(
                "Missing value name and regex".to_string(),
            ));
        }

        self.name = parts[idx].to_string();
        idx += 1;

        if idx >= parts.len() {
            return Err(TextFSMError::TemplateError(
                "Missing regex pattern".to_string(),
            ));
        }

        // Join the remaining parts as the regex
        self.regex = parts[idx..].join(" ");

        // Validate regex format
        if !self.regex.starts_with('(') || !self.regex.ends_with(')') {
            return Err(TextFSMError::TemplateError(format!(
                "Value '{}' must be contained within a '()' pair",
                self.regex
            )));
        }

        // Create template with named group
        self.template = self.regex.replacen('(', &format!("(?P<{}>", self.name), 1);

        // Compile regex for List options
        if self.has_option(&OptionType::List) {
            self.compiled_regex = Some(Regex::new(&self.regex)?);
        }

        // Call option creation callbacks
        for option in &mut self.options {
            option.on_create_options()?;
        }

        Ok(())
    }

    fn add_option(&mut self, name: &str) -> Result<()> {
        // Check for duplicate options
        if self.has_option_name(name) {
            return Err(TextFSMError::TemplateError(format!(
                "Duplicate option \"{}\"",
                name
            )));
        }

        let option_type = OptionType::from_str(name)
            .ok_or_else(|| TextFSMError::TemplateError(format!("Unknown option \"{}\"", name)))?;

        self.options.push(ValueOption::new(option_type));
        Ok(())
    }

    pub fn has_option(&self, option_type: &OptionType) -> bool {
        self.options.iter().any(|opt| {
            std::mem::discriminant(&opt.option_type) == std::mem::discriminant(option_type)
        })
    }

    pub fn has_option_str(&self, option_name: &str) -> bool {
        self.has_option_name(option_name)
    }

    fn has_option_name(&self, name: &str) -> bool {
        self.options
            .iter()
            .any(|opt| match (&opt.option_type, name) {
                (OptionType::Required, "Required") => true,
                (OptionType::Filldown, "Filldown") => true,
                (OptionType::Fillup, "Fillup") => true,
                (OptionType::Key, "Key") => true,
                (OptionType::List, "List") => true,
                _ => false,
            })
    }

    pub fn assign_var(&mut self, value: &str) -> Result<()> {
        self.value = value.to_string();
        for option in &mut self.options {
            option.on_assign_var(value)?;
        }
        Ok(())
    }

    pub fn clear_var(&mut self) -> Result<()> {
        let mut new_value = String::new();
        for option in &mut self.options {
            let result = option.on_clear_var(&self.value)?;
            if !result.is_empty() {
                new_value = result;
            }
        }
        self.value = new_value;
        Ok(())
    }

    pub fn clear_all_var(&mut self) -> Result<()> {
        self.value = String::new();
        for option in &mut self.options {
            option.on_clear_all_var()?;
        }
        Ok(())
    }

    pub fn header(&self) -> String {
        self.name.clone()
    }

    pub fn on_save_record(&self) -> Result<String> {
        let mut result = self.value.clone();
        for option in &self.options {
            result = option.on_save_record(&result)?;
        }
        Ok(result)
    }

    pub fn option_names(&self) -> Vec<String> {
        self.options
            .iter()
            .map(|opt| match opt.option_type {
                OptionType::Required => "Required".to_string(),
                OptionType::Filldown => "Filldown".to_string(),
                OptionType::Fillup => "Fillup".to_string(),
                OptionType::Key => "Key".to_string(),
                OptionType::List => "List".to_string(),
            })
            .collect()
    }
}
