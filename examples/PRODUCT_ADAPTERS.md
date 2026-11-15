# Ficha de adaptador de producto

[README](../README.md) · [Guía de adopción](../docs/ADOPTION.md)

Completa una ficha por caso de uso y guárdala en la documentación del backend que lo implementa. Hazlo antes de escribir código para confirmar que el adaptador conserva el contrato y ownership existentes.

```yaml
product: <nombre>
owner: <responsable>
existing_entrypoint: <controller/handler exacto>
contract_tests: <ruta tests>
identity_source: <guard verificado>
tenant_resolution: <servicio autoritativo>
operation: <caso solo lectura>
model: <si aplica>
data_sources: []
allowed_tools: []
effects: none
privacy_and_retention: <política vigente>
feature_flag: <flag del servidor>
canary_and_rollback: <pasos ensayados>
acceptance_evidence: <comandos/capturas>
```

## Revisión rápida

- [ ] La identidad y el tenant vienen del backend autenticado.
- [ ] El caso es de solo lectura y cada tool tiene el permiso mínimo.
- [ ] Las pruebas caracterizan éxito, error, cuota y aislamiento.
- [ ] No se copian prompts, documentos ni datos personales a fixtures.
- [ ] La bandera se controla en servidor y el rollback está ensayado.

Ejemplos conceptuales: educación `knowledge.retrieve`, CRM `catalog.read`, agenda `help.search`. No son conectores implementados ni dan acceso a datos reales.
