# Arquitectura

[← Índice de documentación](INDEX.md) · [Inicio rápido](QUICKSTART.md)

HarnessOS es un runtime embebible. Coordina procedimientos de IA dentro del backend del producto y delega el acceso a datos a servicios de dominio existentes.

## Recorrido de una solicitud

```mermaid
flowchart TD
  A[Backend autentica y resuelve políticas] --> B[Adaptador arma RunRequest]
  B --> C[HarnessRuntime]
  C --> D[Registry: skill@version]
  C --> E[Valida principal, scopes, entrada y presupuesto]
  E --> D
  D --> F[Skill]
  F --> G[Tool en allowlist]
  G --> H[Servicio de dominio existente]
  F -. si hace falta .-> I[ModelPort]
  D --> J[RunResult + traza de metadata]
  J --> K[Adaptador conserva contrato del producto]
```

El runtime no autentica, no contiene rutas HTTP y no reemplaza el orquestador del producto. El adaptador convierte la solicitud existente al contrato interno y traduce el resultado al DTO actual.

## Piezas

| Pieza | Responsabilidad | Implementación de la semilla |
|---|---|---|
| `HarnessRuntime` | Valida contexto y ejecuta una skill bajo límites | `src/core/runtime.ts` |
| `Registry` | Registra skills versionadas y tools con IDs únicos | `src/core/registry.ts` |
| `Principal` | Expresa identidad verificada, tenant y scopes autorizados | `src/core/contracts.ts`; lo construye el backend |
| `Skill` | Valida una entrada y coordina un procedimiento | `src/skills/` |
| `Tool` | Expone una operación autorizada del servicio dueño del dato | `src/adapters/` en la demo |
| `ModelPort` | Permite usar un proveedor de generación cuando la skill lo necesita | Modelo determinista u Ollama local opt-in |
| `RunResult` | Devuelve salida o error tipado con metadata mínima de ejecución | `src/core/contracts.ts` |

## Controles en cada llamada

1. La versión exacta de skill debe existir en el registro.
2. El principal debe venir del backend y contener los scopes requeridos.
3. La skill valida su propia entrada antes de correr.
4. Cada tool debe estar en la allowlist de la skill, tener scope permitido y validar sus argumentos.
5. La cuenta de tools y el timeout no pueden superar los máximos definidos por servidor.
6. La traza registra correlación, skill, versión, cantidad de tools, duración y resultado; omite inputs, prompts, respuestas y documentos.

El timeout aborta mediante `AbortSignal`, pero una dependencia que ignore esa señal puede seguir ejecutándose. Por eso la semilla no habilita tools con efectos.

## Capacidades del MVP

| Área | Incluido | No incluido |
|---|---|---|
| Skills | `knowledge.retrieve` y `knowledge.answer`, versionadas | Catálogo remoto o marketplace |
| Búsqueda | Fixture sintética aislada por tenant | Índice vectorial o permisos por documento real |
| Modelo | Respuesta determinista y Ollama local opcional | Router productivo de modelos |
| Memoria | Ejemplo efímero en RAM para tests | Persistencia, retención o lifecycle |
| Rutinas | Ninguna | Cola durable, checkpoints, reanudación o idempotencia |
| Transporte | API interna de TypeScript | Servidor HTTP/SSE público |

## Límites de dependencia

- `src/core` no depende de NestJS, ORM, base de datos ni proveedor LLM.
- Las skills acceden a datos mediante tools; no importan tablas privadas de otro módulo.
- Permisos de negocio y ownership por objeto permanecen en el backend y el servicio de dominio.
- El contenido recuperado es dato no confiable. No puede cambiar scopes ni reglas del runtime.
- HarnessOS no debe convertirse en un segundo orquestador paralelo al producto.

Para la secuencia de integración, ve a [Adopción](ADOPTION.md). Para las estructuras concretas, revisa el [Inicio rápido](QUICKSTART.md).
