use clap::Parser;
use cmdparser::NetworkOutputParser;
use std::io::Read;
use std::fs::File;

#[derive(Parser)]
#[command(name = "netssh_textfsm")]
#[command(about = "Parse network device command output using TextFSM templates")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Parse using template file directly
    ParseTemplate {
        /// Path to the TextFSM template file
        template_file: String,
        /// Path to the raw data file
        data_file: String,
    },
    /// Parse using platform and command (auto-select template)
    ParseCommand {
        /// Device platform (e.g., cisco_ios, cisco_asa)
        platform: String,
        /// Command string (e.g., "show version")
        command: String,
        /// Path to the raw data file
        data_file: String,
        /// Optional path to templates directory
        #[arg(long)]
        template_dir: Option<String>,
    },
}

fn parse_with_template_file(template_file: &str, data_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use cmdparser::textfsm::{TextFSM};
    use indexmap::IndexMap;
    use serde_json;

    // Read the template file
    let template = File::open(template_file)?;
    let mut fsm = TextFSM::new(template)?;

    // Read the data file
    let mut data_content = String::new();
    let mut data = File::open(data_file)?;
    data.read_to_string(&mut data_content)?;

    // Parse the data
    let parsed_data: Vec<IndexMap<String, serde_json::Value>> = fsm.parse_text_to_dicts(&data_content)?;

    // Convert to JSON and store in a file
    let json_output = serde_json::to_string_pretty(&parsed_data)?;
    std::fs::write("rust_parsed_data.json", json_output)?;
    println!("Successfully parsed data and saved to rust_parsed_data.json");

    Ok(())
}

fn parse_with_platform_command(
    platform: &str,
    command: &str,
    data_file: &str,
    template_dir: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the data file
    let mut data_content = String::new();
    let mut data = File::open(data_file)?;
    data.read_to_string(&mut data_content)?;

    // Create parser with optional template directory
    let template_dir_path = template_dir.map(std::path::PathBuf::from);
    let parser = NetworkOutputParser::new(template_dir_path);

    // Parse the data
    match parser.parse_to_json(platform, command, &data_content)? {
        Some(json_output) => {
            std::fs::write("rust_parsed_data.json", &json_output)?;
            println!("Successfully parsed data and saved to rust_parsed_data.json");
            println!("Platform: {}, Command: {}", platform, command);
        }
        None => {
            eprintln!("Failed to parse data - no suitable template found or parsing failed");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::ParseTemplate { template_file, data_file } => {
            parse_with_template_file(&template_file, &data_file)?;
        }
        Commands::ParseCommand { platform, command, data_file, template_dir } => {
            parse_with_platform_command(&platform, &command, &data_file, template_dir)?;
        }
    }

    Ok(())
}
