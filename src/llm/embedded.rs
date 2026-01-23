use crate::config::Config;
use crate::llm::{CommandResponse, LlmBackend, Message};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use candle_core::{Device, quantized::gguf_file};
use candle_transformers::models::quantized_llama as model;
use hf_hub::api::sync::ApiBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;

// Global lazy-loaded model for REPL mode efficiency
static LOADED_MODEL: OnceCell<Arc<LoadedModel>> = OnceCell::new();

use std::sync::Mutex;

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
        // Priority: Metal (macOS) -> CUDA (NVIDIA) -> CPU
        #[cfg(target_os = "macos")]
        {
            if let Ok(device) = Device::new_metal(0) {
                eprintln!("🚀 Using Metal GPU acceleration");
                return Ok(device);
            }
        }

        #[cfg(feature = "cuda")]
        {
            if let Ok(device) = Device::new_cuda(0) {
                eprintln!("🚀 Using CUDA GPU acceleration");
                return Ok(device);
            }
        }

        eprintln!("💻 Using CPU (no GPU acceleration available)");
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

        let api = ApiBuilder::new().with_progress(true).build()?;
        let repo = api.model(model_id.to_string());

        eprintln!("📦 Checking model cache for {}...", model_id);
        let path = repo.get(filename)?;
        eprintln!("✅ Model ready: {}", filename);
        Ok(path)
    }

    fn download_tokenizer(&self) -> Result<Tokenizer> {
        let model_id = self
            .config
            .embedded_model
            .as_deref()
            .unwrap_or("microsoft/Phi-3-mini-4k-instruct-gguf");
        let api = ApiBuilder::new().with_progress(true).build()?;
        let repo = api.model(model_id.to_string());

        let tokenizer_path = match repo.get("tokenizer.json") {
            Ok(p) => p,
            Err(_) => {
                // Try base model without -gguf suffix
                let base_model = model_id.replace("-gguf", "");
                let base_repo = api.model(base_model);
                base_repo.get("tokenizer.json")?
            }
        };

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

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
                .template("{spinner:.yellow} {msg}")?,
        );
        pb.set_message("Loading model into memory...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        let model_path = self.download_model()?;
        let device = Self::get_device()?;

        let mut file = std::fs::File::open(&model_path)?;
        let gguf = gguf_file::Content::read(&mut file)?;
        let model_weights = model::ModelWeights::from_gguf(gguf, &mut file, &device)?;

        let tokenizer = self.download_tokenizer()?;

        pb.finish_and_clear();
        eprintln!("✅ Model loaded successfully");

        let loaded = Arc::new(LoadedModel {
            model: Mutex::new(model_weights),
            tokenizer,
            device,
        });

        // Initialize Global State
        let _ = LOADED_MODEL.set(Arc::clone(&loaded));

        Ok(loaded)
    }

    fn generate_text(
        &self,
        loaded: &LoadedModel,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<String> {
        use candle_core::Tensor;
        use candle_transformers::generation::LogitsProcessor;

        let tokens = loaded
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let input_ids = tokens.get_ids();
        let mut all_tokens = input_ids.to_vec();

        let mut logits_processor = LogitsProcessor::new(42, Some(0.7), Some(0.9));
        let mut model = loaded
            .model
            .lock()
            .map_err(|e| anyhow!("Model lock failed: {}", e))?;

        for _ in 0..max_tokens {
            let input = Tensor::new(&all_tokens[..], &loaded.device)?.unsqueeze(0)?;
            let logits = model.forward(&input, all_tokens.len())?;
            let logits = logits.squeeze(0)?;
            let next_token = logits_processor.sample(&logits)?;

            all_tokens.push(next_token);

            // EOS token check (2 = EOS for Llama/Phi usually, 32000 for some others)
            if next_token == 2 || next_token == 32000 || next_token == 32007 {
                break;
            }
        }

        let output = loaded
            .tokenizer
            .decode(&all_tokens[input_ids.len()..], true)
            .map_err(|e| anyhow!("Decoding failed: {}", e))?;

        Ok(output)
    }
}

#[async_trait]
impl LlmBackend for EmbeddedClient {
    async fn generate_command(&self, messages: &[Message]) -> Result<CommandResponse> {
        let loaded = self.load_or_get_model()?;
        let system_prompt = crate::llm::get_system_prompt();

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
                    // Treat system/tool output as user context in the chat
                    prompt.push_str(&format!(
                        "<|user|>\nPrevious Command Output: {}\n<|end|>\n",
                        msg.content
                    ));
                }
            }
        }
        prompt.push_str("<|assistant|>\n");

        let raw_response = self.generate_text(&loaded, &prompt, 256)?;

        crate::llm::parse_llm_response(&raw_response)
    }
}
