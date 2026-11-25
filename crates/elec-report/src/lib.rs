//! Reportes: BOM, cuadro de cargas y plano.
//!
//! Las estructuras son el contrato de datos compartido por CSV, PDF y SVG: una sola fuente,
//! varias rutas de salida (ADR-0004). La construcción real llega en M6.

use elec_core::Project;
use elec_profile::NormProfile;
use serde::{Deserialize, Serialize};

/// Línea del listado de materiales.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BomLine {
    pub item: String,
    pub unit: String,
    pub quantity: f64,
    /// Función del conductor (`phase`, `neutral`, `pe`, `return`, `traveler`), si aplica.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    /// Calibre o diámetro, si aplica.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

/// Listado de materiales.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Bom {
    pub lines: Vec<BomLine>,
}

/// Fila del cuadro de cargas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadRow {
    pub circuit: String,
    pub description: String,
    pub connected_loads: String,
    pub power_va: f64,
    pub current_a: f64,
    pub voltage_v: f64,
    pub protection: String,
}

/// Cuadro de cargas.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LoadSchedule {
    pub rows: Vec<LoadRow>,
}

/// Construye el BOM del proyecto.
///
/// Placeholder de M0: devuelve un BOM vacío hasta M6, cuando exista netlist y metrado.
#[must_use]
pub fn bom(_project: &Project, _profile: &NormProfile) -> Bom {
    Bom::default()
}

/// Construye el cuadro de cargas definido por el usuario (D-09).
///
/// Placeholder de M0: sin circuitos, no hay filas.
#[must_use]
pub fn load_schedule(project: &Project) -> LoadSchedule {
    LoadSchedule {
        rows: project
            .circuits
            .iter()
            .map(|circuit| LoadRow {
                circuit: circuit.number.to_string(),
                description: circuit.description.clone(),
                connected_loads: String::new(),
                power_va: 0.0,
                current_a: 0.0,
                voltage_v: circuit.voltage.get(),
                protection: String::new(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_schedule_projects_user_circuits() {
        let mut project = Project::new("Casa");
        project.circuits.push(elec_core::Circuit {
            id: elec_core::CircuitId::new(),
            number: 1,
            description: "Iluminación".into(),
            voltage: elec_core::Volts::new(120.0).expect("válido"),
        });
        let schedule = load_schedule(&project);
        assert_eq!(schedule.rows.len(), 1);
        assert_eq!(schedule.rows[0].circuit, "1");
        assert!((schedule.rows[0].voltage_v - 120.0).abs() < f64::EPSILON);
    }
}
