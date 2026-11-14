# Seguridad

[← Índice de documentación](INDEX.md) · [Riesgos](RISKS.md)

Esta página describe las fronteras de seguridad de la semilla. Los controles del runtime reducen riesgos comunes, pero no reemplazan autenticación ni políticas del producto.

## Identidad y permisos

- El backend autentica al usuario y resuelve tenant, subject, roles y permisos.
- Solo el servidor construye `Principal`; no lo aceptes desde texto libre o payload del cliente.
- Cada skill declara scopes y una allowlist de tools.
- La tool y el servicio de dominio vuelven a comprobar autorización por objeto.
- Una API key de servicio no identifica por sí sola a una persona.

## Entradas y contenido recuperado

- Valida entrada de skill y argumentos de cada tool con esquemas estrictos.
- Trata prompts, documentos, resultados de tools y respuestas de modelos como datos no confiables.
- El contenido recuperado no puede conceder scopes, seleccionar tools ni cambiar políticas.
- La salida de un LLM puede ser incorrecta; una cita no demuestra fundamentación.

## Recursos y red

- El servidor fija el máximo de llamadas a tools y timeout; una solicitud solo puede reducirlos.
- La cancelación requiere que tools y proveedores cooperen con `CancellationToken`; una tarea que no coopera conserva su worker hasta terminar.
- El runtime limita concurrencia con 8 workers por defecto y admite configuración explícita de cantidad y stack.
- `RuntimeConfig` rechaza entradas, resultados de tools y salidas que superen sus estimaciones de memoria. El control ocurre después de que el valor existe en el proceso y no limita asignaciones internas de tools/modelos.
- Ollama de ejemplo solo acepta loopback, desactiva proxies y no sigue redirecciones. No permitas URL elegida por usuarios ni amplíes la red sin controles SSRF y autenticación.
- No habilites shell, SQL libre, filesystem ni HTTP arbitrario como tools.

## Datos y observabilidad

- La traza del runtime contiene metadata de ejecución, nunca prompts, respuestas ni documentos.
- Los IDs de correlación también pueden ser sensibles; controla acceso y cardinalidad.
- Define minimización, consentimiento, retención, exportación, borrado y revocación en el backend antes de usar datos reales.
- `EphemeralMemory` es solo una muestra en RAM con límites configurables; contabiliza bytes de claves y valores, limita entradas y no almacena de forma durable ni segura.
- HarnessOS no reserva toda la RAM libre. El presupuesto de memoria debe configurarlo el operador de acuerdo con otras cargas y los límites del host.

## Antes de una escritura

La semilla no ofrece tools de escritura. Para añadirlas se requieren autorización por operación y objeto, idempotencia durable, auditoría, reconciliación, cancelación y aprobación humana cuando corresponda. Diseña y prueba esos controles antes de activar efectos reales.

## Lista de revisión

- [ ] Principal derivado de auth verificada en servidor.
- [ ] Tenant y ownership resueltos por fuentes autoritativas.
- [ ] Tools mínimas, validadas y autorizadas por operación.
- [ ] Presupuestos definidos por servidor y deadlines en dependencias.
- [ ] Trazas sin payload ni contenido personal.
- [ ] Retención, borrado y acceso a logs documentados.
- [ ] Evals de inyección, abstención y calidad para el dominio.
- [ ] Rollback y responsables definidos.

Esta guía no es una auditoría de seguridad ni una aprobación para producción.
