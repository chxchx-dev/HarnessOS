use std::sync::Arc;

use super::{RegistryError, Skill, SkillDescriptor, Tool};

#[derive(Default)]
pub struct Registry {
    skills: Vec<Arc<dyn Skill>>,
    tools: Vec<Arc<dyn Tool>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_skill<S: Skill + 'static>(&mut self, skill: S) -> Result<(), RegistryError> {
        let key = format!("{}@{}", skill.id(), skill.version());
        if self
            .skills
            .iter()
            .any(|entry| entry.id() == skill.id() && entry.version() == skill.version())
        {
            return Err(RegistryError {
                kind: "Skill",
                id: key,
            });
        }
        self.skills.push(Arc::new(skill));
        Ok(())
    }

    pub fn register_tool<T: Tool + 'static>(&mut self, tool: T) -> Result<(), RegistryError> {
        if self.tools.iter().any(|entry| entry.id() == tool.id()) {
            return Err(RegistryError {
                kind: "Tool",
                id: tool.id().to_owned(),
            });
        }
        self.tools.push(Arc::new(tool));
        Ok(())
    }

    pub fn get_skill(&self, id: &str, version: &str) -> Option<Arc<dyn Skill>> {
        self.skills
            .iter()
            .find(|skill| skill.id() == id && skill.version() == version)
            .cloned()
    }

    pub fn get_tool(&self, id: &str) -> Option<Arc<dyn Tool>> {
        self.tools.iter().find(|tool| tool.id() == id).cloned()
    }

    pub fn list_skills(&self) -> Vec<SkillDescriptor> {
        self.skills
            .iter()
            .map(|skill| SkillDescriptor {
                id: skill.id().to_owned(),
                version: skill.version().to_owned(),
            })
            .collect()
    }
}
