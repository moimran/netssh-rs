use crate::textfsm::TextFSM;
use indexmap::IndexMap;
use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Custom error type for parse output operations
pub type ParseOutputResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Template entry from the index file
#[derive(Debug, Clone)]
struct TemplateEntry {
    /// Template filename
    template: String,
    /// Command pattern (may contain regex)
    command: String,
}

/// Network output parser for TextFSM templates
/// 
/// This struct provides functionality to parse network device command outputs
/// using TextFSM templates. It handles template location, selection, and output parsing.
#[derive(Debug)]
pub struct NetworkOutputParser {
    /// Path to the templates directory
    template_dir: PathBuf,
    /// Whether the index has been loaded
    index_loaded: bool,
    /// Platform to templates mapping
    platform_templates: HashMap<String, Vec<TemplateEntry>>,
}

impl NetworkOutputParser {
    /// Create a new NetworkOutputParser
    /// 
    /// # Arguments
    /// * `template_dir` - Optional path to template directory. If None, uses default.
    pub fn new(template_dir: Option<PathBuf>) -> Self {
        let default_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("templates");
        
        Self {
            template_dir: template_dir.unwrap_or(default_dir),
            index_loaded: false,
            platform_templates: HashMap::new(),
        }
    }

    /// Load the template index file and build a dictionary of templates by platform
    fn load_template_index(&mut self) -> ParseOutputResult<&HashMap<String, Vec<TemplateEntry>>> {
        if self.index_loaded {
            return Ok(&self.platform_templates);
        }

        let index_file = self.template_dir.join("index");
        if !index_file.exists() {
            eprintln!("Index file not found at {}", index_file.display());
            self.index_loaded = true;
            return Ok(&self.platform_templates);
        }

        let file = File::open(&index_file)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Find the header line (first non-comment line)
        let mut header_line = None;
        for line in lines.by_ref() {
            let line = line?;
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            header_line = Some(trimmed.to_string());
            break;
        }

        let header = header_line.ok_or("No header found in index file")?;
        
        // Parse header to get column positions
        let headers: Vec<&str> = header.split(',').map(|h| h.trim()).collect();
        
        let template_col = headers.iter().position(|&h| h.to_lowercase() == "template")
            .ok_or("Missing 'template' column in index file")?;
        let platform_col = headers.iter().position(|&h| {
            let lower = h.to_lowercase();
            lower == "platform" || lower == "vendor"
        }).ok_or("Missing 'platform' or 'vendor' column in index file")?;
        let command_col = headers.iter().position(|&h| h.to_lowercase() == "command")
            .ok_or("Missing 'command' column in index file")?;

        // Parse the remaining lines
        for line in lines {
            let line = line?;
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Parse CSV line (simple split for now, could be enhanced for quoted fields)
            let fields: Vec<&str> = trimmed.split(',').map(|f| f.trim()).collect();
            
            if fields.len() <= template_col.max(platform_col).max(command_col) {
                continue;
            }

            let template_file = fields[template_col].to_string();
            let platform = fields[platform_col].to_lowercase();
            let command_pattern = fields[command_col];

            // Convert command pattern: remove [[]] completion patterns
            // sh[[ow]] ver[[sion]] -> show version
            let command = self.normalize_command_pattern(command_pattern);

            let entry = TemplateEntry {
                template: template_file,
                command,
            };

            self.platform_templates
                .entry(platform)
                .or_insert_with(Vec::new)
                .push(entry);
        }

        self.index_loaded = true;
        println!("Loaded templates for {} platforms", self.platform_templates.len());
        Ok(&self.platform_templates)
    }

    /// Normalize command pattern by removing completion brackets
    /// Converts "sh[[ow]] ver[[sion]]" to "show version"
    fn normalize_command_pattern(&self, pattern: &str) -> String {
        let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        re.replace_all(pattern, "$1").to_lowercase()
    }

