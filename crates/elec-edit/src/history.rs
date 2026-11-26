//! Historial de undo por snapshots inmutables (INV-05, ADR-0003).
//!
//! El comando es la unidad semántica; el snapshot es la unidad de restauración. El modelo de una
//! vivienda es pequeño, así que copiar es más barato que mantener una inversa por operación.

use elec_core::{Command, CommandError, Project};

/// Historial de estados del proyecto.
#[derive(Debug, Clone)]
pub struct History {
    snapshots: Vec<Project>,
    cursor: usize,
}

impl History {
    #[must_use]
    pub fn new(project: Project) -> Self {
        Self {
            snapshots: vec![project],
            cursor: 0,
        }
    }

    #[must_use]
    pub fn current(&self) -> &Project {
        &self.snapshots[self.cursor]
    }

    /// Aplica un comando y guarda el estado resultante.
    ///
    /// # Errors
    /// Devuelve el error del comando sin alterar el historial.
    pub fn execute(&mut self, command: &dyn Command) -> Result<(), CommandError> {
        let mut next = self.current().clone();
        command.apply(&mut next)?;
        // Cualquier cambio descarta la rama de redo.
        self.snapshots.truncate(self.cursor + 1);
        self.snapshots.push(next);
        self.cursor = self.snapshots.len() - 1;
        Ok(())
    }

    /// Deshace un paso. Devuelve `false` si ya está en el estado inicial.
    pub fn undo(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        true
    }

    /// Rehace un paso. Devuelve `false` si no hay rama de redo.
    pub fn redo(&mut self) -> bool {
        if self.cursor + 1 >= self.snapshots.len() {
            return false;
        }
        self.cursor += 1;
        true
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.snapshots.len()
    }

    /// Estados guardados (incluido el actual).
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

/// Huella determinista del proyecto, para comprobar que undo/redo restauran exactamente.
///
/// FNV-1a sobre el JSON: no es criptográfico, solo necesita distinguir estados.
#[must_use]
pub fn fingerprint(project: &Project) -> u64 {
    let json = serde_json::to_string(project).unwrap_or_default();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in json.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use elec_core::{Millimeters, PointMm, WallId};

    use super::*;
    use crate::commands::AddWall;

    fn add_wall(_index: u128) -> AddWall {
        AddWall {
            id: WallId::new(),
            start: PointMm::new(Millimeters(0), Millimeters(0)),
            end: PointMm::new(Millimeters(1000), Millimeters(0)),
            thickness: Millimeters(150),
            height: Millimeters(2700),
        }
    }

    #[test]
    fn req_io_002_hundred_undos_restore_original_state() {
        let mut history = History::new(Project::new("Casa"));
        let original = fingerprint(history.current());

        for index in 1..=100 {
            history.execute(&add_wall(index)).expect("comando válido");
        }
        assert_eq!(history.current().geometry.walls.len(), 100);
        assert_ne!(fingerprint(history.current()), original);

        for _ in 0..100 {
            assert!(history.undo());
        }
        assert_eq!(fingerprint(history.current()), original);
        assert!(!history.can_undo());

        for _ in 0..100 {
            assert!(history.redo());
        }
        assert_eq!(history.current().geometry.walls.len(), 100);
    }

    #[test]
    fn new_command_discards_redo_branch() {
        let mut history = History::new(Project::new("Casa"));
        history.execute(&add_wall(1)).expect("válido");
        history.execute(&add_wall(2)).expect("válido");
        assert!(history.undo());
        assert!(history.can_redo());

        history.execute(&add_wall(3)).expect("válido");
        assert!(!history.can_redo());
        assert_eq!(history.current().geometry.walls.len(), 2);
    }

    #[test]
    fn undo_at_start_is_a_noop() {
        let mut history = History::new(Project::new("Casa"));
        assert!(!history.undo());
        assert_eq!(history.len(), 1);
    }
}
