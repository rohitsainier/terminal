use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AIEngine {
    client: Client,
    pub provider: AIProvider,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AIProvider {
    Ollama { model: String, base_url: String },
    OpenAI { api_key: String, model: String },
    Anthropic { api_key: String, model: String },
}

#[derive(Serialize, Deserialize)]
pub struct AIResponse {
    pub command: String,
    pub explanation: String,
    pub dangerous: bool,
    pub danger_reason: Option<String>,
}

impl AIEngine {
    pub fn new(provider: AIProvider) -> Self {
        Self {
            client: Client::new(),
            provider,
        }
    }

    pub async fn translate_to_command(
        &self,
        natural_language: &str,
        os: &str,
        shell: &str,
        cwd: &str,
    ) -> Result<AIResponse, String> {
        let system_prompt = format!(
            r#"You are a shell command translator. Convert natural language to shell commands.

OS: {}
Shell: {}
Current directory: {}

Rules:
1. Return ONLY the command, no explanation in the command field
2. Set dangerous=true if the command could delete data, modify system files, or is irreversible
3. Provide a brief explanation
4. If the request is ambiguous, provide the most common interpretation

Respond in JSON format:
{{"command": "the shell command", "explanation": "brief explanation", "dangerous": false, "danger_reason": null}}"#,
            os, shell, cwd
        );

        let response_text = match &self.provider {
            AIProvider::Ollama { model, base_url } => {
                self.call_ollama(base_url, model, &system_prompt, natural_language)
                    .await?
            }
            AIProvider::OpenAI { api_key, model } => {
                self.call_openai(api_key, model, &system_prompt, natural_language)
                    .await?
            }
            AIProvider::Anthropic { api_key, model } => {
                self.call_anthropic(api_key, model, &system_prompt, natural_language)
                    .await?
            }
        };

        // Parse JSON from response
        let response: AIResponse = serde_json::from_str(&response_text)
            .or_else(|_| {
                // Try to extract JSON from markdown code block
                if let Some(start) = response_text.find('{') {
                    if let Some(end) = response_text.rfind('}') {
                        let json_str = &response_text[start..=end];
                        return serde_json::from_str(json_str);
                    }
                }
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Could not parse AI response",
                )))
            })
            .map_err(|e| format!("Failed to parse AI response: {} | Raw: {}", e, response_text))?;

        Ok(response)
    }

    pub async fn explain_command(&self, command: &str) -> Result<String, String> {
        let prompt = format!(
            "Explain this shell command in simple terms. Be brief (2-3 sentences max):\n\n{}",
            command
        );

        match &self.provider {
            AIProvider::Ollama { model, base_url } => {
                self.call_ollama(base_url, model, "You are a helpful shell expert. Be brief.", &prompt)
                    .await
            }
            AIProvider::OpenAI { api_key, model } => {
                self.call_openai(api_key, model, "You are a helpful shell expert. Be brief.", &prompt)
                    .await
            }
            AIProvider::Anthropic { api_key, model } => {
                self.call_anthropic(api_key, model, "You are a helpful shell expert. Be brief.", &prompt)
                    .await
            }
        }
    }

    pub async fn suggest_fix(
        &self,
        command: &str,
        error_output: &str,
    ) -> Result<AIResponse, String> {
        let prompt = format!(
            r#"This command failed:
Command: {}
Error: {}

Suggest the corrected command. Respond in JSON:
{{"command": "fixed command", "explanation": "what was wrong and how this fixes it", "dangerous": false, "danger_reason": null}}"#,
            command, error_output
        );

        let response_text = match &self.provider {
            AIProvider::Ollama { model, base_url } => {
                self.call_ollama(base_url, model, "You fix shell commands. Respond in JSON only.", &prompt)
                    .await?
            }
            AIProvider::OpenAI { api_key, model } => {
                self.call_openai(api_key, model, "You fix shell commands. Respond in JSON only.", &prompt)
                    .await?
            }
            AIProvider::Anthropic { api_key, model } => {
                self.call_anthropic(api_key, model, "You fix shell commands. Respond in JSON only.", &prompt)
                    .await?
            }
        };

        serde_json::from_str(&response_text)
            .or_else(|_| {
                if let Some(start) = response_text.find('{') {
                    if let Some(end) = response_text.rfind('}') {
                        return serde_json::from_str(&response_text[start..=end]);
                    }
                }
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Parse error",
                )))
            })
            .map_err(|e| format!("Parse error: {}", e))
    }

    // ─── Provider Calls ───────────────────────────

    async fn call_ollama(
        &self,
        base_url: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "system": system,
            "stream": false,
            "options": {
                "temperature": 0.1,
                "num_predict": 500
            }
        });

        let resp = self
            .client
            .post(format!("{}/api/generate", base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}. Is Ollama running?", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        data["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No response field from Ollama".to_string())
    }

    async fn call_openai(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.1,
            "max_tokens": 500
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        data["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in OpenAI response".to_string())
    }

    async fn call_anthropic(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 500,
            "system": system,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Anthropic request failed: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

        data["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in Anthropic response".to_string())
    }
}

/// Fetch installed models from a running Ollama instance
    pub async fn list_ollama_models(base_url: &str) -> Result<Vec<String>, String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("HTTP client error: {}", e))?;

        let resp = client
            .get(format!("{}/api/tags", base_url))
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Cannot connect to Ollama at {}. Is it running? ({})",
                    base_url, e
                )
            })?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Invalid response from Ollama: {}", e))?;

        let models = data["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }