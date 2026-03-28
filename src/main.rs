use anyhow::Result;
use atty::Stream;
use clap::Parser;
use colored::*;
use dialoguer::{Confirm, FuzzySelect, Input, Select};
use hi_shell::command::{execute_command, truncate_output};
use hi_shell::config::{CloudProviderType, Config, LlmProvider, LocalProviderType};
use hi_shell::llm::{LlmBackend, cloud::CloudClient, embedded::EmbeddedClient, local::LocalClient};
use hi_shell::telemetry::Telemetry;
use hi_shell::update::{UpdateChecker, should_check_update};
use serde::Deserialize;
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[command(name = "hi-shell")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "An intelligent terminal assistant that translates your natural language descriptions into executable bash commands."
)]
struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    input: Vec<String>,

    #[arg(long, help = "Initialize configuration")]
    init: bool,

    #[arg(long, help = "Force non-interactive mode")]
    non_interactive: bool,

    #[arg(short, long, help = "Override the model name")]
    model: Option<String>,

    #[arg(long, help = "Do not save the model override to configuration")]
    temp: bool,

    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(long, help = "Manage downloaded embedded models")]
    models: bool,
}

#[derive(Debug, Deserialize)]
struct HfTreeEntry {
    #[serde(rename = "type")]
    entry_type: String,
    path: String,
}

async fn fetch_gguf_files(repo: &str) -> Result<Vec<String>> {
    let url = format!("https://huggingface.co/api/models/{}/tree/main", repo);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch model files from HuggingFace");
    }

    let entries: Vec<HfTreeEntry> = response.json().await?;
    let gguf_files: Vec<String> = entries
        .into_iter()
        .filter(|e| e.entry_type == "file" && e.path.ends_with(".gguf"))
        .map(|e| e.path)
        .collect();

    Ok(gguf_files)
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        eprintln!("\n{} {}: {}", "❌".red(), "Error".red().bold(), e);
        std::process::exit(1);
    }
    Ok(())
}

async fn run() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let filter = if args.verbose {
        "hi_shell=debug"
    } else {
        "hi_shell=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    if args.init {
        run_init().await?;
        return Ok(());
    }

    if args.models {
        run_model_management().await?;
        return Ok(());
    }

    let mut config = Config::load()?;

    let last_check = config.get_last_update_check();
    if should_check_update(last_check) {
        let current_version = env!("CARGO_PKG_VERSION");
        let checker = UpdateChecker::new(current_version)?;

        if let Ok(update_info) = checker.check_for_updates().await {
            if update_info.needs_update {
                println!("{}", hi_shell::update::format_update_message(&update_info));
            }

            let _ = config.update_last_check();
        }
    }

    // Override model if provided via CLI
    if let Some(model_override) = args.model {
        match config.llm_provider {
            LlmProvider::Embedded => config.embedded_model = Some(model_override),
            LlmProvider::Local => config.local_model = Some(model_override),
            LlmProvider::Cloud => config.cloud_model = Some(model_override),
        }

        // Save the config by default, unless --temp is specified
        if !args.temp {
            config.save()?;
            println!(
                "{} Model updated permanently to configuration.",
                "⚙".yellow()
            );
        }
    }

    let telemetry = Telemetry::new(&config);

    let mut history = Vec::new();

    if !atty::is(Stream::Stdin) {
        // Pipe mode
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        process_request(&buffer, &config, &telemetry, &mut history).await?;
    } else if !args.input.is_empty() {
        // One-shot mode
        let user_input = args.input.join(" ");
        process_request(&user_input, &config, &telemetry, &mut history).await?;
    } else {
        // REPL mode
        run_repl(&config, &telemetry).await?;
    }

    telemetry.flush().await;
    Ok(())
}

