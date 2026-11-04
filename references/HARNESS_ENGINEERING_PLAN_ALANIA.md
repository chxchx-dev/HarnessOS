# Plan de transición de Alania a Harness Engineering

Fecha: 2026-11-03. Estado: **propuesta; implementación no iniciada**.

Este documento define trabajo futuro. No activa funcionalidades, no modifica
endpoints, no aplica migraciones y no acredita el estado del servidor actual.
El código y las pruebas de caracterización tendrán prioridad ante discrepancias
con documentación histórica. Cada fase debe dejar evidencia antes de avanzar.

## 1. Resultado buscado y alcance

Alania debe ejecutar tareas educativas con contexto autorizado, procedimientos
versionados, herramientas limitadas, validación, trazabilidad y recuperación
ante fallos. La transición debe conservar los contratos usados por los clientes.

Se trabajará sobre dos ámbitos relacionados:

- **Ejecución del producto:** las siete capas del texto adjunto: modelo, memoria,
  conexiones, skills, rutinas, reglas y biblioteca de fallos.
- **Ingeniería del repositorio:** contexto navegable, límites arquitectónicos
  verificables, entorno reproducible, pruebas de contratos, evaluaciones y un
  ciclo de trabajo que permita a humanos y agentes cambiar el sistema con evidencia.

Las siete capas son una organización propuesta para Alania, no un estándar que
obligue a crear siete servicios. Una skill será un procedimiento ejecutable y
versionado; su descripción o prompt no sustituye su implementación ni sus pruebas.

La transición inicial conserva NestJS, PostgreSQL/Prisma, MongoDB, Ollama, Wiki,
LibreTranslate, STT/TTS y `packages/learning-core`. Se amplían las fronteras
existentes por incrementos. Un cambio de modelo, Redis, MCP, una plataforma de
agentes, microservicios o un nuevo monorepo requerirán una necesidad medida y
una decisión aparte. No son prerrequisitos del plan.

## 2. Punto de partida comprobado en el repositorio

La revisión fue de código y documentación local; no incluyó pruebas live,
ejecución de suites ni auditoría exhaustiva de todas las rutas.

| Capacidad | Base existente | Implicación para la transición |
|---|---|---|
| Orquestación | [RuntimeExecutionService](../src/modules/ai-orchestrator/runtime-execution.service.ts), [UseCaseRegistryService](../src/modules/ai-orchestrator/use-case-registry.service.ts), [AiOrchestratorService](../src/modules/ai-orchestrator/ai-orchestrator.service.ts) | Evolucionar estas piezas; evitar un segundo orquestador con reglas duplicadas. |
| Modelos | [LlmPort](../src/application/ports/llm.port.ts), Ollama y [ModelPolicyService](../src/application/services/model-policy.service.ts) | Reutilizar puertos y políticas. `ModelPolicyService` aplica política de chat; no asumir que ya es un router general. |
| Uso de modelos | El runtime selecciona `qwen3:4b-instruct`; el estado documentado indica modelo forzado en modo todo en uno | El router 1B/4B del adjunto es una hipótesis futura, no el punto de partida. Confirmar configuración efectiva en fase 0. |
| Conocimiento | [KnowledgeChatService](../src/modules/knowledge/retrieval/knowledge-chat.service.ts), RAG y `KnowledgeContextService` | Mantener políticas diferentes para chat general, chat estricto y educación. |
| Lecciones educativas | [wiki-first-lesson.ts](../src/modules/education-generation/wiki-first-lesson.ts) y ADR-074/077 | Preservar composición determinista con Wiki, ámbito `lessons`, evidencia suficiente y error explícito; sin fallback generativo. |
| Traducción | [TranslateService](../src/modules/translate/translate.service.ts) y ADR-070 | Mantener LibreTranslate y su caché separado; sin fallback a LLM. |
| Trabajos largos | [AiGenerationQueueService](../src/modules/ai-generation-queue/ai-generation-queue.service.ts) y `AiGenerationJob` | Reutilizar cola, prioridades y recuperación. Hoy su arranque asume una sola instancia. |
| Idempotencia | `RuntimeExecutionService` usa un `Map` con TTL de cinco minutos | No ofrece garantía durable ni entre réplicas para nuevas rutinas con efectos. |
| Memoria y progreso | Conversaciones, perfil, sesiones guiadas Mongo, evidencia/progreso PostgreSQL y `learning-core` | Definir un contrato de acceso sobre datos existentes; evitar otra copia autoritativa del progreso. |
| Activación | [FeatureFlagsService](../src/modules/features/feature-flags.service.ts) | Reutilizar flag global y override por cuenta; diseñar aparte alcance por cliente sin JWT. |
| Privacidad | [PrivacyService](../src/modules/legal/privacy.service.ts), exportación/borrado y consentimiento existentes | Incluir cualquier estado nuevo, cachés, jobs y trazas en el ciclo de vida de datos. |
| Verificación | [TESTING.md](TESTING.md), Vitest y typecheck | Separar pruebas rápidas, integración y evaluaciones con servicios reales. |

Riesgos que condicionan el plan:

1. La cola vuelve a poner todos los trabajos `running` en `queued` al iniciar.
   Dos gateways compartiendo esa base podrían duplicar ejecuciones. El canary
   inicial debe ocurrir dentro de una instancia; escalar requiere otro diseño.
