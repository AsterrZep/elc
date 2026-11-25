//! Dispositivos y circuitos del MVP.
//!
//! Protecciones y netlist llegan en M2/M4; aquí solo lo que el editor necesita colocar.

use serde::{Deserialize, Serialize};

use crate::geometry::PointMm;
use crate::ids::{CircuitId, DeviceId, RoomId};
use crate::units::{Amps, Volts, Watts};

/// Tipo de interruptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchKind {
    Simple,
    /// Par de conmutación; los dos comparten `group`.
    ThreeWay {
        group: u32,
    },
    /// Grupo de cuatro vías.
    FourWay {
        group: u32,
    },
}

/// Altura de montaje por tipo de dispositivo; las cotas viven en el perfil normativo (INV-06).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MountKind {
    Switch,
    Receptacle,
    Panel,
    Luminaire,
}

/// Naturaleza del dispositivo.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeviceKind {
    Luminaire {
        watts: Watts,
    },
    Switch {
        kind: SwitchKind,
    },
    Receptacle {
        rating: Amps,
        gfci: bool,
        dedicated: bool,
    },
    Panel {
        main_rating: Amps,
    },
}

impl DeviceKind {
    /// Clave para buscar la altura de montaje en el perfil.
    #[must_use]
    pub const fn mount_kind(&self) -> MountKind {
        match self {
            Self::Luminaire { .. } => MountKind::Luminaire,
            Self::Switch { .. } => MountKind::Switch,
            Self::Receptacle { .. } => MountKind::Receptacle,
            Self::Panel { .. } => MountKind::Panel,
        }
    }
}

/// Dispositivo colocado en el plano.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub kind: DeviceKind,
    pub position: PointMm,
    pub room: Option<RoomId>,
    pub circuit: Option<CircuitId>,
    /// Altura de montaje; si es `None` se resuelve con el perfil.
    pub mount_z: Option<crate::units::Millimeters>,
}

/// Circuito del cuadro de cargas (D-09).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    pub id: CircuitId,
    pub number: u16,
    pub description: String,
    pub voltage: Volts,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Millimeters;

    #[test]
    fn luminaire_mounts_at_ceiling_height() {
        let device = Device {
            id: DeviceId::new(),
            kind: DeviceKind::Luminaire {
                watts: Watts::new(60.0).expect("válido"),
            },
            position: PointMm::new(Millimeters(1000), Millimeters(1000)),
            room: None,
            circuit: None,
            mount_z: None,
        };
        assert_eq!(device.kind.mount_kind(), MountKind::Luminaire);
    }
}
