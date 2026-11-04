# Estado actual

[← Índice de documentación](INDEX.md) · [Roadmap](ROADMAP.md)

**Fecha:** 2026-11-04  
**Versión:** `0.1.0` · semilla experimental · TypeScript

## Disponible hoy

- Runtime interno con resolución `skill@version`, validación de identidad/scopes, allowlist de tools, presupuesto y timeout.
- Skills `knowledge.retrieve@1.0.0` y `knowledge.answer@1.0.0`.
- Fixtures sintéticas multi-tenant, respuesta determinista offline y Ollama local opt-in.
- CLI de demo legible, guía de inicio rápido y prueba de humo del ejemplo.
- README como página de entrada, índice de documentación y guías conectadas por navegación interna.
- Trazas de runtime limitadas a metadata, sin inputs, respuestas ni documentos.

## Fuera de alcance

No hay autenticación real, permisos de dominio, API HTTP/SSE, integración con productos, base de datos, memoria durable, RAG vectorial, jobs, colas, workflows reanudables, tools de escritura ni despliegue.

## Evidencia de esta revisión

| Comando | Resultado |
|---|---|
| `node --experimental-strip-types --test tests/*.test.mjs` | Aprobado: **12/12** |
| `node --experimental-strip-types scripts/demo.mjs` | Aprobado: dos casos sintéticos y fuentes `DEMO-` |
| Enlaces Markdown locales activos | Aprobados |
| `git diff --check` | Aprobado |
| `npm run check` | Incompleto: falta `tsc`; instalar TypeScript falló con `EACCES` en el registro npm |

Entorno: Node.js `v24.22.2`. No se probó Ollama, despliegue ni integración con un backend.

## Siguientes hitos

1. Validar la guía con una persona que no conozca el código y anotar fricciones.
2. Cuando se elija un consumidor, completar la [matriz de contrato](CONSUMER_CONTRACT_TEMPLATE.md) con su propio repositorio y fixtures sintéticas.
3. Caracterizar un caso de solo lectura en ese consumidor antes de implementar un adaptador.

Consulta el [roadmap](ROADMAP.md) para gates y criterios de avance.
