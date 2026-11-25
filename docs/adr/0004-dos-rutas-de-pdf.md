# 0004 — Dos rutas de PDF: plano por SVG, tablas por printpdf

- **Estado:** aceptado
- **Fecha:** 2026-11-25

## Contexto

El spec v0.2 proponía una sola ruta de dibujo: SVG canónico y PDF por conversión, para que plano
y lámina compartieran motor. La investigación muestra que `svg2pdf` usa `usvg`, y `usvg` convierte
**todo texto a curvas**: fidelidad perfecta, pero el texto deja de ser seleccionable y buscable, y
los archivos crecen. En un plano es aceptable; en un BOM de 200 filas es un defecto.

## Opciones consideradas

1. Una sola ruta SVG → PDF para todo.
2. Plano por SVG → PDF; tablas generadas directo con `printpdf`.
3. Todo directo con `printpdf`, incluido el plano.

## Decisión

La opción 2.

## Consecuencias

- El plano mantiene fidelidad vectorial y no depende de fuentes del sistema.
- BOM y cuadro de cargas conservan texto real y paginación; `REQ-REP-004` verifica que
  `pdftotext` devuelve las filas.
- Hay dos renderizadores, pero **una sola fuente de datos** (`Bom`, `LoadSchedule`): el riesgo de
  divergencia es de presentación, no de cifras.
