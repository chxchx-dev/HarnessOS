# Documentación de HarnessOS

[← Volver al README](../README.md)

Elige una ruta según lo que quieras hacer. La demo funciona sin servicios externos; la adopción en un producto requiere caracterizar primero su backend.

## Empezar

| Objetivo | Documento |
|---|---|
| Ejecutar la demo y entender el flujo | [Inicio rápido](QUICKSTART.md) |
| Saber qué está implementado hoy | [Estado actual](STATE.md) |
| Ver las decisiones de arquitectura | [Arquitectura](ARCHITECTURE.md) |

## Reutilizar en un backend

| Objetivo | Documento |
|---|---|
| Planear una integración gradual | [Guía de adopción](ADOPTION.md) |
| Registrar contratos antes del cambio | [Matriz de compatibilidad](CONSUMER_CONTRACT_TEMPLATE.md) |
| Preparar la ficha de un adaptador | [Plantilla de adaptadores](../examples/PRODUCT_ADAPTERS.md) |

## Revisar límites y calidad

| Objetivo | Documento |
|---|---|
| Revisar fronteras de seguridad | [Seguridad](SECURITY.md) |
| Ver riesgos conocidos | [Riesgos](RISKS.md) |
| Entender las pruebas y evals | [Evals](../evals/README.md) |
| Consultar fases pendientes | [Roadmap](ROADMAP.md) |

## Decisiones

- [ADR 0001: núcleo embebible antes que plataforma](DECISIONS/0001-monolith-first.md)
- [ADR 0002: port incremental a Rust](DECISIONS/0002-rust-port.md)
- [ADR 0003: workers acotados y reutilizables](DECISIONS/0003-bounded-workers.md)
