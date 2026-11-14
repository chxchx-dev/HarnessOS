use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use super::resource::fits_memory_budget;
use super::{
    Budget, CancellationToken, ErrorCode, Principal, Registry, RunFailure, RunRequest, RunResult,
    RunTrace, SkillContext, TraceOutcome,
};

pub const SERVER_BUDGET: Budget = Budget {
    max_tool_calls: 4,
    timeout: Duration::from_millis(8_000),
};
pub const DEFAULT_MEMORY_STACK_BYTES: usize = 256 * 1024;
pub const DEFAULT_MAX_CONCURRENT_RUNS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    pub max_concurrent_runs: usize,
    pub worker_stack_bytes: usize,
    pub max_input_bytes: usize,
    pub max_tool_result_bytes: usize,
    pub max_output_bytes: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_runs: DEFAULT_MAX_CONCURRENT_RUNS,
            worker_stack_bytes: DEFAULT_MEMORY_STACK_BYTES,
            max_input_bytes: 1024 * 1024,
            max_tool_result_bytes: 4 * 1024 * 1024,
            max_output_bytes: 4 * 1024 * 1024,
        }
    }
}

pub struct HarnessRuntime {
    registry: Arc<Registry>,
    config: RuntimeConfig,
    workers: OnceLock<Result<WorkerPool, ()>>,
    active_runs: Arc<AtomicUsize>,
}

impl HarnessRuntime {
    pub fn new(registry: Registry) -> Self {
        Self::with_config(registry, RuntimeConfig::default())
    }

