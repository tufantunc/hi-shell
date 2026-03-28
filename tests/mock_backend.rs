use async_trait::async_trait;
use hi_shell::error::Result;
use hi_shell::llm::{CommandResponse, LlmBackend, Message};
use std::sync::{Arc, Mutex};

pub struct MockBackend {
    responses: Arc<Mutex<Vec<CommandResponse>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockBackend {
    pub fn new(responses: Vec<CommandResponse>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    pub fn single(command: &str, explanation: &str, dangerous: bool) -> Self {
        Self::new(vec![CommandResponse {
            command: command.to_string(),
            explanation: Some(explanation.to_string()),
            dangerous,
        }])
    }
}

#[async_trait]
impl LlmBackend for MockBackend {
    async fn generate_command(
        &self,
        _messages: &[Message],
        _repair_context: Option<&str>,
    ) -> Result<CommandResponse> {
        let mut count = self.call_count.lock().unwrap();
        let idx = *count;
        *count += 1;

        let responses = self.responses.lock().unwrap();
        if idx < responses.len() {
            Ok(responses[idx].clone())
        } else if let Some(last) = responses.last() {
            Ok(last.clone())
        } else {
            Ok(CommandResponse {
                command: "echo 'no response configured'".to_string(),
                explanation: Some("mock fallback".to_string()),
                dangerous: false,
            })
        }
    }
}

pub struct MockInteraction {
    pub allow_dangerous: bool,
    pub allow_repair: bool,
}

impl MockInteraction {
    pub fn new(allow_dangerous: bool, allow_repair: bool) -> Self {
        Self {
            allow_dangerous,
            allow_repair,
        }
    }

    pub fn allow_all() -> Self {
        Self::new(true, true)
    }

    pub fn _deny_all() -> Self {
        Self::new(false, false)
    }
}

impl hi_shell::service::UserInteraction for MockInteraction {
    fn confirm_dangerous(&self, _command: &str) -> bool {
        self.allow_dangerous
    }

    fn confirm_repair(&self, _error_output: &str) -> bool {
        self.allow_repair
    }
}