2. [La consulta de jobs](../src/modules/ai-generation-queue/ai-generation-queue.controller.ts)
   usa API key y el modelo `AiGenerationJob` no tiene `userId` propio. Antes de
   guardar memoria o progreso personal en jobs, definir ownership y autorización.
   No cambiar silenciosamente el contrato legado ni copiarlo a nuevas funciones personales.
3. Hay memoria y evidencia ya implementadas en Práctica guiada. Su cierre no
   acredita automáticamente dominio; `LESSONS` y `DISCOVER` son independientes.
4. El estado documenta pendientes de restauración de datos y contenido guiado
   ausente en la base desplegada. Son verificaciones pendientes, no hechos live
   confirmados en esta sesión ni problemas que deba ocultar el harness.
5. Hay textos históricos sobre modelos, streaming y fallback superados por
   decisiones posteriores. El inventario inicial debe resolverlos con código,
   configuración no secreta y pruebas; no copiar valores antiguos como contrato.

## 3. Condiciones de compatibilidad

Para cada ruta se registrarán método, URL, DTO, validaciones, autenticación,
roles, cuotas, status, errores, cabeceras, formato y efectos de persistencia.
También los campos opcionales, `null`, defaults, límites y aliases vigentes.

- Los clientes seguirán usando sus endpoints HTTP/SSE actuales. La adopción del
  harness será interna; no se exigirá migrarlos a `/api/v1/executions`.
- El runtime v1 exige identidad de usuario aunque su controller acepte API key
  o JWT. No se puede usar ese requisito para alterar rutas que hoy funcionan
  con API key sin usuario. El contexto interno distinguirá ambas identidades.
- La estructura JSON, códigos de error y semántica de los campos seguirán siendo
  compatibles. Los DTOs tienen `forbidNonWhitelisted`; no agregar parámetros
  obligatorios de harness ni pedir al cliente versiones internas.
- SSE conserva **el contrato de cada familia**: nombres, payloads, correlación,
  orden permitido, heartbeat, evento terminal y cierre. No imponer a todas las
  rutas un único conjunto de eventos. Los tokens intercalados de Explora deben
  conservar `perspectiveIndex`; traducción entrega bloques completos por idioma.
- Distinguir validaciones HTTP anteriores al stream de errores posteriores a
  abrirlo. Caracterizar primero las diferencias existentes antes de corregirlas.
- La desconexión de generaciones académicas conserva el job y su resultado.
  Cancelación explícita, timeout y desconexión no serán tratados como lo mismo.
  Chat/voz y otras familias deben mantener su política particular documentada.
- Mantener IDs y relaciones: `jobId`, `conversationId`, sesiones, intentos,
  `topicCode`, `objectiveCode` y claves privadas de respuesta. El cambio interno
  no crea otro intento, sesión ni cargo de cuota.
- Mantener CORS, proxy, allowlists, credenciales y permisos de cada familia.
  Una mejora de seguridad que exija cambiar clientes se diseñará y desplegará
  explícitamente, con contrato de transición y evaluación del riesgo.
- Conservar decisiones pedagógicas: score/IDs/cantidad válidos, referencias,
  umbrales actuales, español/contexto geográfico cuando corresponda y separación
  de generación educativa frente al banco de Lecciones de Idiomas.
- El canal uncensored mantiene aislamiento y ausencia de persistencia de texto.
  Las trazas compartidas no deben introducir allí memoria, replay ni datasets.

No se promete ausencia absoluta de regresiones: estos límites se verifican con
contratos ejecutables, canary, observación y reversión ensayada.

## 4. Diseño objetivo incremental

```text
Endpoints y transporte actuales
  → guards + DTO + identidad/cuota existentes
  → adaptador interno del caso de uso
  → contexto de ejecución y selección de procedimiento/versiones
  → memoria autorizada + conexiones permitidas
  → pasos deterministas o inferencia, según el caso
  → validadores y reglas del módulo
  → persistencia/efectos según contrato existente
  → serialización JSON/SSE actual

Transversal: presupuesto, correlación, métricas, fallos y activación gradual
```

La integración no implica que toda persistencia espere al resultado final:
se conservará el orden requerido por cada operación. Por ejemplo, un job nace
antes de ejecutar; progreso calificable requiere evaluación válida; ciertos
resultados educativos tienen persistencia posterior best-effort.

| Capa | Responsabilidad propuesta | Control verificable |
|---|---|---|
| 0. Modelo | Resolver proveedor/modelo/presupuesto por capacidad y configuración; admitir procedimientos sin LLM | El cliente no decide permisos ni eleva recursos; las lecciones deterministas y la traducción no invocan inferencia. |
| 1. Memoria | Leer contexto de sesión, preferencias permitidas y evidencia desde sus fuentes actuales | Ownership, expiración, procedencia y límites; una conversación no prueba dominio. |
| 2. Conexiones | Adaptar servicios públicos de módulos, RAG, Wiki, voz y traducción | Lista permitida, schemas, timeout, límites, clasificación lectura/escritura y autorización por operación. |
| 3. Skills | Ejecutar procedimientos versionados con entrada/salida y fallos tipados | Versión de prompt, contenido, reglas, herramientas y evaluaciones; sin ejecutar código enviado por el usuario. |
| 4. Rutinas | Coordinar pasos y reanudar trabajo autorizado | Estado explícito, checkpoint, identidad/versiones fijadas e idempotencia por efecto. |
| 5. Reglas | Aplicar seguridad, formato, dominio, fundamentación y política pedagógica | Reglas críticas en código y servicios autoritativos; el modelo no puede derogarlas. |
| 6. Fallos | Convertir incidentes en casos reproducibles y correcciones evaluadas | Datos saneados, causa, versión afectada, test y revisión antes de promover una mejora. |

