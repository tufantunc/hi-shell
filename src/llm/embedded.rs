use crate::config::Config;
use crate::error::{HiShellError, Result};
use crate::llm::{CommandResponse, LlmBackend, Message};
use async_trait::async_trait;
use candle_core::{Device, Tensor, quantized::gguf_file};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama as llama_model;
use candle_transformers::models::quantized_phi3 as phi3_model;
use candle_transformers::models::quantized_qwen2 as qwen2_model;
use hf_hub::api::sync::ApiBuilder;

use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

// Global lazy-loaded model for REPL mode efficiency
static LOADED_MODEL: OnceCell<Arc<LoadedModel>> = OnceCell::new();

enum ModelWeights {
    Llama(llama_model::ModelWeights),
    Phi3(phi3_model::ModelWeights),
    Qwen2(qwen2_model::ModelWeights),
}

struct LoadedModel {
    model: Mutex<ModelWeights>,
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
            .unwrap_or("lmstudio-community/Llama-3.2-1B-Instruct-GGUF");
        let filename = self
            .config
            .embedded_model_file
            .as_deref()
            .unwrap_or("Llama-3.2-1B-Instruct-Q4_K_M.gguf");

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
            .unwrap_or("lmstudio-community/Llama-3.2-1B-Instruct-GGUF");

        let api = ApiBuilder::new()
            .with_progress(true)
            .build()
            .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
        let repo = api.model(model_id.to_string());
        let tokenizer_path = match repo.get("tokenizer.json") {
            Ok(p) => p,
            Err(_) => {
                // Try deriving base model name first
                let base_model = model_id.replace("-GGUF", "").replace("-gguf", "");
                let base_repo = api.model(base_model);
                match base_repo.get("tokenizer.json") {
                    Ok(p) => p,
                    Err(_) => {
                        // Fallback to a known reliable repo for Llama 3.2 1B
                        if model_id.to_lowercase().contains("llama-3.2-1b") {
                            let fallback_repo =
                                api.model("unsloth/Llama-3.2-1B-Instruct".to_string());
                            fallback_repo.get("tokenizer.json").map_err(|e| {
                                HiShellError::LlmLoad(format!(
                                    "Failed to download tokenizer from all sources: {}",
                                    e
                                ))
                            })?
                        } else {
                            return Err(HiShellError::LlmLoad(
                                "Failed to download tokenizer from primary or base repo"
                                    .to_string(),
                            ));
                        }
                    }
                }
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

        let model_path = self.download_model()?;
        let device = Self::get_device()?;
        let tokenizer = self.download_tokenizer()?;

        let pb = crate::llm::create_spinner("Loading model into memory...")?;

        let mut file = std::fs::File::open(&model_path)?;
        let gguf = gguf_file::Content::read(&mut file)
            .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

        // Detect model type from GGUF metadata
        let model_weights = if gguf.metadata.contains_key("phi3.block_count") {
            info!("Detected Phi-3 model architecture");
            let weights = phi3_model::ModelWeights::from_gguf(false, gguf, &mut file, &device)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
            ModelWeights::Phi3(weights)
        } else if gguf.metadata.contains_key("qwen2.block_count") {
            info!("Detected Qwen2 model architecture");
            let weights = qwen2_model::ModelWeights::from_gguf(gguf, &mut file, &device)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
            ModelWeights::Qwen2(weights)
        } else if gguf.metadata.contains_key("llama.block_count")
            || gguf.metadata.contains_key("llama.attention.head_count")
        {
            info!("Detected Llama model architecture");
            let weights = llama_model::ModelWeights::from_gguf(gguf, &mut file, &device)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
            ModelWeights::Llama(weights)
        } else {
            // Get architecture name from metadata if available
            let arch = gguf
                .metadata
                .get("general.architecture")
                .map(|v| format!("{:?}", v))
                .unwrap_or_else(|| "unknown".to_string());
            return Err(HiShellError::LlmLoad(format!(
                "Unsupported model architecture: {}. Supported: Llama, Phi-3, Qwen2",
                arch
            )));
        };

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
        let mut generated_tokens = Vec::new();

        let mut logits_processor = LogitsProcessor::new(42, Some(0.7), Some(0.9));
        let mut model = loaded
            .model
            .lock()
            .map_err(|e| HiShellError::LlmLoad(format!("Model lock failed: {}", e)))?;

        debug!(
            "Starting token generation for prompt length {}...",
            input_ids.len()
        );

        let mut pos = 0;
        let mut next_token;

        // Determine EOS tokens based on model type
        let eos_tokens: Vec<u32> = match &*model {
            ModelWeights::Phi3(_) => {
                // Phi-3 stop tokens: <|end|>=32007, <|endoftext|>=32000
                vec![2, 32000, 32007]
            }
            ModelWeights::Qwen2(_) => {
                // Qwen2 stop tokens: <|endoftext|>=151643, <|im_end|>=151645
                vec![2, 151643, 151645]
            }
            ModelWeights::Llama(_) => {
                // Llama 3/3.x Stop tokens: <|end_of_text|>=128000/1, <|eot_id|>=128009
                vec![2, 128000, 128001, 128009]
            }
        };

        // Process the prompt in one go
        {
            let input = Tensor::new(input_ids, &loaded.device)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?
                .unsqueeze(0)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;
            let logits = match &mut *model {
                ModelWeights::Llama(llama) => llama.forward(&input, pos),
                ModelWeights::Phi3(phi3) => phi3.forward(&input, pos),
                ModelWeights::Qwen2(qwen2) => qwen2.forward(&input, pos),
            }
            .map_err(|e| HiShellError::LlmLoad(format!("Forward pass failed: {}", e)))?;
            let logits = logits
                .squeeze(0)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

            next_token = logits_processor
                .sample(&logits)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

            generated_tokens.push(next_token);
            pos += input_ids.len();
        }

        // Generate subsequent tokens
        for _ in 0..max_tokens {
            if eos_tokens.contains(&next_token) {
                break;
            }

            let input = Tensor::new(&[next_token], &loaded.device)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?
                .unsqueeze(0)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

            let logits = match &mut *model {
                ModelWeights::Llama(llama) => llama.forward(&input, pos),
                ModelWeights::Phi3(phi3) => phi3.forward(&input, pos),
                ModelWeights::Qwen2(qwen2) => qwen2.forward(&input, pos),
            }
            .map_err(|e| HiShellError::LlmLoad(format!("Forward pass failed: {}", e)))?;
            let logits = logits
                .squeeze(0)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

            next_token = logits_processor
                .sample(&logits)
                .map_err(|e| HiShellError::LlmLoad(e.to_string()))?;

            generated_tokens.push(next_token);
            pos += 1;
        }

        let output = loaded
            .tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| HiShellError::Parsing(format!("Decoding failed: {}", e)))?;

        Ok(output)
    }

