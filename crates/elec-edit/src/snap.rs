//! Snap con índice espacial.
//!
//! Prioridad: extremo de elemento > restricción ortogonal > rejilla. Un snap fuerte gana a la
//! restricción de ángulo, como en cualquier CAD: si el cursor está sobre un extremo, se va al
//! extremo aunque el trazo esté en modo ortogonal.

use elec_core::{DeviceId, OpeningId, Project, RoomId, WallId};
use rstar::{AABB, PointDistance, RTree, RTreeObject};

/// Elemento del proyecto referenciable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Element {
    Wall(WallId),
    Opening(OpeningId),
    Room(RoomId),
    Device(DeviceId),
}

/// Punto candidato de snap.
#[derive(Debug, Clone, Copy)]
pub struct SnapPoint {
    pub at: [f64; 2],
    pub element: Element,
}

impl RTreeObject for SnapPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.at)
    }
}

impl PointDistance for SnapPoint {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.at[0] - point[0];
        let dy = self.at[1] - point[1];
        dx * dx + dy * dy
    }
}

/// Tipo de captura aplicada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapKind {
    Endpoint,
    Ortho,
    Grid,
}

/// Resultado del snap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapResult {
    pub point: [i64; 2],
    pub kind: SnapKind,
    pub hit: Option<Element>,
}

/// Configuración de snap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snap {
    pub grid_mm: i64,
    /// Radio de captura de extremos, en milímetros (lo resuelve la app desde píxeles y zoom).
    pub endpoint_radius_mm: f64,
    pub ortho: bool,
}

impl Default for Snap {
    fn default() -> Self {
        Self {
            grid_mm: 100,
            endpoint_radius_mm: 250.0,
            ortho: false,
        }
    }
}

/// Índice de puntos capturables del proyecto.
#[derive(Debug, Clone, Default)]
pub struct SnapEngine {
    points: RTree<SnapPoint>,
}

impl SnapEngine {
    /// Construye el índice con vértices de muros, habitaciones, puntos de agua y dispositivos.
    ///
    /// Se reconstruye al cambiar el proyecto; con un plano residencial es despreciable.
    #[must_use]
    pub fn build(project: &Project) -> Self {
        let mut points = Vec::new();

        for wall in &project.geometry.walls {
            points.push(SnapPoint {
                at: [wall.start.x.get() as f64, wall.start.y.get() as f64],
                element: Element::Wall(wall.id),
            });
            points.push(SnapPoint {
                at: [wall.end.x.get() as f64, wall.end.y.get() as f64],
                element: Element::Wall(wall.id),
            });
        }
        for room in &project.geometry.rooms {
            for vertex in &room.polygon {
                points.push(SnapPoint {
                    at: [vertex.x.get() as f64, vertex.y.get() as f64],
                    element: Element::Room(room.id),
                });
            }
        }
        for device in &project.devices {
            points.push(SnapPoint {
                at: [
                    device.position.x.get() as f64,
                    device.position.y.get() as f64,
                ],
                element: Element::Device(device.id),
            });
        }
        Self {
            points: RTree::bulk_load(points),
        }
    }

    /// Resuelve el snap para un punto del mundo.
    ///
    /// `anchor` es el punto de referencia del trazo (primer clic de un muro), necesario para la
    /// restricción ortogonal.
    #[must_use]
    pub fn resolve(&self, world: [i64; 2], anchor: Option<[i64; 2]>, cfg: &Snap) -> SnapResult {
        let probe = [world[0] as f64, world[1] as f64];
        let radius_sq = cfg.endpoint_radius_mm * cfg.endpoint_radius_mm;

        if cfg.endpoint_radius_mm > 0.0
            && let Some(candidate) = self.points.nearest_neighbor(&probe)
        {
            let dx = candidate.at[0] - probe[0];
            let dy = candidate.at[1] - probe[1];
            if dx * dx + dy * dy <= radius_sq {
                return SnapResult {
                    point: [
                        candidate.at[0].round() as i64,
                        candidate.at[1].round() as i64,
                    ],
                    kind: SnapKind::Endpoint,
                    hit: Some(candidate.element),
                };
            }
        }

        if cfg.ortho
            && let Some(anchor) = anchor
        {
            let dx = world[0] - anchor[0];
            let dy = world[1] - anchor[1];
            let (x, y) = if dx.abs() >= dy.abs() {
                (world[0], anchor[1])
            } else {
                (anchor[0], world[1])
            };
            return SnapResult {
                point: [x, y],
                kind: SnapKind::Ortho,
                hit: None,
            };
        }

        let grid = cfg.grid_mm.max(1);
        SnapResult {
            point: [round_to_grid(world[0], grid), round_to_grid(world[1], grid)],
            kind: SnapKind::Grid,
            hit: None,
        }
    }

