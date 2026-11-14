# Riesgos y límites conocidos

[← Índice de documentación](INDEX.md) · [Seguridad](SECURITY.md) · [Roadmap](ROADMAP.md)

Los tests de fixture prueban contratos internos. No prueban una integración de producto ni preparan HarnessOS para producción.

| ID | Riesgo | Impacto | Estado y salida necesaria |
|---|---|---|---|
| R-01 | El `Principal` puede ser falso si el runtime se expone directamente por HTTP | Crítico | No hay servidor incluido; autenticar y construirlo en backend verificado |
| R-02 | Una dependencia puede ignorar `CancellationToken` y seguir ocupando un worker tras timeout | Alto | Bloqueante para efectos; exigir deadlines y cancelación cooperativa; admisión acotada |
| R-03 | Memoria efímera en RAM sin lifecycle ni controles durables | Alto | Límites de bytes/entradas reducen crecimiento, pero sigue siendo solo demo/tests; no usar para datos personales |
| R-04 | Búsqueda de fixtures sin permisos por documento | Alto | Sustituir por servicio de dominio y probar ownership |
| R-05 | El modelo puede alucinar o seguir prompt injection | Alto | Evals, validación de salida y revisión humana según impacto |
| R-06 | No hay idempotencia, workflow store ni claims distribuidos | Crítico | No habilitar escrituras ni retries con efectos |
| R-07 | No hay pruebas HTTP/SSE de un consumidor real | Alto | Caracterizar compatibilidad antes de integrar |
| R-08 | Ollama no tiene SLO ni presupuesto productivo | Medio | Medir latencia, costo, tokens y disponibilidad |
| R-09 | No hay lifecycle real de consentimiento, retención o borrado | Crítico con datos sensibles | Definirlo y probarlo de extremo a extremo |
| R-10 | CI prueba solo el runtime y fixtures, no servicios reales ni contrato de un consumidor | Alto | Caracterizar en el repositorio consumidor antes del piloto |

## Regla de salida

No marques una capacidad como lista para producción por compilar o pasar tests de demo. Cada riesgo aplicable requiere owner, evidencia y criterio de aceptación antes de ampliar el uso.
