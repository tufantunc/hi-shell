use crate::command::truncate_output;
use crate::error::Result;
use crate::llm::{CommandResponse, LlmBackend, Message, Role};

const MAX_HISTORY_LEN: usize = 30;
const MAX_OUTPUT_LEN: usize = 1000;

pub struct CommandService {
    history: Vec<Message>,
}

pub struct ProcessResult {
    pub response: CommandResponse,
    pub output: String,
    pub success: bool,
    pub was_repair: bool,
}

pub trait UserInteraction: Send + Sync {
    fn confirm_dangerous(&self, command: &str) -> bool;
    fn confirm_repair(&self, error_output: &str) -> bool;
}

impl CommandService {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn history(&self) -> &[Message] {
        &self.history
    }

    pub fn push_user_message(&mut self, content: String) {
        self.history.push(Message {
            role: Role::User,
            content,
        });
    }

    pub async fn generate_command(
        &mut self,
        backend: &dyn LlmBackend,
        repair_context: Option<&str>,
    ) -> Result<CommandResponse> {
        let response = backend
            .generate_command(&self.history, repair_context)
            .await?;

        self.history.push(Message {
            role: Role::Assistant,
            content: serde_json::to_string(&response)?,
        });

        Ok(response)
    }

    pub fn record_command_output(&mut self, output: &str, success: bool) {
        let truncated = truncate_output(output, MAX_OUTPUT_LEN);

        self.history.push(Message {
            role: Role::System,
            content: if success {
                format!("Command output:\n{}", truncated)
            } else {
                format!("Command failed with output:\n{}", truncated)
            },
        });

        if self.history.len() > MAX_HISTORY_LEN {
            self.history.drain(0..(self.history.len() - MAX_HISTORY_LEN));
        }
    }

    pub async fn process_request(
        &mut self,
        request: &str,
        backend: &dyn LlmBackend,
        interaction: &dyn UserInteraction,
        execute_fn: impl Fn(&str) -> std::result::Result<(String, bool), Box<dyn std::error::Error + Send + Sync>>,
    ) -> Result<ProcessResult> {
        self.push_user_message(request.to_string());

        let mut current_repair_context: Option<String> = None;

        loop {
            let response = self
                .generate_command(backend, current_repair_context.as_deref())
                .await?;

            let was_repair = current_repair_context.is_some();

            if response.dangerous
                && !interaction.confirm_dangerous(&response.command)
            {
                return Ok(ProcessResult {
                    response,
                    output: String::new(),
                    success: false,
                    was_repair,
                });
            }

            let (output, success) = execute_fn(&response.command).map_err(|e| crate::error::HiShellError::Config(e.to_string()))?;
            self.record_command_output(&output, success);

            if success {
                return Ok(ProcessResult {
                    response,
                    output,
                    success: true,
                    was_repair,
                });
            } else {
                if interaction.confirm_repair(&output) {
                    current_repair_context = Some(output);
                    continue;
                } else {
                    return Ok(ProcessResult {
                        response,
                        output,
                        success: false,
                        was_repair,
                    });
                }
            }
        }
    }
}

impl Default for CommandService {
    fn default() -> Self {
        Self::new()
    }
}
