use std::env;
use std::process::ExitCode;
use std::sync::Arc;

use harnessos::bootstrap::create_demo_harness;
use harnessos::core::{AuthType, Principal, RunRequest};
use harnessos::models::{DemoModel, OllamaConfig, OllamaModel};
use serde_json::{Value, json};

fn main() -> ExitCode {
    let model: Arc<dyn harnessos::core::ModelPort> = if env::var("HARNESS_MODEL").as_deref()
        == Ok("ollama")
    {
        let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3:4b-instruct".to_owned());
        let url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_owned());
        match OllamaModel::new(OllamaConfig { url, model }) {
            Ok(model) => Arc::new(model),
            Err(message) => {
                eprintln!("{message}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        Arc::new(DemoModel)
    };
    let runtime = match create_demo_harness(model) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    println!("HarnessOS · demostración educativa");
    println!("DATOS FICTICIOS: sin conexión con sistemas ni datos reales.\n");
    println!("Capacidades disponibles:");
    for skill in runtime.registry().list_skills() {
        println!("- {}@{}", skill.id, skill.version);
    }

    let cases = [
        ("education-demo", "¿Qué incluyen las lecciones educativas?"),
        ("appointments-demo", "¿Qué ofrece la agenda de prueba?"),
    ];
    let mut failed = false;
    for (tenant_id, question) in cases {
        let result = runtime.execute(RunRequest {
            skill_id: "knowledge.answer".to_owned(),
            version: "1.0.0".to_owned(),
            input: json!({ "question": question }),
            principal: Principal {
                tenant_id: tenant_id.to_owned(),
                subject_id: Some("local-test-user".to_owned()),
                auth_type: AuthType::Jwt,
                scopes: vec!["knowledge:read".to_owned(), "model:use".to_owned()],
            },
            correlation_id: format!("local-demo-{tenant_id}"),
            requested_budget: None,
        });
        println!("\nCaso: {tenant_id} (tenant simulado)");
        println!("Pregunta: {question}");
        match result {
            harnessos::core::RunResult::Failure { error, .. } => {
                println!("Resultado: {} — {}", error.code.as_str(), error.message);
                failed = true;
            }
            harnessos::core::RunResult::Success { output, trace } => {
                println!("Respuesta: {}", json_string(&output, "answer"));
                println!("Fuentes ficticias:");
                if let Some(citations) = output.get("citations").and_then(Value::as_array) {
                    for citation in citations {
                        println!(
                            "- {}: {}",
                            json_string(citation, "id"),
                            json_string(citation, "title")
                        );
                    }
                }
                println!("Modelo: {}", json_string(&output, "modelUsed"));
                println!(
                    "Verificada: {}",
                    if output.get("verified").and_then(Value::as_bool) == Some(true) {
                        "sí"
                    } else {
                        "no; requiere evaluación"
                    }
                );
                println!(
                    "Traza: {} herramienta(s), {} ms",
                    trace.tool_call_count, trace.duration_ms
                );
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn json_string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}
