//! Tareas de desarrollo.
//!
//! `cargo xtask coverage-req` informa qué requisitos del spec tienen prueba. No sustituye a la
//! revisión: verifica que exista una prueba que declare el ID con `#[requirement("REQ-...")]`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("coverage-req") => {
            let strict = args.iter().any(|arg| arg == "--strict");
            coverage_req(strict)
        }
        _ => {
            eprintln!("uso: cargo xtask coverage-req [--strict]");
            ExitCode::FAILURE
        }
    }
}

fn coverage_req(strict: bool) -> ExitCode {
    let root = match workspace_root() {
        Ok(root) => root,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let declared = match read_declared_requirements(&root) {
        Ok(set) => set,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    if declared.is_empty() {
        eprintln!("error: no se encontró ningún REQ en el spec");
        return ExitCode::FAILURE;
    }

    let mut covered = BTreeSet::new();
    scan_dir(&root.join("crates"), &mut covered);
    scan_dir(&root.join("xtask"), &mut covered);

    let missing: Vec<&String> = declared.difference(&covered).collect();
    let percent = (declared.len() - missing.len()) as f64 / declared.len() as f64 * 100.0;

    println!(
        "cobertura de requisitos: {}/{} ({percent:.1} %)",
        declared.len() - missing.len(),
        declared.len()
    );
    if !missing.is_empty() {
        println!("\nsin prueba:");
        for id in missing.iter().take(25) {
            println!("  - {id}");
        }
        if missing.len() > 25 {
            println!("  … y {} más", missing.len() - 25);
        }
    }

    if strict && !missing.is_empty() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask no tiene directorio padre".to_string())
}

/// Lee los `REQ-*` declarados en la primera columna de las tablas del spec.
fn read_declared_requirements(root: &Path) -> Result<BTreeSet<String>, String> {
    let entries = fs::read_dir(root).map_err(|err| format!("no se pudo leer el repo: {err}"))?;
    let spec = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("SPEC-v1.0") && name.ends_with(".md"))
        })
        .ok_or_else(|| "no se encontró el archivo SPEC-v1.0".to_string())?;

    let text =
        fs::read_to_string(&spec).map_err(|err| format!("no se pudo leer el spec: {err}"))?;

    let mut declared = BTreeSet::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("| `REQ-") else {
            continue;
        };
        let Some(end) = rest.find('`') else { continue };
        let id = format!("REQ-{}", &rest[..end]);
        // La tabla de convenciones documenta `REQ-<ÁREA>-<nnn>`: no es un requisito real.
        let is_concrete = id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if is_concrete {
            declared.insert(id);
        }
    }
    Ok(declared)
}

/// Recorre un directorio buscando atributos `#[requirement("REQ-...")]`.
fn scan_dir(dir: &Path, covered: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, covered);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            if let Ok(text) = fs::read_to_string(&path) {
                extract_requirements(&text, covered);
            }
        }
    }
}

/// Extrae requisitos cubiertos por dos vías:
///
/// 1. Atributo explícito: `#[requirement("REQ-SIM-004")]`.
/// 2. Nombre de prueba: `fn req_sim_004_descripcion(...)`.
fn extract_requirements(text: &str, covered: &mut BTreeSet<String>) {
    let marker = "#[requirement(\"";
    let mut rest = text;
    while let Some(start) = rest.find(marker) {
        rest = &rest[start + marker.len()..];
        let Some(end) = rest.find("\")]") else { break };
        covered.insert(rest[..end].to_string());
        rest = &rest[end..];
    }

    for (index, _) in text.match_indices("fn req_") {
        let name = &text[index + "fn req_".len()..];
        if let Some(id) = requirement_id_from_test_name(name) {
            covered.insert(id);
        }
    }
}

/// `geo_001_area_exacta` -> `REQ-GEO-001`.
fn requirement_id_from_test_name(name: &str) -> Option<String> {
    let (area, rest) = name.split_once('_')?;
    if area.is_empty() || !area.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    Some(format!("REQ-{}-{digits}", area.to_ascii_uppercase()))
}
