//! Modelo del proyecto y unidad de mutación.
//!
//! Toda escritura pasa por un `Command` y el undo guarda snapshots inmutables (INV-05/D-10).

use serde::{Deserialize, Serialize};

use crate::devices::{Circuit, Device};
use crate::geometry::{Opening, Room, Wall, WaterPoint};
use crate::migrate::CURRENT_SCHEMA_VERSION;

/// Versión del esquema del archivo (INV-09).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(pub u32);

impl Default for SchemaVersion {
    fn default() -> Self {
        Self(CURRENT_SCHEMA_VERSION)
    }
}

/// Todo lo que es geometría del plano.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Geometry {
    pub walls: Vec<Wall>,
    pub openings: Vec<Opening>,
    pub rooms: Vec<Room>,
    pub water_points: Vec<WaterPoint>,
}

/// Proyecto completo: es lo que se guarda en un `.elec.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub schema_version: SchemaVersion,
    pub name: String,
    pub geometry: Geometry,
    pub devices: Vec<Device>,
    pub circuits: Vec<Circuit>,
}

impl Project {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema_version: SchemaVersion::default(),
            name: name.into(),
            geometry: Geometry::default(),
            devices: Vec::new(),
            circuits: Vec::new(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new("Proyecto sin título")
    }
}

/// Error de aplicación de un comando.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    #[error("el elemento referenciado no existe")]
    MissingElement,
    #[error("la operación dejaría el proyecto en un estado inválido: {0}")]
    InvalidState(String),
}

/// Unidad de mutación reversible.
///
/// `apply` no se llama nunca directamente desde la UI: el motor registra el comando, toma un
/// snapshot y aplica. Así el undo es una lista de estados inmutables.
pub trait Command {
    /// Etiqueta legible para el historial de undo.
    fn label(&self) -> &str;

    /// Aplica el cambio sobre el proyecto.
    ///
    /// # Errors
    /// Devuelve [`CommandError`] si el cambio no es aplicable.
    fn apply(&self, project: &mut Project) -> Result<(), CommandError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RenameProject {
        name: String,
    }

    impl Command for RenameProject {
        fn label(&self) -> &str {
            "renombrar proyecto"
        }

        fn apply(&self, project: &mut Project) -> Result<(), CommandError> {
            project.name = self.name.clone();
            Ok(())
        }
    }

    #[test]
    fn command_mutates_project_through_trait() {
        let mut project = Project::new("original");
        RenameProject {
            name: "nuevo".into(),
        }
        .apply(&mut project)
        .expect("comando válido");
        assert_eq!(project.name, "nuevo");
    }
}