async fn run_init() -> Result<()> {
    println!("{}", "Welcome to hi-shell Configuration!".bold().green());

    let provider_idx = Select::new()
        .with_prompt("How would you like to run LLM?")
        .items(&[
            "Embedded (Llama-3.2-1B)",
            "Local (Ollama/LM Studio)",
            "Cloud (OpenRouter/Gemini/Anthropic)",
        ])
        .default(0)
        .interact()?;

    let mut config = Config::default();

    match provider_idx {
        0 => {
            config.llm_provider = LlmProvider::Embedded;

            let repo: String = Input::new()
                .with_prompt("HuggingFace Model Repo")
                .default("lmstudio-community/Llama-3.2-1B-Instruct-GGUF".to_string())
                .interact_text()?;

            config.embedded_model = Some(repo.clone());

            println!("{} Fetching available GGUF files...", "⏳".yellow());

            let gguf_file = match fetch_gguf_files(&repo).await {
                Ok(files) if !files.is_empty() => {
                    let idx = FuzzySelect::new()
                        .with_prompt("Select GGUF file")
                        .items(&files)
                        .default(0)
                        .interact()?;
                    files[idx].clone()
                }
                Ok(_) => {
                    println!(
                        "{} No GGUF files found, please enter manually.",
                        "⚠".yellow()
                    );
                    Input::new().with_prompt("GGUF Filename").interact_text()?
                }
                Err(e) => {
                    println!(
                        "{} Could not fetch files ({}), please enter manually.",
                        "⚠".yellow(),
                        e
                    );
                    Input::new()
                        .with_prompt("GGUF Filename")
                        .default("Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string())
                        .interact_text()?
                }
            };

            config.embedded_model_file = Some(gguf_file);

            println!(
                "{} Model configured. It will be downloaded on first use.",
                "ℹ".blue()
            );
        }
        1 => {
            config.llm_provider = LlmProvider::Local;
            let local_type_idx = Select::new()
                .with_prompt("Choose local provider")
                .items(&["Ollama", "LM Studio"])
                .interact()?;

            config.local_provider = Some(if local_type_idx == 0 {
                LocalProviderType::Ollama
            } else {
                LocalProviderType::LmStudio
            });

            let default_url = if local_type_idx == 0 {
                "http://localhost:11434"
            } else {
                "http://localhost:1234"
            };
            let url: String = Input::new()
                .with_prompt("Base URL")
                .default(default_url.to_string())
                .interact_text()?;
            config.local_url = Some(url.clone());

            // Try to fetch models automatically
            println!("{} Fetching available models...", "⏳".blue());
            let models =
                LocalClient::list_models(config.local_provider.as_ref().unwrap(), &url).await;

            match models {
                Ok(model_list) if !model_list.is_empty() => {
                    let model_idx = FuzzySelect::new()
                        .with_prompt("Select a model")
                        .items(&model_list)
                        .default(0)
                        .interact()?;
                    config.local_model = Some(model_list[model_idx].clone());
                }
                _ => {
                    println!(
                        "{} Could not fetch models automatically. Please enter manually.",
                        "⚠️".yellow()
                    );
                    config.local_model = Some(
                        Input::new()
                            .with_prompt("Model name")
                            .default("phi3".to_string())
                            .interact_text()?,
                    );
                }
            }
        }
        2 => {
            config.llm_provider = LlmProvider::Cloud;
            let cloud_type_idx = Select::new()
                .with_prompt("Choose cloud provider")
                .items(&[
                    "OpenRouter",
                    "Gemini",
                    "Anthropic",
                    "OpenAI",
                    "Custom (OpenAI-compatible)",
                ])
                .interact()?;

            config.cloud_provider = Some(match cloud_type_idx {
                0 => CloudProviderType::OpenRouter,
                1 => CloudProviderType::Gemini,
                2 => CloudProviderType::Anthropic,
                3 => CloudProviderType::OpenAI,
                4 => CloudProviderType::Custom,
                _ => unreachable!(), // Should not happen with `Select`
            });

            if config.cloud_provider == Some(CloudProviderType::Custom) {
                config.cloud_custom_url = Some(
                    Input::new()
                        .with_prompt("API Base URL (e.g. https://api.openai.com/v1)")
                        .interact_text()?,
                );
            }

            config.api_key = Some(
                Input::<String>::new()
                    .with_prompt("Enter API Key")
                    .interact_text()?,
            );

            // Try to fetch models for Cloud too
            println!("{} Fetching cloud models...", "⏳".blue());
            let cloud_models = CloudClient::list_models(
                config.cloud_provider.as_ref().unwrap(),
                config.api_key.as_deref(),
            )
            .await;

            match cloud_models {
                Ok(model_list) if !model_list.is_empty() => {
                    let model_idx = FuzzySelect::new()
                        .with_prompt("Select a cloud model")
                        .items(&model_list)
                        .default(0)
                        .interact()?;
                    config.cloud_model = Some(model_list[model_idx].clone());
                }
                _ => {
                    println!(
                        "{} Could not fetch models automatically. Please enter manually.",
                        "⚠️".yellow()
                    );
                    config.cloud_model = Some(
                        Input::new()
                            .with_prompt("Cloud Model Name")
                            .default("google/gemini-2.0-flash-exp:free".to_string())
                            .interact_text()?,
                    );
                }
            }
        }
        _ => unreachable!(),
    }

    config.telemetry_enabled = Confirm::new()
        .with_prompt("Do you allow anonymous telemetry (usage stats and error reports)?")
        .default(true)
        .interact()?;

    config.save()?;
    println!("{}", "\nConfiguration saved successfully!".green());
    Ok(())
}

async fn run_model_management() -> Result<()> {
    println!("{}", "\n--- Embedded Model Management ---".bold().green());

    loop {
        let models = EmbeddedClient::list_downloaded_models()?;

        if models.is_empty() {
            println!("No embedded models currently downloaded.");
            return Ok(());
        }

        println!("\nCurrently downloaded models:");
        for (i, (name, size)) in models.iter().enumerate() {
            let size_mb = size / 1024 / 1024;
            println!("{}. {} ({} MB)", i + 1, name.cyan(), size_mb);
        }

        let options = vec!["Delete a model", "Exit"];
        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        if selection == 1 {
            return Ok(());
        }

        let model_names: Vec<String> = models.iter().map(|(n, _)| n.clone()).collect();
        let to_delete = FuzzySelect::new()
            .with_prompt("Select model to delete")
            .items(&model_names)
            .interact()?;

        let confirm = Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {}?",
                model_names[to_delete]
            ))
            .default(false)
            .interact()?;

        if confirm {
            EmbeddedClient::delete_model(&model_names[to_delete])?;
            println!("{} Model deleted successfully.", "✔".green());
        }
    }
}

