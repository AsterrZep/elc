//! Núcleo de dominio de elc.
//!
//! Aquí viven las unidades, la geometría entera, el modelo del proyecto, los identificadores
//! y la persistencia versionada. Este crate **no** conoce la interfaz, ni la red, ni el sistema
//! de ficheros (INV-02).

pub mod devices;
pub mod geometry;
pub mod ids;
pub mod migrate;
pub mod project;
pub mod units;

pub use devices::{Circuit, Device, DeviceKind, MountKind, SwitchKind};
pub use geometry::{Opening, OpeningKind, PointMm, Room, RoomKind, Wall, WaterKind, WaterPoint};
pub use ids::{
    BranchId, CircuitId, DeviceId, EdgeId, NetId, NodeId, OpeningId, RoomId, WallId, WaterPointId,
};
pub use project::{Command, CommandError, Geometry, Project, SchemaVersion};
pub use units::{Amps, Milliamps, Millimeters, Ohms, UnitError, Volts, Watts};
