pub mod textfsm;
pub mod parse_output;
pub mod config;

pub use textfsm::*;
pub use parse_output::*;
pub use config::*;

#[cfg(test)]
mod tests {
    use super::*;
    use colored::*;
    use serde_json::{self, Value as JsonValue};
    use std::fs::{self, File};
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    /// Test configuration and command-line arguments
    #[derive(Debug)]
    struct TestConfig {
        generate_json: bool,
        tests_dir: PathBuf,
        vendor_filter: Option<String>,
        command_filter: Option<String>,
        max_failures: usize,
        continue_on_failure: bool,
    }

    impl TestConfig {
        fn from_env() -> Self {
            let generate_json = std::env::var("GENERATE_JSON")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            let tests_dir = std::env::var("TESTS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("tests"));

            let vendor_filter = std::env::var("VENDOR_FILTER").ok();
            let command_filter = std::env::var("COMMAND_FILTER").ok();

            let max_failures = std::env::var("MAX_FAILURES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10);

            let continue_on_failure = std::env::var("CONTINUE_ON_FAILURE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            Self {
                generate_json,
                tests_dir,
                vendor_filter,
                command_filter,
                max_failures,
                continue_on_failure,
            }
        }
    }

    /// Statistics for test execution
    #[derive(Debug, Default)]
    struct TestStats {
        directories_processed: usize,
        files_processed: usize,
        files_successful: usize,
        files_failed: usize,
        files_matching: usize,
        files_differing: usize,
        parsing_errors: Vec<String>,
        comparison_errors: Vec<String>,
    }

    /// Result of processing a single command directory
    #[derive(Debug)]
    struct DirectoryResult {
        path: PathBuf,
        template_file: Option<PathBuf>,
        raw_files: Vec<PathBuf>,
        rust_outputs: Vec<(PathBuf, std::result::Result<(), String>)>,
        comparisons: Vec<ComparisonResult>,
    }

    /// Result of comparing Rust vs Python output
    #[derive(Debug)]
    struct ComparisonResult {
        raw_file: PathBuf,
        rust_file: PathBuf,
        python_file: PathBuf,
        matches: bool,
        error: Option<String>,
        differences: Option<String>,
    }

    /// Comprehensive TextFSM validation test with filtering support
    ///
    /// Environment variables:
    /// - GENERATE_JSON=true/false: Generate JSON files vs compare with existing
    /// - VENDOR_FILTER=<string>: Only test vendors containing this string (e.g., "cisco")
    /// - COMMAND_FILTER=<string>: Only test commands containing this string (e.g., "show_interface")
    /// - MAX_FAILURES=<number>: Stop after this many failures (default: 10)
    /// - CONTINUE_ON_FAILURE=true/false: Continue processing after failures (default: false)
    /// - TESTS_DIR=<path>: Path to tests directory (default: "tests")
    ///
    /// Examples:
    /// ```bash
    /// # Generate JSON for all cisco vendors
    /// GENERATE_JSON=true VENDOR_FILTER=cisco cargo test test_comprehensive_textfsm_validation -- --nocapture
    ///
    /// # Test only show_interface commands
    /// COMMAND_FILTER=show_interface cargo test test_comprehensive_textfsm_validation -- --nocapture
    ///
    /// # Test cisco_asa show_interface specifically
    /// VENDOR_FILTER=cisco_asa COMMAND_FILTER=show_interface cargo test test_comprehensive_textfsm_validation -- --nocapture
    ///
    /// # Continue processing even after failures, with higher failure limit
    /// CONTINUE_ON_FAILURE=true MAX_FAILURES=50 cargo test test_comprehensive_textfsm_validation -- --nocapture
    /// ```

    #[test]
    fn test_comprehensive_textfsm_validation() {
        let config = TestConfig::from_env();

        println!("{}", "=".repeat(80).bright_blue());
        println!(
            "{}",
            "TextFSM Comprehensive Validation Test Suite"
                .bright_blue()
                .bold()
        );
        println!("{}", "=".repeat(80).bright_blue());
        println!("Configuration:");
        println!("  Tests directory: {}", config.tests_dir.display());
        println!("  Generate JSON: {}", config.generate_json);
        if let Some(ref vendor) = config.vendor_filter {
            println!("  Vendor filter: {}", vendor.bright_yellow());
        }
        if let Some(ref command) = config.command_filter {
            println!("  Command filter: {}", command.bright_yellow());
        }
        println!("  Max failures: {}", config.max_failures);
        println!("  Continue on failure: {}", config.continue_on_failure);
        println!();

        let mut stats = TestStats::default();
        let mut directory_results = Vec::new();

        // Find all command directories
        let command_dirs = find_command_directories(&config.tests_dir, &config);
        println!(
            "Found {} command directories to process",
            command_dirs.len()
        );
        println!();

        // Process each directory
        for (i, dir_path) in command_dirs.iter().enumerate() {
            print!(
                "Processing [{}/{}] {} ... ",
                i + 1,
                command_dirs.len(),
                dir_path
                    .strip_prefix(&config.tests_dir)
                    .unwrap_or(dir_path)
                    .display()
            );

            match process_command_directory(dir_path, &config) {
                Ok(result) => {
                    stats.directories_processed += 1;
                    stats.files_processed += result.raw_files.len();

                    let mut dir_has_failures = false;

                    // Count successful/failed parsing
                    for (_, parse_result) in &result.rust_outputs {
                        match parse_result {
                            Ok(_) => stats.files_successful += 1,
                            Err(e) => {
                                stats.files_failed += 1;
                                dir_has_failures = true;
                                stats
                                    .parsing_errors
                                    .push(format!("{}: {}", dir_path.display(), e));
                            }
                        }
                    }

                    // Count matching/differing comparisons
                    for comparison in &result.comparisons {
                        if comparison.matches {
                            stats.files_matching += 1;
                        } else {
                            stats.files_differing += 1;
                            dir_has_failures = true;
                            if let Some(error) = &comparison.error {
                                stats.comparison_errors.push(format!(
                                    "{}: {}",
                                    comparison.raw_file.display(),
                                    error
                                ));
                            }
                        }
                    }

                    if dir_has_failures {
                        println!("{}", "FAILED".red());
                    } else {
                        println!("{}", "OK".green());
                    }
                    directory_results.push(result);

                    // Check if we should stop due to too many failures
                    if !config.continue_on_failure
                        && stats.parsing_errors.len() >= config.max_failures
                    {
                        println!();
                        println!(
                            "{}",
                            format!(
                                "Stopping after {} failures (max_failures reached)",
                                config.max_failures
                            )
                            .yellow()
                        );
                        break;
                    }
                }
                Err(e) => {
                    println!("{}: {}", "ERROR".red(), e);
                    stats
                        .parsing_errors
                        .push(format!("{}: {}", dir_path.display(), e));

                    // Check if we should stop due to too many failures
                    if !config.continue_on_failure
                        && stats.parsing_errors.len() >= config.max_failures
                    {
                        println!();
                        println!(
                            "{}",
                            format!(
                                "Stopping after {} failures (max_failures reached)",
                                config.max_failures
                            )
                            .yellow()
                        );
                        break;
                    }
                }
            }
        }

        // Print detailed test report
        print_test_report(&stats, &directory_results, &config);

        // Assert overall success
        if stats.files_failed > 0 {
            panic!("Test failed: {} files failed to parse", stats.files_failed);
        }

        if !config.generate_json && stats.files_differing > 0 {
            panic!(
                "Test failed: {} files have differences between Rust and Python outputs",
                stats.files_differing
            );
        }
    }

    /// Find all command directories in the tests directory
    fn find_command_directories(tests_dir: &Path, config: &TestConfig) -> Vec<PathBuf> {
        let mut command_dirs = Vec::new();

        if !tests_dir.exists() {
            return command_dirs;
        }

        // Walk through vendor directories (e.g., cisco_asa, juniper_junos)
        for vendor_entry in WalkDir::new(tests_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let vendor_path = vendor_entry.path();

            // Skip special directories
            let vendor_name = vendor_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if vendor_name.starts_with('.')
                || vendor_name == "__pycache__"
                || vendor_name == "mocks"
            {
                continue;
            }

            // Apply vendor filter if specified
            if let Some(ref filter) = config.vendor_filter {
                if !vendor_name.contains(filter) {
                    continue;
                }
            }

            // Walk through command directories within vendor directories
            for command_entry in WalkDir::new(vendor_path)
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_dir())
            {
                let command_path = command_entry.path();

                // Apply command filter if specified
                if let Some(ref filter) = config.command_filter {
                    let command_name = command_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    if !command_name.contains(filter) {
                        continue;
                    }
                }

                // Check if this directory has .textfsm and .raw files
                if has_textfsm_files(command_path) {
                    command_dirs.push(command_path.to_path_buf());
                }
            }
        }

        command_dirs.sort();
        command_dirs
    }