Piezas internas a especificar antes de programar:

- Contexto de ejecución: identidad de usuario/cliente, caso de uso, versiones,
  correlación, permisos resueltos, presupuesto temporal/tokens/pasos, señal de
  cancelación y política de persistencia. Los campos derivados son del servidor.
- Registro de procedimientos: partir de `UseCaseRegistryService`, separar los
  handlers a medida que se migren y resolver versiones de forma explícita.
- Adaptadores de compatibilidad: entrada/salida y eventos internos mapeados al
  contrato existente; evitar ampliar el parcheado de texto SSE del runtime.
- Puertos de herramientas y memoria: contratos pequeños en la capa existente
  de aplicación; módulos conservan su dominio y el acceso a sus tablas.
- Versiones fijadas: código/skill, prompt, reglas, modelo real, contenido y
  estado de fuentes identificables; publicar una versión no cambia jobs activos.
  Los permisos y revocaciones vigentes se comprueban antes de entregar datos o
  aplicar efectos; fijar versiones nunca autoriza ignorar una revocación.
- Límite total de intentos: contar reparaciones, reintentos del upstream y de
  cola dentro del mismo presupuesto. Prohibir reintentos que dupliquen efectos
  o deltas. Una reparación no garantiza exactitud pedagógica.
- La secuencia inicial la decide el servidor. No suponer que el modelo actual
  ejecuta herramientas de forma fiable: cualquier bucle autónomo futuro necesita
  evaluación específica, límite de pasos y salida segura antes de habilitarse.

No se fija un árbol nuevo de directorios como objetivo. El código compartido
puro sigue en `learning-core`; Nest, Prisma y proveedores permanecen fuera de él.
Las dependencias nuevas entre módulos deben pasar por puertos/servicios públicos
y revisión de ciclos de importación e inyección.

## 5. Fases, entregables y condiciones de avance

Orden principal: **0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8**.
La observabilidad, seguridad y biblioteca de fallos comienzan en las primeras
fases y se amplían durante todo el trabajo. No se dejan para el final.

### Fase 0 — Inventario, línea base y riesgos

- Crear inventario completo de rutas a partir de controllers, DTOs, guards y
  bootstrap. Swagger es una ayuda, no cubre por sí solo SSE, cuotas ni efectos.
- Agrupar consumidores externos y versiones conocidas; recopilar muestras
  saneadas de solicitudes/respuestas, sin copiar datos personales a fixtures.
- Revisar código de cada familia conforme entre al alcance. Conciliar estado,
  contexto de módulo, decisiones vigentes y guía de integración del consumidor.
- Ejecutar typecheck y suite actual. Clasificar fallos previos, dependencias
  faltantes y pruebas omitidas; no presentarlos como cambios del harness.
- Verificar en entorno aislado migraciones pendientes, contenido publicado,
  índices vectoriales/TTL y disponibilidad de servicios. Restauraciones o cambios
  operativos se preparan por separado antes de ejecutarse.
- Medir por familia latencia de cola, primer evento útil, total, errores,
  reintentos, CPU/RAM y calidad. Separar modelo frío/caliente y niveles de carga.

**Entregable:** matriz de cobertura y compatibilidad, informe de línea base,
riesgos priorizados, fixture educativo inicial y backlog por incremento.
**Salida:** toda ruta clasificada como migrar, adaptar o preservar; cada riesgo
crítico tiene resolución o bloquea expresamente la fase dependiente.
**Reversión:** sin cambios de runtime; los hallazgos quedan como evidencia.

### Fase 1 — Entorno de ingeniería y contratos ejecutables

- Mantener `AGENTS.md` como mapa y `docs/` como memoria operativa. Separar estado
  actual de historial y planes; no hacer una reescritura documental masiva.
- Preparar arranque reproducible de tests y servicios aislados para desarrollo
  y CI, con bases/volúmenes/puertos propios. Los agentes no necesitan producción.
- Añadir verificaciones de límites de módulo y pureza de `learning-core` sobre
  invariantes reales, con mensajes que expliquen cómo corregir una infracción.
- Automatizar los contratos críticos y ampliar hasta cubrir todas las familias;
  verificar JWT/API key/roles/cuotas, errores y efectos además del schema.
- Definir pipeline de CI con typecheck, tests, contratos y validación documental;
  integrar con el proveedor usado por el equipo, sin asumir un workflow existente.
- Incorporar desde aquí registro de fallos saneados y evaluaciones iniciales.

**Entregable:** verificación reproducible y contrato de entrega por incremento.
**Salida:** un cambio deliberadamente incompatible es detectado por los checks;
un cambio compatible pasa; las pruebas críticas no quedan omitidas silenciosamente.
**Reversión:** ajustar checks defectuosos con evidencia; no rebajar contratos
para aceptar una regresión.

