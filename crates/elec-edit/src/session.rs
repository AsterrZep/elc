//! Sesión de edición: el estado que la UI dibuja y modifica.
//!
//! La UI no toca el `Project` directamente: llama a `execute`, `undo` o `redo`, y reconstruye el
//! índice espacial cuando cambia el documento.

use elec_core::{Command, CommandError, PointMm, Project};

use crate::camera::Camera;
use crate::history::{History, fingerprint};
use crate::snap::{Element, Snap, SnapEngine, SnapResult};

/// Distancia máxima para seleccionar un elemento, en milímetros.
pub const HIT_RADIUS_MM: i64 = 120;

/// Estado completo del editor.
#[derive(Debug, Clone)]
pub struct Session {
    pub history: History,
    pub camera: Camera,
    pub selection: Option<Element>,
    pub grid_mm: i64,
    pub ortho: bool,
    snap: SnapEngine,
}

impl Session {
    #[must_use]
    pub fn new(project: Project) -> Self {
        let snap = SnapEngine::build(&project);
        Self {
            history: History::new(project),
            camera: Camera::default(),
            selection: None,
            grid_mm: 100,
            ortho: false,
            snap,
        }
    }

    #[must_use]
    pub fn project(&self) -> &Project {
        self.history.current()
    }

    /// Aplica un comando y actualiza el índice espacial.
    ///
    /// # Errors
    /// Propaga el error del comando sin modificar el estado.
    pub fn execute(&mut self, command: &dyn Command) -> Result<(), CommandError> {
        self.history.execute(command)?;
        self.reindex();
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        let changed = self.history.undo();
        if changed {
            self.reindex();
            self.selection = None;
        }
        changed
    }

    pub fn redo(&mut self) -> bool {
        let changed = self.history.redo();
        if changed {
            self.reindex();
            self.selection = None;
        }
        changed
    }

    fn reindex(&mut self) {
        self.snap = SnapEngine::build(self.history.current());
    }

    /// Configuración de snap derivada del zoom: un radio fijo en píxeles se traduce a mm.
    #[must_use]
    pub fn snap_config(&self, mm_per_px: f64) -> Snap {
        Snap {
            grid_mm: self.grid_mm,
            endpoint_radius_mm: 12.0 * mm_per_px,
            ortho: self.ortho,
        }
    }

    #[must_use]
    pub fn resolve_snap(
        &self,
        world: [i64; 2],
        anchor: Option<[i64; 2]>,
        mm_per_px: f64,
    ) -> SnapResult {
        self.snap
            .resolve(world, anchor, &self.snap_config(mm_per_px))
    }

    /// Habitación que contiene el punto, si hay alguna.
    #[must_use]
    pub fn room_at(&self, world: [i64; 2]) -> Option<elec_core::RoomId> {
        self.project()
            .geometry
            .rooms
            .iter()
            .find(|room| point_in_polygon(world, &room.polygon))
            .map(|room| room.id)
    }

    /// Busca el elemento bajo el cursor, con prioridad dispositivo > muro > habitación.
    #[must_use]
    pub fn hit_test(&self, world: [i64; 2], radius_mm: i64) -> Option<Element> {
        let project = self.project();
        let radius_sq = (radius_mm * radius_mm) as f64;

        for device in &project.devices {
            let dx = (device.position.x.get() - world[0]) as f64;
            let dy = (device.position.y.get() - world[1]) as f64;
            if dx * dx + dy * dy <= radius_sq {
                return Some(Element::Device(device.id));
            }
        }

        for wall in &project.geometry.walls {
            let allowed = radius_mm + wall.thickness.get() / 2;
            if distance_sq_to_segment(world, wall.start, wall.end) <= (allowed * allowed) as f64 {
                return Some(Element::Wall(wall.id));
            }
        }

        for room in &project.geometry.rooms {
            if point_in_polygon(world, &room.polygon) {
                return Some(Element::Room(room.id));
            }
        }

        None
    }

    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        fingerprint(self.project())
    }

    /// Rectángulo que contiene todo el proyecto, para encuadrar la vista.
    #[must_use]
    pub fn bounds(&self) -> Option<([i64; 2], [i64; 2])> {
        let project = self.project();
        let mut min = [i64::MAX, i64::MAX];
        let mut max = [i64::MIN, i64::MIN];
        let mut any = false;

        let mut include = |point: [i64; 2]| {
            min[0] = min[0].min(point[0]);
            min[1] = min[1].min(point[1]);
            max[0] = max[0].max(point[0]);
            max[1] = max[1].max(point[1]);
        };

        for wall in &project.geometry.walls {
            include([wall.start.x.get(), wall.start.y.get()]);
            include([wall.end.x.get(), wall.end.y.get()]);
            any = true;
        }
        for room in &project.geometry.rooms {
            for vertex in &room.polygon {
                include([vertex.x.get(), vertex.y.get()]);
                any = true;
            }
        }
        for device in &project.devices {
            include([device.position.x.get(), device.position.y.get()]);
            any = true;
        }

        any.then_some((min, max))
    }

    /// Nombre legible del elemento seleccionado.
    #[must_use]
    pub fn selection_label(&self) -> Option<&'static str> {
        self.selection.map(|element| match element {
            Element::Wall(_) => "muro",
            Element::Opening(_) => "abertura",
            Element::Room(_) => "habitación",
            Element::Device(_) => "dispositivo",
        })
    }
}

