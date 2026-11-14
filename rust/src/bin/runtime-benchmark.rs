use std::io::Write;
use std::thread;
use std::time::Instant;

use harnessos::adapters::DemoKnowledgeTool;
use harnessos::core::{AuthType, HarnessRuntime, Principal, Registry, RunRequest};
use harnessos::skills::KnowledgeRetrieveSkill;
use serde_json::json;

const ITERATIONS: usize = 10_000;
const SAMPLES: usize = 5;

fn main() {
    let mut registry = Registry::new();
    registry
        .register_tool(DemoKnowledgeTool::default())
        .expect("tool ID único");
    registry
        .register_skill(KnowledgeRetrieveSkill::default())
        .expect("skill ID único");
    let runtime = HarnessRuntime::new(registry);
    let principal = Principal {
        tenant_id: "education-demo".to_owned(),
        subject_id: Some("u1".to_owned()),
        auth_type: AuthType::Jwt,
        scopes: vec!["knowledge:read".to_owned()],
    };
    let _warmup = runtime.execute(RunRequest {
        skill_id: "knowledge.retrieve".to_owned(),
        version: "1.0.0".to_owned(),
        input: json!({ "question": "lecciones educativas" }),
        principal: principal.clone(),
        correlation_id: "warmup".to_owned(),
        requested_budget: None,
    });
    let mut failures = 0usize;
    let mut sample_ms = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            let result = runtime.execute(RunRequest {
                skill_id: "knowledge.retrieve".to_owned(),
                version: "1.0.0".to_owned(),
                input: json!({ "question": "lecciones educativas" }),
                principal: principal.clone(),
                correlation_id: "bench".to_owned(),
                requested_budget: None,
            });
            if !result.is_ok() {
                failures += 1;
            }
        }
        sample_ms.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    println!(
        "{}",
        json!({ "iterations": ITERATIONS, "samples_ms": sample_ms, "failures": failures })
    );
    let _ = std::io::stdout().flush();
    // Pause for an external process sampler to record post workload working set.
    thread::sleep(std::time::Duration::from_secs(3));
}
