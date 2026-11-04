import type { SourceExcerpt, Tool } from "../core/contracts.ts";

interface Doc extends SourceExcerpt { tenantId: string; }
const docs: readonly Doc[] = [
  { tenantId: "education-demo", id: "DEMO-EDU-01", title: "Ejemplo ficticio: material educativo", excerpt: "En este ejemplo simulado, las lecciones educativas incluyen fuentes de consulta y una estructura clara." },
  { tenantId: "education-demo", id: "DEMO-EDU-02", title: "Ejemplo ficticio: continuidad de una actividad", excerpt: "En este ejemplo simulado, una actividad conserva su identificador para que el usuario pueda retomarla." },
  { tenantId: "appointments-demo", id: "DEMO-APP-01", title: "Ejemplo ficticio: agenda", excerpt: "La agenda de prueba permite consultar horarios simulados sin datos de pacientes." },
  { tenantId: "catalog-demo", id: "DEMO-CAT-01", title: "Ejemplo ficticio: catálogo comercial", excerpt: "Una consulta de prueba muestra artículos de un catálogo ficticio dentro de su tenant." },
];

export const demoKnowledgeTool: Tool = {
  id: "knowledge.search",
  requiredScope: "knowledge:read",
  validate(input: unknown): boolean {
    return typeof input === "object" && input !== null &&
      "query" in input && typeof input.query === "string" &&
      input.query.trim().length >= 2 && input.query.length <= 300;
  },
  async execute(input, { principal, signal }): Promise<SourceExcerpt[]> {
    if (signal.aborted) return [];
    const query = (input as { query: string }).query.toLocaleLowerCase("es");
    const tokens = query.match(/[\p{L}\p{N}]{3,}/gu) ?? [];
    const tenantDocs = docs.filter((d) => d.tenantId === principal.tenantId);
    // Relevancia simplificada para una demo; NO es búsqueda productiva ni pgvector.
    const relevant = tenantDocs.filter((d) => tokens.some((token) =>
      `${d.title} ${d.excerpt}`.toLocaleLowerCase("es").includes(token)));
    return relevant.slice(0, 3).map(({ id, title, excerpt }) => ({ id, title, excerpt }));
  },
};