    /// Find the appropriate template for a given platform and command
    /// 
    /// # Arguments
    /// * `platform` - Device platform (e.g., "cisco_ios")
    /// * `command` - Command string (e.g., "show version")
    /// 
    /// # Returns
    /// Path to template file or None if not found
    pub fn find_template(&mut self, platform: &str, command: &str) -> ParseOutputResult<Option<PathBuf>> {
        let templates = self.load_template_index()?;
        let platform_lower = platform.to_lowercase();
        let command_lower = command.to_lowercase();

        let platform_entries = match templates.get(&platform_lower) {
            Some(entries) => entries.clone(),
            None => {
                eprintln!("No templates found for platform '{}'", platform);
                return Ok(None);
            }
        };

        println!(
            "Looking for command '{}' in platform '{}' with {} templates",
            command, platform, platform_entries.len()
        );

        // Clone template_dir to avoid borrowing issues
        let template_dir = self.template_dir.clone();

        // First try exact regex match
        for entry in &platform_entries {
            if let Ok(re) = Regex::new(&entry.command) {
                if re.is_match(&command_lower) {
                    let template_path = template_dir.join(&entry.template);
                    if template_path.exists() {
                        println!("Found exact match template: {}", template_path.display());
                        return Ok(Some(template_path));
                    }
                }
            }
        }

        // Then try substring match
        for entry in &platform_entries {
            if command_lower.contains(&entry.command) || entry.command.contains(&command_lower) {
                let template_path = template_dir.join(&entry.template);
                if template_path.exists() {
                    println!("Found substring match template: {}", template_path.display());
                    return Ok(Some(template_path));
                }
            }
        }

        eprintln!("No template found for '{}' command '{}'", platform, command);
        Ok(None)
    }

    /// Parse command output using TextFSM
    /// 
    /// # Arguments
    /// * `platform` - Device platform (e.g., "cisco_ios")
    /// * `command` - Command string (e.g., "show version")
    /// * `data` - Command output as string
    /// 
    /// # Returns
    /// Vector of dictionaries containing parsed data, or None if parsing fails
    pub fn parse_output(
        &mut self,
        platform: &str,
        command: &str,
        data: &str,
    ) -> ParseOutputResult<Option<Vec<IndexMap<String, serde_json::Value>>>> {
        if data.is_empty() {
            eprintln!("Empty output provided for parsing");
            return Ok(None);
        }

        let template_path = match self.find_template(platform, command)? {
            Some(path) => path,
            None => {
                eprintln!("No template found for {}, {}", platform, command);
                return Ok(None);
            }
        };

        println!("Parsing output using template: {}", template_path.display());
        
        let template_file = File::open(&template_path)?;
        
        let mut fsm = TextFSM::new(template_file)?;
        let parsed_data = fsm.parse_text_to_dicts(data)?;
        
        Ok(Some(parsed_data))
    }

    /// Parse command output and return as JSON string
    /// 
    /// # Arguments
    /// * `platform` - Device platform (e.g., "cisco_ios")
    /// * `command` - Command string (e.g., "show version")
    /// * `data` - Command output as string
    /// 
    /// # Returns
    /// JSON string of parsed data, or None if parsing fails
    pub fn parse_to_json(
        &mut self,
        platform: &str,
        command: &str,
        data: &str,
    ) -> ParseOutputResult<Option<String>> {
        match self.parse_output(platform, command, data)? {
            Some(result) => {
                let json = serde_json::to_string_pretty(&result)?;
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }
}

// Global parser instance for backward compatibility
lazy_static::lazy_static! {
    static ref GLOBAL_PARSER: std::sync::Mutex<NetworkOutputParser> = 
        std::sync::Mutex::new(NetworkOutputParser::new(None));
}

/// Parse command output using TextFSM (function interface)
/// 
/// # Arguments
/// * `platform` - Device platform (e.g., "cisco_ios")
/// * `command` - Command string (e.g., "show version")
/// * `data` - Command output as string
/// 
/// # Returns
/// Vector of dictionaries containing parsed data, or None if parsing fails
pub fn parse_output(
    platform: &str,
    command: &str,
    data: &str,
) -> ParseOutputResult<Option<Vec<IndexMap<String, serde_json::Value>>>> {
    let mut parser = GLOBAL_PARSER.lock().unwrap();
    parser.parse_output(platform, command, data)
}

/// Parse command output and return as JSON string (function interface)
/// 
/// # Arguments
/// * `platform` - Device platform (e.g., "cisco_ios")
/// * `command` - Command string (e.g., "show version")
/// * `data` - Command output as string
/// 
/// # Returns
/// JSON string of parsed data, or None if parsing fails
pub fn parse_output_to_json(
    platform: &str,
    command: &str,
    data: &str,
) -> ParseOutputResult<Option<String>> {
    let mut parser = GLOBAL_PARSER.lock().unwrap();
    parser.parse_to_json(platform, command, data)
}
