use std::sync::Arc;

use crate::adapters::DemoKnowledgeTool;
use crate::core::{HarnessRuntime, ModelPort, Registry, RegistryError};
use crate::skills::{KnowledgeAnswerSkill, KnowledgeRetrieveSkill};

pub fn create_demo_harness(model: Arc<dyn ModelPort>) -> Result<HarnessRuntime, RegistryError> {
    let mut registry = Registry::new();
    registry.register_tool(DemoKnowledgeTool::default())?;
    registry.register_skill(KnowledgeRetrieveSkill::default())?;
    registry.register_skill(KnowledgeAnswerSkill::new(model))?;
    Ok(HarnessRuntime::new(registry))
}