### Fase 2 — Contexto interno y primer caso sin efectos nuevos

- Diseñar el contexto, puertos y selección versionada descritos en la sección 4.
- Integrar un adaptador mínimo en `ai-orchestrator`, conservando transportes y
  los permisos originales; evitar un servicio central que replique todo el dominio.
- Primer piloto: respuesta estricta con conocimiento de `/api/chat/rag`.
  Reutilizar `KnowledgeChatService`, su política sin contexto y sus controles;
  no cambiar al mismo tiempo embeddings, búsqueda, modelo y prompts.
- El piloto conserva auditoría/logs existentes, pero no agrega memoria personal,
  progreso ni herramientas de escritura. La comparación offline suprime también
  los efectos existentes usando dobles o almacenes aislados.
- Añadir trazas de etapas y métricas con correlación; no guardar razonamiento
  interno del modelo ni texto crudo por defecto.

**Entregable:** un caso real operando por la frontera interna con flag apagada
por defecto y equivalencia verificable en fixtures.
**Salida:** mismo contrato, fuentes y política de fallos; sin dependencia nueva
de identidad JWT para clientes API key existentes.
**Reversión:** desactivar la selección del adaptador; conservar la ruta actual.

### Fase 3 — Skills, conexiones y reglas educativas

- Registrar por procedimiento schemas, permisos, dependencias, política de
  fuentes, presupuestos, versiones, códigos de fallo y criterios de aceptación.
- Incorporar primero la lección Wiki determinista como procedimiento sin LLM.
  Preservar umbral vigente, colección, referencias, cinco secciones y error sin
  evidencia; no agregar ejercicios al payload de lección.
- Incorporar actividades/tutor/contenido docente en incrementos separados;
  reutilizar validadores, JSON Schema admitido por Ollama, idioma y reparación.
- Conectar servicios mediante sus interfaces autorizadas. Distinguir fuente
  institucional publicada, Wiki y material enviado por el usuario.
- Tratar documentos y resultados de herramientas como datos no confiables:
  no pueden modificar políticas, seleccionar secretos ni autorizar escrituras.
- Preservar revisión/publicación de conocimiento y contenido guiado; la ingesta
  administrativa directa no se convierte en herramienta disponible para alumnos.

**Entregable:** catálogo mínimo de procedimientos con versiones y dos recorridos
verificados: lección determinista y actividad generativa validada.
**Salida:** fallos estructurales, evidencia insuficiente e inyección se detectan
sin producir resultados marcados como válidos; reglas autoritativas no se duplican.
**Reversión:** flag por caso de uso y retorno a su servicio actual.

### Fase 4 — Memoria autorizada y privacidad

- Inventariar cada dato por finalidad, propietario, origen, sensibilidad,
  retención, ubicación, autoridad para modificarlo y comportamiento al borrarlo.
- Exponer puertos sobre conversaciones/perfiles/sesiones existentes. Distinguir
  contexto de ejecución, memoria efímera y evidencia durable de aprendizaje.
- Leer progreso solo del dominio correspondiente. Las preferencias pueden
  orientar una explicación; no cambiar calificaciones, cuotas ni permisos.
- Especificar presupuesto de contexto y selección/resumen con procedencia.
  Los resúmenes no adquieren autoridad sobre evidencia curricular.
- Para escrituras nuevas, validar identidad, finalidad y consentimiento aplicable
  según mecanismos existentes. No inferir datos sensibles de la conversación.
- Integrar exportación, borrado/anonimización, expiración, cachés y revocación.
  Considerar jobs en curso: no deben recrear memoria después del borrado.
- Revisar registros actuales que guardan preguntas/respuestas; reducir exposición
  sin afirmar que todo el logging existente ya está saneado.
- Resolver ownership de jobs/resultados antes de persistir datos personales
  nuevos; documentar compatibilidad y riesgos del acceso legado por API key.

**Entregable:** contrato de memoria, matriz de datos y pruebas de aislamiento y
ciclo de vida, con migraciones aditivas solo si son necesarias.
**Salida:** ningún usuario/cliente puede recuperar memoria ajena; una API key
sin identidad no habilita memoria personal; exportación/borrado cubren datos nuevos.
**Reversión:** detener lecturas/escrituras nuevas por flag; conservar estructuras
aditivas y resolver su retención. Desactivar una flag no deshace escrituras previas.

### Fase 5 — Rutinas durables y efectos controlados

- Elegir un recorrido con identidad y evidencia ya definidas: consultar estado
  de aprendizaje → presentar contenido aprobado → iniciar actividad/intento →
  recibir respuesta → evaluar → registrar evidencia → recomendar siguiente paso.
- Separar preparación de contenido de la interacción del alumno. La rutina
  espera su respuesta en un estado persistido; no fabrica respuestas ni completa
  una evaluación en nombre del estudiante.
- En Idiomas, reutilizar intentos, grader, sesiones y mapeos publicados. No unir
  el generador `/education/lessons` al progreso de Idiomas suponiendo equivalencia.
  Una rutina educativa general necesita primero su propio contrato de evidencia.
- Definir estados, errores terminales/reintentables, pasos y checkpoints; fijar
  versión del flujo y contenido al inicio y resolver reanudación tras reinicio.
