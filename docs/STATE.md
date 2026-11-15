# Estado actual

[README](../README.md) · [Roadmap](ROADMAP.md)

**Fecha:** 2026-11-14 · **Versión:** `0.2.0` · semilla experimental · Rust

## Disponible hoy

- Biblioteca Rust con resolución `skill@version`, validación de identidad/scopes, allowlist de tools, presupuesto, timeout y hasta 8 workers reutilizables por defecto.
- Skills `knowledge.retrieve@1.0.0` y `knowledge.answer@1.0.0`.
- Fixtures sintéticas multi-tenant, respuesta determinista offline y Ollama local opt-in con proxy y redirecciones desactivados.
- `RuntimeConfig` limita estimaciones de entrada/salida y ajusta workers/stack. `EphemeralMemory` volátil y configurable: por defecto 16 MiB de bytes contabilizados, 4096 entradas y 1 KiB por registro.
- CLI Rust de demo y benchmark repetible.
- CI de Rust, guía de inicio rápido y documentación conectada.
- Trazas limitadas a metadata, sin inputs, respuestas ni documentos.

## Fuera de alcance

No hay autenticación real, permisos de dominio, API HTTP/SSE, integración con productos, base de datos, memoria durable, RAG vectorial, jobs, colas, workflows reanudables, tools de escritura ni despliegue. HarnessOS asigna memoria al necesitarla; no reserva toda la RAM libre del host.

## Evidencia de esta revisión

| Comando | Resultado |
|---|---|
| `cargo test --locked` | Aprobado: **23/23** (22 unitarias y 1 de demo) |
| `cargo fmt --check` | Aprobado |
| `cargo check --all-targets --locked` | Aprobado |
| `cargo run --release --locked --bin harnessos-demo` | Aprobado: dos casos sintéticos y fuentes `DEMO-` |
| Enlaces Markdown locales activos | Aprobados |
| `git diff --check` | Aprobado |

Entorno local: Rust `1.99.1`, Windows `x86_64`. Ollama, despliegue e integración con un backend no se probaron.

### Medición sintética de recursos

Una corrida de referencia con un calentamiento y cinco muestras de 10 000 ejecuciones de `knowledge.retrieve`, en modo release:

| Implementación | Muestras de tiempo (ms) | Mediana | Working set/RSS observado |
|---|---|---:|---:|
| Rust 1.99, pool de 8 workers y límites JSON | 370,73 · 367,93 · 380,18 · 413,47 · 389,68 | 380,18 ms | pico 3,47 MB |

El muestreo es exploratorio y depende del host; no predice latencia o consumo con Ollama, modelos o servicios del producto. Repetir con tráfico representativo en el PC/VPS objetivo antes de ajustar workers o establecer un SLO.

## Siguientes hitos

1. Validar el inicio rápido con una persona que no conozca el código y anotar fricciones.
2. Cuando se elija un consumidor, registrar compatibilidad y fixtures sintéticas en su propio repositorio.
3. Repetir la carga con tráfico representativo en el host objetivo antes de configurar sus límites.

Consulta el [roadmap](ROADMAP.md) para gates y criterios de avance.
