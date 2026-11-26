//! Editor 2D de elc.
//!
//! Cliente fino del motor de edición: dibuja el proyecto, traduce eventos del ratón a comandos y
//! muestra el estado. Toda la lógica probabilística (snap, cámara, undo) vive en `elec-edit` y se
//! prueba sin abrir ventana.

use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, FontId, Pos2, Sense, Stroke, Vec2};
use elec_core::migrate::{load_project, save_project};
use elec_core::{
    Amps, DeviceId, DeviceKind, Millimeters, OpeningId, OpeningKind, PointMm, Project, RoomId,
    RoomKind, SwitchKind, WallId, Watts,
};
use elec_edit::camera::View;
use elec_edit::commands::{AddDevice, AddOpening, AddRoom, AddWall};
use elec_edit::{Element, Session};

const DEFAULT_FILE: &str = "proyecto.elec.json";
const GRID_COLOR: Color32 = Color32::from_rgb(34, 38, 44);
const WALL_COLOR: Color32 = Color32::from_rgb(220, 224, 230);
const WALL_SELECTED: Color32 = Color32::from_rgb(120, 200, 255);
const ROOM_FILL: Color32 = Color32::from_rgba_premultiplied(70, 110, 150, 24);
const ROOM_SELECTED: Color32 = Color32::from_rgba_premultiplied(120, 200, 255, 40);
const DEVICE_COLOR: Color32 = Color32::from_rgb(255, 190, 90);
const DEVICE_SELECTED: Color32 = Color32::from_rgb(255, 235, 150);
const HINT_COLOR: Color32 = Color32::from_rgb(150, 158, 168);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Wall,
    Room,
    Device,
    Opening,
}

struct App {
    session: Session,
    file: PathBuf,
    tool: Tool,
    device_kind: DeviceKind,
    room_kind: RoomKind,
    wall_start: Option<[i64; 2]>,
    room_points: Vec<[i64; 2]>,
    status: String,
    smoke: bool,
    frames: u32,
}

impl App {
    fn new(file: PathBuf, smoke: bool, ctx: &egui::Context) -> Self {
        ctx.set_visuals(egui::Visuals::dark());
        let project = load_or_default(&file);
        let mut session = Session::new(project);
        if let Some((min, max)) = session.bounds() {
            session.camera.fit(min, max, View::new(1200.0, 760.0));
        }
        Self {
            session,
            file,
            tool: Tool::Select,
            device_kind: DeviceKind::Receptacle {
                rating: Amps::new(15.0).expect("válido"),
                gfci: false,
                dedicated: false,
            },
            room_kind: RoomKind::Living,
            wall_start: None,
            room_points: Vec::new(),
            status: String::new(),
            smoke,
            frames: 0,
        }
    }

    fn save(&mut self) {
        match save_project(self.session.project())
            .map_err(|err| err.to_string())
            .and_then(|text| {
                std::fs::write(&self.file, text)
                    .map_err(|err| format!("no se pudo escribir {}: {err}", self.file.display()))
            }) {
            Ok(()) => self.status = format!("guardado en {}", self.file.display()),
            Err(err) => self.status = err,
        }
    }

    fn add_wall(&mut self, start: [i64; 2], end: [i64; 2]) {
        let command = AddWall {
            id: WallId::new(),
            start: PointMm::new(Millimeters(start[0]), Millimeters(start[1])),
            end: PointMm::new(Millimeters(end[0]), Millimeters(end[1])),
            thickness: Millimeters(150),
            height: Millimeters(2700),
        };
        match self.session.execute(&command) {
            Ok(()) => self.status = "muro añadido".into(),
            Err(err) => self.status = err.to_string(),
        }
    }

    fn add_room(&mut self, points: Vec<[i64; 2]>) {
        let command = AddRoom {
            id: RoomId::new(),
            polygon: points
                .iter()
                .map(|point| PointMm::new(Millimeters(point[0]), Millimeters(point[1])))
                .collect(),
            kind: self.room_kind,
            name: format!("{:?}", self.room_kind),
        };
        match self.session.execute(&command) {
            Ok(()) => {
                self.status = "habitación añadida".into();
                self.room_points.clear();
            }
            Err(err) => self.status = err.to_string(),
        }
    }

