import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const projectRoot = fileURLToPath(new URL("..", import.meta.url));

test("demo quickstart runs offline with synthetic evidence and readable output", () => {
  const result = spawnSync(
    process.execPath,
    ["--experimental-strip-types", "scripts/demo.mjs"],
    {
      cwd: projectRoot,
      encoding: "utf8",
      timeout: 10_000,
      env: { ...process.env, HARNESS_MODEL: "demo" },
    },
  );

  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /DATOS FICTICIOS: sin conexión con sistemas ni datos reales/);
  assert.doesNotMatch(result.stdout, /Alania|DentyKit|ACore/i);
  assert.match(result.stdout, /Caso: education-demo/);
  assert.match(result.stdout, /Respuesta:.*lecciones educativas/);
  assert.match(result.stdout, /DEMO-EDU-01: Ejemplo ficticio/);
  assert.match(result.stdout, /Caso: appointments-demo/);
  assert.match(result.stdout, /DEMO-APP-01: Ejemplo ficticio/);
});
