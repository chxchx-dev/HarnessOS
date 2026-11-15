# Evals y biblioteca de fallos

[README](../README.md) · [Seguridad](../docs/SECURITY.md)

Las pruebas unitarias verifican reglas del runtime. Las evals comprueban si una capacidad responde bien al dominio, usa evidencia y se comporta correctamente con entradas difíciles.

## Estado actual

- `tests/`: suite automática del runtime y smoke test de la demo.
- `cases.json`: catálogo inicial; los casos con estado `pending` son ideas y **no se ejecutan ni cuentan como aprobados**.
- No hay un juez LLM ni evaluación automática de calidad semántica.

## Añadir un caso

Usa únicamente datos sintéticos o anonimizados. Registra:

| Campo | Qué debe indicar |
|---|---|
| Input | Solicitud ficticia y contexto mínimo |
| Principal | Tenant/scopes simulados para la prueba |
| Tools | Dependencias disponibles y efectos permitidos |
| Expected | Respuesta, abstención o error aceptable |
| Evidence | Fuentes sintéticas que sustentan la salida |
| Version | Skill y versión evaluadas |
| Risk | Prompt injection, fuga entre tenants, salida inválida u otro riesgo |

No incluyas transcripciones reales, prompts completos de producción, documentos personales ni secretos en fixtures.

## Criterios recomendados

- Fundamentación y referencias correctas.
- Abstención cuando falta evidencia.
- Aislamiento por tenant y permisos.
- Resistencia a instrucciones inyectadas en documentos.
- Regresión de salida y cumplimiento del esquema.
- Revisión humana para acciones o dominios de alto impacto.

Una respuesta plausible no demuestra que sea verdadera. Publica el comando, versión de skill, datos sintéticos y resultado de cada suite para que la evaluación se pueda repetir.
