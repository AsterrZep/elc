//! Sesión de edición sin interfaz.
//!
//! Aquí vive todo lo que el editor hace y que se puede probar sin abrir ventana: transformación
//! de cámara, snap, índice espacial, comandos e historial de undo. `elec-app` es solo un cliente
//! que dibuja y traduce eventos (ADR-0006).

pub mod camera;
pub mod commands;
pub mod history;
pub mod session;
pub mod snap;

pub use camera::{Camera, View};
pub use commands::{AddDevice, AddOpening, AddRoom, AddWall};
pub use history::{History, fingerprint};
pub use session::Session;
pub use snap::{Element, Snap, SnapResult};
