use serde_json::{Value, json};

use crate::core::{ErrorCode, HarnessError, Skill, SkillContext};

pub struct KnowledgeRetrieveSkill {
    required_scopes: Vec<String>,
    allowed_tools: Vec<String>,
}

impl Default for KnowledgeRetrieveSkill {
    fn default() -> Self {
        Self {
            required_scopes: vec!["knowledge:read".to_owned()],
            allowed_tools: vec!["knowledge.search".to_owned()],
        }
    }
}

impl Skill for KnowledgeRetrieveSkill {
    fn id(&self) -> &str {
        "knowledge.retrieve"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn required_scopes(&self) -> &[String] {
        &self.required_scopes
    }
    fn allowed_tools(&self) -> &[String] {
        &self.allowed_tools
    }

    fn validate(&self, input: &Value) -> bool {
        valid_question(input)
    }

    fn run(&self, input: &Value, context: &SkillContext<'_>) -> Result<Value, HarnessError> {
        let question = input
            .get("question")
            .and_then(Value::as_str)
            .ok_or(HarnessError::new(
                ErrorCode::BadInput,
                "Entrada inválida para la skill.",
            ))?;
        let sources = context.call_tool("knowledge.search", &json!({ "query": question }))?;
        if sources.as_array().is_none_or(Vec::is_empty) {
            return Err(HarnessError::new(
                ErrorCode::EvidenceMissing,
                "No hay evidencia documental suficiente.",
            ));
        }
        Ok(json!({ "sources": sources, "answer": null, "modelUsed": null }))
    }
}

pub(crate) fn valid_question(input: &Value) -> bool {
    input
        .get("question")
        .and_then(Value::as_str)
        .is_some_and(|question| {
            let length = question.encode_utf16().count();
            (2..=300).contains(&length) && question.trim().encode_utf16().count() >= 2
        })
}
