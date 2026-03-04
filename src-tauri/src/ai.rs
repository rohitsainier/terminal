use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AIResponse {
    pub command: String,
    pub explanation: String,
    pub dangerous: bool,
    pub danger_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ChatResponse {
    #[serde(rename = "message")]
    Message(String),
    #[serde(rename = "tool_call")]
    ToolCall {
        server: String,
        tool: String,
        arguments: serde_json::Value,
    },
}

impl AIEngine {
    pub fn new(provider: AIProvider) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, provider }
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

Respond in JSON format ONLY (no markdown, no code blocks):
{{"command": "the shell command", "explanation": "brief explanation", "dangerous": false, "danger_reason": null}}"#,
            os, shell, cwd
        );

        let response_text = self
            .call_provider(&system_prompt, natural_language)
            .await?;

        parse_ai_json(&response_text)
    }

    pub async fn explain_command(&self, command: &str) -> Result<String, String> {
        let prompt = format!(
            "Explain this shell command in simple terms. Be brief (2-3 sentences max). No markdown formatting:\n\n{}",
            command
        );

        self.call_provider(
            "You are a helpful shell expert. Be brief and clear. No markdown.",
            &prompt,
        )
        .await
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

Suggest the corrected command. Respond in JSON ONLY (no markdown):
{{"command": "fixed command", "explanation": "what was wrong and how this fixes it", "dangerous": false, "danger_reason": null}}"#,
            command, error_output
        );

        let response_text = self
            .call_provider(
                "You fix shell commands. Respond in JSON only. No markdown code blocks.",
                &prompt,
            )
            .await?;

        parse_ai_json(&response_text)
    }

    /// Route to the correct provider
    async fn call_provider(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<String, String> {
        match &self.provider {
            AIProvider::Ollama { model, base_url } => {
                self.call_ollama(base_url, model, system, prompt).await
            }
            AIProvider::OpenAI { api_key, model } => {
                self.call_openai(api_key, model, system, prompt).await
            }
            AIProvider::Anthropic { api_key, model } => {
                self.call_anthropic(api_key, model, system, prompt).await
            }
        }
    }

    // ─── Ollama ───────────────────────────────────

    async fn call_ollama(
        &self,
        base_url: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<String, String> {
        // First try /api/chat (newer Ollama versions prefer this)
        match self
            .call_ollama_chat(base_url, model, system, prompt)
            .await
        {
            Ok(response) => return Ok(response),
            Err(chat_err) => {
                eprintln!(
                    "[AI] /api/chat failed ({}), trying /api/generate...",
                    chat_err
                );
            }
        }

        // Fallback to /api/generate
        self.call_ollama_generate(base_url, model, system, prompt)
            .await
    }

    async fn call_ollama_chat(
        &self,
        base_url: &str,
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
            "stream": false,
            "options": {
                "temperature": 0.1,
                "num_predict": 500
            }
        });

        let url = format!("{}/api/chat", base_url);
        eprintln!("[AI] POST {} with model={}", url, model);

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Cannot connect to Ollama at {}. Is it running? Error: {}",
                    base_url, e
                )
            })?;

        let status = resp.status();
        let raw_text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read Ollama response: {}", e))?;

        eprintln!(
            "[AI] Ollama /api/chat status={}, body_len={}",
            status,
            raw_text.len()
        );

        if !status.is_success() {
            return Err(format_ollama_error(&raw_text, status.as_u16(), model));
        }

        let data: serde_json::Value = serde_json::from_str(&raw_text)
            .map_err(|e| format!("Invalid JSON from Ollama: {} | Raw: {}", e, truncate(&raw_text, 200)))?;

        // Check for error field
        if let Some(err) = data["error"].as_str() {
            return Err(format_ollama_model_error(err, model));
        }

        // /api/chat returns: { "message": { "content": "..." } }
        if let Some(content) = data["message"]["content"].as_str() {
            if !content.is_empty() {
                return Ok(content.to_string());
            }
        }

        Err(format!(
            "Unexpected Ollama /api/chat response structure. Raw: {}",
            truncate(&raw_text, 300)
        ))
    }

    async fn call_ollama_generate(
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

        let url = format!("{}/api/generate", base_url);
        eprintln!("[AI] POST {} with model={}", url, model);

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Cannot connect to Ollama at {}. Is it running? Error: {}",
                    base_url, e
                )
            })?;

        let status = resp.status();
        let raw_text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read Ollama response: {}", e))?;

        eprintln!(
            "[AI] Ollama /api/generate status={}, body_len={}",
            status,
            raw_text.len()
        );

        if !status.is_success() {
            return Err(format_ollama_error(&raw_text, status.as_u16(), model));
        }

        let data: serde_json::Value = serde_json::from_str(&raw_text)
            .map_err(|e| format!("Invalid JSON from Ollama: {} | Raw: {}", e, truncate(&raw_text, 200)))?;

        // Check for error field first
        if let Some(err) = data["error"].as_str() {
            return Err(format_ollama_model_error(err, model));
        }

        // /api/generate returns: { "response": "..." }
        if let Some(response) = data["response"].as_str() {
            if !response.is_empty() {
                return Ok(response.to_string());
            }
        }

        // Sometimes response comes as "output" in newer versions
        if let Some(output) = data["output"].as_str() {
            if !output.is_empty() {
                return Ok(output.to_string());
            }
        }

        // Log full response for debugging
        eprintln!(
            "[AI] Ollama response had no 'response' field. Full JSON: {}",
            truncate(&raw_text, 500)
        );

        Err(format!(
            "Ollama returned an empty response. Model '{}' may not support this request.\n\
             Try a different model (e.g., llama3.2 or mistral).\n\
             Debug: {}",
            model,
            truncate(&raw_text, 200)
        ))
    }

    // ─── OpenAI ───────────────────────────────────

    async fn call_openai(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<String, String> {
        if api_key.is_empty() {
            return Err("OpenAI API key is empty. Add it in Settings → AI Provider.".into());
        }

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

        let status = resp.status();
        let raw_text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read OpenAI response: {}", e))?;

        if !status.is_success() {
            let data: serde_json::Value =
                serde_json::from_str(&raw_text).unwrap_or_default();
            let msg = data["error"]["message"]
                .as_str()
                .unwrap_or("Unknown OpenAI error");
            return Err(format!("OpenAI error ({}): {}", status, msg));
        }

        let data: serde_json::Value = serde_json::from_str(&raw_text)
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        data["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in OpenAI response".to_string())
    }

    // ─── Anthropic ────────────────────────────────

    async fn call_anthropic(
        &self,
        api_key: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<String, String> {
        if api_key.is_empty() {
            return Err(
                "Anthropic API key is empty. Add it in Settings → AI Provider.".into(),
            );
        }

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

        let status = resp.status();
        let raw_text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read Anthropic response: {}", e))?;

        if !status.is_success() {
            let data: serde_json::Value =
                serde_json::from_str(&raw_text).unwrap_or_default();
            let msg = data["error"]["message"]
                .as_str()
                .unwrap_or("Unknown Anthropic error");
            return Err(format!("Anthropic error ({}): {}", status, msg));
        }

        let data: serde_json::Value = serde_json::from_str(&raw_text)
            .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

        data["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in Anthropic response".to_string())
    }

    /// Generate a task execution plan from a user request.
    /// Returns structured JSON with title and steps.
    pub async fn plan_task(
        &self,
        messages: &[ChatMessage],
        mcp_tools_context: &str,
        os: &str,
        shell: &str,
    ) -> Result<serde_json::Value, String> {
        let system_prompt = format!(
            r#"You are a task planner with access to MCP tools.

OS: {os}
Shell: {shell}

{mcp_tools_context}

Your job is to analyze the user's request and create an execution plan.
Break the task into clear, sequential steps. Each step should be a single action.

You MUST respond with this exact JSON format. No markdown, no extra text:
{{"plan": {{"title": "Short descriptive title", "steps": [{{"step": 1, "description": "What this step does", "tool": "tool_name_or_null"}}, {{"step": 2, "description": "...", "tool": "..."}}]}}}}

Rules:
- Keep the title short (under 60 chars)
- Each step description should be clear and actionable
- Set "tool" to the exact MCP tool name that will be used, or null if no tool needed
- Order steps logically — later steps may depend on earlier results
- For simple questions that need no tools, return a single step with tool: null
- Include 2-10 steps maximum. Be practical, not granular."#,
            os = os, shell = shell, mcp_tools_context = mcp_tools_context
        );

        let mut prompt_parts: Vec<String> = Vec::new();
        for m in messages {
            match m.role.as_str() {
                "user" => prompt_parts.push(format!("User: {}", m.content)),
                "assistant" => prompt_parts.push(format!("Assistant: {}", m.content)),
                other => prompt_parts.push(format!("{}: {}", other, m.content)),
            }
        }
        let user_prompt = prompt_parts.join("\n\n");

        let response_text = self.call_provider(&system_prompt, &user_prompt).await?;
        let cleaned = extract_json_from_text(&response_text);

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&cleaned) {
            if value.get("plan").is_some() {
                return Ok(value);
            }
        }

        // Fallback: single-step plan
        let user_text = messages.last().map(|m| m.content.as_str()).unwrap_or("Complete task");
        Ok(serde_json::json!({
            "plan": {
                "title": user_text.chars().take(60).collect::<String>(),
                "steps": [{"step": 1, "description": user_text, "tool": null}]
            }
        }))
    }

    pub async fn chat_with_tools(
        &self,
        messages: &[ChatMessage],
        mcp_tools_context: &str,
        os: &str,
        shell: &str,
        plan_context: Option<&str>,
    ) -> Result<ChatResponse, String> {
        let plan_section = plan_context
            .map(|p| format!("\n\nCURRENT PLAN STEP:\n{}\nFocus on completing THIS step. When done, respond with a message summarizing the result.", p))
            .unwrap_or_default();

        let system_prompt = format!(
            r#"You are an AI assistant with access to MCP tools.

OS: {os}
Shell: {shell}

{mcp_tools_context}

You MUST respond with exactly ONE of these JSON formats. No markdown, no extra text.

To use a tool:
{{"tool_call": {{"server": "server_name", "tool": "tool_name", "arguments": {{}}}}}}

To reply to the user:
{{"message": "your response"}}

Rules:
- Use EXACT tool names from the list (e.g. "create_frame" not "figma/create_frame")
- For multi-step tasks: call ONE tool per response. You will see the result and can call another.
- Only send a "message" when you are truly done or have nothing more to do with tools.
- If a tool errors, try to fix the issue or explain what happened.{plan_section}"#,
            os = os, shell = shell, mcp_tools_context = mcp_tools_context, plan_section = plan_section
        );

        // Build a single prompt from all messages
        let mut prompt_parts: Vec<String> = Vec::new();
        for m in messages {
            match m.role.as_str() {
                "user" => prompt_parts.push(format!("User: {}", m.content)),
                "assistant" => prompt_parts.push(format!("Assistant: {}", m.content)),
                "tool_result" => prompt_parts.push(format!("[Tool Result]\n{}", m.content)),
                "tool_error" => prompt_parts.push(format!("[Tool Error]\n{}", m.content)),
                other => prompt_parts.push(format!("{}: {}", other, m.content)),
            }
        }

        let user_prompt = prompt_parts.join("\n\n");

        let response_text = self.call_provider(&system_prompt, &user_prompt).await?;

        // Parse response
        let cleaned = extract_json_from_text(&response_text);

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&cleaned) {
            // Tool call
            if let Some(tc) = value.get("tool_call") {
                return Ok(ChatResponse::ToolCall {
                    server: tc["server"].as_str().unwrap_or("").to_string(),
                    tool: tc["tool"].as_str().unwrap_or("").to_string(),
                    arguments: tc.get("arguments").cloned().unwrap_or(serde_json::json!({})),
                });
            }
            // Message
            if let Some(msg) = value["message"].as_str() {
                return Ok(ChatResponse::Message(msg.to_string()));
            }
        }

        // Fallback: treat as plain message
        Ok(ChatResponse::Message(response_text))
    }
}

