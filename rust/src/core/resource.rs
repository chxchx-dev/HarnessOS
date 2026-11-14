use std::mem::size_of;

use serde_json::Value;

const MAX_VALUE_DEPTH: usize = 64;
const MAP_ENTRY_OVERHEAD_BYTES: usize = 48;

/// Conservative boundary estimate for owned JSON values. This is not an RSS or allocator cap.
pub(crate) fn fits_memory_budget(value: &Value, max_bytes: usize) -> bool {
    let mut accounted = 0usize;
    visit(value, 0, max_bytes, &mut accounted)
}

fn visit(value: &Value, depth: usize, limit: usize, accounted: &mut usize) -> bool {
    if depth > MAX_VALUE_DEPTH || !charge(size_of::<Value>(), limit, accounted) {
        return false;
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => true,
        Value::String(text) => charge(text.len(), limit, accounted),
        Value::Array(items) => {
            if !charge(
                items.capacity().saturating_mul(size_of::<Value>()),
                limit,
                accounted,
            ) {
                return false;
            }
            items
                .iter()
                .all(|item| visit(item, depth + 1, limit, accounted))
        }
        Value::Object(fields) => fields.iter().all(|(key, item)| {
            charge(
                size_of::<String>()
                    .saturating_add(key.len())
                    .saturating_add(MAP_ENTRY_OVERHEAD_BYTES),
                limit,
                accounted,
            ) && visit(item, depth + 1, limit, accounted)
        }),
    }
}

fn charge(bytes: usize, limit: usize, accounted: &mut usize) -> bool {
    *accounted = accounted.saturating_add(bytes);
    *accounted <= limit
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::fits_memory_budget;

    #[test]
    fn estimates_json_memory_without_serializing_or_recursing_without_limit() {
        assert!(fits_memory_budget(&json!({ "q": "hi" }), 1_024));
        assert!(!fits_memory_budget(&json!({ "q": "x".repeat(256) }), 64));
        let deeply_nested = (0..70).fold(json!(null), |value, _| json!([value]));
        assert!(!fits_memory_budget(&deeply_nested, 1_000_000));
    }
}
