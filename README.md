# elc — CAD eléctrico residencial en Rust

[![CI](https://github.com/AsterrZep/elc/actions/workflows/ci.yml/badge.svg)](https://github.com/AsterrZep/elc/actions/workflows/ci.yml)

Diseño de instalaciones eléctricas residenciales en 2D: plano, circuito lógico, simulación
(sobrecarga, diferencial, cortocircuito, caída de tensión), verificación de reglas normativas
y reportes BOM / cuadro de cargas.

> **Estado: M0 completado.** Workspace de crates headless, CI en tres sistemas, CLI y migrador
> de esquema funcionando. El editor 2D (`elec-app`) llega en M1.

## Qué es y qué no es

Es una herramienta para dibujar la instalación de una vivienda, comprobar si funciona y si es
segura antes de comprar material, y obtener la lista de materiales y el cuadro de cargas.

**No certifica cumplimiento normativo** ni sustituye la revisión de un profesional habilitado.
Los informes indican cuántas reglas implementadas se evaluaron y cuántas están pendientes de
verificación (`verify`) contra la edición oficial de la norma.

## Alcance del MVP (v1.0)

- Plano 2D: muros con espesor, aberturas, habitaciones, plano de fondo calibrado.
- Catálogo: luminaria, interruptor simple y 3-way, toma, toma GFCI, tablero, breaker, RCD.
- Conexión: el **netlist lógico** es la única fuente de verdad eléctrica.
- Simulación: solver nodal (MNA), curvas tiempo-corriente, diferencial por desbalance,
  cortocircuito, flotantes, caída de tensión.
- DRC con reglas como datos, cita y severidad; un perfil normativo (NEC 2023).
- Reportes: BOM y cuadro de cargas en CSV/PDF; plano en SVG/PDF.
- Undo/redo ilimitado y archivos JSON versionados con migraciones.

**Fuera del MVP:** 3D, segundo perfil normativo, auto-enrutado general, trifásico, IFC,
cualquier forma de certificación legal.

## Documentación

| Documento | Contenido |
| --- | --- |
| [`SPEC-v1.0 — SDD App eléctrica residencial.md`](SPEC-v1.0%20%E2%80%94%20SDD%20App%20el%C3%A9ctrica%20residencial.md) | Requisitos (`REQ-*`), invariantes, criterios de aceptación, hitos |
| [`INVESTIGACIÓN-v1 — Stack, normativa y metodología.md`](INVESTIGACI%C3%93N-v1%20%E2%80%94%20Stack%2C%20normativa%20y%20metodolog%C3%ADa.md) | Evidencia de stack, números normativos y metodología |
| [`App eléctrica residencial en Rust — Análisis crítico y SPEC v0.2.md`](App%20el%C3%A9ctrica%20residencial%20en%20Rust%20%E2%80%94%20An%C3%A1lisis%20cr%C3%ADtico%20y%20SPEC%20v0.2.md) | Crítica que originó el recorte del alcance |
| [`Especificación de App Eléctrica en Rust.pdf`](Especificaci%C3%B3n%20de%20App%20El%C3%A9ctrica%20en%20Rust.pdf) | Conversación original (SPEC v1.0 con 3D) |

## Stack

Rust · `egui`/`eframe` · `nalgebra` (MNA) · `serde` JSON versionado · `rstar` (índice espacial) ·
`proptest` / `insta` / `rstest` · SVG propio → PDF para planos, `printpdf` para tablas.

Metodología: Spec-Driven Development. Sin requisito no hay código; el criterio de aceptación se
convierte en prueba antes de la implementación.

## Compilar y probar

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Usar el CLI

```bash
cargo run -p elec-cli -- --version
cargo run -p elec-cli -- info examples/casa-minima.elec.json
cargo run -p elec-cli -- validate examples/casa-minima.elec.json --profile examples/perfil-demo.json
```

`simulate` y `report` responden con su hito pendiente (M3 y M6): fallan a propósito antes que
mentir con un resultado vacío.

## Estructura

```
crates/elec-core      unidades, geometría entera, modelo, comandos, migraciones
crates/elec-profile   NormProfile como datos (reglas, cotas, curvas)
crates/elec-sim       estado de simulación, reloj, protecciones
crates/elec-rules     motor DRC y hallazgos
crates/elec-report    BOM y cuadro de cargas
crates/elec-cli       interfaz headless para pruebas y automatización
xtask                 cobertura de requisitos (cargo xtask coverage-req)
docs/adr              decisiones de arquitectura (append-only)
examples              proyecto y perfil de demostración
```

## Proceso

Spec-Driven Development: sin `REQ` no hay código, y el criterio de aceptación se convierte en
prueba antes de la implementación. Ver `CONSTITUTION.md`, `SPEC-v1.0` y `PLAN.md`.

## Licencia

**PolyForm Noncommercial License 1.0.0** — ver [`LICENSE`](LICENSE).

Uso, estudio, modificación y redistribución permitidos para fines **no comerciales**
(incluye uso personal, educativo, investigación y organizaciones sin ánimo de lucro).
**No se permite uso comercial** ni la explotación comercial de la app o de trabajos derivados.

Esto es *source available*, no "open source" según la definición de la OSI: la licencia
restringe el campo de uso. Para un uso comercial se requiere una licencia aparte del titular.

---

© 2026 AsterrZep.
