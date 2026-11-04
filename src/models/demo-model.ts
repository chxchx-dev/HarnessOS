import type { ModelPort, SourceExcerpt } from "../core/contracts.ts";

/** Salida determinista, sin inferencia: permite ejecutar la demo offline. */
export const demoModel: ModelPort = {
  name: "deterministic-demo (NO LLM)",
  async generate(_question: string, sources: readonly SourceExcerpt[]): Promise<string> {
    return `[DEMO - no generado por IA] ${sources.map((s) => s.excerpt).join(" ")}`;
  },
};