fn distance_sq_to_segment(point: [i64; 2], a: PointMm, b: PointMm) -> f64 {
    let px = point[0] as f64;
    let py = point[1] as f64;
    let ax = a.x.get() as f64;
    let ay = a.y.get() as f64;
    let bx = b.x.get() as f64;
    let by = b.y.get() as f64;

    let dx = bx - ax;
    let dy = by - ay;
    let length_sq = dx * dx + dy * dy;
    if length_sq == 0.0 {
        let ex = px - ax;
        let ey = py - ay;
        return ex * ex + ey * ey;
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / length_sq).clamp(0.0, 1.0);
    let ex = px - (ax + t * dx);
    let ey = py - (ay + t * dy);
    ex * ex + ey * ey
}

fn point_in_polygon(point: [i64; 2], polygon: &[PointMm]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let px = point[0] as f64;
    let py = point[1] as f64;
    let mut inside = false;
    let n = polygon.len();
    for index in 0..n {
        let a = polygon[index];
        let b = polygon[(index + 1) % n];
        let (xi, yi) = (a.x.get() as f64, a.y.get() as f64);
        let (xj, yj) = (b.x.get() as f64, b.y.get() as f64);
        let crosses = (yi > py) != (yj > py);
        if crosses {
            let x_at_y = (xj - xi) * (py - yi) / (yj - yi) + xi;
            if px < x_at_y {
                inside = !inside;
            }
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use elec_core::{Amps, DeviceId, DeviceKind, Millimeters, RoomKind, WallId};

    use super::*;
    use crate::commands::{AddDevice, AddRoom, AddWall};

    fn sample_session() -> Session {
        let mut session = Session::new(Project::new("Casa"));
        session
            .execute(&AddWall {
                id: WallId::new(),
                start: PointMm::new(Millimeters(0), Millimeters(0)),
                end: PointMm::new(Millimeters(5000), Millimeters(0)),
                thickness: Millimeters(150),
                height: Millimeters(2700),
            })
            .expect("muro válido");
        session
            .execute(&AddRoom {
                id: elec_core::RoomId::new(),
                polygon: vec![
                    PointMm::new(Millimeters(0), Millimeters(0)),
                    PointMm::new(Millimeters(4000), Millimeters(0)),
                    PointMm::new(Millimeters(4000), Millimeters(3000)),
                    PointMm::new(Millimeters(0), Millimeters(3000)),
                ],
                kind: RoomKind::Bathroom,
                name: "Baño".into(),
            })
            .expect("habitación válida");
        session
            .execute(&AddDevice {
                id: DeviceId::new(),
                kind: DeviceKind::Receptacle {
                    rating: Amps::new(20.0).expect("válido"),
                    gfci: true,
                    dedicated: false,
                },
                position: PointMm::new(Millimeters(1000), Millimeters(1500)),
                room: None,
                circuit: None,
            })
            .expect("dispositivo válido");
        session
    }

    #[test]
    fn req_ux_004_adding_walls_is_fast_enough_for_interaction() {
        let mut session = Session::new(Project::new("Casa"));
        let start = std::time::Instant::now();
        for index in 0..100 {
            session
                .execute(&AddWall {
                    id: WallId::new(),
                    start: PointMm::new(Millimeters(index), Millimeters(0)),
                    end: PointMm::new(Millimeters(index), Millimeters(3000)),
                    thickness: Millimeters(150),
                    height: Millimeters(2700),
                })
                .expect("muro válido");
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 300,
            "100 muros tardaron {elapsed:?}; el presupuesto interactivo es 3 s y el objetivo 300 ms"
        );
    }

    #[test]
    fn hit_test_prefers_device_then_wall_then_room() {
        let session = sample_session();
        assert!(matches!(
            session.hit_test([1000, 1500], HIT_RADIUS_MM),
            Some(Element::Device(_))
        ));
        assert!(matches!(
            session.hit_test([2500, 40], HIT_RADIUS_MM),
            Some(Element::Wall(_))
        ));
        assert!(matches!(
            session.hit_test([3500, 2800], HIT_RADIUS_MM),
            Some(Element::Room(_))
        ));
        assert!(session.hit_test([9000, 9000], HIT_RADIUS_MM).is_none());
    }

    #[test]
    fn undo_clears_selection_and_reindexes() {
        let mut session = sample_session();
        session.selection = Some(Element::Wall(WallId::new()));
        let before = session.fingerprint();
        assert!(session.undo());
        assert!(session.selection.is_none());
        assert_ne!(session.fingerprint(), before);
        assert!(session.redo());
    }

    #[test]
    fn bounds_cover_all_elements() {
        let session = sample_session();
        let (min, max) = session.bounds().expect("hay geometría");
        assert_eq!(min, [0, 0]);
        assert_eq!(max, [5000, 3000]);
    }

    #[test]
    fn snap_radius_scales_with_zoom() {
        let session = Session::new(Project::new("Casa"));
        assert_eq!(session.snap_config(1.0).endpoint_radius_mm, 12.0);
        assert_eq!(session.snap_config(10.0).endpoint_radius_mm, 120.0);
    }

    #[test]
    fn wall_hit_uses_segment_not_endpoints() {
        let session = sample_session();
        // Punto medio del muro: no hay vértice cerca, pero debe seleccionarse.
        assert!(matches!(
            session.hit_test([2500, 0], HIT_RADIUS_MM),
            Some(Element::Wall(_))
        ));
    }

    #[test]
    fn room_at_finds_the_containing_room() {
        let session = sample_session();
        assert!(session.room_at([3500, 2800]).is_some());
        assert!(session.room_at([9000, 9000]).is_none());
    }

    #[test]
    fn empty_project_has_no_bounds() {
        let session = Session::new(Project::new("vacío"));
        assert!(session.bounds().is_none());
        assert!(session.selection_label().is_none());
    }
}
