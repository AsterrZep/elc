# 0006 — El motor de edición vive en un crate headless (`elec-edit`)

- **Estado:** aceptado
- **Fecha:** 2026-11-26

## Contexto

M1 necesita cámara, snap con índice espacial, hit-test, selección e historial de undo. Si todo
eso vive dentro de `elec-app`, solo se puede probar abriendo una ventana: CI no tiene pantalla y
las pruebas de la puerta (`REQ-IO-002`, `REQ-GEO-005`, `REQ-UX-004`) quedarían sin verificar.

## Opciones consideradas

1. Todo en `elec-app`, con pruebas de UI.
2. Motor de edición en un crate nuevo sin dependencias gráficas; `elec-app` como cliente fino.

## Decisión

La opción 2. `elec-edit` contiene `Camera`, `SnapEngine` (rstar), `Session`, los comandos de
edición (`AddWall`, `AddRoom`, `AddDevice`, `AddOpening`) y `History`. No conoce `egui`.
`elec-app` traduce eventos, dibuja y guarda.

## Consecuencias

- Las pruebas de snap, calibración, undo y rendimiento corren en CI sin GPU ni display.
- El spec mantiene siete crates de producto; `elec-edit` es un crate de soporte, no una capa
  nueva de dominio: sigue sin depender de la UI y no contiene reglas normativas.
- La UI no puede mutar el `Project` directamente: toda edición pasa por `Session::execute`.
