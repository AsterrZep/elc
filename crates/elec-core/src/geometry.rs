//! Geometría del plano en milímetros enteros (INV-04).
//!
//! Las habitaciones se dibujan explícitamente: no se detectan desde muros (D-05). El área se
//! calcula con aritmética entera, sin tolerancias.

use serde::{Deserialize, Serialize};

use crate::ids::{OpeningId, RoomId, WallId, WaterPointId};
use crate::units::Millimeters;

/// Punto en el plano, en milímetros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PointMm {
    pub x: Millimeters,
    pub y: Millimeters,
}

impl PointMm {
    #[must_use]
    pub const fn new(x: Millimeters, y: Millimeters) -> Self {
        Self { x, y }
    }
}

/// Muro con espesor y altura reales.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wall {
    pub id: WallId,
    pub start: PointMm,
    pub end: PointMm,
    pub thickness: Millimeters,
    pub height: Millimeters,
    pub base_z: Millimeters,
}

impl Wall {
    /// Longitud del eje del muro.
    #[must_use]
    pub fn length(&self) -> Millimeters {
        let dx = i128::from(self.end.x.get() - self.start.x.get());
        let dy = i128::from(self.end.y.get() - self.start.y.get());
        // Raíz entera: evita decimales en la topología.
        Millimeters(integer_sqrt(dx * dx + dy * dy) as i64)
    }
}

/// Tipo de abertura.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpeningKind {
    Door,
    Window,
}

/// Puerta o ventana dentro de un muro.
///
/// Ocupa un tramo del muro y define una zona restringida para canalizaciones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opening {
    pub id: OpeningId,
    pub wall: WallId,
    pub kind: OpeningKind,
    /// Distancia desde el inicio del muro hasta el borde de la abertura.
    pub offset: Millimeters,
    pub width: Millimeters,
    /// Cota del antepecho (0 en puertas).
    pub sill_z: Millimeters,
    /// Cota del dintel.
    pub head_z: Millimeters,
    /// Si bloquea el paso de conductos (REQ-DRC-008).
    pub blocks_conduits: bool,
}

/// Tipo de ambiente: clave de las reglas por zona (GFCI, cargas mínimas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomKind {
    Bathroom,
    Kitchen,
    Living,
    Bedroom,
    Laundry,
    Garage,
    Outdoor,
    Crawlspace,
    UnfinishedBasement,
    Other,
}

/// Habitación o zona, dibujada explícitamente como polígono cerrado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub polygon: Vec<PointMm>,
    pub kind: RoomKind,
    pub name: String,
}

impl Room {
    /// Área exacta del polígono en mm², usando la fórmula del cordón con enteros.
    #[must_use]
    pub fn area_mm2(&self) -> i128 {
        let n = self.polygon.len();
        if n < 3 {
            return 0;
        }
        let mut acc: i128 = 0;
        for i in 0..n {
            let a = self.polygon[i];
            let b = self.polygon[(i + 1) % n];
            acc += i128::from(a.x.get()) * i128::from(b.y.get())
                - i128::from(b.x.get()) * i128::from(a.y.get());
        }
        (acc / 2).abs()
    }

    /// Área en m², solo para presentación.
    #[must_use]
    pub fn area_m2(&self) -> f64 {
        self.area_mm2() as f64 / 1_000_000.0
    }

    /// Rectángulo cerrado, útil para pruebas y plantillas.
    #[must_use]
    pub fn rectangle(
        kind: RoomKind,
        name: impl Into<String>,
        width: Millimeters,
        height: Millimeters,
    ) -> Self {
        let w = width.get();
        let h = height.get();
        Self {
            id: RoomId::new(),
            polygon: vec![
                PointMm::new(Millimeters(0), Millimeters(0)),
                PointMm::new(Millimeters(w), Millimeters(0)),
                PointMm::new(Millimeters(w), Millimeters(h)),
                PointMm::new(Millimeters(0), Millimeters(h)),
            ],
            kind,
            name: name.into(),
        }
    }
}

/// Tipo de punto de agua.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterKind {
    Sink,
    Tub,
    Shower,
}

/// Punto de agua con posición.
///
/// Un booleano por habitación no permite medir la distancia de 6 ft que exige la regla GFCI
/// (REQ-DRC-002, REQ-GEO-007).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaterPoint {
    pub id: WaterPointId,
    pub kind: WaterKind,
    pub position: PointMm,
}

/// Raíz cuadrada entera por el método de Newton.
fn integer_sqrt(value: i128) -> i128 {
    if value < 2 {
        return value.max(0);
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn req_geo_006_room_area_is_exact() {
        let room = Room::rectangle(
            RoomKind::Bedroom,
            "Dormitorio",
            Millimeters(4000),
            Millimeters(3000),
        );
        assert_eq!(room.area_mm2(), 12_000_000);
        assert!((room.area_m2() - 12.0).abs() < f64::EPSILON);
    }

    #[test]
    fn degenerate_polygon_has_zero_area() {
        let room = Room {
            id: RoomId::new(),
            polygon: vec![PointMm::default(), PointMm::default()],
            kind: RoomKind::Other,
            name: "vacío".into(),
        };
        assert_eq!(room.area_mm2(), 0);
    }

    #[test]
    fn wall_length_is_integer() {
        let wall = Wall {
            id: WallId::new(),
            start: PointMm::new(Millimeters(0), Millimeters(0)),
            end: PointMm::new(Millimeters(3000), Millimeters(4000)),
            thickness: Millimeters(150),
            height: Millimeters(2700),
            base_z: Millimeters(0),
        };
        assert_eq!(wall.length(), Millimeters(5000));
    }
}
