import type { Budget, Principal, RunRequest, RunResult, SkillContext } from "./contracts.ts";
import { HarnessError, isHarnessError } from "./errors.ts";
import { Registry } from "./registry.ts";

const SERVER_BUDGET: Readonly<Budget> = { maxToolCalls: 4, timeoutMs: 8_000 };

function validPrincipal(value: Principal): boolean {
  return !!value && typeof value.tenantId === "string" && value.tenantId.trim().length > 0
    && (value.subjectId === null || (typeof value.subjectId === "string" && value.subjectId.trim().length > 0))
    && (value.authType === "jwt" || value.authType === "service")
    && Array.isArray(value.scopes) && value.scopes.every((scope: unknown) => typeof scope === "string");
}

function reduceBudget(requested?: Partial<Budget>): Budget {
  const maxToolCalls = requested?.maxToolCalls ?? SERVER_BUDGET.maxToolCalls;
  const timeoutMs = requested?.timeoutMs ?? SERVER_BUDGET.timeoutMs;
  if (!Number.isSafeInteger(maxToolCalls) || maxToolCalls < 1 || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1) {
    throw new HarnessError("INVALID_REQUEST", "El presupuesto debe contener enteros positivos.");
  }
  return {
    maxToolCalls: Math.min(maxToolCalls, SERVER_BUDGET.maxToolCalls),
    timeoutMs: Math.min(timeoutMs, SERVER_BUDGET.timeoutMs),
  };
}

/** No es transporte HTTP. El servidor debe autenticar y construir Principal antes de llamar. */
export class HarnessRuntime {
  private readonly registry: Registry;
  constructor(registry: Registry) { this.registry = registry; }

  async execute(request: RunRequest): Promise<RunResult> {
    const started = Date.now();
    let toolCallCount = 0;
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;
    const trace = (outcome: "ok" | "error") => ({
      correlationId: typeof request?.correlationId === "string" ? request.correlationId : "invalid",
      skill: typeof request?.skillId === "string" ? request.skillId : "invalid",
      version: typeof request?.version === "string" ? request.version : "invalid",
      toolCallCount, durationMs: Date.now() - started, outcome,
    } as const);
    try {
      if (!request || !validPrincipal(request.principal) ||
          typeof request.correlationId !== "string" || request.correlationId.length < 1 || request.correlationId.length > 128 ||
          typeof request.skillId !== "string" || typeof request.version !== "string") {
        throw new HarnessError("INVALID_REQUEST", "Contexto interno de ejecución inválido.");
      }
      const budget = reduceBudget(request.requestedBudget);
      const skill = this.registry.getSkill(request.skillId, request.version);
      if (!skill) throw new HarnessError("UNKNOWN_SKILL", "Skill o versión no registrada.");
      if (!skill.requiredScopes.every((scope) => request.principal.scopes.includes(scope))) {
        throw new HarnessError("UNAUTHORIZED", "Permiso insuficiente para ejecutar la skill.");
      }
      if (!skill.validate(request.input)) throw new HarnessError("BAD_INPUT", "Entrada inválida para la skill.");
      const authorizedPrincipal: Readonly<Principal> = Object.freeze({
        ...request.principal, scopes: Object.freeze([...request.principal.scopes]),
      });
      const context: SkillContext = {
        principal: authorizedPrincipal, signal: controller.signal,
        callTool: async (id, input) => {
          if (controller.signal.aborted) throw new HarnessError("BUDGET_EXCEEDED", "Tiempo excedido.");
          if (!skill.allowedTools.includes(id)) throw new HarnessError("TOOL_FORBIDDEN", "Tool no permitida para la skill.");
          const tool = this.registry.getTool(id);
          if (!tool) throw new HarnessError("TOOL_FORBIDDEN", "Tool no registrada.");
          if (!authorizedPrincipal.scopes.includes(tool.requiredScope)) {
            throw new HarnessError("UNAUTHORIZED", "Permiso insuficiente para la herramienta.");
          }
          if (!tool.validate(input)) throw new HarnessError("BAD_INPUT", "Argumentos de herramienta inválidos.");
          if (toolCallCount >= budget.maxToolCalls) throw new HarnessError("BUDGET_EXCEEDED", "Límite de herramientas excedido.");
          toolCallCount++;
          try {
            return await tool.execute(input, { principal: authorizedPrincipal, signal: controller.signal });
          } catch (e) {
            if (isHarnessError(e)) throw e;
            throw new HarnessError("DEPENDENCY_FAILED", "Falló una dependencia autorizada.");
          }
        },
      };
      const deadline = new Promise<never>((_, reject) => {
        timer = setTimeout(() => {
          controller.abort();
          reject(new HarnessError("BUDGET_EXCEEDED", "Tiempo de ejecución excedido."));
        }, budget.timeoutMs);
      });
      const output = await Promise.race([skill.run(request.input, context), deadline]);
      return { ok: true, output, trace: trace("ok") };
    } catch (error) {
      const safe = isHarnessError(error)
        ? error : new HarnessError("INTERNAL_ERROR", "Fallo inesperado en la ejecución.");
      return { ok: false, error: { code: safe.code, message: safe.message }, trace: trace("error") };
    } finally {
      if (timer) clearTimeout(timer);
    }
  }
}
