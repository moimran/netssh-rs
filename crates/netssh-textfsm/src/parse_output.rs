use crate::textfsm::TextFSM;
use indexmap::IndexMap;
use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Custom error type for parse output operations
pub type ParseOutputResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Compiled template entry with pre-compiled regex and validated path
#[derive(Debug, Clone)]
struct CompiledTemplateEntry {
    /// Pre-compiled command regex pattern
    command_regex: Option<Regex>,
    /// Original command pattern for fallback
    command_pattern: String,
    /// Full path to template file (pre-validated)
    template_path: PathBuf,
}

/// Global compiled index cache
#[derive(Debug, Clone)]
struct CompiledIndex {
    /// Platform to compiled templates mapping
    platform_templates: HashMap<String, Vec<CompiledTemplateEntry>>,
}

lazy_static::lazy_static! {
    /// Pre-compiled regex for command pattern normalization
    static ref COMPLETION_PATTERN: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    /// Global cache for compiled indexes
    static ref GLOBAL_INDEX_CACHE: Arc<Mutex<HashMap<String, CompiledIndex>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Get global index cache
fn get_global_cache() -> Arc<Mutex<HashMap<String, CompiledIndex>>> {
    GLOBAL_INDEX_CACHE.clone()
}

/// Convert completion patterns to regex patterns
/// Converts "sh[[ow]]" to "sh(o(w)?)?" and "ver[[sion]]" to "ver(s(i(o(n)?)?)?)?)"
fn convert_completion_to_regex(pattern: &str) -> String {
    COMPLETION_PATTERN.replace_all(pattern, |caps: &regex::Captures| {
        let word = &caps[1];
        let mut result = String::new();
        for ch in word.chars() {
            result.push('(');
            result.push(ch);
        }
        result.push_str(&")?".repeat(word.len()));
        result
    }).to_string()
}

/// Normalize command pattern by removing completion brackets and converting to lowercase
fn normalize_command_pattern(pattern: &str) -> String {
    COMPLETION_PATTERN.replace_all(pattern, "$1").to_lowercase()
}

/// Network output parser for TextFSM templates
///
/// This struct provides functionality to parse network device command outputs
/// using TextFSM templates. It handles template location, selection, and output parsing.
#[derive(Debug)]
pub struct NetworkOutputParser {
    /// Path to the templates directory
    template_dir: PathBuf,
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
        }
    }

    /// Get or load the compiled index for this template directory
    fn get_compiled_index(&self) -> ParseOutputResult<CompiledIndex> {
        let cache = get_global_cache();
        let cache_key = self.template_dir.to_string_lossy().to_string();

        // Check if already cached
        {
            let cache_guard = cache.lock().unwrap();
            if let Some(compiled_index) = cache_guard.get(&cache_key) {
                return Ok(compiled_index.clone());
            }
        }

        // Load and compile index
        let compiled_index = self.load_and_compile_index()?;

        // Cache the compiled index
        {
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.insert(cache_key, compiled_index.clone());
        }

        Ok(compiled_index)
    }

    /// Load and compile the template index file
    fn load_and_compile_index(&self) -> ParseOutputResult<CompiledIndex> {
        let index_file = self.template_dir.join("index");
        if !index_file.exists() {
            eprintln!("Index file not found at {}", index_file.display());
            return Ok(CompiledIndex {
                platform_templates: HashMap::new(),
            });
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

        let mut platform_templates: HashMap<String, Vec<CompiledTemplateEntry>> = HashMap::new();

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

            let template_files = fields[template_col];
            let platform = fields[platform_col].to_lowercase();
            let command_pattern = fields[command_col];

            // Handle multiple template files separated by colons - use only the first one
            let template_file = template_files.split(':').next().unwrap_or(template_files).to_string();

            // Build full template path and validate existence
            let template_path = self.template_dir.join(&template_file);
            if !template_path.exists() {
                // Skip silently - some templates may not exist in this distribution
                continue;
            }

            // Convert completion patterns to regex and compile
            let regex_pattern = convert_completion_to_regex(command_pattern);
            let command_regex = match Regex::new(&regex_pattern) {
                Ok(regex) => Some(regex),
                Err(_) => {
                    // Skip patterns that can't be compiled (e.g., lookahead/lookbehind)
                    None
                }
            };

            let entry = CompiledTemplateEntry {
                command_regex,
                command_pattern: command_pattern.to_string(),
                template_path,
            };

            platform_templates
                .entry(platform)
                .or_insert_with(Vec::new)
                .push(entry);
        }


        Ok(CompiledIndex {
            platform_templates,
        })
    }

    /// Find the appropriate template for a given platform and command
    ///
    /// # Arguments
    /// * `platform` - Device platform (e.g., "cisco_ios")
    /// * `command` - Command string (e.g., "show version")
    ///
    /// # Returns
    /// Path to template file or None if not found
    pub fn find_template(&self, platform: &str, command: &str) -> ParseOutputResult<Option<PathBuf>> {
        let compiled_index = self.get_compiled_index()?;
        let platform_lower = platform.to_lowercase();
        let command_lower = command.to_lowercase();

        let platform_entries = match compiled_index.platform_templates.get(&platform_lower) {
            Some(entries) => entries,
            None => {
                return Ok(None);
            }
        };

        // First try pre-compiled regex match (fast!)
        for entry in platform_entries {
            if let Some(ref regex) = entry.command_regex {
                if regex.is_match(&command_lower) {
                    return Ok(Some(entry.template_path.clone()));
                }
            }
        }

        // Fallback to substring match for patterns that couldn't be compiled as regex
        for entry in platform_entries {
            let normalized_pattern = normalize_command_pattern(&entry.command_pattern);
            if command_lower.contains(&normalized_pattern) || normalized_pattern.contains(&command_lower) {
                return Ok(Some(entry.template_path.clone()));
            }
        }

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
        &self,
        platform: &str,
        command: &str,
        data: &str,
    ) -> ParseOutputResult<Option<Vec<IndexMap<String, serde_json::Value>>>> {
        if data.is_empty() {
            return Ok(None);
        }

        let template_path = match self.find_template(platform, command)? {
            Some(path) => path,
            None => {
                return Ok(None);
            }
        };

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
        &self,
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
    let parser = GLOBAL_PARSER.lock().unwrap();
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
    let parser = GLOBAL_PARSER.lock().unwrap();
    parser.parse_to_json(platform, command, data)
}
