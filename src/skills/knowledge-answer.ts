import type { ModelPort, Skill, SourceExcerpt } from "../core/contracts.ts";
import { HarnessError } from "../core/errors.ts";
import { isQuestionInput } from "./knowledge-retrieve.ts";

export function knowledgeAnswerSkill(model: ModelPort): Skill {
  return {
    id: "knowledge.answer", version: "1.0.0", requiredScopes: ["knowledge:read", "model:use"],
    allowedTools: ["knowledge.search"], validate: isQuestionInput,
    async run(input, ctx) {
      const question = (input as {question: string}).question;
      const sources = await ctx.callTool("knowledge.search", { query: question }) as SourceExcerpt[];
      if (sources.length === 0) throw new HarnessError("EVIDENCE_MISSING", "No hay evidencia documental suficiente.");
      const answer = await model.generate(question, sources, ctx.signal);
      // Las citas provienen de la herramienta, NO de la prosa del modelo.
      // Una cita presente no garantiza que la respuesta esté fundamentada.
      return { answer, citations: sources.map(({ id, title }) => ({ id, title })), modelUsed: model.name, verified: false };
    },
  };
}
