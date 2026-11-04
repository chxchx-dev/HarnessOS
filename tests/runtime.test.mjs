import test from "node:test";
import assert from "node:assert/strict";
import { createDemoHarness } from "../src/bootstrap.ts";
import { Registry } from "../src/core/registry.ts";
import { HarnessRuntime } from "../src/core/runtime.ts";
import { EphemeralMemory } from "../src/adapters/ephemeral-memory.ts";

const principal = (tenantId="education-demo", scopes=["knowledge:read", "model:use"]) =>
  ({ tenantId, subjectId: "u1", authType: "jwt", scopes });
const req = (overrides={}) => ({
  skillId: "knowledge.retrieve", version: "1.0.0",
  input: { question: "lecciones educativas" }, principal: principal(), correlationId: "t1", ...overrides,
});
const run = async (overrides) => createDemoHarness().runtime.execute(req(overrides));

test("consultar documentos aprobados sin inferencia", async () => {
  const result = await run();
  assert.equal(result.ok, true);
  assert.equal(result.output.sources[0].id, "DEMO-EDU-01");
  assert.match(result.output.sources[0].title, /^Ejemplo ficticio:/);
  assert.equal(result.trace.toolCallCount, 1);
  assert.equal("question" in result.trace, false);
});

test("evidencia ausente => abstención, no fallback inventado", async () => {
  const result = await run({ input: {question: "hipopotamos"} });
  assert.equal(result.ok, false);
  assert.equal(result.error.code, "EVIDENCE_MISSING");
});

test("aislamiento por tenant: una organización no ve documentos de otra", async () => {
  const result = await run({ principal: principal("appointments-demo") });
  assert.equal(result.ok, false);
  assert.equal(result.error.code, "EVIDENCE_MISSING");
});

test("scope insuficiente bloquea antes de llamar herramientas", async () => {
  const result = await run({ principal: principal("education-demo",[]) });
  assert.equal(result.ok, false);
  assert.equal(result.error.code, "UNAUTHORIZED");
  assert.equal(result.trace.toolCallCount, 0);
});

test("versión inexistente no ejecuta una versión accidental", async () => {
  const result = await run({version:"2.0.0"});
  assert.equal(result.ok, false);
  assert.equal(result.error.code, "UNKNOWN_SKILL");
});

test("input inválido bloqueado", async () => {
  const result = await run({input:{question:"x"}});
  assert.equal(result.ok, false);
  assert.equal(result.error.code, "BAD_INPUT");
});

test("presupuesto inválido o muy pequeño", async () => {
  const invalid = await run({requestedBudget:{maxToolCalls:0}});
  assert.equal(invalid.error.code, "INVALID_REQUEST");
  const small = await run({requestedBudget:{timeoutMs:0}});
  assert.equal(small.error.code, "INVALID_REQUEST");
});

test("budget no ampliable; herramienta fuera de allowlist denegada", async () => {
  const registry = new Registry();
  registry.registerSkill({
    id:"no-tools",version:"1",requiredScopes:[],allowedTools:[],validate:()=>true,
    run:(_input,ctx)=>ctx.callTool("knowledge.search",{query:"hola"}),
  });
  registry.registerTool({id:"knowledge.search",requiredScope:"knowledge:read",validate:()=>true, execute:async()=>[]});
  const result = await new HarnessRuntime(registry).execute(req({skillId:"no-tools",version:"1", requestedBudget:{maxToolCalls:9999}}));
  assert.equal(result.ok,false);
  assert.equal(result.error.code,"TOOL_FORBIDDEN");
});

test("respuesta demo documenta que no está verificada", async () => {
  const result = await run({skillId:"knowledge.answer"});
  assert.equal(result.ok,true);
  assert.equal(result.output.verified,false);
  assert.match(result.output.modelUsed,/deterministic-demo/);
});

test("memoria efímera: exige identidad, scopes y separa tenants", () => {
  const memory = new EphemeralMemory();
  const a = principal("education-demo", ["memory:read","memory:write"]);
  const b = principal("appointments-demo", ["memory:read","memory:write"]);
  memory.set(a,"short","test");
  assert.equal(memory.get(a,"short"),"test");
  assert.equal(memory.get(b,"short"),undefined);
  assert.throws(()=>memory.get({...a,subjectId:null},"short"),/identidad|usuario/);
  assert.throws(()=>memory.set(principal(),"short","x"),/Escritura no autorizada/);
});

test("timeout termina resultado aunque una skill no coopere", async () => {
  const registry = new Registry();
  registry.registerSkill({
    id:"sleep",version:"1",requiredScopes:[],allowedTools:[],validate:()=>true,
    run:()=>new Promise(resolve=>setTimeout(()=>resolve("late"),80)),
  });
  const result = await new HarnessRuntime(registry).execute(req({skillId:"sleep",version:"1",requestedBudget:{timeoutMs:5}}));
  assert.equal(result.ok,false);
  assert.equal(result.error.code,"BUDGET_EXCEEDED");
});
