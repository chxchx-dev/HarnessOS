use std::sync::Arc;

use serde_json::{Value, json};

use crate::core::{ErrorCode, HarnessError, ModelPort, Skill, SkillContext, SourceExcerpt};

use super::knowledge_retrieve::valid_question;

pub struct KnowledgeAnswerSkill {
    required_scopes: Vec<String>,
    allowed_tools: Vec<String>,
    model: Arc<dyn ModelPort>,
}

impl KnowledgeAnswerSkill {
    pub fn new(model: Arc<dyn ModelPort>) -> Self {
        Self {
            required_scopes: vec!["knowledge:read".to_owned(), "model:use".to_owned()],
            allowed_tools: vec!["knowledge.search".to_owned()],
            model,
        }
    }
}

impl Skill for KnowledgeAnswerSkill {
    fn id(&self) -> &str {
        "knowledge.answer"
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
        let value = context.call_tool("knowledge.search", &json!({ "query": question }))?;
        let sources = parse_sources(&value)?;
        if sources.is_empty() {
            return Err(HarnessError::new(
                ErrorCode::EvidenceMissing,
                "No hay evidencia documental suficiente.",
            ));
        }
        let answer = self.model.generate(question, &sources, context.signal)?;
        let citations: Vec<Value> = sources
            .iter()
            .map(|source| json!({ "id": source.id, "title": source.title }))
            .collect();
        Ok(json!({
            "answer": answer,
            "citations": citations,
            "modelUsed": self.model.name(),
            "verified": false
        }))
    }
}

fn parse_sources(value: &Value) -> Result<Vec<SourceExcerpt>, HarnessError> {
    let Some(items) = value.as_array() else {
        return Err(HarnessError::new(
            ErrorCode::DependencyFailed,
            "Falló una dependencia autorizada.",
        ));
    };
    items
        .iter()
        .map(|item| {
            let Some(id) = item.get("id").and_then(Value::as_str) else {
                return Err(HarnessError::new(
                    ErrorCode::DependencyFailed,
                    "Falló una dependencia autorizada.",
                ));
            };
            let Some(title) = item.get("title").and_then(Value::as_str) else {
                return Err(HarnessError::new(
                    ErrorCode::DependencyFailed,
                    "Falló una dependencia autorizada.",
                ));
            };
            let Some(excerpt) = item.get("excerpt").and_then(Value::as_str) else {
                return Err(HarnessError::new(
                    ErrorCode::DependencyFailed,
                    "Falló una dependencia autorizada.",
                ));
            };
            Ok(SourceExcerpt {
                id: id.to_owned(),
                title: title.to_owned(),
                excerpt: excerpt.to_owned(),
            })
        })
        .collect()
}
