# Roadmap

[README](../README.md) · [Estado actual](STATE.md)

El roadmap separa la semilla que ya corre de las capacidades que requieren adopción, datos o decisiones operativas. Una casilla solo se marca cuando hay evidencia en el entorno indicado.

## R0 · Port completo del MVP a Rust

- [x] Portar runtime, contratos, skills, tools y modelos demo/Ollama.
- [x] Mantener el mismo comportamiento observable en pruebas de scopes, allowlist, aislamiento, abstención, timeout y salida de demo.
- [x] Añadir límites estimados para entradas/resultados/salidas JSON, memoria efímera acotada y concurrencia configurable sin reservar RAM libre.
- [x] Medir latencia y memoria en una carga sintética repetida y documentar el costo observado.
- [ ] Repetir la medición con tráfico representativo en el PC/VPS objetivo.

**Salida:** el MVP local se ejecuta completamente en Rust; no implica que haya integración con un backend o preparación para producción.

## P0 · Semilla utilizable

- [x] Núcleo neutral a producto con contratos internos.
- [x] Demo local de lectura con fixtures sintéticas multi-tenant.
- [x] Pruebas de scopes, allowlist, evidencia, límites y aislamiento.
- [x] Inicio rápido, guía de adopción y documentación navegable.
- [ ] Validar el inicio rápido con una persona nueva en el repositorio.

**Salida:** otra persona ejecuta la demo, entiende los límites y puede señalar qué necesita adaptar.

## P1 · Contrato del primer consumidor

- [ ] Elegir un backend y una capacidad existente de solo lectura.
- [ ] Registrar rutas, DTO, auth, roles, cuotas, errores, SSE, side effects y consumidores en la matriz.
- [ ] Añadir pruebas de caracterización del contrato y de los fallos actuales.
- [ ] Acordar owner, SLO, logs, privacidad y retención.

**Salida:** línea base reproducible y dueño del piloto identificados.

## P2 · Adaptador en piloto

- [ ] Conectar una skill a servicios de dominio existentes detrás de una flag de servidor.
- [ ] Resolver `Principal` desde auth verificada y comprobar ownership por objeto.
- [ ] Comparar calidad, fuentes, latencia, costo y errores contra la línea base.
- [ ] Ensayar rollback sin cambiar el contrato del cliente.

**Gate:** no hay bypass de permisos, regresión contractual ni degradación de SLO sin aceptación explícita.

## P3 · Calidad y operación

- [ ] Evals reproducibles de utilidad, fundamento, prompt injection y abstención.
- [ ] Métricas por etapa y trazas saneadas, con control de acceso y retención.
- [ ] Validadores de salida y límites de tokens/costo cuando un modelo intervenga.

**Gate:** regresiones detectables y evaluación humana definida para los casos de impacto alto.

## P4 · Memoria y rutinas durables

- [ ] Usar fuentes de verdad y ownership existentes; definir consentimiento, retención, exportación y borrado.
- [ ] Diseñar estado durable, checkpoints, reanudación, cancelación e idempotencia antes de jobs o escrituras.
- [ ] Añadir reconciliación, locks/leases y aprobación humana cuando haya efectos sensibles.

**Gate:** reinicios y reintentos no duplican efectos; aislamiento y borrado se prueban de extremo a extremo.

## P5 · Expansión

- [ ] Medir carga y SLO con tráfico representativo no sensible.
- [ ] Activar gradualmente por política del servidor y conservar rollback.
- [ ] Reutilizar fallos anonimizados para mejorar fixtures y evals.

**Gate:** responsables de seguridad, producto y operación aceptan evidencia, costo y límites.

## Fuera de prioridad

No construir una plataforma distribuida, marketplace, agentes autónomos, router avanzado, MCP universal o infraestructura nueva sin una necesidad medida y una decisión registrada.