// ─── Helper Functions ─────────────────────────────

/// Parse JSON from AI response text, handling markdown code blocks
fn parse_ai_json(text: &str) -> Result<AIResponse, String> {
    let cleaned = extract_json_from_text(text);

    serde_json::from_str::<AIResponse>(&cleaned).map_err(|e| {
        format!(
            "Failed to parse AI response as JSON: {}\nRaw text: {}",
            e,
            truncate(text, 300)
        )
    })
}

/// Extract JSON object from text that might contain markdown or extra content
fn extract_json_from_text(text: &str) -> String {
    let text = text.trim();

    // Try direct parse first
    if text.starts_with('{') && text.ends_with('}') {
        return text.to_string();
    }

    // Remove markdown code blocks: ```json ... ``` or ``` ... ```
    let stripped = text
        .trim_start_matches("```json")
        .trim_start_matches("```JSON")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if stripped.starts_with('{') && stripped.ends_with('}') {
        return stripped.to_string();
    }

    // Find first { and last }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
    }

    // Return as-is and let the caller handle the error
    text.to_string()
}

/// Format a user-friendly error when Ollama returns an error
fn format_ollama_error(raw: &str, status: u16, model: &str) -> String {
    let data: serde_json::Value =
        serde_json::from_str(raw).unwrap_or_default();

    if let Some(err) = data["error"].as_str() {
        return format_ollama_model_error(err, model);
    }

    format!(
        "Ollama returned HTTP {} for model '{}'.\n\
         Make sure:\n\
         1. Ollama is running: ollama serve\n\
         2. Model is installed: ollama pull {}\n\
         Raw: {}",
        status,
        model,
        model,
        truncate(raw, 200)
    )
}