    pub fn list_downloaded_models() -> Result<Vec<(String, u64)>> {
        let base_dirs = directories::BaseDirs::new().ok_or_else(|| {
            HiShellError::Config("Could not determine home directory".to_string())
        })?;
        let cache_path = base_dirs.home_dir().join(".cache/huggingface/hub");

        if !cache_path.exists() {
            return Ok(vec![]);
        }

        let mut models = Vec::new();
        for entry in std::fs::read_dir(cache_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("models--") {
                let display_name = name.replace("models--", "").replace("--", "/");

                // Simple size calculation (recursive)
                let size = Self::get_dir_size(&entry.path())?;
                models.push((display_name, size));
            }
        }
        Ok(models)
    }

    fn get_dir_size(path: &std::path::Path) -> Result<u64> {
        let mut size = 0;
        if path.is_file() {
            size += path.metadata()?.len();
        } else if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                size += Self::get_dir_size(&entry?.path())?;
            }
        }
        Ok(size)
    }

    pub fn delete_model(display_name: &str) -> Result<()> {
        let internal_name = format!("models--{}", display_name.replace("/", "--"));
        let base_dirs = directories::BaseDirs::new().ok_or_else(|| {
            HiShellError::Config("Could not determine home directory".to_string())
        })?;
        let cache_path = base_dirs
            .home_dir()
            .join(".cache/huggingface/hub")
            .join(internal_name);

        if cache_path.exists() {
            std::fs::remove_dir_all(cache_path)?;
        }
        Ok(())
    }
}

#[async_trait]
impl LlmBackend for EmbeddedClient {
    async fn generate_command(
        &self,
        messages: &[Message],
        repair_context: Option<&str>,
    ) -> Result<CommandResponse> {
        let system_prompt = crate::llm::get_system_prompt(repair_context);
        let loaded = self.load_or_get_model()?;

        let pb = crate::llm::create_spinner(if repair_context.is_some() {
            "Analyzing error and fixing..."
        } else {
            "Generating command..."
        })?;

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

        pb.finish_and_clear();
        crate::llm::parse_llm_response(&raw_response)
    }
}