    pub fn with_config(registry: Registry, mut config: RuntimeConfig) -> Self {
        config.max_concurrent_runs = config.max_concurrent_runs.max(1);
        config.worker_stack_bytes = config.worker_stack_bytes.max(64 * 1024);
        config.max_input_bytes = config.max_input_bytes.max(1);
        config.max_tool_result_bytes = config.max_tool_result_bytes.max(1);
        config.max_output_bytes = config.max_output_bytes.max(1);
        Self {
            registry: Arc::new(registry),
            config,
            workers: OnceLock::new(),
            active_runs: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn execute(&self, request: RunRequest) -> RunResult {
        let started = Instant::now();
        let trace_info = TraceInfo::from_request(&request);
        if !valid_principal(&request.principal)
            || request.correlation_id.is_empty()
            || request.correlation_id.encode_utf16().count() > 128
        {
            return failure(
                ErrorCode::InvalidRequest,
                "Contexto interno de ejecución inválido.",
                trace_info,
                0,
                started,
            );
        }
        let Some(budget) = reduce_budget(request.requested_budget) else {
            return failure(
                ErrorCode::InvalidRequest,
                "El presupuesto debe contener enteros positivos.",
                trace_info,
                0,
                started,
            );
        };
        if !fits_memory_budget(&request.input, self.config.max_input_bytes) {
            return failure(
                ErrorCode::BadInput,
                "Entrada excede el límite de memoria.",
                trace_info,
                0,
                started,
            );
        }
        let Some(skill) = self.registry.get_skill(&request.skill_id, &request.version) else {
            return failure(
                ErrorCode::UnknownSkill,
                "Skill o versión no registrada.",
                trace_info,
                0,
                started,
            );
        };
        if !skill
            .required_scopes()
            .iter()
            .all(|scope| request.principal.has_scope(scope))
        {
            return failure(
                ErrorCode::Unauthorized,
                "Permiso insuficiente para ejecutar la skill.",
                trace_info,
                0,
                started,
            );
        }
        if !skill.validate(&request.input) {
            return failure(
                ErrorCode::BadInput,
                "Entrada inválida para la skill.",
                trace_info,
                0,
                started,
            );
        }

        let workers = self.workers.get_or_init(|| {
            WorkerPool::new(
                self.config.max_concurrent_runs,
                self.config.worker_stack_bytes,
            )
            .map_err(|_| ())
        });
        let Ok(workers) = workers else {
            return failure(
                ErrorCode::InternalError,
                "No se pudo iniciar el grupo de ejecución.",
                trace_info,
                0,
                started,
            );
        };
        let Some(permit) =
            RunPermit::try_acquire(&self.active_runs, self.config.max_concurrent_runs)
        else {
            return failure(
                ErrorCode::BudgetExceeded,
                "Límite de ejecuciones simultáneas excedido.",
                trace_info,
                0,
                started,
            );
        };
        let tool_call_count = Arc::new(AtomicUsize::new(0));
        let worker_started = Instant::now();
        let signal = Arc::new(CancellationToken::new(worker_started + budget.timeout));
        let (sender, receiver) = mpsc::sync_channel(1);
        let job = RunJob {
            request,
            skill,
            registry: Arc::clone(&self.registry),
            signal: Arc::clone(&signal),
            tool_call_count: Arc::clone(&tool_call_count),
            trace: trace_info.clone(),
            started,
            max_tool_calls: budget.max_tool_calls,
            max_tool_result_bytes: self.config.max_tool_result_bytes,
            max_output_bytes: self.config.max_output_bytes,
            result_sender: sender,
            permit,
        };
        if workers.sender.send(job).is_err() {
            return failure(
                ErrorCode::InternalError,
                "El grupo de ejecución no está disponible.",
                trace_info,
                0,
                started,
            );
        }

        match receiver.recv_timeout(budget.timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                signal.cancel();
                failure(
                    ErrorCode::BudgetExceeded,
                    "Tiempo de ejecución excedido.",
                    trace_info,
                    tool_call_count.load(Ordering::Acquire),
                    started,
                )
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => failure(
                ErrorCode::InternalError,
                "Fallo inesperado en la ejecución.",
                trace_info,
                tool_call_count.load(Ordering::Acquire),
                started,
            ),
        }
    }
}

struct WorkerPool {
    sender: mpsc::SyncSender<RunJob>,
}

impl WorkerPool {
    fn new(count: usize, stack_bytes: usize) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel::<RunJob>(0);
        let receiver = Arc::new(Mutex::new(receiver));
        let (ready_sender, ready_receiver) = mpsc::sync_channel::<()>(0);
        for index in 0..count {
            let worker_receiver = Arc::clone(&receiver);
            let worker_ready = ready_sender.clone();
            thread::Builder::new()
                .name(format!("harness-worker-{index}"))
                .stack_size(stack_bytes)
                .spawn(move || {
                    let _ = worker_ready.send(());
                    loop {
                        let message = {
                            let locked = worker_receiver
                                .lock()
                                .unwrap_or_else(|error| error.into_inner());
                            locked.recv()
                        };
                        match message {
                            Ok(job) => run_job(job),
                            Err(_) => break,
                        }
                    }
                })?;
        }
        drop(ready_sender);
        for _ in 0..count {
            ready_receiver.recv().map_err(|_| {
                std::io::Error::other("El grupo de ejecución no inició todos los workers.")
            })?;
        }
        Ok(Self { sender })
    }
}

struct RunJob {
    request: RunRequest,
    skill: Arc<dyn super::Skill>,
    registry: Arc<Registry>,
    signal: Arc<CancellationToken>,
    tool_call_count: Arc<AtomicUsize>,
    trace: TraceInfo,
    started: Instant,
    max_tool_calls: usize,
    max_tool_result_bytes: usize,
    max_output_bytes: usize,
    result_sender: mpsc::SyncSender<RunResult>,
    permit: RunPermit,
}

fn run_job(job: RunJob) {
    let RunJob {
        request,
        skill,
        registry,
        signal,
        tool_call_count,
        trace,
        started,
        max_tool_calls,
        max_tool_result_bytes,
        max_output_bytes,
        result_sender,
        permit,
    } = job;
    let result = catch_unwind(AssertUnwindSafe(|| {
        let context = SkillContext::new(
            &request.principal,
            &signal,
            &registry,
            skill.allowed_tools(),
            max_tool_calls,
            max_tool_result_bytes,
            &tool_call_count,
        );
        skill.run(&request.input, &context)
    }));
    let count = tool_call_count.load(Ordering::Acquire);
    let result = match result {
        Ok(Ok(output)) if fits_memory_budget(&output, max_output_bytes) => {
            success(output, trace.clone(), count, started)
        }
        Ok(Ok(_)) => failure(
            ErrorCode::BudgetExceeded,
            "Límite de memoria excedido.",
            trace.clone(),
            count,
            started,
        ),
        Ok(Err(error)) => failure(error.code, error.message, trace.clone(), count, started),
        Err(_) => failure(
            ErrorCode::InternalError,
            "Fallo inesperado en la ejecución.",
            trace,
            count,
            started,
        ),
    };
    let _ = result_sender.send(result);
    drop(permit);
}

struct RunPermit(Arc<AtomicUsize>);

impl RunPermit {
    fn try_acquire(active: &Arc<AtomicUsize>, maximum: usize) -> Option<Self> {
        let mut current = active.load(Ordering::Relaxed);
        loop {
            if current >= maximum {
                return None;
            }
            match active.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(Self(Arc::clone(active))),
                Err(actual) => current = actual,
            }
        }
    }
}