async fn process_request(
    request: &str,
    config: &Config,
    telemetry: &Telemetry,
    history: &mut Vec<hi_shell::llm::Message>,
) -> Result<()> {
    let provider_name = format!("{:?}", config.llm_provider);
    let model_name = match config.llm_provider {
        LlmProvider::Embedded => config.embedded_model.clone(),
        LlmProvider::Local => config.local_model.clone(),
        LlmProvider::Cloud => config.cloud_model.clone(),
    };

    // Add user request to history if it's not a repair turn
    history.push(hi_shell::llm::Message {
        role: hi_shell::llm::Role::User,
        content: request.to_string(),
    });

    let backend: Box<dyn LlmBackend> = match config.llm_provider {
        LlmProvider::Embedded => {
            Box::new(EmbeddedClient::new(config.clone())) as Box<dyn LlmBackend>
        }
        LlmProvider::Local => Box::new(LocalClient::new(config.clone())) as Box<dyn LlmBackend>,
        LlmProvider::Cloud => Box::new(CloudClient::new(config.clone())) as Box<dyn LlmBackend>,
    };

    let mut current_repair_context: Option<String> = None;

    loop {
        let start_time = std::time::Instant::now();
        let response_result = backend
            .generate_command(history, current_repair_context.as_deref())
            .await;
        let latency_ms = start_time.elapsed().as_millis() as u64;

        match response_result {
            Ok(response) => {
                telemetry.track_event(
                    "command_generated",
                    serde_json::json!({
                        "provider": provider_name,
                        "model": model_name,
                        "latency_ms": latency_ms,
                        "dangerous": response.dangerous,
                        "success": true,
                        "is_repair": current_repair_context.is_some()
                    }),
                );

                // Add assistant response to history
                history.push(hi_shell::llm::Message {
                    role: hi_shell::llm::Role::Assistant,
                    content: serde_json::to_string(&response)?,
                });

                println!(
                    "\x1B[2K\r{} Proposed command:",
                    if current_repair_context.is_some() {
                        "🔧".yellow()
                    } else {
                        "✔".green().bold()
                    }
                );

                let cmd = response.command.trim();
                let width = cmd.len() + 4;
                let border = "═".repeat(width);
                println!("  ╔{}╗", border);
                println!("  ║  {}  ║", cmd.bold().cyan());
                println!("  ╚{}╝", border);

                if let Some(explanation) = &response.explanation {
                    println!("\n{} {}", "💡".yellow(), explanation.italic());
                }

                if response.dangerous {
                    println!(
                        "\n{} {}",
                        "⚠️  WARNING:".red().bold(),
                        "This command is marked as dangerous!".red()
                    );
                    if !Confirm::new()
                        .with_prompt("Do you definitely want to execute this?")
                        .default(false)
                        .interact()?
                    {
                        telemetry.track_event(
                            "command_aborted",
                            serde_json::json!({
                                "reason": "dangerous_confirmation_rejected"
                            }),
                        );
                        return Ok(());
                    }
                } else {
                    println!("\n{} Executing...", "➜".blue());
                }

                let (output, success) = execute_command_with_output(&response.command)?;

                let truncated_output = truncate_output(&output, 1000);

                history.push(hi_shell::llm::Message {
                    role: hi_shell::llm::Role::System,
                    content: truncated_output,
                });

                if history.len() > 30 {
                    history.drain(0..(history.len() - 30));
                }

                if success {
                    telemetry.track_event(
                        "command_executed",
                        serde_json::json!({
                            "provider": provider_name,
                            "success": true
                        }),
                    );
                    return Ok(());
                } else {
                    telemetry.track_event(
                        "command_executed",
                        serde_json::json!({
                            "provider": provider_name,
                            "success": false
                        }),
                    );

                    println!("\n{} {}", "❌".red(), "Command failed.".red().bold());
                    if Confirm::new()
                        .with_prompt("Would you like me to try and fix this error?")
                        .default(true)
                        .interact()?
                    {
                        current_repair_context = Some(output);
                        continue;
                    } else {
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                telemetry.track_event(
                    "command_failed",
                    serde_json::json!({
                        "provider": provider_name,
                        "latency_ms": latency_ms,
                        "error": e.to_string()
                    }),
                );
                return Err(e.into());
            }
        }
    }
}

fn execute_command_with_output(cmd: &str) -> Result<(String, bool)> {
    let (combined, success) = execute_command(cmd)?;

    let stdout_end = combined.find("\nError:\n").unwrap_or(combined.len());
    let stdout_part = &combined[..stdout_end];
    let stderr_part = if stdout_end < combined.len() {
        &combined[stdout_end + "\nError:\n".len()..]
    } else {
        ""
    };

    if !stdout_part.is_empty() {
        print!("{}", stdout_part);
        io::stdout().flush()?;
    }
    if !stderr_part.is_empty() {
        eprint!("{}", stderr_part);
        io::stderr().flush()?;
    }

    Ok((combined, success))
}

async fn run_repl(config: &Config, telemetry: &Telemetry) -> Result<()> {
    use rustyline::DefaultEditor;
    let mut rl = DefaultEditor::new()?;
    let mut history = Vec::new();

    println!("hi-shell REPL mode. Type 'exit' to quit.");

    loop {
        let readline = rl.readline("hi-shell ➜ ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }

                rl.add_history_entry(line)?;
                if let Err(e) = process_request(line, config, telemetry, &mut history).await {
                    eprintln!("Error: {}", e);
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}
