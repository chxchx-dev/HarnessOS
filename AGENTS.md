# Instrucciones para agentes de desarrollo

1. Leer `README.md`, `docs/STATE.md`, `docs/ARCHITECTURE.md` y `docs/ROADMAP.md` antes de editar.
2. Elegir **una** tarea pendiente; describir el comportamiento observable y las pruebas antes de programar.
3. No añadir dependencias, colas, bases de datos, servidores, MCP o agentes nuevos por moda. Exigir ADR y necesidad medida.
4. No cambiar rutas/DTO/SSE de productos existentes desde este repositorio. Cualquier adopción va detrás de adaptadores y contratos.
5. Nunca aceptar tenant, subject, scopes, permisos de herramientas o budget máximo desde texto libre del usuario. Resolver identidad y política en servidor.
6. Herramientas de lectura por defecto. Una herramienta de escritura requiere autorización por operación, idempotencia, auditoría, reconciliación y aprobación humana cuando corresponda.
7. No guardar secretos, prompts completos, contenido personal, razonamiento del modelo ni documentos en trazas/fixtures.
8. No presentar código de demostración o memoria en RAM como persistencia/seguridad de producción.
9. Antes de marcar tarea completa: ejecutar `npm run check` y añadir/reparar pruebas. No borrar pruebas fallidas para aprobar.
10. Actualizar `docs/STATE.md` y evidencia de pruebas; no afirmar despliegues ni migraciones sin prueba.
11. No realizar commit, push, despliegue ni alterar credenciales/producción sin autorización específica.
