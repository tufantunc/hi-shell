mod config;
mod llm;
mod telemetry;

use crate::config::{CloudProviderType, Config, LlmProvider, LocalProviderType};
use crate::llm::{LlmBackend, cloud::CloudClient, embedded::EmbeddedClient, local::LocalClient};
use crate::telemetry::Telemetry;
use anyhow::Result;
use atty::Stream;
use clap::Parser;
use colored::*;
use dialoguer::{Confirm, FuzzySelect, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
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

    if args.init {
        run_init().await?;
        return Ok(());
    }

    let mut config = Config::load()?;

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
            "Embedded (Phi-3-mini)",
            "Local (Ollama/LM Studio)",
            "Cloud (OpenRouter/Gemini/Anthropic)",
        ])
        .default(0)
        .interact()?;

    let mut config = Config::default();

    match provider_idx {
        0 => {
            config.llm_provider = LlmProvider::Embedded;

            config.embedded_model = Some(
                Input::new()
                    .with_prompt("HuggingFace Model Repo")
                    .default("microsoft/Phi-3-mini-4k-instruct-gguf".to_string())
                    .interact_text()?,
            );

            config.embedded_model_file = Some(
                Input::new()
                    .with_prompt("GGUF Filename")
                    .default("Phi-3-mini-4k-instruct-q4.gguf".to_string())
                    .interact_text()?,
            );

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
                "http://localhost:11434/api/generate"
            } else {
                "http://localhost:1234/v1/chat/completions"
            };
            let url: String = Input::new()
                .with_prompt("API URL")
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

async fn process_request(
    request: &str,
    config: &Config,
    telemetry: &Telemetry,
    history: &mut Vec<crate::llm::Message>,
) -> Result<()> {
    let provider_name = format!("{:?}", config.llm_provider);

    // Add user request to history if it's not a repair turn
    history.push(crate::llm::Message {
        role: crate::llm::Role::User,
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
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.blue} {msg}")?,
        );
        pb.set_message(if current_repair_context.is_some() {
            "Analyzing error and fixing..."
        } else {
            "Generating command..."
        });
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        let response_result = backend
            .generate_command(history, current_repair_context.as_deref())
            .await;
        let latency_ms = start_time.elapsed().as_millis() as u64;

        match response_result {
            Ok(response) => {
                pb.finish_and_clear();

                telemetry.track_event(
                    "command_generated",
                    serde_json::json!({
                        "provider": provider_name,
                        "latency_ms": latency_ms,
                        "dangerous": response.dangerous,
                        "success": true,
                        "is_repair": current_repair_context.is_some()
                    }),
                );

                // Add assistant response to history
                history.push(crate::llm::Message {
                    role: crate::llm::Role::Assistant,
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

                let (output, success) = execute_command(&response.command)?;

                // Add command output to history
                let truncated_output = if output.len() > 1000 {
                    format!("{}... (truncated)", &output[..1000])
                } else {
                    output.clone()
                };

                history.push(crate::llm::Message {
                    role: crate::llm::Role::System,
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
                pb.finish_with_message("Generation failed");
                telemetry.track_event(
                    "command_failed",
                    serde_json::json!({
                        "provider": provider_name,
                        "latency_ms": latency_ms,
                        "error": e.to_string()
                    }),
                );
                return Err(e);
            }
        }
    }
}

fn execute_command(cmd: &str) -> Result<(String, bool)> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stdout.is_empty() {
        print!("{}", stdout);
        io::stdout().flush()?;
    }
    if !stderr.is_empty() {
        eprint!("{}", stderr);
        io::stderr().flush()?;
    }

    let mut combined = stdout;
    if !stderr.is_empty() {
        combined.push_str("\nError:\n");
        combined.push_str(&stderr);
    }

    Ok((combined, output.status.success()))
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
