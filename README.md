# HarnessOS

Runtime Rust experimental para ejecutar capacidades de IA con permisos, presupuestos y límites explícitos. Se integra como biblioteca detrás de un backend existente; no es un servidor HTTP ni reemplaza su autenticación, persistencia o reglas de negocio.

> **Estado:** experimental. La demo usa datos inventados y funciona sin servicios externos. No está lista para producción ni para datos personales.

## Ejecutar

Necesitas Rust estable y Cargo. Desde la raíz del repositorio:

```bash
cargo run --release --locked --bin harnessos-demo
```

La demo ejecuta dos consultas sintéticas con respuestas y fuentes `DEMO-`. Para comprobar el código:

```bash
cargo fmt --check
cargo check --all-targets --locked
cargo test --locked
```

## Qué incluye

- Runtime embebible con skills versionadas, scopes, allowlist de tools, presupuestos y timeout.
- Demo offline determinista y opción de modelo local Ollama.
- Memoria efímera y límites configurables de concurrencia y tamaño; no reserva toda la RAM ni ofrece persistencia.

El servidor debe autenticar al usuario y construir el principal verificado. No expongas el runtime directamente al cliente. Los límites de memoria JSON son estimaciones, no un límite exacto del RSS ni un sandbox para tools.

## Documentación

- [Seguridad y límites](docs/SECURITY.md)
- [Arquitectura](docs/ARCHITECTURE.md)
- [Integración](docs/ADOPTION.md)
- [Estado actual](docs/STATE.md)
- [Roadmap](docs/ROADMAP.md)

Consulta [LICENSE](LICENSE) para los términos de uso.
