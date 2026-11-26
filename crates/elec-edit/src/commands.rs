//! Comandos de edición del MVP.
//!
//! Cada comando lleva su propio identificador para que la UI pueda seleccionar lo que acaba de
//! crear. La validación es deliberadamente mínima en M1: la coherencia geométrica se comprueba en
//! M5 con las reglas.

use elec_core::{
    Command, CommandError, Device, DeviceId, DeviceKind, Millimeters, Opening, OpeningId,
    OpeningKind, PointMm, Project, Room, RoomId, RoomKind, Wall, WallId,
};

/// Añade un muro.
#[derive(Debug, Clone)]
pub struct AddWall {
    pub id: WallId,
    pub start: PointMm,
    pub end: PointMm,
    pub thickness: Millimeters,
    pub height: Millimeters,
}

impl Command for AddWall {
    fn label(&self) -> &'static str {
        "añadir muro"
    }

    fn apply(&self, project: &mut Project) -> Result<(), CommandError> {
        if self.start == self.end {
            return Err(CommandError::InvalidState(
                "un muro no puede empezar y terminar en el mismo punto".into(),
            ));
        }
        project.geometry.walls.push(Wall {
            id: self.id,
            start: self.start,
            end: self.end,
            thickness: self.thickness,
            height: self.height,
            base_z: Millimeters::ZERO,
        });
        Ok(())
    }
}

/// Añade una habitación cerrada.
#[derive(Debug, Clone)]
pub struct AddRoom {
    pub id: RoomId,
    pub polygon: Vec<PointMm>,
    pub kind: RoomKind,
    pub name: String,
}

impl Command for AddRoom {
    fn label(&self) -> &'static str {
        "añadir habitación"
    }

    fn apply(&self, project: &mut Project) -> Result<(), CommandError> {
        if self.polygon.len() < 3 {
            return Err(CommandError::InvalidState(
                "una habitación necesita al menos tres vértices".into(),
            ));
        }
        project.geometry.rooms.push(Room {
            id: self.id,
            polygon: self.polygon.clone(),
            kind: self.kind,
            name: self.name.clone(),
        });
        Ok(())
    }
}

/// Añade un dispositivo.
#[derive(Debug, Clone)]
pub struct AddDevice {
    pub id: DeviceId,
    pub kind: DeviceKind,
    pub position: PointMm,
    pub room: Option<RoomId>,
    pub circuit: Option<elec_core::CircuitId>,
}

impl Command for AddDevice {
    fn label(&self) -> &'static str {
        "añadir dispositivo"
    }

    fn apply(&self, project: &mut Project) -> Result<(), CommandError> {
        project.devices.push(Device {
            id: self.id,
            kind: self.kind,
            position: self.position,
            room: self.room,
            circuit: self.circuit,
            mount_z: None,
        });
        Ok(())
    }
}

/// Añade una puerta o ventana a un muro existente.
#[derive(Debug, Clone)]
pub struct AddOpening {
    pub id: OpeningId,
    pub wall: WallId,
    pub kind: OpeningKind,
    pub offset: Millimeters,
    pub width: Millimeters,
    pub sill_z: Millimeters,
    pub head_z: Millimeters,
    pub blocks_conduits: bool,
}

impl Command for AddOpening {
    fn label(&self) -> &'static str {
        "añadir abertura"
    }

    fn apply(&self, project: &mut Project) -> Result<(), CommandError> {
        let wall = project
            .geometry
            .walls
            .iter()
            .find(|wall| wall.id == self.wall)
            .ok_or(CommandError::MissingElement)?;
        if self.head_z <= self.sill_z {
            return Err(CommandError::InvalidState(
                "el dintel debe estar por encima del antepecho".into(),
            ));
        }
        let end = self.offset.get() + self.width.get();
        if end > wall.length().get() {
            return Err(CommandError::InvalidState(
                "la abertura no cabe en el muro".into(),
            ));
        }
        project.geometry.openings.push(Opening {
            id: self.id,
            wall: self.wall,
            kind: self.kind,
            offset: self.offset,
            width: self.width,
            sill_z: self.sill_z,
            head_z: self.head_z,
            blocks_conduits: self.blocks_conduits,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall_command() -> AddWall {
        AddWall {
            id: WallId::new(),
            start: PointMm::new(Millimeters(0), Millimeters(0)),
            end: PointMm::new(Millimeters(3000), Millimeters(0)),
            thickness: Millimeters(150),
            height: Millimeters(2700),
        }
    }

    #[test]
    fn zero_length_wall_is_rejected() {
        let mut project = Project::new("Casa");
        let mut command = wall_command();
        command.end = command.start;
        assert!(command.apply(&mut project).is_err());
        assert!(project.geometry.walls.is_empty());
    }

    #[test]
    fn opening_must_fit_in_wall() {
        let mut project = Project::new("Casa");
        wall_command().apply(&mut project).expect("muro válido");
        let wall = project.geometry.walls[0].id;

        let too_wide = AddOpening {
            id: OpeningId::new(),
            wall,
            kind: OpeningKind::Window,
            offset: Millimeters(2500),
            width: Millimeters(900),
            sill_z: Millimeters(900),
            head_z: Millimeters(2100),
            blocks_conduits: true,
        };
        assert!(too_wide.apply(&mut project).is_err());

        let fits = AddOpening {
            id: OpeningId::new(),
            wall,
            kind: OpeningKind::Window,
            offset: Millimeters(1000),
            width: Millimeters(900),
            sill_z: Millimeters(900),
            head_z: Millimeters(2100),
            blocks_conduits: true,
        };
        fits.apply(&mut project).expect("cabe en el muro");
        assert_eq!(project.geometry.openings.len(), 1);
    }

    #[test]
    fn opening_requires_existing_wall() {
        let mut project = Project::new("Casa");
        let command = AddOpening {
            id: OpeningId::new(),
            wall: WallId::new(),
            kind: OpeningKind::Door,
            offset: Millimeters(0),
            width: Millimeters(900),
            sill_z: Millimeters(0),
            head_z: Millimeters(2100),
            blocks_conduits: true,
        };
        assert_eq!(
            command.apply(&mut project),
            Err(CommandError::MissingElement)
        );
    }

    #[test]
    fn room_needs_three_vertices() {
        let mut project = Project::new("Casa");
        let command = AddRoom {
            id: RoomId::new(),
            polygon: vec![
                PointMm::new(Millimeters(0), Millimeters(0)),
                PointMm::new(Millimeters(10), Millimeters(0)),
            ],
            kind: RoomKind::Bathroom,
            name: "Baño".into(),
        };
        assert!(command.apply(&mut project).is_err());
    }
}
