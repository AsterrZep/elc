# 0003 — Undo por snapshots, comandos semánticos

- **Estado:** aceptado
- **Fecha:** 2026-11-25

## Contexto

El spec (INV-05) pedía un `Command` reversible con `undo()` por comando. Implementar la inversa
de cada operación es una fuente clásica de bugs silenciosos: basta olvidar restaurar un índice o
un contador para corromper el proyecto sin error visible. Proyectos CAD en Rust (CADCraft,
ShapeCraft) resuelven el undo con snapshots porque el documento es barato de clonar.

## Opciones consideradas

1. `apply` + `undo` por comando.
2. Snapshot completo del proyecto antes de cada comando.
3. Híbrido: comando semántico + snapshot en el stack.

## Decisión

La opción 3. `Command` sigue siendo la unidad semántica (una entrada de historial, etiqueta,
validación), pero el stack de undo guarda el `Project` inmutable resultante. El modelo de una
vivienda es < 1 MB, así que copiar es más barato que un bug de inversa.

## Consecuencias

- `REQ-IO-002` se prueba con hash: 100 comandos y 100 undos devuelven el estado original.
- No hay `undo()` por comando que mantener ni que pueda desincronizarse.
- Si el modelo crece mucho (no se espera), se puede pasar a comandos inversos sin cambiar la API
  pública del historial.