/// Format Ollama model-specific errors with helpful instructions
fn format_ollama_model_error(err: &str, model: &str) -> String {
    let err_lower = err.to_lowercase();

    if err_lower.contains("not found") || err_lower.contains("pull") {
        format!(
            "Model '{}' is not installed.\n\
             Run this in your terminal to install it:\n\n\
             ollama pull {}\n\n\
             Then try again.",
            model, model
        )
    } else if err_lower.contains("loading")
        || err_lower.contains("initializing")
    {
        format!(
            "Model '{}' is still loading. Wait a moment and try again.",
            model
        )
    } else if err_lower.contains("memory") || err_lower.contains("ram") {
        format!(
            "Not enough memory to run '{}'. Try a smaller model:\n\n\
             ollama pull llama3.2:1b\n\n\
             Then select it in Settings.",
            model
        )
    } else {
        format!("Ollama error: {}", err)
    }
}

/// Truncate string for error messages
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Fetch installed models from a running Ollama instance
pub async fn list_ollama_models(base_url: &str) -> Result<Vec<String>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let url = format!("{}/api/tags", base_url);
    eprintln!("[AI] Fetching models from {}", url);

    let resp = client.get(&url).send().await.map_err(|e| {
        format!(
            "Cannot connect to Ollama at {}.\n\
             Make sure Ollama is running:\n\n\
             ollama serve\n\n\
             Error: {}",
            base_url, e
        )
    })?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!(
            "Ollama returned HTTP {}. Is it running correctly?",
            status
        ));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid response from Ollama: {}", e))?;

    let models:  Vec<String>  = data["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    // Try "name" field first, then "model"
                    m["name"]
                        .as_str()
                        .or_else(|| m["model"].as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    eprintln!("[AI] Found {} models: {:?}", models.len(), models);

    if models.is_empty() {
        return Err(
            "Ollama is running but has no models installed.\n\
             Install one with:\n\n\
             ollama pull llama3.2"
                .to_string(),
        );
    }

    Ok(models)
}

/// Fetch available models from OpenAI
pub async fn list_openai_models(api_key: &str) -> Result<Vec<String>, String> {
    if api_key.trim().is_empty() {
        return Err("OpenAI API key is empty. Add it in Settings → AI Provider.".into());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let resp = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    let status = resp.status();
    let raw_text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read OpenAI response: {}", e))?;

    if !status.is_success() {
        let data: serde_json::Value = serde_json::from_str(&raw_text).unwrap_or_default();
        let msg = data["error"]["message"]
            .as_str()
            .unwrap_or("Unknown OpenAI error");
        return Err(format!("OpenAI error ({}): {}", status, msg));
    }

    let data: serde_json::Value = serde_json::from_str(&raw_text)
        .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

    let mut models: Vec<String> = data["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    models.sort();
    models.dedup();

    if models.is_empty() {
        return Err("OpenAI returned no models for this API key.".to_string());
    }

    Ok(models)
}