    fn add_device(&mut self, at: [i64; 2]) {
        let room = self.session.room_at(at);
        let command = AddDevice {
            id: DeviceId::new(),
            kind: self.device_kind,
            position: PointMm::new(Millimeters(at[0]), Millimeters(at[1])),
            room,
            circuit: None,
        };
        match self.session.execute(&command) {
            Ok(()) => self.status = "dispositivo añadido".into(),
            Err(err) => self.status = err.to_string(),
        }
    }

    fn add_opening(&mut self, at: [i64; 2], kind: OpeningKind) {
        let Some(Element::Wall(wall_id)) = self.session.hit_test(at, 200) else {
            self.status = "haz clic sobre un muro para añadir la abertura".into();
            return;
        };
        let (offset, width) = match kind {
            OpeningKind::Door => (600, 900),
            OpeningKind::Window => (1200, 1200),
        };
        let command = AddOpening {
            id: OpeningId::new(),
            wall: wall_id,
            kind,
            offset: Millimeters(offset),
            width: Millimeters(width),
            sill_z: Millimeters(if matches!(kind, OpeningKind::Door) {
                0
            } else {
                900
            }),
            head_z: Millimeters(2100),
            blocks_conduits: true,
        };
        match self.session.execute(&command) {
            Ok(()) => self.status = "abertura añadida".into(),
            Err(err) => self.status = err.to_string(),
        }
    }

    fn handle_click(&mut self, at: [i64; 2], shift: bool) {
        match self.tool {
            Tool::Select => {
                self.session.selection = self.session.hit_test(at, 150);
                self.status = match self.session.selection_label() {
                    Some(label) => format!("seleccionado: {label}"),
                    None => "nada seleccionado".into(),
                };
            }
            Tool::Wall => match self.wall_start.take() {
                None => self.wall_start = Some(at),
                Some(start) => self.add_wall(start, at),
            },
            Tool::Room => {
                if self.room_points.last() != Some(&at) {
                    self.room_points.push(at);
                }
            }
            Tool::Device => self.add_device(at),
            Tool::Opening => {
                let kind = if shift {
                    OpeningKind::Door
                } else {
                    OpeningKind::Window
                };
                self.add_opening(at, kind);
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Select, "Seleccionar");
            ui.selectable_value(&mut self.tool, Tool::Wall, "Muro");
            ui.selectable_value(&mut self.tool, Tool::Room, "Habitación");
            ui.selectable_value(&mut self.tool, Tool::Device, "Dispositivo");
            ui.selectable_value(&mut self.tool, Tool::Opening, "Abertura");
            ui.separator();
            ui.checkbox(&mut self.session.ortho, "Ortogonal");
            ui.add(
                egui::DragValue::new(&mut self.session.grid_mm)
                    .range(10..=1000)
                    .suffix(" mm"),
            );
            ui.separator();
            if ui
                .add_enabled(self.session.history.can_undo(), egui::Button::new("↶"))
                .on_hover_text("Deshacer (Ctrl+Z)")
                .clicked()
            {
                self.session.undo();
            }
            if ui
                .add_enabled(self.session.history.can_redo(), egui::Button::new("↷"))
                .on_hover_text("Rehacer (Ctrl+Shift+Z)")
                .clicked()
            {
                self.session.redo();
            }
            if ui.button("Guardar").clicked() {
                self.save();
            }
            if ui.button("Encuadrar").clicked() {
                if let Some((min, max)) = self.session.bounds() {
                    let size = ui.available_size();
                    self.session.camera.fit(min, max, View::new(size.x, size.y));
                }
            }
        });

