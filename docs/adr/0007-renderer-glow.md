# 0007 — `eframe` con renderer glow, no wgpu

- **Estado:** aceptado
- **Fecha:** 2026-11-26

## Contexto

`eframe` 0.37 activa `wgpu` por defecto. Compilar `wgpu`/`naga`/`ash` en esta máquina tarda más de
diez minutos y añade centenares de crates para un plano 2D. La investigación (§9) ya concluyó que
egui mueve 2–5 mil primitivas a 4 fps o más con OpenGL, y que el MVP no necesita un backend propio.

## Decisión

`eframe` con `default-features = false` y features `glow`, `default_fonts`, `clipboard`, `x11` y
`wayland`. Sin `wgpu`. Se conserva `accesskit` fuera del MVP.

## Consecuencias

- Build de `elec-app` mucho más corto y sin `naga`.
- El renderer por defecto es OpenGL; en Linux hace falta `libGL` y, para Wayland, `libwayland`.
- Si en el futuro se necesita WebGL/WebGPU o captura headless, se reevalúa con un ADR nuevo.
