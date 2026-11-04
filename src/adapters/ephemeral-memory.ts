import { HarnessError } from "../core/errors.ts";
import type { Principal } from "../core/contracts.ts";

/** Memoria efímera SOLO para tests. No durable ni apta para datos personales. */
export class EphemeralMemory {
  private readonly records = new Map<string, string>();
  private key(principal: Principal, name: string): string {
    if (!principal.subjectId) throw new HarnessError("UNAUTHORIZED", "La memoria personal requiere usuario autenticado.");
    if (!principal.scopes.includes("memory:read")) throw new HarnessError("UNAUTHORIZED", "Memoria no autorizada.");
    if (!/^[a-z0-9_-]{1,40}$/i.test(name)) throw new HarnessError("BAD_INPUT", "Nombre inválido.");
    return JSON.stringify([principal.tenantId, principal.subjectId, name]);
  }
  get(principal: Principal, name: string): string | undefined { return this.records.get(this.key(principal, name)); }
  set(principal: Principal, name: string, value: string): void {
    if (!principal.scopes.includes("memory:write")) throw new HarnessError("UNAUTHORIZED", "Escritura no autorizada.");
    if (value.length > 1024) throw new HarnessError("BAD_INPUT", "Memoria demasiado grande.");
    this.records.set(this.key(principal, name), value);
  }
  delete(principal: Principal, name: string): void { this.records.delete(this.key(principal, name)); }
}