        ui.horizontal(|ui| {
            match self.tool {
                Tool::Device => {
                    device_picker(ui, &mut self.device_kind);
                }
                Tool::Room => {
                    ui.label("Ambiente:");
                    for (kind, label) in [
                        (RoomKind::Living, "Sala"),
                        (RoomKind::Bedroom, "Dormitorio"),
                        (RoomKind::Kitchen, "Cocina"),
                        (RoomKind::Bathroom, "Baño"),
                        (RoomKind::Laundry, "Lavandería"),
                        (RoomKind::Garage, "Garaje"),
                        (RoomKind::Outdoor, "Exterior"),
                    ] {
                        ui.selectable_value(&mut self.room_kind, kind, label);
                    }
                    ui.label("Clic por vértice · Enter o doble clic: cerrar");
                }
                Tool::Wall => {
                    ui.label("Clic 1: inicio · Clic 2: fin · Esc: cancelar");
                }
                Tool::Opening => {
                    ui.label("Clic sobre un muro · Shift: puerta · sin Shift: ventana");
                }
                Tool::Select => {
                    ui.label(
                        "Clic para seleccionar · arrastra con botón derecho para mover la vista",
                    );
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "{:.2} mm/px · {} muros · {} dispositivos",
                    self.session.camera.mm_per_px,
                    self.session.project().geometry.walls.len(),
                    self.session.project().devices.len()
                ));
            });
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
        let rect = response.rect;
        let origin = rect.min;
        let view = View::new(rect.width().max(1.0), rect.height().max(1.0));
        let to_screen = |world: [i64; 2]| -> Pos2 {
            let p = self
                .session
                .camera
                .world_to_screen([world[0] as f64, world[1] as f64], view);
            origin + Vec2::new(p[0], p[1])
        };

        painter.rect_filled(
            rect,
            egui::CornerRadius::ZERO,
            Color32::from_rgb(18, 20, 24),
        );
        self.draw_grid(&painter, origin, rect, view);

