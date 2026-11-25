//! Persistencia versionada y migraciones (INV-09).
//!
//! Un archivo se abre siempre por [`load_project`]: nunca con `serde_json::from_str` directo
//! desde la UI. Así una versión futura se rechaza con un error claro en vez de perderse datos.

use serde::{Deserialize, Serialize};

use crate::project::Project;

/// Versión de esquema que entiende este binario.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Vista mínima para leer la versión antes de deserializar el resto.
#[derive(Debug, Deserialize)]
struct VersionProbe {
    schema_version: u32,
}

/// Error de carga de un proyecto.
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error(
        "el archivo usa el esquema {found}, más nuevo que el soportado ({CURRENT_SCHEMA_VERSION}); actualiza la app"
    )]
    TooNew { found: u32 },
    #[error("JSON inválido: {0}")]
    Json(#[from] serde_json::Error),
}

/// Carga un proyecto aplicando las migraciones necesarias.
///
/// # Errors
/// Devuelve [`MigrationError::TooNew`] si el archivo viene del futuro y
/// [`MigrationError::Json`] si no es un proyecto válido.
pub fn load_project(text: &str) -> Result<Project, MigrationError> {
    let probe: VersionProbe = serde_json::from_str(text)?;
    if probe.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(MigrationError::TooNew {
            found: probe.schema_version,
        });
    }
    // v1 es la versión base: no hay migraciones todavía. Cuando exista v2, aquí se encadena
    // `migrate_v1_to_v2` antes de deserializar el modelo final.
    let project: Project = serde_json::from_str(text)?;
    Ok(project)
}

/// Serializa de forma determinista (REQ-IO-004).
///
/// `serde_json` ordena las claves de los mapas por defecto, así que dos guardados del mismo
/// proyecto producen los mismos bytes.
pub fn save_project(project: &Project) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(project)
}

/// Resultado de comparar el esquema de un archivo con el actual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SchemaStatus {
    pub found: u32,
    pub current: u32,
}

impl SchemaStatus {
    /// `true` si el archivo necesita migración.
    #[must_use]
    pub const fn needs_migration(self) -> bool {
        self.found < self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn req_io_004_saving_twice_is_byte_identical() {
        let project = Project::new("Casa");
        let first = save_project(&project).expect("serializa");
        let second = save_project(&project).expect("serializa");
        assert_eq!(first, second);
    }

    #[test]
    fn round_trip_preserves_project() {
        let project = Project::new("Casa");
        let text = save_project(&project).expect("serializa");
        let loaded = load_project(&text).expect("carga");
        assert_eq!(project, loaded);
    }

    #[test]
    fn future_schema_is_rejected() {
        let text = r#"{"schema_version": 99, "name": "x", "geometry": {"walls": [], "openings": [], "rooms": [], "water_points": []}, "devices": [], "circuits": []}"#;
        match load_project(text) {
            Err(MigrationError::TooNew { found }) => assert_eq!(found, 99),
            other => panic!("se esperaba TooNew, se obtuvo {other:?}"),
        }
    }
}
