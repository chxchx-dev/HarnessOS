/** Contratos neutrales a producto. Los campos de identidad llegan del servidor. */
export interface Principal {
  tenantId: string;
  subjectId: string | null;
  authType: "jwt" | "service";
  scopes: readonly string[];
}

export interface Budget {
  maxToolCalls: number;
  timeoutMs: number;
}

export interface RunRequest {
  skillId: string;
  version: string;
  input: unknown;
  principal: Principal;
  correlationId: string;
  /** El solicitante solo puede reducir el límite fijado por servidor. */
  requestedBudget?: Partial<Budget>;
}

export type ErrorCode =
  | "INVALID_REQUEST"
  | "UNAUTHORIZED"
  | "UNKNOWN_SKILL"
  | "BAD_INPUT"
  | "TOOL_FORBIDDEN"
  | "BUDGET_EXCEEDED"
  | "EVIDENCE_MISSING"
  | "MODEL_UNAVAILABLE"
  | "DEPENDENCY_FAILED"
  | "INTERNAL_ERROR";

export interface RunTrace {
  correlationId: string;
  skill: string;
  version: string;
  toolCallCount: number;
  durationMs: number;
  outcome: "ok" | "error";
  /** Intencionalmente sin payload, prompt, secretos ni argumentos. */
}

export type RunResult =
  | { ok: true; output: unknown; trace: RunTrace }
  | { ok: false; error: { code: ErrorCode; message: string }; trace: RunTrace };

export interface ToolContext {
  principal: Readonly<Principal>;
  signal: AbortSignal;
}
export interface Tool {
  id: string;
  requiredScope: string;
  validate(input: unknown): boolean;
  execute(input: unknown, context: ToolContext): Promise<unknown>;
}
export interface SkillContext {
  principal: Readonly<Principal>;
  signal: AbortSignal;
  callTool(id: string, input: unknown): Promise<unknown>;
}
export interface Skill {
  id: string;
  version: string;
  requiredScopes: readonly string[];
  allowedTools: readonly string[];
  validate(input: unknown): boolean;
  run(input: unknown, context: SkillContext): Promise<unknown>;
}
export interface SourceExcerpt {
  id: string;
  title: string;
  excerpt: string;
}
export interface ModelPort {
  readonly name: string;
  generate(question: string, sources: readonly SourceExcerpt[], signal: AbortSignal): Promise<string>;
}