impl Drop for RunPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Clone)]
struct TraceInfo {
    correlation_id: String,
    skill: String,
    version: String,
}

impl TraceInfo {
    fn from_request(request: &RunRequest) -> Self {
        Self {
            correlation_id: if request.correlation_id.is_empty() {
                "invalid".to_owned()
            } else {
                request.correlation_id.clone()
            },
            skill: request.skill_id.clone(),
            version: request.version.clone(),
        }
    }

    fn into_trace(self, outcome: TraceOutcome, count: usize, started: Instant) -> RunTrace {
        RunTrace {
            correlation_id: self.correlation_id,
            skill: self.skill,
            version: self.version,
            tool_call_count: count,
            duration_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            outcome,
        }
    }
}

fn valid_principal(principal: &Principal) -> bool {
    !principal.tenant_id.trim().is_empty()
        && principal
            .subject_id
            .as_ref()
            .is_none_or(|subject| !subject.trim().is_empty())
}

fn reduce_budget(requested: Option<super::RequestedBudget>) -> Option<Budget> {
    let requested = requested.unwrap_or_default();
    if requested.max_tool_calls == Some(0) || requested.timeout_ms == Some(0) {
        return None;
    }
    Some(Budget {
        max_tool_calls: requested
            .max_tool_calls
            .unwrap_or(SERVER_BUDGET.max_tool_calls)
            .min(SERVER_BUDGET.max_tool_calls),
        timeout: Duration::from_millis(
            requested
                .timeout_ms
                .unwrap_or(SERVER_BUDGET.timeout.as_millis() as u64)
                .min(SERVER_BUDGET.timeout.as_millis() as u64),
        ),
    })
}

fn success(
    output: serde_json::Value,
    trace: TraceInfo,
    count: usize,
    started: Instant,
) -> RunResult {
    RunResult::Success {
        output,
        trace: trace.into_trace(TraceOutcome::Ok, count, started),
    }
}