- Idempotencia durable por principal + ejecución + paso + operación y fingerprint.
  Una clave repetida no debe consumir cuota ni registrar progreso dos veces.
- Hacer atómicas las escrituras relacionadas cuando sea posible. Para efectos
  entre PostgreSQL/Mongo/servicios, definir reconciliación y compensaciones
  explícitas; evaluar patrón outbox si la frontera lo exige.
- Ante respuesta incierta del proveedor, verificar el efecto antes de repetir.
  No prometer ejecución exactamente una vez en sistemas distribuidos.
- Ampliar la cola existente si basta. Un registro de rutina/step será aditivo;
  no insertar nuevos estados en respuestas públicas de jobs sin contrato aparte.
- Mantener una sola instancia con la cola actual. Claims/leases/locks y pruebas
  de competencia son un requisito previo si se decide añadir otra instancia.

**Entregable:** rutina educativa autenticada reanudable, con efecto idempotente y
evidencia de éxito/fallo. Las operaciones actuales siguen disponibles.
**Salida:** reinicio en cada frontera no duplica evidencia ni cuota; cierre de
conversación no equivale a dominio; progreso conserva `LESSONS`/`DISCOVER`.
**Reversión:** impedir nuevas rutinas; permitir finalizar o reconciliar las
existentes con su versión fijada. No borrar evidencia ni reejecutar pasos confirmados.

### Fase 6 — Evaluación comparativa y despliegue gradual

- Ejecutar fixtures antiguos/nuevos sobre el mismo corpus y configuración.
  Para LLM, varias repeticiones y revisión por rúbrica; no exigir igualdad de prosa.
- Realizar replay offline saneado en almacenamiento aislado. Comparación en
  sombra solo si no duplica escrituras, cuotas, ingesta bajo demanda ni carga que
  afecte usuarios. No duplicar inferencias pesadas en la CPU de producción.
- Ensayar despliegue, compatibilidad de migraciones y rollback en staging.
- Activar primero cuentas internas, después cohortes estables propuestas de
  1% → 5% → 25% → 100%, únicamente si hay volumen suficiente para evaluarlas.
  Para tráfico bajo, usar cuentas controladas y una ventana mayor.
- Usar overrides existentes para usuarios; alcance por cliente API key requiere
  identidad de cliente comprobable y selección del servidor, no un header libre.
- Hacer canary dentro de la instancia actual. No implementar blue/green con dos
  consumidores de cola sobre la misma base sin resolver antes sus claims.
- Mantener flags distintas para orquestación, memoria, rutinas y modelos.
  La asignación de una ejecución activa queda fijada aunque cambie la flag.

**Entregable:** reporte comparativo, matriz de versiones, runbook de activación y
reversión, revisión de seguridad/dominio y responsables identificados.
**Salida:** cumplir los criterios de la sección 8, con muestra suficiente y sin
incidentes críticos abiertos; aprobación humana del release por su responsable.
**Reversión:** detener admisiones nuevas al camino afectado, drenar o reconciliar
jobs, conservar el lector compatible y volver al artefacto probado cuando proceda.

### Fase 7 — Ampliación por familias

Cada fila es un incremento o varios, con baseline, flag, contratos y evaluación
propios. Adoptar una interfaz común no obliga a reemplazar todos los servicios.

| Orden | Familia | Protección específica |
|---|---|---|
| 1 | Chat estricto/RAG | Sin contexto no inventa; caída de una fuente no elimina la otra; mantener `useWiki:false`. |
| 2 | Lecciones Wiki | Determinismo, colección, referencias y error por evidencia insuficiente; no LLM. |
| 3 | Actividades, tutor, temarios, transformaciones y juegos | Cantidad, claves, idioma, límites, semilla/reintentos y separación de pistas/respuestas. |
| 4 | Contenido docente/exámenes | Hilos estables, validación específica por tipo y borradores/resultados según contrato. |
| 5 | Chat general, historial y Explora | Fallback propio, cuotas, cancelación y correlación de streams paralelos. |
| 6 | Idiomas, lecciones de banco y Práctica guiada | Publicación, sesiones/TTL, idempotencia, rúbricas, respuestas privadas y tracks separados. |
| 7 | Traducción y voz | LibreTranslate sin fallback LLM, caché por proveedor, binarios/audio, límites y degradación independiente. |
| 8 | Conocimiento administrativo, documentos y automatizaciones | Roles/secret key, aprobación editorial y efectos autorizados; sin SQL/HTTP arbitrarios para el modelo. |
| 9 | Resto de superficies | Organizador de textos sigue determinista. Auth, consentimiento, pagos, suscripciones, flags, salud, métricas y exportaciones se caracterizan y preservan; solo se adaptan si hay beneficio concreto. |
| 10 | Canal uncensored y consumidores offline | Aislamiento/no persistencia; paridad de `learning-core` y artefactos desktop sin introducir dependencias del gateway. |

**Salida:** todas las rutas del inventario tienen estado final y evidencia:
adoptada, adaptada, o preservada con justificación. Los contratos se prueban
también sobre rutas que no atraviesan el harness.

### Fase 8 — Operación y mejora continua

- Cada incidente reproducible produce caso saneado, clasificación, causa,
  corrección y prueba/evaluación. No entrenar ni publicar reglas automáticamente.
