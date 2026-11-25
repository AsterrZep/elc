//! Identificadores estables en archivo (UUID v7) y su forma tipada.
//!
//! Cada identificador es un newtype sobre `Uuid`: impide pasar un `WallId` donde se espera un
//! `RoomId` y sobrevive a guardar y reabrir el proyecto.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Genera un identificador nuevo, ordenable por tiempo.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(
    /// Muro.
    WallId
);
id_type!(
    /// Puerta o ventana.
    OpeningId
);
id_type!(
    /// Habitación o zona.
    RoomId
);
id_type!(
    /// Punto de agua (fregadero, bañera, ducha).
    WaterPointId
);
id_type!(
    /// Dispositivo eléctrico.
    DeviceId
);
id_type!(
    /// Circuito del cuadro de cargas.
    CircuitId
);
id_type!(
    /// Nodo eléctrico del netlist.
    NodeId
);
id_type!(
    /// Rama del netlist.
    BranchId
);
id_type!(
    /// Hilo lógico.
    NetId
);
id_type!(
    /// Arista de un hilo entre dos nodos.
    EdgeId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_orderable() {
        let a = WallId::new();
        let b = WallId::new();
        assert_ne!(a, b);
        assert!(a < b, "UUID v7 debe ser ordenable por tiempo");
    }
}
