//! Cámara 2D: transformación entre milímetros del modelo y píxeles de pantalla.
//!
//! La geometría sigue siendo entera; la cámara es la única que usa decimales, y solo para
//! presentación. `screen_to_world` redondea a milímetros.

/// Dimensiones del área de dibujo en píxeles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub width: f32,
    pub height: f32,
}

impl View {
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    fn center(self) -> [f32; 2] {
        [self.width / 2.0, self.height / 2.0]
    }
}

/// Cámara del lienzo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Punto del mundo que queda en el centro de la vista, en mm.
    pub center_mm: [f64; 2],
    /// Escala: milímetros por píxel.
    pub mm_per_px: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center_mm: [0.0, 0.0],
            // 1 px = 20 mm: una vivienda de 10 m ocupa ~500 px.
            mm_per_px: 20.0,
        }
    }
}

impl Camera {
    /// Escala mínima y máxima para no perder el control del zoom.
    pub const MIN_MM_PER_PX: f64 = 0.2;
    pub const MAX_MM_PER_PX: f64 = 200.0;

    #[must_use]
    pub fn new(center_mm: [f64; 2], mm_per_px: f64) -> Self {
        Self {
            center_mm,
            mm_per_px: mm_per_px.clamp(Self::MIN_MM_PER_PX, Self::MAX_MM_PER_PX),
        }
    }

    /// Convierte un punto del mundo (mm) a coordenadas de pantalla (px).
    #[must_use]
    pub fn world_to_screen(&self, world: [f64; 2], view: View) -> [f32; 2] {
        let [cx, cy] = view.center();
        [
            (cx as f64 + (world[0] - self.center_mm[0]) / self.mm_per_px) as f32,
            // Y del mundo crece hacia arriba; la pantalla crece hacia abajo.
            (cy as f64 - (world[1] - self.center_mm[1]) / self.mm_per_px) as f32,
        ]
    }

    /// Convierte píxeles de pantalla a milímetros del mundo.
    #[must_use]
    pub fn screen_to_world(&self, screen: [f32; 2], view: View) -> [i64; 2] {
        let [cx, cy] = view.center();
        let x = self.center_mm[0] + (screen[0] as f64 - cx as f64) * self.mm_per_px;
        let y = self.center_mm[1] - (screen[1] as f64 - cy as f64) * self.mm_per_px;
        [x.round() as i64, y.round() as i64]
    }

    /// Desplaza la vista por un delta en píxeles.
    pub fn pan_px(&mut self, delta: [f32; 2]) {
        self.center_mm[0] -= f64::from(delta[0]) * self.mm_per_px;
        self.center_mm[1] += f64::from(delta[1]) * self.mm_per_px;
    }

    /// Hace zoom manteniendo fijo el punto del mundo que está bajo el cursor.
    pub fn zoom_at(&mut self, factor: f64, cursor: [f32; 2], view: View) {
        let anchor = self.screen_to_world_f64(cursor, view);
        let new_scale = (self.mm_per_px / factor).clamp(Self::MIN_MM_PER_PX, Self::MAX_MM_PER_PX);
        if (new_scale - self.mm_per_px).abs() < f64::EPSILON {
            return;
        }
        self.mm_per_px = new_scale;
        // Recoloca el centro para que `anchor` siga bajo el cursor.
        let [cx, cy] = view.center();
        let dx_px = f64::from(cursor[0]) - f64::from(cx);
        let dy_px = f64::from(cursor[1]) - f64::from(cy);
        self.center_mm = [
            anchor[0] - dx_px * self.mm_per_px,
            anchor[1] + dy_px * self.mm_per_px,
        ];
    }

    fn screen_to_world_f64(&self, screen: [f32; 2], view: View) -> [f64; 2] {
        let [cx, cy] = view.center();
        [
            self.center_mm[0] + (f64::from(screen[0]) - f64::from(cx)) * self.mm_per_px,
            self.center_mm[1] - (f64::from(screen[1]) - f64::from(cy)) * self.mm_per_px,
        ]
    }

    /// Ajusta la vista para mostrar un rectángulo del mundo con margen.
    pub fn fit(&mut self, min: [i64; 2], max: [i64; 2], view: View) {
        let width_mm = (max[0] - min[0]).max(1) as f64;
        let height_mm = (max[1] - min[1]).max(1) as f64;
        let scale_x = width_mm / f64::from(view.width.max(1.0)) * 1.15;
        let scale_y = height_mm / f64::from(view.height.max(1.0)) * 1.15;
        self.mm_per_px = scale_x
            .max(scale_y)
            .clamp(Self::MIN_MM_PER_PX, Self::MAX_MM_PER_PX);
        self.center_mm = [
            (min[0] as f64 + max[0] as f64) / 2.0,
            (min[1] as f64 + max[1] as f64) / 2.0,
        ];
    }

    /// Calibra el plano de fondo con dos puntos de pantalla y una cota real conocida
    /// (REQ-GEO-005).
    ///
    /// Devuelve la escala resultante en mm/px.
    pub fn calibrate(&mut self, a: [f32; 2], b: [f32; 2], real_mm: f64) -> f64 {
        let dx = f64::from(b[0]) - f64::from(a[0]);
        let dy = f64::from(b[1]) - f64::from(a[1]);
        let pixel_distance = (dx * dx + dy * dy).sqrt();
        if pixel_distance < f64::EPSILON || real_mm <= 0.0 {
            return self.mm_per_px;
        }
        self.mm_per_px = (real_mm / pixel_distance).clamp(Self::MIN_MM_PER_PX, Self::MAX_MM_PER_PX);
        self.mm_per_px
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: View = View::new(1000.0, 800.0);

    #[test]
    fn req_geo_005_calibration_maps_real_distance_within_one_mm() {
        let mut camera = Camera::default();
        // 1000 px equivalen a 5000 mm (una cota de 5 m).
        let scale = camera.calibrate([100.0, 100.0], [1100.0, 100.0], 5000.0);
        assert!((scale - 5.0).abs() < 1e-9);

        let a = camera.screen_to_world([100.0, 100.0], VIEW);
        let b = camera.screen_to_world([1100.0, 100.0], VIEW);
        let measured = (b[0] - a[0]).abs() as f64;
        assert!((measured - 5000.0).abs() <= 1.0);
    }

    #[test]
    fn zoom_keeps_point_under_cursor_stable() {
        let mut camera = Camera::default();
        let cursor = [321.0, 240.0];
        let before = camera.screen_to_world(cursor, VIEW);

        camera.zoom_at(1.5, cursor, VIEW);

        let after = camera.screen_to_world(cursor, VIEW);
        assert!((after[0] - before[0]).abs() <= 1, "{after:?} vs {before:?}");
        assert!((after[1] - before[1]).abs() <= 1, "{after:?} vs {before:?}");
    }

    #[test]
    fn zoom_is_clamped() {
        let mut camera = Camera::default();
        for _ in 0..40 {
            camera.zoom_at(10.0, [0.0, 0.0], VIEW);
        }
        assert!(camera.mm_per_px >= Camera::MIN_MM_PER_PX);
        for _ in 0..40 {
            camera.zoom_at(0.1, [0.0, 0.0], VIEW);
        }
        assert!(camera.mm_per_px <= Camera::MAX_MM_PER_PX);
    }

    #[test]
    fn pan_moves_the_world_under_the_pointer() {
        let mut camera = Camera::default();
        let before = camera.screen_to_world([500.0, 400.0], VIEW);
        camera.pan_px([100.0, 0.0]);
        let after = camera.screen_to_world([600.0, 400.0], VIEW);
        assert_eq!(before, after);
    }
}