- Revisar calidad educativa con responsable pedagógico; no aprobar solo porque
  un LLM juez declare correcta la respuesta o el JSON sea válido.
- Vigilar latencia, cola, fallbacks, fallos por dependencia, uso de memoria,
  consumo de recursos y resultados de tareas completas.
- Revisar documentos y límites de arquitectura periódicamente; retirar capas
  que no aporten valor medido y duplicaciones detectadas.
- Mantener un catálogo de versiones activas/retiradas. Retirar el camino previo
  de cada caso solo tras estabilidad observada y cierre de sus jobs/sesiones.
- Evaluar router 1B/4B u otros proveedores como experimento independiente, con
  la misma batería, calidad, recursos y posibilidad de volver al modelo vigente.

**Salida de la transición:** cobertura completa del inventario, al menos una
rutina con efectos verificada, operación documentada y responsables de mantenimiento.
La operación posterior continúa sin declararse una migración perpetua.

## 6. Matriz de pruebas y evaluaciones

Ubicar pruebas conforme a [TESTING.md](TESTING.md): specs junto a módulos,
integración junto a la capacidad y Python en su servicio. Los datasets de
evaluación pueden acompañar al módulo; cualquier runner fuera de los patrones
de Vitest debe tener un comando explícito y ejecución en CI documentados.

| Nivel | Qué demuestra | Cuándo |
|---|---|---|
| Unitario determinista | Reglas, schemas, permisos, semilla, presupuestos y transiciones | Cada incremento. |
| Contrato HTTP/SSE | DTO, guards, status, errores, eventos y finalización con handlers controlados | Cada cambio que toque una frontera. |
| Integración | PostgreSQL+pgvector, Mongo/TTL, jobs, locks, transacciones y privacidad reales | Antes de admitir persistencia/efectos y en CI con servicios aislados. |
| E2E educativo | Identidad → contenido → respuesta del alumno → evaluación → evidencia → recomendación | Rutina piloto y cada ampliación relevante. |
| Evaluación de contenido | Corrección, fundamento, nivel, idioma y utilidad; repeticiones/rúbrica humana | Antes de promover skill, prompt, corpus, regla o modelo. |
| Rendimiento/resiliencia | Carga, cola, saturación, timeout, reinicio, caída parcial y rollback | Antes de canary y ante cambios operativos. |

Comandos existentes para caracterizar la base al comenzar la implementación:

```bash
pnpm exec tsc -p tsconfig.build.json --noEmit
pnpm exec vitest run
pnpm exec vitest run src/modules/knowledge/retrieval/knowledge-chat.service.spec.ts
pnpm exec vitest run src/modules/ai-orchestrator src/modules/ai-generation-queue
```

Los smoke tests en `tests/manual/` no sustituyen la suite automática. Validar
servicios Python con las pruebas pertinentes cuando se modifiquen; compilar su
sintaxis por sí solo no valida comportamiento. Los builds y pruebas de clientes
se ejecutan en sus repositorios; no se presume paridad por compilar el gateway.

### Casos iniciales obligatorios

Cada caso incluirá entrada, estado inicial, dependencias simuladas o reales,
resultado esperado, efectos permitidos/prohibidos, versión y evidencia de corrida.
Estos 25 escenarios son el conjunto inicial; no reemplazan cubrir el inventario.

| ID | Escenario | Resultado que se debe verificar |
|---|---|---|
| HE-01 | Pregunta con fuente institucional y Wiki pertinentes | Respuesta con fuentes reales, formato vigente y trazabilidad. |
| HE-02 | Institucional/pgvector falla, Wiki funciona | Degradación independiente; preservar la regresión del spec existente. |
| HE-03 | `useWiki:false` con fuente institucional | No consultar Wiki; conservar respuesta institucional. |
| HE-04 | Sin contexto en chat estricto frente a chat general | Abstención del estricto y política propia del general; no homogeneizar. |
| HE-05 | Lección con Wiki suficiente | Cinco secciones, referencias, score/ámbito correctos y cero llamadas LLM. |
| HE-06 | Wiki vacía, débil o caída en lección | Error explícito; no producir lección de respaldo inventada. |
| HE-07 | Búsqueda con material docente extenso | Consulta acotada; país explícito prevalece y no se infiere desde la fuente. |
| HE-08 | Job reintentado con la misma semilla | Variante, IDs y asociaciones reproducibles. |
| HE-09 | Actividad con cantidad/IDs/clave inválidos | Reparación acotada o error; no devolver éxito con actividad incorregible. |
| HE-10 | Examen sin opciones / palabras con pistas desalineadas | Política específica del tipo; mantener respuesta y pista vinculadas. |
| HE-11 | Inyección en usuario, documento o resultado de herramienta | No elevar permisos, revelar secretos ni ejecutar escritura no autorizada. |
| HE-12 | API key sin usuario / JWT / roles insuficientes | Contrato original por ruta; memoria personal requiere identidad/ownership. |
| HE-13 | Usuario B solicita memoria, job o sesión de A | Denegación según contrato aplicable; hallazgos legados quedan explícitos. |
| HE-14 | Sesión expirada, reanudación o sin contenido publicado | No inventar sesión, publicar borrador ni cobrar cuota dos veces. |
| HE-15 | Dos turnos o submits concurrentes y repetidos | Orden consistente e idempotencia; no evidencia duplicada. |
| HE-16 | Cierre manual o sesión completada sin rúbrica válida | No declarar dominio; conservar separación de tracks. |
| HE-17 | Desconexión SSE de una generación académica | Job continúa y resultado consultable con el mismo identificador. |
| HE-18 | Error antes/después de abrir SSE y fragmentación de chunks | HTTP o evento correcto; único cierre terminal según contrato, sin stream colgado. |
| HE-19 | Cola llena, dependencia lenta y timeout tras deltas | Error previsto; presupuesto respetado y sin generación duplicada. |
| HE-20 | Reinicio entre efecto persistido y checkpoint | Recuperación/reconciliación sin repetir progreso, cuota ni escritura. |
| HE-21 | Publicación nueva y rollback con jobs activos | Cada ejecución conserva versiones y resultado interpretable. |
| HE-22 | Exportación/borrado o revocación entre lectura y entrega/escritura | Datos nuevos cubiertos; bloquear salida/efectos no autorizados y evitar recrear datos borrados al terminar el job o en replay. |
| HE-23 | LibreTranslate caído o caché antiguo | Error de dependencia / caché separado; no inferencia generativa. |
| HE-24 | STT/TTS falla o audio excede límites | Error/degradación propios; no bloquear otras capacidades. |
| HE-25 | Explora paralelo y canal uncensored instrumentados | Índices/eventos correctos; canal aislado sin guardar texto ni memoria. |

