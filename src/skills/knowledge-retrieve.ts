import type { Skill, SourceExcerpt } from "../core/contracts.ts";
import { HarnessError } from "../core/errors.ts";

export function isQuestionInput(input: unknown): boolean {
  return typeof input === "object" && input !== null &&
    "question" in input && typeof input.question === "string" &&
    input.question.trim().length >= 2 && input.question.length <= 300;
}

export const knowledgeRetrieveSkill: Skill = {
  id: "knowledge.retrieve", version: "1.0.0", requiredScopes: ["knowledge:read"],
  allowedTools: ["knowledge.search"], validate: isQuestionInput,
  async run(input, ctx) {
    const question = (input as {question: string}).question;
    const sources = await ctx.callTool("knowledge.search", { query: question }) as SourceExcerpt[];
    if (sources.length === 0) throw new HarnessError("EVIDENCE_MISSING", "No hay evidencia documental suficiente.");
    return { sources, answer: null, modelUsed: null };
  },
};
