use anyhow::{anyhow, Result};
use serde_json::json;
use std::env;

pub enum AIProvider {
    Ollama,
    OpenAI,
    Gemini,
}

pub struct AIService {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub model: String,
}

impl AIService {
    pub fn from_env() -> Self {
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            Self {
                provider: AIProvider::OpenAI,
                api_key: Some(key),
                model: env::var("RUNBOOK_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
            }
        } else if let Ok(key) = env::var("GEMINI_API_KEY") {
            Self {
                provider: AIProvider::Gemini,
                api_key: Some(key),
                model: env::var("RUNBOOK_MODEL").unwrap_or_else(|_| "gemini-1.5-pro".to_string()),
            }
        } else {
            Self {
                provider: AIProvider::Ollama,
                api_key: None,
                model: env::var("RUNBOOK_MODEL").unwrap_or_else(|_| "llama3".to_string()),
            }
        }
    }

    pub async fn summarize(&self, context: &str) -> Result<String> {
        match self.provider {
            AIProvider::Ollama => self.call_ollama(context).await,
            AIProvider::OpenAI => self.call_openai(context).await,
            AIProvider::Gemini => self.call_gemini(context).await,
        }
    }

    async fn call_ollama(&self, context: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let res = client
            .post("http://localhost:11434/api/generate")
            .json(&json!({
                "model": self.model,
                "prompt": format!("Summarize this engineering session into a concise problem description and solution. Output as Markdown.\n\nContext:\n{}", context),
                "stream": false
            }))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Ollama call failed: {}", res.status()));
        }

        let body: serde_json::Value = res.json().await?;
        Ok(body["response"].as_str().unwrap_or("").to_string())
    }

    async fn call_openai(&self, context: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let res = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key.as_ref().unwrap()))
            .json(&json!({
                "model": self.model,
                "messages": [
                    {"role": "system", "content": "You are a senior engineer summarizing a debugging/development session. Be concise and technical."},
                    {"role": "user", "content": format!("Summarize this session into 'Problem' and 'Solution' sections.\n\n{}", context)}
                ]
            }))
            .send()
            .await?;

        let body: serde_json::Value = res.json().await?;
        Ok(body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn call_gemini(&self, context: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model,
            self.api_key.as_ref().unwrap()
        );
        let res = client
            .post(url)
            .json(&json!({
                "contents": [{
                    "parts": [{
                        "text": format!("Summarize this engineering session into a concise problem description and solution. Output as Markdown.\n\nContext:\n{}", context)
                    }]
                }]
            }))
            .send()
            .await?;

        let body: serde_json::Value = res.json().await?;
        Ok(body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
}
