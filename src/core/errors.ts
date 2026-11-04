import type { ErrorCode } from "./contracts.ts";

export class HarnessError extends Error {
  readonly code: ErrorCode;
  constructor(code: ErrorCode, message: string) {
    super(message);
    this.code = code;
    this.name = "HarnessError";
  }
}

export function isHarnessError(error: unknown): error is HarnessError {
  return error instanceof HarnessError;
}
