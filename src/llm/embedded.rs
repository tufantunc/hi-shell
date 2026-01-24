use crate::config::Config;
use crate::error::{HiShellError, Result};
use crate::llm::{CommandResponse, LlmBackend, Message};
use async_trait::async_trait;
use candle_core::{Device, Tensor, quantized::gguf_file};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama as model;
use hf_hub::api::sync::ApiBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

// Global lazy-loaded model for REPL mode efficiency
static LOADED_MODEL: OnceCell<Arc<LoadedModel>> = OnceCell::new();

struct LoadedModel {
    model: Mutex<model::ModelWeights>,
    tokenizer: Tokenizer,
    device: Device,
}

pub struct EmbeddedClient {
    config: Config,
}

impl EmbeddedClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    fn get_device() -> Result<Device> {
        #[cfg(target_os = "macos")]
        {
            if let Ok(device) = Device::new_metal(0) {
                info!("Using Metal GPU acceleration");
                return Ok(device);
            }
        }

        #[cfg(feature = "cuda")]
        {
            if let Ok(device) = Device::new_cuda(0) {
                info!("Using CUDA GPU acceleration");
                return Ok(device);
            }
        }

        warn!("Using CPU (no GPU acceleration available)");
        Ok(Device::Cpu)
    }

    fn download_model(&self) -> Result<PathBuf> {
        let model_id = self
            .config
            .embedded_model
            .as_deref()
            .unwrap_or("microsoft/Phi-3-mini-4k-instruct-gguf");
        let filename = self
            .config
            .embedded_model_file
            .as_deref()
            .unwrap_or("Phi-3-mini-4k-instruct-q4.gguf");

        let api = ApiBuilder::new()
            .with_progress(true)
            .build()
            .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
        let repo = api.model(model_id.to_string());

        debug!("Checking model cache for {}...", model_id);
        let path = repo
            .get(filename)
            .map_err(|e| HiShellError::LlmLoad(format!("Failed to download model: {}", e)))?;
        Ok(path)
    }

    fn download_tokenizer(&self) -> Result<Tokenizer> {
        let model_id = self
            .config
            .embedded_model
            .as_deref()
            .unwrap_or("microsoft/Phi-3-mini-4k-instruct-gguf");
        let api = ApiBuilder::new()
            .with_progress(true)
            .build()
            .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
        let repo = api.model(model_id.to_string());

        let tokenizer_path = match repo.get("tokenizer.json") {
            Ok(p) => p,
            Err(_) => {
                let base_model = model_id.replace("-gguf", "");
                let base_repo = api.model(base_model);
                base_repo.get("tokenizer.json").map_err(|e| {
                    HiShellError::LlmLoad(format!("Failed to download tokenizer: {}", e))
                })?
            }
        };

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| HiShellError::LlmLoad(format!("Failed to load tokenizer: {}", e)))?;

        Ok(tokenizer)
    }

    fn load_or_get_model(&self) -> Result<Arc<LoadedModel>> {
        if let Some(m) = LOADED_MODEL.get() {
            return Ok(Arc::clone(m));
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.yellow} {msg}")
                .map_err(|e| HiShellError::Config(e.to_string()))?,
        );
        pb.set_message("Loading model into memory...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        let model_path = self.download_model()?;
        let device = Self::get_device()?;

        let mut file = std::fs::File::open(&model_path)?;
        let gguf = gguf_file::Content::read(&mut file)
            .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
        let model_weights = model::ModelWeights::from_gguf(gguf, &mut file, &device)
            .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

        let tokenizer = self.download_tokenizer()?;

        pb.finish_and_clear();
        info!("Model loaded successfully on {:?}", device);

        let loaded = Arc::new(LoadedModel {
            model: Mutex::new(model_weights),
            tokenizer,
            device,
        });

        let _ = LOADED_MODEL.set(Arc::clone(&loaded));

        Ok(loaded)
    }

    fn generate_text(
        &self,
        loaded: &LoadedModel,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<String> {
        let tokens = loaded
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| HiShellError::Parsing(format!("Tokenization failed: {}", e)))?;

        let input_ids = tokens.get_ids();
        let mut all_tokens = input_ids.to_vec();

        let mut logits_processor = LogitsProcessor::new(42, Some(0.7), Some(0.9));
        let mut model = loaded
            .model
            .lock()
            .map_err(|e| HiShellError::LlmLoad(format!("Model lock failed: {}", e)))?;

        debug!("Starting token generation...");
        for _ in 0..max_tokens {
            let input = Tensor::new(&all_tokens[..], &loaded.device)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?
                .unsqueeze(0)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
            let logits = model
                .forward(&input, all_tokens.len())
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
            let logits = logits
                .squeeze(0)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
            let next_token = logits_processor
                .sample(&logits)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

            all_tokens.push(next_token);

            if next_token == 2 || next_token == 32000 || next_token == 32007 {
                break;
            }
        }

        let output = loaded
            .tokenizer
            .decode(&all_tokens[input_ids.len()..], true)
            .map_err(|e| HiShellError::Parsing(format!("Decoding failed: {}", e)))?;

        Ok(output)
    }
}

#[async_trait]
impl LlmBackend for EmbeddedClient {
    async fn generate_command(
        &self,
        messages: &[Message],
        repair_context: Option<&str>,
    ) -> Result<CommandResponse> {
        let loaded = self.load_or_get_model()?;
        let system_prompt = crate::llm::get_system_prompt(repair_context);

        let mut prompt = format!("<|system|>\n{}\n<|end|>\n", system_prompt);
        for msg in messages {
            match msg.role {
                crate::llm::Role::User => {
                    prompt.push_str(&format!("<|user|>\n{}\n<|end|>\n", msg.content));
                }
                crate::llm::Role::Assistant => {
                    prompt.push_str(&format!("<|assistant|>\n{}\n<|end|>\n", msg.content));
                }
                crate::llm::Role::System => {
                    prompt.push_str(&format!(
                        "<|user|>\nPrevious Command Output: {}\n<|end|>\n",
                        msg.content
                    ));
                }
            }
        }
        prompt.push_str("<|assistant|>\n");

        debug!("Internal prompt for embedded model: {}", prompt);
        let raw_response = self.generate_text(&loaded, &prompt, 256)?;

        crate::llm::parse_llm_response(&raw_response)
    }
}