    /// Check if a directory contains TextFSM template and raw files
    fn has_textfsm_files(dir: &Path) -> bool {
        let entries: Vec<_> = fs::read_dir(dir)
            .map(|entries| entries.filter_map(|e| e.ok()).collect())
            .unwrap_or_default();

        let has_textfsm = entries.iter().any(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "textfsm")
                .unwrap_or(false)
        });

        let has_raw = entries.iter().any(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "raw")
                .unwrap_or(false)
        });

        has_textfsm && has_raw
    }

    /// Process a single command directory
    fn process_command_directory(
        dir_path: &Path,
        config: &TestConfig,
    ) -> std::result::Result<DirectoryResult, String> {
        // Find template file
        let template_file = find_template_file(dir_path)?;

        // Find all raw files
        let raw_files = find_raw_files(dir_path)?;

        if raw_files.is_empty() {
            return Err("No raw files found".to_string());
        }

        let mut rust_outputs = Vec::new();
        let mut comparisons = Vec::new();

        // Create rust output directory if generating JSON
        if config.generate_json {
            let rust_dir = dir_path.join("rust");
            if let Err(e) = fs::create_dir_all(&rust_dir) {
                return Err(format!("Failed to create rust directory: {}", e));
            }
        }

        // Process each raw file
        for raw_file in &raw_files {
            let raw_filename = raw_file
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid raw filename")?;

            let rust_output_file = dir_path.join("rust").join(format!("{}.json", raw_filename));

            // Parse with Rust TextFSM
            let parse_result = if config.generate_json {
                parse_and_save_rust(template_file.as_ref().unwrap(), raw_file, &rust_output_file)
            } else {
                // Just validate parsing without saving
                validate_rust_parsing(template_file.as_ref().unwrap(), raw_file)
            };

            rust_outputs.push((rust_output_file.clone(), parse_result));

            // Compare with Python output if not generating
            if !config.generate_json {
                let python_output_file = dir_path
                    .join("python")
                    .join(format!("{}.json", raw_filename));
                let comparison = compare_outputs(raw_file, &rust_output_file, &python_output_file);
                comparisons.push(comparison);
            }
        }

        Ok(DirectoryResult {
            path: dir_path.to_path_buf(),
            template_file,
            raw_files,
            rust_outputs,
            comparisons,
        })
    }

    /// Find the TextFSM template file in a directory
    fn find_template_file(dir: &Path) -> std::result::Result<Option<PathBuf>, String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut textfsm_files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "textfsm")
                .unwrap_or(false)
            {
                textfsm_files.push(path);
            }
        }

        match textfsm_files.len() {
            0 => Ok(None),
            1 => Ok(Some(textfsm_files[0].clone())),
            _ => {
                // If multiple templates, prefer one matching directory name
                let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

                for template_file in &textfsm_files {
                    if let Some(filename) = template_file.file_name().and_then(|n| n.to_str()) {
                        if filename.contains(dir_name) {
                            return Ok(Some(template_file.clone()));
                        }
                    }
                }

                // Return first one if no match
                Ok(Some(textfsm_files[0].clone()))
            }
        }
    }

    /// Find all raw files in a directory
    fn find_raw_files(dir: &Path) -> std::result::Result<Vec<PathBuf>, String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut raw_files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "raw")
                .unwrap_or(false)
            {
                raw_files.push(path);
            }
        }

        raw_files.sort();
        Ok(raw_files)
    }

    /// Parse a raw file with Rust TextFSM and save to JSON
    fn parse_and_save_rust(
        template_file: &Path,
        raw_file: &Path,
        output_file: &Path,
    ) -> std::result::Result<(), String> {
        // Read template
        let template = File::open(template_file).map_err(|e| {
            format!(
                "Failed to open template file '{}': {}",
                template_file.display(),
                e
            )
        })?;

        let mut fsm = TextFSM::new(template).map_err(|e| {
            format!(
                "Failed to create TextFSM from template '{}': {}",
                template_file.display(),
                e
            )
        })?;

        // Read raw data
        let mut raw_content = String::new();
        let mut raw_file_handle = File::open(raw_file)
            .map_err(|e| format!("Failed to open raw file '{}': {}", raw_file.display(), e))?;
        raw_file_handle
            .read_to_string(&mut raw_content)
            .map_err(|e| format!("Failed to read raw file '{}': {}", raw_file.display(), e))?;

        // Parse
        let parsed_data = fsm.parse_text_to_dicts(&raw_content).map_err(|e| {
            format!(
                "Failed to parse raw file '{}' with template '{}': {}",
                raw_file.display(),
                template_file.display(),
                e
            )
        })?;

        // Save to JSON
        let json_output = serde_json::to_string_pretty(&parsed_data)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;

        fs::write(output_file, json_output).map_err(|e| {
            format!(
                "Failed to write output file '{}': {}",
                output_file.display(),
                e
            )
        })?;

        Ok(())
    }

    /// Validate Rust parsing without saving (for comparison mode)
    fn validate_rust_parsing(
        template_file: &Path,
        raw_file: &Path,
    ) -> std::result::Result<(), String> {
        // Read template
        let template = File::open(template_file).map_err(|e| {
            format!(
                "Failed to open template file '{}': {}",
                template_file.display(),
                e
            )
        })?;

        let mut fsm = TextFSM::new(template).map_err(|e| {
            format!(
                "Failed to create TextFSM from template '{}': {}",
                template_file.display(),
                e
            )
        })?;

        // Read raw data
        let mut raw_content = String::new();
        let mut raw_file_handle = File::open(raw_file)
            .map_err(|e| format!("Failed to open raw file '{}': {}", raw_file.display(), e))?;
        raw_file_handle
            .read_to_string(&mut raw_content)
            .map_err(|e| format!("Failed to read raw file '{}': {}", raw_file.display(), e))?;

        // Parse (just validate, don't save)
        let _parsed_data = fsm.parse_text_to_dicts(&raw_content).map_err(|e| {
            format!(
                "Failed to parse raw file '{}' with template '{}': {}",
                raw_file.display(),
                template_file.display(),
                e
            )
        })?;

        Ok(())
    }

    /// Compare Rust and Python JSON outputs
    fn compare_outputs(raw_file: &Path, rust_file: &Path, python_file: &Path) -> ComparisonResult {
        let mut result = ComparisonResult {
            raw_file: raw_file.to_path_buf(),
            rust_file: rust_file.to_path_buf(),
            python_file: python_file.to_path_buf(),
            matches: false,
            error: None,
            differences: None,
        };

        // Check if both files exist
        if !rust_file.exists() {
            result.error = Some("Rust output file does not exist".to_string());
            return result;
        }

        if !python_file.exists() {
            result.error = Some("Python output file does not exist".to_string());
            return result;
        }

        // Read and parse JSON files
        let rust_content = match fs::read_to_string(rust_file) {
            Ok(content) => content,
            Err(e) => {
                result.error = Some(format!("Failed to read Rust file: {}", e));
                return result;
            }
        };

        let python_content = match fs::read_to_string(python_file) {
            Ok(content) => content,
            Err(e) => {
                result.error = Some(format!("Failed to read Python file: {}", e));
                return result;
            }
        };

        let rust_json: JsonValue = match serde_json::from_str(&rust_content) {
            Ok(json) => json,
            Err(e) => {
                result.error = Some(format!("Failed to parse Rust JSON: {}", e));
                return result;
            }
        };

        let python_json: JsonValue = match serde_json::from_str(&python_content) {
            Ok(json) => json,
            Err(e) => {
                result.error = Some(format!("Failed to parse Python JSON: {}", e));
                return result;
            }
        };

        // Deep comparison of JSON content
        if rust_json == python_json {
            result.matches = true;
        } else {
            result.differences = Some(format_json_differences(&rust_json, &python_json));
        }

        result
    }

    /// Format differences between two JSON values
    fn format_json_differences(rust_json: &JsonValue, python_json: &JsonValue) -> String {
        // Simple difference reporting - could be enhanced with more detailed diff
        format!(
            "JSON content differs:\nRust output: {}\nPython output: {}",
            serde_json::to_string_pretty(rust_json).unwrap_or_else(|_| "Invalid JSON".to_string()),
            serde_json::to_string_pretty(python_json)
                .unwrap_or_else(|_| "Invalid JSON".to_string())
        )
    }

    /// Print detailed test report
    fn print_test_report(stats: &TestStats, _results: &[DirectoryResult], config: &TestConfig) {
        println!();
        println!("{}", "=".repeat(80).bright_blue());
        println!("{}", "TEST REPORT".bright_blue().bold());
        println!("{}", "=".repeat(80).bright_blue());

        println!("Configuration:");
        println!("  Tests directory: {}", config.tests_dir.display());
        println!("  Generate JSON: {}", config.generate_json);
        println!();

        println!("Statistics:");
        println!("  Directories processed: {}", stats.directories_processed);
        println!("  Files processed: {}", stats.files_processed);
        println!(
            "  Files successfully parsed: {}",
            stats.files_successful.to_string().green()
        );
        println!(
            "  Files failed to parse: {}",
            if stats.files_failed > 0 {
                stats.files_failed.to_string().red()
            } else {
                stats.files_failed.to_string().green()
            }
        );

        if !config.generate_json {
            println!(
                "  Files matching Python output: {}",
                stats.files_matching.to_string().green()
            );
            println!(
                "  Files differing from Python: {}",
                if stats.files_differing > 0 {
                    stats.files_differing.to_string().red()
                } else {
                    stats.files_differing.to_string().green()
                }
            );
        }

        // Show parsing errors
        if !stats.parsing_errors.is_empty() {
            println!();
            println!("{}", "Parsing Errors:".red().bold());
            for (i, error) in stats.parsing_errors.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, error);
            }
            if stats.parsing_errors.len() > 10 {
                println!("  ... and {} more errors", stats.parsing_errors.len() - 10);
            }
        }

        // Show comparison errors
        if !config.generate_json && !stats.comparison_errors.is_empty() {
            println!();
            println!("{}", "Comparison Errors:".red().bold());
            for (i, error) in stats.comparison_errors.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, error);
            }
            if stats.comparison_errors.len() > 10 {
                println!(
                    "  ... and {} more errors",
                    stats.comparison_errors.len() - 10
                );
            }
        }

        println!();
        let overall_status =
            if stats.files_failed == 0 && (config.generate_json || stats.files_differing == 0) {
                "PASSED".green().bold()
            } else {
                "FAILED".red().bold()
            };
        println!("Overall Status: {}", overall_status);
        println!("{}", "=".repeat(80).bright_blue());
    }
}
