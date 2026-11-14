use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde_json::Value;

use super::{HarnessError, Registry, resource::fits_memory_budget};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthType {
    Jwt,
    Service,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Principal {
    pub tenant_id: String,
    pub subject_id: Option<String>,
    pub auth_type: AuthType,
    pub scopes: Vec<String>,
}

impl Principal {
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|candidate| candidate == scope)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Budget {
    pub max_tool_calls: usize,
    pub timeout: Duration,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RequestedBudget {
    pub max_tool_calls: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    InvalidRequest,
    Unauthorized,
    UnknownSkill,
    BadInput,
    ToolForbidden,
    BudgetExceeded,
    EvidenceMissing,
    ModelUnavailable,
    DependencyFailed,
    InternalError,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::UnknownSkill => "UNKNOWN_SKILL",
            Self::BadInput => "BAD_INPUT",
            Self::ToolForbidden => "TOOL_FORBIDDEN",
            Self::BudgetExceeded => "BUDGET_EXCEEDED",
            Self::EvidenceMissing => "EVIDENCE_MISSING",
            Self::ModelUnavailable => "MODEL_UNAVAILABLE",
            Self::DependencyFailed => "DEPENDENCY_FAILED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceOutcome {
    Ok,
    Error,
}

impl TraceOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunTrace {
    pub correlation_id: String,
    pub skill: String,
    pub version: String,
    pub tool_call_count: usize,
    pub duration_ms: u64,
    pub outcome: TraceOutcome,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunFailure {
    pub code: ErrorCode,
    pub message: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RunResult {
    Success { output: Value, trace: RunTrace },
    Failure { error: RunFailure, trace: RunTrace },
}

impl RunResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn output(&self) -> Option<&Value> {
        match self {
            Self::Success { output, .. } => Some(output),
            Self::Failure { .. } => None,
        }
    }

    pub fn error(&self) -> Option<&RunFailure> {
        match self {
            Self::Success { .. } => None,
            Self::Failure { error, .. } => Some(error),
        }
    }

    pub fn trace(&self) -> &RunTrace {
        match self {
            Self::Success { trace, .. } | Self::Failure { trace, .. } => trace,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunRequest {
    pub skill_id: String,
    pub version: String,
    pub input: Value,
    /// Construido por el backend autenticado; nunca se acepta desde texto libre.
    pub principal: Principal,
    pub correlation_id: String,
    /// El solicitante interno solo puede reducir los límites fijados por servidor.
    pub requested_budget: Option<RequestedBudget>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceExcerpt {
    pub id: String,
    pub title: String,
    pub excerpt: String,
}

#[derive(Debug)]
pub struct CancellationToken {
    cancelled: AtomicBool,
    deadline: Instant,
}

impl CancellationToken {
    pub(crate) fn new(deadline: Instant) -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            deadline,
        }
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire) || Instant::now() >= self.deadline
    }

    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
}

pub struct ToolContext<'a> {
    pub principal: &'a Principal,
    pub signal: &'a CancellationToken,
}

pub struct SkillContext<'a> {
    pub principal: &'a Principal,
    pub signal: &'a CancellationToken,
    registry: &'a Registry,
    allowed_tools: &'a [String],
    max_tool_calls: usize,
    max_tool_result_bytes: usize,
    tool_call_count: &'a std::sync::atomic::AtomicUsize,
}

impl<'a> SkillContext<'a> {
    pub(crate) fn new(
        principal: &'a Principal,
        signal: &'a CancellationToken,
        registry: &'a Registry,
        allowed_tools: &'a [String],
        max_tool_calls: usize,
        max_tool_result_bytes: usize,
        tool_call_count: &'a std::sync::atomic::AtomicUsize,
    ) -> Self {
        Self {
            principal,
            signal,
            registry,
            allowed_tools,
            max_tool_calls,
            max_tool_result_bytes,
            tool_call_count,
        }
    }

    pub fn call_tool(&self, id: &str, input: &Value) -> Result<Value, HarnessError> {
        if self.signal.is_cancelled() {
            return Err(HarnessError::new(
                ErrorCode::BudgetExceeded,
                "Tiempo excedido.",
            ));
        }
        if !self.allowed_tools.iter().any(|allowed| allowed == id) {
            return Err(HarnessError::new(
                ErrorCode::ToolForbidden,
                "Tool no permitida para la skill.",
            ));
        }
        let Some(tool) = self.registry.get_tool(id) else {
            return Err(HarnessError::new(
                ErrorCode::ToolForbidden,
                "Tool no registrada.",
            ));
        };
        if !self.principal.has_scope(tool.required_scope()) {
            return Err(HarnessError::new(
                ErrorCode::Unauthorized,
                "Permiso insuficiente para la herramienta.",
            ));
        }
        if !tool.validate(input) {
            return Err(HarnessError::new(
                ErrorCode::BadInput,
                "Argumentos de herramienta inválidos.",
            ));
        }
        let mut count = self.tool_call_count.load(Ordering::Relaxed);
        loop {
            if count >= self.max_tool_calls {
                return Err(HarnessError::new(
                    ErrorCode::BudgetExceeded,
                    "Límite de herramientas excedido.",
                ));
            }
            match self.tool_call_count.compare_exchange_weak(
                count,
                count + 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => count = actual,
            }
        }
        let context = ToolContext {
            principal: self.principal,
            signal: self.signal,
        };
        let result = tool.execute(input, &context)?;
        if !fits_memory_budget(&result, self.max_tool_result_bytes) {
            return Err(HarnessError::new(
                ErrorCode::BudgetExceeded,
                "Límite de memoria excedido.",
            ));
        }
        Ok(result)
    }
}

pub trait Tool: Send + Sync {
    fn id(&self) -> &str;
    fn required_scope(&self) -> &str;
    fn validate(&self, input: &Value) -> bool;
    fn execute(&self, input: &Value, context: &ToolContext<'_>) -> Result<Value, HarnessError>;
}

pub trait Skill: Send + Sync {
    fn id(&self) -> &str;
    fn version(&self) -> &str;
    fn required_scopes(&self) -> &[String];
    fn allowed_tools(&self) -> &[String];
    fn validate(&self, input: &Value) -> bool;
    fn run(&self, input: &Value, context: &SkillContext<'_>) -> Result<Value, HarnessError>;
}

pub trait ModelPort: Send + Sync {
    fn name(&self) -> &str;
    fn generate(
        &self,
        question: &str,
        sources: &[SourceExcerpt],
        signal: &CancellationToken,
    ) -> Result<String, HarnessError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillDescriptor {
    pub id: String,
    pub version: String,
}
