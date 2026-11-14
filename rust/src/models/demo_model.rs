use crate::core::{CancellationToken, HarnessError, ModelPort, SourceExcerpt};

pub struct DemoModel;

impl ModelPort for DemoModel {
    fn name(&self) -> &str {
        "deterministic-demo (NO LLM)"
    }

    fn generate(
        &self,
        _question: &str,
        sources: &[SourceExcerpt],
        _signal: &CancellationToken,
    ) -> Result<String, HarnessError> {
        Ok(format!(
            "[DEMO - no generado por IA] {}",
            sources
                .iter()
                .map(|source| source.excerpt.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        ))
    }
}
