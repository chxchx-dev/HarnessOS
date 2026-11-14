# ADR 0002 · Port incremental del MVP a Rust

[← Índice de documentación](../INDEX.md) · [Arquitectura](../ARCHITECTURE.md)

**Estado:** adoptada para el port solicitado; beneficios de rendimiento pendientes de comparar
**Alcance:** implementación local de HarnessOS. No cambia contratos de productos consumidores.

## Contexto y medición de referencia

El mantenedor solicita migrar HarnessOS completamente a Rust y mejorar el uso de memoria. La semilla actual no tiene almacenamiento durable ni una carga que justifique consumir la RAM libre del host; su único almacén es `EphemeralMemory`, limitado a 1 KiB por valor y marcado para tests.

Medición local de una corrida, con Node.js 25.0.0, GC explícito y 10 000 ejecuciones de `knowledge.retrieve` sobre fixtures sintéticas:

| Métrica | Antes | Después | Cambio |
|---|---:|---:|---:|
| Heap usado | 7 792 192 B | 8 098 568 B | +306 376 B |
| RSS del proceso | 51 634 176 B | 55 951 360 B | +4 317 184 B |
| Tiempo | — | 135,1 ms | — |

Es una línea base exploratoria de una sola corrida, no una prueba de fuga ni una afirmación de que Rust será más rápido o usará menos memoria. Se deben comparar las mismas operaciones tras completar el port antes de declarar una mejora.

## Decisión

- Portar por etapas el MVP a una biblioteca y demo en Rust, conservando contratos observables durante la transición.
- Mantener identidad, permisos, límites de herramientas, errores seguros y trazas mínimas bajo control del runtime.
- Usar `serde_json` para conservar entradas y salidas JSON flexibles del contrato TypeScript y `ureq` para el cliente HTTP bloqueante de Ollama local. Ambas dependencias se limitan a capacidades ya existentes; no se incorpora un executor async.
- Mantener la memoria efímera explícita, volátil y acotada por instancia con límites de bytes, entradas y tamaño por registro. Aplicar estimaciones máximas configurables a entrada, resultado de tool y salida. No reservar automáticamente toda la RAM del host ni presentar memoria en RAM como persistencia.
- Durante el port se conservó TypeScript para comparar comportamiento y recursos; se retiró tras aprobar paridad y benchmark repetido. El port no cambia rutas, DTO, SSE ni despliegues de consumidores.

## Consecuencias y gates

- La memoria disponible de cada PC/VPS la administra el sistema operativo y el allocator. HarnessOS solo asigna memoria al necesitarla y aplica el presupuesto configurado por el operador.
- Las ejecuciones respetan cancelación y timeout con workers de stack acotado; una dependencia que ignore cancelación puede seguir activa después del timeout, igual que en la semilla.
- El benchmark repetido de la carga sintética quedó documentado en ADR 0003 y `docs/STATE.md`; no es una predicción de producción ni una afirmación de SLO.
- No declarar mejora de memoria o latencia sin medir ambas implementaciones en condiciones comparables.

## Revisar esta decisión cuando

El benchmark no muestre una ventaja que justifique el costo de migración, o cuando un consumidor requiera persistencia, ejecución concurrente o políticas de memoria distintas. Registrar evidencia y una ADR nueva antes de añadir infraestructura.
