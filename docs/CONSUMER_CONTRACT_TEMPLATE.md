# Matriz de compatibilidad del consumidor

[← Índice de documentación](INDEX.md) · [Guía de adopción](ADOPTION.md)

Usa esta ficha en el repositorio del consumidor antes de implementar un adaptador. Complétala con evidencia de código y pruebas; no rellenes huecos con suposiciones.

## Contexto

```yaml
product: <nombre>
owner: <responsable>
use_case: <operación existente de solo lectura>
entrypoint: <controller o handler>
feature_flag: <flag de servidor>
slo: <latencia/disponibilidad acordadas>
rollback: <paso probado>
```

## Contrato que debe conservarse

| Campo | Valor actual | Evidencia (archivo/prueba) | Invariante |
|---|---|---|---|
| Método y ruta HTTP | Pendiente | Pendiente | Mantener |
| DTO, validaciones y límites | Pendiente | Pendiente | Mantener |
| Auth y fuente de identidad | Pendiente | Pendiente | No confiar en identidad del cliente |
| Tenant, owner, roles y permisos | Pendiente | Pendiente | No reducir permisos |
| Cuotas y facturación | Pendiente | Pendiente | No duplicar ni eludir |
| Status y forma de errores | Pendiente | Pendiente | Mantener |
| SSE, eventos y heartbeat | Pendiente | Pendiente | Mantener por familia |
| Jobs, reintentos y cancelación | Pendiente | Pendiente | No duplicar efectos |
| Persistencia, logs y retención | Pendiente | Pendiente | Minimizar contenido y conservar política |
| Consumidores conocidos | Pendiente | Pendiente | Sin ruptura no aprobada |

## Evidencia de compatibilidad

- [ ] Caso válido y respuesta esperada.
- [ ] Entrada inválida y status/error esperado.
- [ ] Falta de identidad y permisos insuficientes.
- [ ] Límite/cuota y comportamiento de retry.
- [ ] Cancelación, timeout y cierre de stream si aplica.
- [ ] Fixtures sintéticas; sin transcripts ni documentos personales.
- [ ] Flag apagada, canary y rollback ensayado.

## Aprobación del piloto

```yaml
contract_tests: <comando y rutas>
privacy_and_retention: <política revisada>
allowed_tools: []
effects: none
acceptance_evidence: <ubicación de resultados>
approved_by: <responsables>
```

**Gate:** no iniciar el adaptador hasta que contrato, owner, política de datos y rollback tengan evidencia revisable.