        let project = self.session.project().clone();
        for room in &project.geometry.rooms {
            let selected =
                matches!(self.session.selection, Some(Element::Room(id)) if id == room.id);
            let points: Vec<Pos2> = room
                .polygon
                .iter()
                .map(|vertex| to_screen([vertex.x.get(), vertex.y.get()]))
                .collect();
            if points.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(
                    points.clone(),
                    if selected { ROOM_SELECTED } else { ROOM_FILL },
                    Stroke::new(1.0, HINT_COLOR.gamma_multiply(0.6)),
                ));
                if let Some(center) = average(&points) {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        &room.name,
                        FontId::proportional(12.0),
                        HINT_COLOR,
                    );
                }
            }
        }

        for wall in &project.geometry.walls {
            let selected =
                matches!(self.session.selection, Some(Element::Wall(id)) if id == wall.id);
            let color = if selected { WALL_SELECTED } else { WALL_COLOR };
            painter.line_segment(
                [
                    to_screen([wall.start.x.get(), wall.start.y.get()]),
                    to_screen([wall.end.x.get(), wall.end.y.get()]),
                ],
                Stroke::new(3.0_f32.max(wall.thickness.get() as f32 / 40.0), color),
            );
        }

        for opening in &project.geometry.openings {
            if let Some(wall) = project
                .geometry
                .walls
                .iter()
                .find(|wall| wall.id == opening.wall)
            {
                let a = to_screen([wall.start.x.get(), wall.start.y.get()]);
                let b = to_screen([wall.end.x.get(), wall.end.y.get()]);
                let dir = (b - a).normalized();
                let total = (b - a).length();
                let wall_len = wall.length().get().max(1) as f32;
                let start = a + dir * (opening.offset.get() as f32 / wall_len * total);
                let end = start + dir * (opening.width.get() as f32 / wall_len * total);
                let color = if matches!(opening.kind, OpeningKind::Door) {
                    Color32::from_rgb(120, 220, 170)
                } else {
                    Color32::from_rgb(120, 180, 255)
                };
                painter.line_segment([start, end], Stroke::new(6.0, color));
            }
        }

        for device in &project.devices {
            let selected =
                matches!(self.session.selection, Some(Element::Device(id)) if id == device.id);
            let color = if selected {
                DEVICE_SELECTED
            } else {
                DEVICE_COLOR
            };
            let center = to_screen([device.position.x.get(), device.position.y.get()]);
            match device.kind {
                DeviceKind::Luminaire { .. } => {
                    painter.circle_filled(center, 7.0, color);
                }
                DeviceKind::Switch { .. } => {
                    painter.rect_filled(
                        egui::Rect::from_center_size(center, Vec2::splat(10.0)),
                        egui::CornerRadius::same(2),
                        color,
                    );
                }
                DeviceKind::Receptacle { gfci, .. } => {
                    painter.circle_stroke(center, 7.0, Stroke::new(2.0, color));
                    if gfci {
                        painter.circle_filled(center, 3.0, color);
                    }
                }
                DeviceKind::Panel { .. } => {
                    painter.rect_stroke(
                        egui::Rect::from_center_size(center, Vec2::new(18.0, 24.0)),
                        egui::CornerRadius::same(2),
                        Stroke::new(2.0, color),
                        egui::StrokeKind::Middle,
                    );
                }
            }
        }

        if let Some(pos) = response.hover_pos() {
            let anchor = self.wall_start.or_else(|| self.room_points.last().copied());
            let world = self
                .session
                .camera
                .screen_to_world([pos.x - origin.x, pos.y - origin.y], view);
            let result = self
                .session
                .resolve_snap(world, anchor, self.session.camera.mm_per_px);
            let snapped = to_screen(result.point);
            painter.circle_stroke(snapped, 5.0, Stroke::new(1.5, HINT_COLOR));
            if let Some(element) = result.hit {
                painter.text(
                    snapped + Vec2::new(8.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{element:?}"),
                    FontId::monospace(10.0),
                    HINT_COLOR,
                );
            }

            // Previsualización del trazo en curso.
            if let Some(start) = self.wall_start {
                painter.line_segment(
                    [to_screen(start), snapped],
                    Stroke::new(1.5, Color32::from_rgb(120, 200, 255)),
                );
                painter.circle_filled(to_screen(start), 4.0, HINT_COLOR);
            }
            if self.room_points.len() >= 2 {
                let mut points: Vec<Pos2> =
                    self.room_points.iter().map(|p| to_screen(*p)).collect();
                points.push(snapped);
                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(1.5, Color32::from_rgb(120, 200, 255)),
                ));
            }
        }

        self.canvas_input(ui, &response, view, origin);
    }

    fn draw_grid(&self, painter: &egui::Painter, origin: Pos2, rect: egui::Rect, view: View) {
        let step = self.session.grid_mm.max(10);
        let step_px = step as f64 / self.session.camera.mm_per_px;
        if step_px < 6.0 {
            return;
        }
        let top_left = self.session.camera.screen_to_world([0.0, 0.0], view);
        let bottom_right = self
            .session
            .camera
            .screen_to_world([view.width, view.height], view);
        let mut x = top_left[0] / step * step;
        while x <= bottom_right[0] {
            let p = self.session.camera.world_to_screen([x as f64, 0.0], view);
            let screen_x = origin.x + p[0];
            painter.line_segment(
                [
                    Pos2::new(screen_x, rect.top()),
                    Pos2::new(screen_x, rect.bottom()),
                ],
                Stroke::new(1.0, GRID_COLOR),
            );
            x += step;
        }
        let mut y = bottom_right[1] / step * step;
        while y <= top_left[1] {
            let p = self.session.camera.world_to_screen([0.0, y as f64], view);
            let screen_y = origin.y + p[1];
            painter.line_segment(
                [
                    Pos2::new(rect.left(), screen_y),
                    Pos2::new(rect.right(), screen_y),
                ],
                Stroke::new(1.0, GRID_COLOR),
            );
            y += step;
        }
    }

    fn canvas_input(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        view: View,
        origin: Pos2,
    ) {
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            let delta = response.drag_delta();
            self.session.camera.pan_px([delta.x, delta.y]);
        }

        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > 0.0
                && let Some(pos) = response.hover_pos()
            {
                let factor = if scroll > 0.0 { 1.12 } else { 1.0 / 1.12 };
                self.session
                    .camera
                    .zoom_at(factor, [pos.x - origin.x, pos.y - origin.y], view);
            }
        }

        if let Some(pos) = response.interact_pointer_pos() {
            let screen = [pos.x - origin.x, pos.y - origin.y];
            let world = self.session.camera.screen_to_world(screen, view);
            let anchor = self.wall_start.or_else(|| self.room_points.last().copied());
            let snapped = self
                .session
                .resolve_snap(world, anchor, self.session.camera.mm_per_px);

            let shift = ui.input(|input| input.modifiers.shift);
            if response.double_clicked() && self.tool == Tool::Room {
                if self.room_points.len() >= 3 {
                    let points = std::mem::take(&mut self.room_points);
                    self.add_room(points);
                }
                return;
            }
            if response.clicked() {
                self.handle_click(snapped.point, shift);
            }
        }

        let (undo, redo, save, escape, enter) = ui.input(|input| {
            let command = input.modifiers.command;
            (
                command && input.key_pressed(egui::Key::Z) && !input.modifiers.shift,
                command && input.key_pressed(egui::Key::Z) && input.modifiers.shift,
                command && input.key_pressed(egui::Key::S),
                input.key_pressed(egui::Key::Escape),
                input.key_pressed(egui::Key::Enter),
            )
        });

        if undo {
            self.session.undo();
        }
        if redo {
            self.session.redo();
        }
        if save {
            self.save();
        }
        if escape {
            self.wall_start = None;
            self.room_points.clear();
            self.status = "cancelado".into();
        }
        if enter && self.tool == Tool::Room && self.room_points.len() >= 3 {
            let points = std::mem::take(&mut self.room_points);
            self.add_room(points);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(match self.tool {
                    Tool::Select => "Seleccionar",
                    Tool::Wall => "Muro",
                    Tool::Room => "Habitación",
                    Tool::Device => "Dispositivo",
                    Tool::Opening => "Abertura",
                });
                if !self.status.is_empty() {
                    ui.separator();
                    ui.label(&self.status);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Ctrl+Z deshacer · Ctrl+Shift+Z rehacer · Ctrl+S guardar");
                });
            });
        });

        egui::CentralPanel::default().show(ui, |ui| self.canvas(ui));

        self.frames += 1;
        if self.smoke && self.frames >= 3 {
            println!("smoke ok: {} frames", self.frames);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn average(points: &[Pos2]) -> Option<Pos2> {
    if points.is_empty() {
        return None;
    }
    let sum = points
        .iter()
        .fold(Vec2::ZERO, |acc, point| acc + point.to_vec2());
    Some(Pos2::new(
        sum.x / points.len() as f32,
        sum.y / points.len() as f32,
    ))
}

fn load_or_default(file: &PathBuf) -> Project {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|text| load_project(&text).ok())
        .unwrap_or_else(|| Project::new("Proyecto nuevo"))
}

