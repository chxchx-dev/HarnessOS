use serde_json::{Value, json};

use crate::core::{ErrorCode, HarnessError, Tool, ToolContext};

struct DemoDocument {
    tenant_id: &'static str,
    id: &'static str,
    title: &'static str,
    excerpt: &'static str,
}

const DOCUMENTS: [DemoDocument; 4] = [
    DemoDocument {
        tenant_id: "education-demo",
        id: "DEMO-EDU-01",
        title: "Ejemplo ficticio: material educativo",
        excerpt: "En este ejemplo simulado, las lecciones educativas incluyen fuentes de consulta y una estructura clara.",
    },
    DemoDocument {
        tenant_id: "education-demo",
        id: "DEMO-EDU-02",
        title: "Ejemplo ficticio: continuidad de una actividad",
        excerpt: "En este ejemplo simulado, una actividad conserva su identificador para que el usuario pueda retomarla.",
    },
    DemoDocument {
        tenant_id: "appointments-demo",
        id: "DEMO-APP-01",
        title: "Ejemplo ficticio: agenda",
        excerpt: "La agenda de prueba permite consultar horarios simulados sin datos de pacientes.",
    },
    DemoDocument {
        tenant_id: "catalog-demo",
        id: "DEMO-CAT-01",
        title: "Ejemplo ficticio: catálogo comercial",
        excerpt: "Una consulta de prueba muestra artículos de un catálogo ficticio dentro de su tenant.",
    },
];

pub struct DemoKnowledgeTool {
    id: String,
    required_scope: String,
}

impl Default for DemoKnowledgeTool {
    fn default() -> Self {
        Self {
            id: "knowledge.search".to_owned(),
            required_scope: "knowledge:read".to_owned(),
        }
    }
}

impl Tool for DemoKnowledgeTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn required_scope(&self) -> &str {
        &self.required_scope
    }

    fn validate(&self, input: &Value) -> bool {
        input
            .get("query")
            .and_then(Value::as_str)
            .is_some_and(|query| {
                let trimmed_length = query.trim().encode_utf16().count();
                (2..=300).contains(&trimmed_length)
            })
    }

    fn execute(&self, input: &Value, context: &ToolContext<'_>) -> Result<Value, HarnessError> {
        if context.signal.is_cancelled() {
            return Ok(json!([]));
        }
        let query = input
            .get("query")
            .and_then(Value::as_str)
            .ok_or(HarnessError::new(
                ErrorCode::BadInput,
                "Argumentos de herramienta inválidos.",
            ))?
            .to_lowercase();
        let tokens = tokenize(&query);
        let results: Vec<Value> = DOCUMENTS
            .iter()
            .filter(|document| document.tenant_id == context.principal.tenant_id)
            .filter(|document| {
                let content = format!("{} {}", document.title, document.excerpt).to_lowercase();
                tokens.iter().any(|token| content.contains(token))
            })
            .take(3)
            .map(|document| json!({ "id": document.id, "title": document.title, "excerpt": document.excerpt }))
            .collect();
        Ok(Value::Array(results))
    }
}

fn tokenize(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in query.chars() {
        if character.is_alphanumeric() {
            current.extend(character.to_lowercase());
        } else if current.chars().count() >= 3 {
            tokens.push(std::mem::take(&mut current));
        } else {
            current.clear();
        }
    }
    if current.chars().count() >= 3 {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::tokenize;

    #[test]
    fn tokenizes_spanish_words_and_discards_short_tokens() {
        assert_eq!(
            tokenize("¿Qué incluyen las lecciones educativas?"),
            ["qué", "incluyen", "las", "lecciones", "educativas"],
        );
    }
}