La biblioteca inicial debe incluir incidentes ya documentados: consultas Wiki
demasiado largas, evidencia débil para páramos, exámenes con opciones ausentes,
streams sin cierre y contenido guiado no publicado. Recrearlos con fixtures
saneados; no reutilizar jobs/transcripts de usuarios como datos de entrenamiento.

## 7. Seguridad, datos y migraciones

- Denegar por defecto herramientas fuera del catálogo del procedimiento. Validar
  argumentos, identidad y permiso en el adaptador y en el servicio autoritativo.
- Ningún prompt concede acceso libre a SQL, shell, URLs internas o secretos.
  Si se habilitan URLs externas, limitar destinos, tamaño y protocolos y proteger
  contra SSRF. Herramientas con efectos requieren finalidad y autorización verificables.
- Verificar HTML/renderizado, uploads y contenido remoto en sus fronteras
  existentes. Las credenciales siguen fuera de prompts, respuestas y fixtures.
- Distinguir correlación operativa de contenido personal. Métricas con etiquetas
  de baja cardinalidad; identificadores solo en trazas restringidas cuando proceda.
- Inventariar payload/result de jobs, cachés y almacenes externos: no basta con
  añadir `userId` a una tabla para cubrir exportación y borrado de todo el flujo.
- Antes de cualquier cambio persistente: ADR revisada, migración aditiva,
  índices/constraints, plan de backfill idempotente, respaldo restaurable y prueba
  con versión anterior/nueva. Separar expansión y retirada del schema.
- No borrar ni renombrar tablas/campos públicos durante el canary. El rollback
  de código debe poder leer el schema expandido; si no puede, el release se bloquea.
- Nuevas autorizaciones y contratos de progreso se resuelven antes de usar datos
  reales; este documento no reemplaza revisión de los requisitos legales vigentes.

## 8. Criterios de promoción y reversión

Son objetivos propuestos, no resultados medidos. En fase 0 se fijarán por
familia: ventana, carga comparable, mínimo de muestras, responsable y presupuesto.

| Dimensión | Criterio propuesto |
|---|---|
| Compatibilidad | 100% de los casos de contrato aplicables pasan; ninguna ruptura no acordada de DTO, auth, errores o SSE. |
| Seguridad | Cero fugas entre usuarios, accesos indebidos o secretos en casos críticos. Un solo incidente bloquea o revierte el caso afectado. |
| Efectos | Cero duplicaciones en pruebas de reintento/concurrencia/reinicio; reconciliación demostrada ante fallos parciales. |
| Calidad | 100% de validadores deterministas críticos; calidad por rúbrica al menos igual a la línea base y sin errores educativos críticos. Registrar tasa y dispersión, no solo un promedio. |
| Rendimiento | Propuesta inicial: p95 total y primer evento útil no empeoran más de 10% bajo carga equivalente; fijar además presupuesto absoluto por familia y medir overhead del harness por separado. |
| Fiabilidad | Sin aumento material de errores atribuibles al cambio; propuesta inicial: aumento menor a 0,5 puntos porcentuales con muestra suficiente. Tráfico escaso requiere más observación y casos controlados. |
| Recursos | CPU/RAM y ocupación dentro del presupuesto acordado; respetar límites de inferencia, cola y timeout existentes. |
| Recuperación | Rollback ensayado con jobs en curso; propuesta de objetivo inicial: desactivar admisiones nuevas al camino afectado en menos de cinco minutos. |

Un resultado no concluyente no autoriza avanzar. No atribuir mejoras al harness
si cambiaron a la vez modelo, corpus, hardware o estado de carga.

Procedimiento de reversión:

1. Registrar incidente y ejecución/versiones afectadas; bloquear nueva promoción.
2. Apagar flag de admisión del caso; no cambiar de implementación una ejecución
   a mitad de camino ni volver a ejecutar un efecto ya confirmado.
   Ante fuga, revocación de permisos o borrado, bloquear inmediatamente salida y
   efectos pendientes; cancelar o poner en cuarentena los trabajos afectados.
   La versión fijada sirve para reconciliar, nunca para continuar una operación insegura.
