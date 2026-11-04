import { createDemoHarness } from "../src/bootstrap.ts";
import { createOllamaModel } from "../src/models/ollama.ts";

const useOllama = process.env.HARNESS_MODEL === "ollama";
const model = useOllama ? createOllamaModel({
  model: process.env.OLLAMA_MODEL ?? "qwen3:4b-instruct",
  url: process.env.OLLAMA_URL ?? "http://127.0.0.1:11434",
}) : undefined;
const { runtime, registry } = createDemoHarness(model);
console.log("HarnessOS · demostración educativa");
console.log("DATOS FICTICIOS: sin conexión con sistemas ni datos reales.\n");
console.log("Capacidades disponibles:");
for (const skill of registry.listSkills()) {
  console.log(`- ${skill.id}@${skill.version}`);
}

for (const [tenantId, question] of [
  ["education-demo", "¿Qué incluyen las lecciones educativas?"],
  ["appointments-demo", "¿Qué ofrece la agenda de prueba?"],
]) {
  const result = await runtime.execute({
    skillId: "knowledge.answer", version: "1.0.0", input: { question },
    principal: { tenantId, subjectId: "local-test-user", authType: "jwt", scopes: ["knowledge:read", "model:use"] },
    correlationId: `local-demo-${tenantId}`,
  });
  console.log(`\nCaso: ${tenantId} (tenant simulado)`);
  console.log(`Pregunta: ${question}`);
  if (!result.ok) {
    console.log(`Resultado: ${result.error.code} — ${result.error.message}`);
    process.exitCode = 1;
    continue;
  }

  console.log(`Respuesta: ${result.output.answer}`);
  console.log("Fuentes ficticias:");
  for (const citation of result.output.citations) {
    console.log(`- ${citation.id}: ${citation.title}`);
  }
  console.log(`Modelo: ${result.output.modelUsed}`);
  console.log(`Verificada: ${result.output.verified ? "sí" : "no; requiere evaluación"}`);
  console.log(`Traza: ${result.trace.toolCallCount} herramienta(s), ${result.trace.durationMs} ms`);
}
