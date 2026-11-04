import { Registry } from "./core/registry.ts";
import { HarnessRuntime } from "./core/runtime.ts";
import type { ModelPort } from "./core/contracts.ts";
import { demoKnowledgeTool } from "./adapters/demo-knowledge.ts";
import { knowledgeRetrieveSkill } from "./skills/knowledge-retrieve.ts";
import { knowledgeAnswerSkill } from "./skills/knowledge-answer.ts";
import { demoModel } from "./models/demo-model.ts";

export function createDemoHarness(model: ModelPort = demoModel) {
  const registry = new Registry();
  registry.registerTool(demoKnowledgeTool);
  registry.registerSkill(knowledgeRetrieveSkill);
  registry.registerSkill(knowledgeAnswerSkill(model));
  return { runtime: new HarnessRuntime(registry), registry };
}
