use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::{ErrorCode, HarnessError, Principal};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryLimits {
    pub max_entries: usize,
    pub max_total_bytes: usize,
    pub max_record_bytes: usize,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_entries: 4_096,
            max_total_bytes: 16 * 1024 * 1024,
            max_record_bytes: 1_024,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryUsage {
    pub entries: usize,
    /// Cuenta bytes UTF-8 de claves y valores; el límite de entradas también acota el overhead del mapa.
    pub accounted_bytes: usize,
}

#[derive(Default)]
struct State {
    records: HashMap<MemoryKey, String>,
    accounted_bytes: usize,
}

#[derive(Eq, Hash, PartialEq)]
struct MemoryKey {
    tenant_id: String,
    subject_id: String,
    name: String,
}

impl MemoryKey {
    fn accounted_bytes(&self) -> usize {
        self.tenant_id.len() + self.subject_id.len() + self.name.len()
    }
}

/// Almacén volátil de ejemplo. No es persistencia ni está aprobado para datos personales.
pub struct EphemeralMemory {
    limits: MemoryLimits,
    state: Mutex<State>,
}

impl EphemeralMemory {
    pub fn new(limits: MemoryLimits) -> Result<Self, HarnessError> {
        if limits.max_entries == 0
            || limits.max_total_bytes == 0
            || limits.max_record_bytes == 0
            || limits.max_record_bytes > limits.max_total_bytes
        {
            return Err(HarnessError::new(
                ErrorCode::InvalidRequest,
                "Los límites de memoria deben ser enteros positivos y consistentes.",
            ));
        }
        Ok(Self {
            limits,
            state: Mutex::new(State::default()),
        })
    }

    pub fn with_defaults() -> Self {
        Self::new(MemoryLimits::default()).expect("límites de memoria por defecto válidos")
    }

    pub fn get(&self, principal: &Principal, name: &str) -> Result<Option<String>, HarnessError> {
        let key = memory_key(principal, name)?;
        let state = self.state.lock().map_err(|_| internal_error())?;
        Ok(state.records.get(&key).cloned())
    }

    pub fn set(&self, principal: &Principal, name: &str, value: &str) -> Result<(), HarnessError> {
        let key = memory_key(principal, name)?;
        if !principal.has_scope("memory:write") {
            return Err(HarnessError::new(
                ErrorCode::Unauthorized,
                "Escritura no autorizada.",
            ));
        }
        if value.len() > self.limits.max_record_bytes {
            return Err(HarnessError::new(
                ErrorCode::BadInput,
                "Memoria demasiado grande.",
            ));
        }
        let key_bytes = key.accounted_bytes();
        let new_charge = key_bytes.saturating_add(value.len());
        if new_charge > self.limits.max_total_bytes {
            return Err(memory_limit_error());
        }

        let mut state = self.state.lock().map_err(|_| internal_error())?;
        let previous_charge = state
            .records
            .get(&key)
            .map(|old| key_bytes + old.len())
            .unwrap_or(0);
        if !state.records.contains_key(&key) && state.records.len() >= self.limits.max_entries {
            return Err(memory_limit_error());
        }
        let adjusted_total = state
            .accounted_bytes
            .saturating_sub(previous_charge)
            .saturating_add(new_charge);
        if adjusted_total > self.limits.max_total_bytes {
            return Err(memory_limit_error());
        }
        state.records.insert(key, value.to_owned());
        state.accounted_bytes = adjusted_total;
        Ok(())
    }

    pub fn delete(&self, principal: &Principal, name: &str) -> Result<bool, HarnessError> {
        let key = memory_key(principal, name)?;
        if !principal.has_scope("memory:write") {
            return Err(HarnessError::new(
                ErrorCode::Unauthorized,
                "Escritura no autorizada.",
            ));
        }
        let mut state = self.state.lock().map_err(|_| internal_error())?;
        if let Some(value) = state.records.remove(&key) {
            state.accounted_bytes = state
                .accounted_bytes
                .saturating_sub(key.accounted_bytes() + value.len());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn usage(&self) -> Result<MemoryUsage, HarnessError> {
        let state = self.state.lock().map_err(|_| internal_error())?;
        Ok(MemoryUsage {
            entries: state.records.len(),
            accounted_bytes: state.accounted_bytes,
        })
    }
}

fn memory_key(principal: &Principal, name: &str) -> Result<MemoryKey, HarnessError> {
    if !principal.has_scope("memory:read") {
        return Err(HarnessError::new(
            ErrorCode::Unauthorized,
            "Memoria no autorizada.",
        ));
    }
    let Some(subject_id) = principal
        .subject_id
        .as_ref()
        .filter(|subject| !subject.trim().is_empty())
    else {
        return Err(HarnessError::new(
            ErrorCode::Unauthorized,
            "La memoria personal requiere usuario autenticado.",
        ));
    };
    if name.is_empty()
        || name.len() > 40
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(HarnessError::new(ErrorCode::BadInput, "Nombre inválido."));
    }
    Ok(MemoryKey {
        tenant_id: principal.tenant_id.clone(),
        subject_id: subject_id.clone(),
        name: name.to_owned(),
    })
}

fn memory_limit_error() -> HarnessError {
    HarnessError::new(ErrorCode::BudgetExceeded, "Límite de memoria excedido.")
}

fn internal_error() -> HarnessError {
    HarnessError::new(
        ErrorCode::InternalError,
        "Fallo inesperado en la ejecución.",
    )
}

#[cfg(test)]
mod tests {
    use super::{EphemeralMemory, MemoryLimits};
    use crate::core::{AuthType, ErrorCode, Principal};

    fn principal(tenant_id: &str, scopes: &[&str]) -> Principal {
        Principal {
            tenant_id: tenant_id.to_owned(),
            subject_id: Some("user-1".to_owned()),
            auth_type: AuthType::Jwt,
            scopes: scopes.iter().map(|scope| (*scope).to_owned()).collect(),
        }
    }

    #[test]
    fn scopes_and_tenants_isolate_ephemeral_records() {
        let memory = EphemeralMemory::with_defaults();
        let owner = principal("tenant-a", &["memory:read", "memory:write"]);
        memory.set(&owner, "short", "test").unwrap();
        assert_eq!(
            memory.get(&owner, "short").unwrap().as_deref(),
            Some("test")
        );
        assert_eq!(
            memory
                .get(&principal("tenant-b", &["memory:read"]), "short")
                .unwrap(),
            None
        );
        assert_eq!(
            memory
                .get(&principal("tenant-a", &[]), "short")
                .unwrap_err()
                .code,
            ErrorCode::Unauthorized
        );
        assert_eq!(
            memory
                .set(&principal("tenant-a", &["memory:read"]), "x", "y")
                .unwrap_err()
                .code,
            ErrorCode::Unauthorized
        );
    }

    #[test]
    fn enforces_entry_and_byte_budgets_without_eager_allocation() {
        let memory = EphemeralMemory::new(MemoryLimits {
            max_entries: 1,
            max_total_bytes: 16,
            max_record_bytes: 8,
        })
        .unwrap();
        let owner = principal("t", &["memory:read", "memory:write"]);
        memory.set(&owner, "a", "one").unwrap();
        assert_eq!(memory.usage().unwrap().entries, 1);
        assert_eq!(
            memory.set(&owner, "b", "two").unwrap_err().code,
            ErrorCode::BudgetExceeded
        );
        assert_eq!(
            memory.set(&owner, "a", "123456789").unwrap_err().code,
            ErrorCode::BadInput
        );
        memory.delete(&owner, "a").unwrap();
        assert_eq!(memory.usage().unwrap().accounted_bytes, 0);
    }

    #[test]
    fn enforces_total_accounted_bytes_across_records() {
        let memory = EphemeralMemory::new(MemoryLimits {
            max_entries: 4,
            max_total_bytes: 18,
            max_record_bytes: 8,
        })
        .unwrap();
        let owner = principal("t", &["memory:read", "memory:write"]);
        memory.set(&owner, "a", "12345678").unwrap();
        assert_eq!(memory.usage().unwrap().accounted_bytes, 16);
        assert_eq!(
            memory.set(&owner, "b", "x").unwrap_err().code,
            ErrorCode::BudgetExceeded
        );
    }
}
