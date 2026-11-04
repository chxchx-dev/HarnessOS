# ADR 0001 · Núcleo embebible antes que plataforma independiente

[← Índice de documentación](../INDEX.md) · [Arquitectura](../ARCHITECTURE.md)

**Estado:** adoptada para la semilla  
**Alcance:** HarnessOS; no modifica decisiones de los productos consumidores

## Decisión

Mantener HarnessOS como runtime y contratos simples que se ejecutan dentro del backend de cada producto. Cada producto conserva su autenticación, rutas, datos, colas, facturación y políticas.

## Motivo

Un servicio central prematuro añade despliegues, migraciones y acceso transversal antes de demostrar una necesidad. Un núcleo embebible permite validar una capacidad con menos superficie operativa y mantiene el ownership de datos en el dominio existente.

## Consecuencias

- Los adaptadores viven en el backend dueño del caso de uso.
- El runtime no expone HTTP ni crea otro orquestador paralelo.
- Contratos y pruebas pertenecen al consumidor y se caracterizan antes de integrar.
- Los productos pueden adoptar el mismo núcleo en momentos distintos.

## Revisar esta decisión cuando

Dos o más consumidores demuestren una necesidad compartida de coordinación u operación y existan SLO, modelo de permisos, costo y riesgos medidos. Registrar una ADR nueva antes de separar el runtime.
