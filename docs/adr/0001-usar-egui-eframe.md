# 0001 — UI con egui/eframe

- **Estado:** aceptado
- **Fecha:** 2026-11-25

## Contexto

Necesitamos un editor 2D con lienzo propio (muros, símbolos, snap, zoom) en Rust, multiplataforma,
sin construir dos aplicaciones. La medición de referencia muestra que egui sostiene decenas de
miles de primitivas y que un plano residencial del MVP está muy por debajo de ese umbral
(`INVESTIGACIÓN-v1` §1).

## Opciones consideradas

1. `egui` + `eframe`.
2. Tauri + front web (HTML/JS).
3. `iced` o `slint`.

## Decisión

`egui`/`eframe`, con `default-features = false` y renderer explícito.

## Consecuencias

- Un solo lenguaje y un solo ejecutable; el motor no conoce la UI.
- Menú y bandeja propios (egui no tiene menú nativo); aceptable para esta app.
- Accesibilidad nativa vía AccessKit en Windows/macOS/Linux; el WASM queda experimental.
- `culling` de viewport obligatorio desde el primer día (REQ-UX-009), y vía de escape a
  `paint_callback` retenido si un proyecto supera ~50 000 primitivas.
- Si alguna vez se migra a otra UI, el motor sigue intacto: es un cliente más.