3. Drenar o reconciliar jobs de esa versión; si requieren intervención, conservar
   estado y evidencia. Mantener disponibles sus handlers mientras sean necesarios.
4. Restaurar artefacto previo solo tras comprobar compatibilidad con schema y
   trabajos pendientes. Con la cola actual, detener/drenar el proceso anterior
   antes de iniciar otro sobre la misma base.
5. Verificar contratos, salud y muestras reales autorizadas. Promover otra vez
   únicamente después de reproducir, corregir y evaluar la causa.

## 9. Backlog ejecutable y responsables

Todos los bloques están **pendientes**. Los roles son responsabilidades a asignar,
no aprobaciones ya obtenidas ni permiso para desplegar.

| Bloque | Depende de | Responsable principal | Evidencia de cierre |
|---|---|---|---|
| HE-P0 | — | Backend + responsable de operación | Inventario, baseline, riesgos y configuración efectiva. |
| HE-P1 | P0 | Backend/QA | Entorno aislado, CI, contratos y fallos iniciales reproducibles. |
| HE-P2 | P1 | Architect → coder/coder-strong | Contratos internos revisados y piloto RAG equivalente. |
| HE-P3 | P2 | Backend + responsable pedagógico | Skills versionadas, políticas de fuentes y validadores. |
| HE-P4 | P2/P3 | Backend + seguridad/privacidad | Aislamiento, finalidad, ownership y exportación/borrado. |
| HE-P5 | P3/P4 | Architect → coder-strong + QA | Rutina durable con idempotencia y evidencia correcta. |
| HE-P6 | P1–P5 | Operación + QA + responsable del producto | Evaluación, canary y reversión ensayados. |
| HE-P7 | P6 por cada familia | Responsable de cada módulo/cliente | Matriz de cobertura completa y contratos de consumidores. |
| HE-P8 | Desde P1; cierre tras P7 | Mantenimiento + pedagogía + operación | Métricas, biblioteca de fallos y ownership continuo. |

La primera entrega implementable comprende P0/P1 y un piloto P2; no pretende
resolver toda la memoria, modelos y rutinas en una semana. Después se estimará
esfuerzo con el inventario y la capacidad reales. No se adopta el plazo de dos
semanas del adjunto como compromiso para toda la transición.

Cada tarea futura debe indicar: problema observable, alcance/no alcance,
archivos, contratos afectados, dependencias, riesgos, prueba de aceptación,
migración/operación si aplica, flag/reversión y evidencia obtenida.

Aplicar el enrutamiento existente: `fast` para consultas/documentación;
`architect` para decisiones, seguridad y persistencia; implementación en
`coder`/`coder-strong`. Trabajar por incrementos revisables, sin commits, push,
despliegues ni cambios operativos implícitos en una tarea de planificación.

Al cerrar cada incremento significativo, actualizar `PROJECT_STATE.md`, el
contexto del módulo y las decisiones que realmente se hayan adoptado. No marcar
una fase completa por crear sus carpetas o escribir sus interfaces.

## 10. Preguntas que se resuelven durante la fase 0

- ¿Qué consumidores y contratos están activos y cómo se ejecutarán sus pruebas
  de aceptación desde repositorios externos?
- ¿Cuál es la situación real de datos, contenido publicado, migraciones,
  configuración de modelos y dependencias en staging/producción?
- ¿Qué reglas de memoria/finalidad/retención ya cubren cada dato y cuáles faltan?
- ¿Quién revisa calidad pedagógica, seguridad, operación y autoriza cada release?
- ¿Qué nivel de carga y presupuesto por capacidad hacen viable cada cohorte?
- ¿Qué mejoras de permisos del acceso legado a jobs requieren una transición
  independiente antes de admitir información personal?

Estas preguntas no impiden preparar contratos y fixtures locales. Sí bloquean
las fases que dependan de sus respuestas; no deben resolverse con supuestos
silenciosos durante un despliegue.

## 11. Fuentes y relación con el texto adjunto

El adjunto aportó la organización en siete capas y el objetivo educativo. Este
plan adapta esa propuesta a las fronteras que ya existen; no verifica ni atribuye
al video original afirmaciones que solo aparecen en el resumen proporcionado.

La separación entre entorno de ingeniería, conocimiento navegable, restricciones
verificables y feedback se apoya en
[Harness engineering, de OpenAI](https://openai.com/index/harness-engineering/).
El trabajo incremental con estado explícito y comprobación antes de declarar
terminada una tarea se apoya en
[Effective harnesses for long-running agents, de Anthropic](https://www.anthropic.com/research/effective-harnesses-for-long-running-agents).
Son referencias conceptuales; no obligan a cambiar proveedor ni adoptar su SDK.

Referencias locales: [arquitectura](ARCHITECTURE.md),
[decisiones](DECISIONS.md), [pruebas](TESTING.md),
[backend](modules/backend.md), [conocimiento](modules/knowledge.md),
[idiomas](modules/languages.md), [voz](modules/voice.md) y
[contrato del consumidor](INTEGRACION_PROYECTO_CONSUMIDOR.md).
