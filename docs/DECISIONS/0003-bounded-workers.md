# ADR 0003 · Workers Rust acotados y reutilizables

[← Índice de documentación](../INDEX.md) · [ADR 0002: port a Rust](0002-rust-port.md)

**Estado:** adoptada

## Contexto y medición

La primera implementación Rust creaba un hilo por ejecución para preservar el timeout incluso si una skill no cooperaba. Con la misma consulta sintética de la línea base de ADR 0002, una corrida optimizada de 10 000 ejecuciones tardó **1 347,6 ms**; el working set observado fue **3,61 MB**. Node 25 tardó **135,1 ms** y reportó **55,95 MB de RSS** después de GC. Es una comparación exploratoria de una corrida, y las métricas de memoria no son idénticas.

El spawn repetido añade cerca de 10× de latencia secuencial frente a Node y crea trabajo del sistema operativo en cada llamada. La medición justifica reutilizar workers; no justifica reservar RAM libre del host.

## Decisión

- Usar un número fijo y configurable de workers con stacks limitados.
- Entregar trabajo mediante canal de rendezvous de capacidad cero. El contador de admisión rechaza cuando todos los workers están ocupados; el canal no almacena trabajos en una cola.
- Mantener el límite de concurrencia ocupado hasta que la skill termine, incluso si el caller ya recibió timeout. Una skill que ignore cancelación no puede multiplicar workers huérfanos.
- Dejar timeout y cancelación por ejecución, presupuestos del servidor y trazas mínimas sin cambios.
- Mantener la medición reproducible en `harnessos-benchmark` y volver a comparar antes de afirmar mejoras.

## Resultado de la implementación

Con un pool fijo de 8 workers, límites JSON y un canal de rendezvous sin cola, cinco muestras de 10 000 ejecuciones Rust tuvieron tiempos de **370,73 · 367,93 · 380,18 · 413,47 · 389,68 ms** (mediana **380,18 ms**) y cero fallos. Una corrida equivalente en Node tuvo **148,9 · 198,2 · 164,0 · 139,9 · 194,6 ms** (mediana **164,0 ms**). Rust mantuvo aproximadamente **3,47 MB** de working set en Windows; Node llegó a **59,68 MB** de working set máximo y reportó **58,61 MB** de RSS tras GC.

El pool redujo el tiempo Rust cerca de 3,5× frente al spawn por llamada y mantuvo una huella residente menor en esta carga. Aun así, la ruta Rust fue aproximadamente 2,3× más lenta que Node. Las muestras son de un único host Windows y una operación sintética sin Ollama ni tráfico concurrente; el harness de Node reutilizó un principal y el de Rust lo clona por solicitud. La comparación es direccional, no una prueba de rendimiento equivalente, y no debe convertirse en SLO.

## Consecuencias

- La memoria de workers tiene un máximo previsible según su conteo y stack, pero el stack reservado no equivale exactamente a RSS.
- Cuando todos los workers estén ocupados, la runtime devuelve `BUDGET_EXCEEDED`; no espera en una cola ni aumenta el consumo por demanda.
- El operador ajusta cantidad y stack a su host y SLO. HarnessOS no detecta ni consume automáticamente toda la RAM.
