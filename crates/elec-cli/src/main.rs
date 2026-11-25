//! CLI headless de elc.
//!
//! Es la puerta de la suite de aceptación: todo comportamiento del motor debe poder ejercitarse
//! aquí, sin abrir ventana (regla 5 de la constitución).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use elec_core::Project;
use elec_core::migrate::load_project;
use elec_profile::NormProfile;
use serde_json::json;

#[derive(Parser)]
#[command(
    name = "elec-cli",
    version,
    about = "Herramientas headless de elc (CAD eléctrico residencial)",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Resume un proyecto .elec.json
    Info {
        /// Ruta del proyecto
        file: PathBuf,
    },
    /// Evalúa las reglas del perfil normativo sobre el proyecto
    Validate {
        /// Ruta del proyecto
        file: PathBuf,
        /// Perfil normativo en JSON (opcional hasta M5)
        #[arg(long)]
        profile: Option<PathBuf>,
    },
    /// Simula un escenario (M3)
    Simulate {
        /// Ruta del proyecto
        file: PathBuf,
    },
    /// Genera BOM, cuadro de cargas y plano (M6)
    Report {
        /// Ruta del proyecto
        file: PathBuf,
        /// Directorio de salida
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Info { file } => {
            let project = read_project(&file)?;
            let summary = json!({
                "name": project.name,
                "schema_version": project.schema_version.0,
                "walls": project.geometry.walls.len(),
                "openings": project.geometry.openings.len(),
                "rooms": project.geometry.rooms.len(),
                "water_points": project.geometry.water_points.len(),
                "devices": project.devices.len(),
                "circuits": project.circuits.len(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?
            );
            Ok(())
        }
        Command::Validate { file, profile } => {
            let project = read_project(&file)?;
            let profile = match profile {
                Some(path) => {
                    let text = std::fs::read_to_string(&path)
                        .map_err(|err| format!("no se pudo leer {}: {err}", path.display()))?;
                    NormProfile::from_json(&text).map_err(|err| err.to_string())?
                }
                None => {
                    println!("sin --profile: 0 reglas evaluadas (el DRC llega en M5)");
                    return Ok(());
                }
            };
            let ctx = elec_rules::RuleContext {
                project: &project,
                profile: &profile,
            };
            let findings = elec_rules::evaluate(&ctx);
            println!(
                "perfil {} ({}): {} reglas implementadas, {} verificadas, {} hallazgos",
                profile.id,
                profile.edition,
                profile.total_rules(),
                profile.verified_rules(),
                findings.len()
            );
            Ok(())
        }
        Command::Simulate { file } => {
            let project = read_project(&file)?;
            // El perfil entra cuando exista data real; hasta entonces el solver no corre.
            let profile = empty_profile();
            match elec_sim::simulate(&project, &profile) {
                Ok(state) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&state).map_err(|err| err.to_string())?
                    );
                    Ok(())
                }
                Err(err) => Err(err.to_string()),
            }
        }
        Command::Report { file, out } => {
            let _ = read_project(&file)?;
            Err(format!(
                "la generación de reportes llega en M6 (salida prevista: {})",
                out.display()
            ))
        }
    }
}

fn read_project(path: &Path) -> Result<Project, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("no se pudo leer {}: {err}", path.display()))?;
    load_project(&text).map_err(|err| err.to_string())
}

/// Perfil vacío de transición: sin reglas, no participa en el DRC.
fn empty_profile() -> NormProfile {
    NormProfile {
        id: "sin-perfil".into(),
        edition: "0".into(),
        supply: elec_profile::SupplySystem::SplitPhase120_240,
        frequency_hz: 60,
        rules: Vec::new(),
        standard_heights_mm: Default::default(),
    }
}
