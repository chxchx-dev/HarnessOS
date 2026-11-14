use std::process::Command;

#[test]
fn offline_demo_keeps_synthetic_readable_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_harnessos-demo"))
        .env("HARNESS_MODEL", "demo")
        .output()
        .expect("demo binary starts");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("demo output is UTF-8");
    assert!(stdout.contains("DATOS FICTICIOS: sin conexión con sistemas ni datos reales."));
    assert!(stdout.contains("Caso: education-demo (tenant simulado)"));
    assert!(stdout.contains("DEMO-EDU-01: Ejemplo ficticio: material educativo"));
    assert!(stdout.contains("Caso: appointments-demo (tenant simulado)"));
    assert!(stdout.contains("DEMO-APP-01: Ejemplo ficticio: agenda"));
    assert!(stdout.contains("Verificada: no; requiere evaluación"));
    assert!(!stdout.contains("Alania"));
}
