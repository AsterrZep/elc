# 0005 — `elec-app` se crea en M1, no en M0

- **Estado:** aceptado
- **Fecha:** 2026-11-25

## Contexto

El spec §12.1 lista siete crates, incluido `elec-app`. M0 debe dejar CI verde en tres sistemas
operativos en una semana. Añadir `eframe`/`wgpu` en M0 arrastra minutos de compilación y
dependencias de ventana antes de tener algo que dibujar.

## Opciones consideradas

1. Crear los siete crates en M0.
2. Crear seis crates headless + `xtask` en M0 y `elec-app` en M1.

## Decisión

La opción 2. M0 tiene `elec-core`, `elec-profile`, `elec-sim`, `elec-rules`, `elec-report`,
`elec-cli` y `xtask`. `elec-app` nace en M1 con su primera pantalla real.

## Consecuencias

- CI de M0 es rápido y no depende de bibliotecas gráficas.
- La regla constitucional «el núcleo no depende de la UI» se cumple por construcción: todavía no
  existe la UI.
- El spec no cambia: sigue habiendo siete crates de producto al cerrar M1.