fn device_picker(ui: &mut egui::Ui, kind: &mut DeviceKind) {
    ui.label("Dispositivo:");
    let mut selected = match kind {
        DeviceKind::Luminaire { .. } => 0,
        DeviceKind::Switch { .. } => 1,
        DeviceKind::Receptacle { gfci: true, .. } => 3,
        DeviceKind::Receptacle { .. } => 2,
        DeviceKind::Panel { .. } => 4,
    };
    egui::ComboBox::from_id_salt("device_kind")
        .selected_text(
            [
                "Luminaria",
                "Interruptor",
                "Toma 15 A",
                "Toma GFCI",
                "Tablero",
            ][selected],
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut selected, 0, "Luminaria");
            ui.selectable_value(&mut selected, 1, "Interruptor");
            ui.selectable_value(&mut selected, 2, "Toma 15 A");
            ui.selectable_value(&mut selected, 3, "Toma GFCI");
            ui.selectable_value(&mut selected, 4, "Tablero");
        });
    *kind = match selected {
        0 => DeviceKind::Luminaire {
            watts: Watts::new(18.0).expect("válido"),
        },
        1 => DeviceKind::Switch {
            kind: SwitchKind::Simple,
        },
        3 => DeviceKind::Receptacle {
            rating: Amps::new(20.0).expect("válido"),
            gfci: true,
            dedicated: false,
        },
        4 => DeviceKind::Panel {
            main_rating: Amps::new(100.0).expect("válido"),
        },
        _ => DeviceKind::Receptacle {
            rating: Amps::new(15.0).expect("válido"),
            gfci: false,
            dedicated: false,
        },
    };
}

fn main() -> eframe::Result {
    let mut file = PathBuf::from(DEFAULT_FILE);
    let mut smoke = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => {
                if let Some(value) = args.next() {
                    file = PathBuf::from(value);
                }
            }
            "--smoke" => smoke = true,
            "--help" | "-h" => {
                println!("uso: elec-app [--file RUTA] [--smoke]");
                return Ok(());
            }
            other => eprintln!("argumento desconocido: {other}"),
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 820.0])
            .with_title("elc — CAD eléctrico residencial")
            .with_visible(!smoke),
        ..Default::default()
    };

    eframe::run_native(
        "elc",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(file, smoke, &cc.egui_ctx)))),
    )
}