    /// Número de puntos capturables indexados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.size()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.size() == 0
    }
}

fn round_to_grid(value: i64, grid: i64) -> i64 {
    let half = grid / 2;
    let shifted = value.saturating_add(if value >= 0 { half } else { -half });
    shifted / grid * grid
}

#[cfg(test)]
mod tests {
    use elec_core::{
        Amps, Device, DeviceKind, Millimeters, PointMm, Project, Room, RoomKind, Wall,
    };

    use super::*;

    fn wall_project() -> Project {
        let mut project = Project::new("Prueba");
        project.geometry.walls.push(Wall {
            id: WallId::new(),
            start: PointMm::new(Millimeters(0), Millimeters(0)),
            end: PointMm::new(Millimeters(5000), Millimeters(0)),
            thickness: Millimeters(150),
            height: Millimeters(2700),
            base_z: Millimeters(0),
        });
        project
    }

    #[test]
    fn req_ux_009_snap_uses_index_and_finds_endpoint() {
        let engine = SnapEngine::build(&wall_project());
        assert_eq!(engine.len(), 2);

        let cfg = Snap {
            endpoint_radius_mm: 300.0,
            grid_mm: 100,
            ortho: false,
        };
        // A 120 mm del extremo final (5000, 0): gana el extremo, no la rejilla.
        let result = engine.resolve([4880, 70], None, &cfg);
        assert_eq!(result.kind, SnapKind::Endpoint);
        assert_eq!(result.point, [5000, 0]);
    }

    #[test]
    fn grid_rounds_to_nearest_step() {
        let engine = SnapEngine::build(&Project::new("vacío"));
        let cfg = Snap::default();
        let result = engine.resolve([1249, -260], None, &cfg);
        assert_eq!(result.kind, SnapKind::Grid);
        assert_eq!(result.point, [1200, -300]);
    }

    #[test]
    fn ortho_aligns_with_the_dominant_axis() {
        let engine = SnapEngine::build(&Project::new("vacío"));
        let cfg = Snap {
            endpoint_radius_mm: 0.0,
            grid_mm: 10,
            ortho: true,
        };
        let result = engine.resolve([3210, 1180], Some([3000, 1000]), &cfg);
        assert_eq!(result.kind, SnapKind::Ortho);
        assert_eq!(result.point, [3210, 1000]);
    }

    #[test]
    fn endpoint_beats_ortho() {
        let engine = SnapEngine::build(&wall_project());
        let cfg = Snap {
            endpoint_radius_mm: 300.0,
            grid_mm: 100,
            ortho: true,
        };
        let result = engine.resolve([4950, 60], Some([0, 0]), &cfg);
        assert_eq!(result.kind, SnapKind::Endpoint);
    }

    #[test]
    fn devices_are_snappable() {
        let mut project = Project::new("Prueba");
        project.devices.push(Device {
            id: DeviceId::new(),
            kind: DeviceKind::Receptacle {
                rating: Amps::new(15.0).expect("válido"),
                gfci: false,
                dedicated: false,
            },
            position: PointMm::new(Millimeters(1000), Millimeters(2000)),
            room: None,
            circuit: None,
            mount_z: None,
        });
        let engine = SnapEngine::build(&project);
        let cfg = Snap {
            endpoint_radius_mm: 200.0,
            grid_mm: 10,
            ortho: false,
        };
        let result = engine.resolve([1080, 2050], None, &cfg);
        assert_eq!(result.kind, SnapKind::Endpoint);
        assert!(matches!(result.hit, Some(Element::Device(_))));
    }

    #[test]
    fn rooms_contribute_their_vertices() {
        let mut project = Project::new("Prueba");
        project.geometry.rooms.push(Room::rectangle(
            RoomKind::Bathroom,
            "Baño",
            Millimeters(2000),
            Millimeters(2000),
        ));
        let engine = SnapEngine::build(&project);
        assert_eq!(engine.len(), 4);
    }
}