fn failure(
    code: ErrorCode,
    message: &'static str,
    trace: TraceInfo,
    count: usize,
    started: Instant,
) -> RunResult {
    RunResult::Failure {
        error: RunFailure { code, message },
        trace: trace.into_trace(TraceOutcome::Error, count, started),
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use serde_json::{Value, json};

    use super::HarnessRuntime;
    use crate::bootstrap::create_demo_harness;
    use crate::core::{
        AuthType, ErrorCode, HarnessError, Principal, Registry, RequestedBudget, RunRequest,
        RuntimeConfig, Skill, SkillContext, Tool, ToolContext,
    };

    fn principal(tenant_id: &str, scopes: &[&str]) -> Principal {
        Principal {
            tenant_id: tenant_id.to_owned(),
            subject_id: Some("user-1".to_owned()),
            auth_type: AuthType::Jwt,
            scopes: scopes.iter().map(|scope| (*scope).to_owned()).collect(),
        }
    }

    fn request(skill_id: &str, input: Value, scopes: &[&str]) -> RunRequest {
        RunRequest {
            skill_id: skill_id.to_owned(),
            version: "1.0.0".to_owned(),
            input,
            principal: principal("education-demo", scopes),
            correlation_id: "test-correlation".to_owned(),
            requested_budget: None,
        }
    }

    fn demo_runtime() -> HarnessRuntime {
        create_demo_harness(std::sync::Arc::new(crate::models::DemoModel)).unwrap()
    }

    #[test]
    fn retrieves_synthetic_sources_with_minimal_trace() {
        let result = demo_runtime().execute(request(
            "knowledge.retrieve",
            json!({ "question": "lecciones educativas" }),
            &["knowledge:read"],
        ));
        assert!(result.is_ok());
        assert_eq!(result.output().unwrap()["sources"][0]["id"], "DEMO-EDU-01");
        assert_eq!(result.trace().tool_call_count, 1);
        assert_eq!(result.trace().outcome.as_str(), "ok");
    }

    #[test]
    fn missing_evidence_returns_typed_abstention() {
        let result = demo_runtime().execute(request(
            "knowledge.retrieve",
            json!({ "question": "hipopotamos" }),
            &["knowledge:read"],
        ));
        assert_eq!(result.error().unwrap().code, ErrorCode::EvidenceMissing);
    }

    #[test]
    fn tenant_isolation_hides_other_demo_documents() {
        let mut request = request(
            "knowledge.retrieve",
            json!({ "question": "agenda horarios" }),
            &["knowledge:read"],
        );
        request.principal.tenant_id = "education-demo".to_owned();
        let result = demo_runtime().execute(request);
        assert_eq!(result.error().unwrap().code, ErrorCode::EvidenceMissing);
    }

    #[test]
    fn scope_denial_happens_before_tools() {
        let result = demo_runtime().execute(request(
            "knowledge.retrieve",
            json!({ "question": "lecciones educativas" }),
            &[],
        ));
        assert_eq!(result.error().unwrap().code, ErrorCode::Unauthorized);
        assert_eq!(result.trace().tool_call_count, 0);
    }

    #[test]
    fn exact_skill_version_is_required() {
        let mut request = request(
            "knowledge.retrieve",
            json!({ "question": "lecciones educativas" }),
            &["knowledge:read"],
        );
        request.version = "2.0.0".to_owned();
        assert_eq!(
            demo_runtime().execute(request).error().unwrap().code,
            ErrorCode::UnknownSkill
        );
    }

    #[test]
    fn invalid_question_is_rejected_before_tools() {
        let result = demo_runtime().execute(request(
            "knowledge.retrieve",
            json!({ "question": "x" }),
            &["knowledge:read"],
        ));
        assert_eq!(result.error().unwrap().code, ErrorCode::BadInput);
        assert_eq!(result.trace().tool_call_count, 0);
    }

    #[test]
    fn invalid_or_expansive_requested_budget_is_handled_safely() {
        let mut invalid = request(
            "knowledge.retrieve",
            json!({ "question": "lecciones educativas" }),
            &["knowledge:read"],
        );
        invalid.requested_budget = Some(RequestedBudget {
            max_tool_calls: Some(0),
            timeout_ms: None,
        });
        assert_eq!(
            demo_runtime().execute(invalid).error().unwrap().code,
            ErrorCode::InvalidRequest
        );

        let mut reduced = request(
            "knowledge.retrieve",
            json!({ "question": "lecciones educativas" }),
            &["knowledge:read"],
        );
        reduced.requested_budget = Some(RequestedBudget {
            max_tool_calls: Some(99),
            timeout_ms: Some(99_999),
        });
        assert!(demo_runtime().execute(reduced).is_ok());
    }

    struct FixedSkill {
        id: &'static str,
        allowed_tools: Vec<String>,
        behavior: Behavior,
    }

    enum Behavior {
        Call(&'static str),
        CallTwice,
        Output(usize),
        Sleep(Duration),
    }

    impl Skill for FixedSkill {
        fn id(&self) -> &str {
            self.id
        }
        fn version(&self) -> &str {
            "1.0.0"
        }
        fn required_scopes(&self) -> &[String] {
            &[]
        }
        fn allowed_tools(&self) -> &[String] {
            &self.allowed_tools
        }
        fn validate(&self, _input: &Value) -> bool {
            true
        }
        fn run(&self, input: &Value, context: &SkillContext<'_>) -> Result<Value, HarnessError> {
            match self.behavior {
                Behavior::Call(id) => context.call_tool(id, input),
                Behavior::CallTwice => {
                    context.call_tool("knowledge.search", input)?;
                    context.call_tool("knowledge.search", input)
                }
                Behavior::Output(bytes) => Ok(json!("x".repeat(bytes))),
                Behavior::Sleep(duration) => {
                    thread::sleep(duration);
                    Ok(json!("late"))
                }
            }
        }
    }

    struct EmptyTool;

    impl Tool for EmptyTool {
        fn id(&self) -> &str {
            "knowledge.search"
        }
        fn required_scope(&self) -> &str {
            "knowledge:read"
        }
        fn validate(&self, _input: &Value) -> bool {
            true
        }
        fn execute(
            &self,
            _input: &Value,
            _context: &ToolContext<'_>,
        ) -> Result<Value, HarnessError> {
            Ok(json!([]))
        }
    }

    #[test]
    fn allowlist_is_enforced_even_if_tool_is_registered() {
        let mut registry = Registry::new();
        registry
            .register_skill(FixedSkill {
                id: "deny-tools",
                allowed_tools: vec![],
                behavior: Behavior::Call("knowledge.search"),
            })
            .unwrap();
        registry.register_tool(EmptyTool).unwrap();
        let runtime = HarnessRuntime::new(registry);
        let mut request = request("deny-tools", json!({}), &["knowledge:read"]);
        request.version = "1.0.0".to_owned();
        assert_eq!(
            runtime.execute(request).error().unwrap().code,
            ErrorCode::ToolForbidden
        );
    }

    #[test]
    fn tool_call_budget_cannot_be_exceeded() {
        let mut registry = Registry::new();
        registry
            .register_skill(FixedSkill {
                id: "twice",
                allowed_tools: vec!["knowledge.search".to_owned()],
                behavior: Behavior::CallTwice,
            })
            .unwrap();
        registry.register_tool(EmptyTool).unwrap();
        let runtime = HarnessRuntime::new(registry);
        let mut request = request("twice", json!({}), &["knowledge:read"]);
        request.requested_budget = Some(RequestedBudget {
            max_tool_calls: Some(1),
            timeout_ms: None,
        });
        let result = runtime.execute(request);
        assert_eq!(result.error().unwrap().code, ErrorCode::BudgetExceeded);
        assert_eq!(result.trace().tool_call_count, 1);
    }

    struct LargeTool;

    impl Tool for LargeTool {
        fn id(&self) -> &str {
            "large-result"
        }
        fn required_scope(&self) -> &str {
            "knowledge:read"
        }
        fn validate(&self, _input: &Value) -> bool {
            true
        }
        fn execute(
            &self,
            _input: &Value,
            _context: &ToolContext<'_>,
        ) -> Result<Value, HarnessError> {
            Ok(json!("x".repeat(512)))
        }
    }

    #[test]
    fn oversized_input_is_rejected_before_skill_lookup() {
        let runtime = HarnessRuntime::with_config(
            Registry::new(),
            RuntimeConfig {
                max_input_bytes: 64,
                ..RuntimeConfig::default()
            },
        );
        let result = runtime.execute(request("missing", json!("x".repeat(256)), &[]));
        assert_eq!(result.error().unwrap().code, ErrorCode::BadInput);
    }

    #[test]
    fn oversized_tool_result_is_rejected_at_the_boundary() {
        let mut registry = Registry::new();
        registry
            .register_skill(FixedSkill {
                id: "large-result-skill",
                allowed_tools: vec!["large-result".to_owned()],
                behavior: Behavior::Call("large-result"),
            })
            .unwrap();
        registry.register_tool(LargeTool).unwrap();
        let runtime = HarnessRuntime::with_config(
            registry,
            RuntimeConfig {
                max_tool_result_bytes: 128,
                ..RuntimeConfig::default()
            },
        );
        let result = runtime.execute(request(
            "large-result-skill",
            json!({}),
            &["knowledge:read"],
        ));
        assert_eq!(result.error().unwrap().code, ErrorCode::BudgetExceeded);
    }

    #[test]
    fn oversized_skill_output_is_rejected_before_return() {
        let mut registry = Registry::new();
        registry
            .register_skill(FixedSkill {
                id: "large-output",
                allowed_tools: vec![],
                behavior: Behavior::Output(512),
            })
            .unwrap();
        let runtime = HarnessRuntime::with_config(
            registry,
            RuntimeConfig {
                max_output_bytes: 128,
                ..RuntimeConfig::default()
            },
        );
        let result = runtime.execute(request("large-output", json!({}), &[]));
        assert_eq!(result.error().unwrap().code, ErrorCode::BudgetExceeded);
    }

    #[test]
    fn timeout_returns_even_if_skill_does_not_cooperate() {
        let mut registry = Registry::new();
        registry
            .register_skill(FixedSkill {
                id: "sleep",
                allowed_tools: vec![],
                behavior: Behavior::Sleep(Duration::from_millis(80)),
            })
            .unwrap();
        let runtime = HarnessRuntime::new(registry);
        let mut request = request("sleep", json!({}), &[]);
        request.requested_budget = Some(RequestedBudget {
            max_tool_calls: None,
            timeout_ms: Some(5),
        });
        assert_eq!(
            runtime.execute(request).error().unwrap().code,
            ErrorCode::BudgetExceeded
        );
    }

    #[test]
    fn concurrent_runs_are_bounded_and_timed_out_workers_keep_their_slot() {
        let mut registry = Registry::new();
        registry
            .register_skill(FixedSkill {
                id: "sleep",
                allowed_tools: vec![],
                behavior: Behavior::Sleep(Duration::from_millis(80)),
            })
            .unwrap();
        let runtime = HarnessRuntime::with_config(
            registry,
            super::RuntimeConfig {
                max_concurrent_runs: 1,
                ..super::RuntimeConfig::default()
            },
        );
        let mut first = request("sleep", json!({}), &[]);
        first.requested_budget = Some(RequestedBudget {
            max_tool_calls: None,
            timeout_ms: Some(5),
        });
        assert_eq!(
            runtime.execute(first).error().unwrap().code,
            ErrorCode::BudgetExceeded
        );
        let second = runtime.execute(request("sleep", json!({}), &[]));
        assert_eq!(second.error().unwrap().code, ErrorCode::BudgetExceeded);
        thread::sleep(Duration::from_millis(85));
        assert!(runtime.execute(request("sleep", json!({}), &[])).is_ok());
    }

    #[test]
    fn demo_model_marks_generated_answer_unverified() {
        let result = demo_runtime().execute(request(
            "knowledge.answer",
            json!({ "question": "lecciones educativas" }),
            &["knowledge:read", "model:use"],
        ));
        assert!(result.is_ok());
        assert_eq!(result.output().unwrap()["verified"], false);
        assert!(
            result.output().unwrap()["modelUsed"]
                .as_str()
                .unwrap()
                .contains("NO LLM")
        );
    }

    #[test]
    fn malformed_internal_context_is_rejected() {
        let mut request = request(
            "knowledge.retrieve",
            json!({ "question": "lecciones educativas" }),
            &["knowledge:read"],
        );
        request.principal.tenant_id = "  ".to_owned();
        assert_eq!(
            demo_runtime().execute(request).error().unwrap().code,
            ErrorCode::InvalidRequest
        );
    }
}
