# PLAN de desarrollo — elc

**Versión:** 1.0 · 25 nov 2026
**Base:** `SPEC-v1.0` (requisitos) + `INVESTIGACIÓN-v1` (stack y física)
**Objetivo del plan:** llegar a un MVP usable sin construir de más, con cada paso verificable.

---

## 1. Principio de ejecución

Cuatro reglas que ordenan todo lo demás:

1. **Núcleo headless primero, UI después.** Todo se prueba desde `elec-cli` antes de tener botón. La UI es un cliente del motor, nunca al revés.
2. **Una rebanada vertical por hito.** Cada hito entrega algo que el usuario puede ver funcionar de punta a punta, no capas a medias.
3. **La puerta manda.** Un hito no empieza si el anterior no tiene su suite de aceptación en verde y su demo grabada.
4. **Recortar features, nunca física.** Si falta tiempo se cae el plano de fondo o las plantillas; no se cae el solver, la curva de protección ni el DRC con cita.

**Lo que este plan evita deliberadamente:** empezar por la UI bonita, implementar 3D antes de tiempo, y escribir 9 reglas DRC antes de que exista un netlist que evaluar.

---

## 2. Mapa de hitos

| Hito | Entrega | REQ que cierra | Duración |
| --- | --- | --- | --- |
| **M0** | Workspace, CI, constitución, CLI vacío, migrador, xtask de cobertura | — (infraestructura) | 1 sem |
| **M1** | Editor 2D mínimo + modelo + persistencia + undo | `REQ-GEO-001..007`, `REQ-DOM-010`, `REQ-IO-001..004`, `REQ-UX-004..007` | 2–3 sem |
| **M2** | Netlist lógico + plantillas + terminales | `REQ-NET-001..006`, `REQ-DOM-011..015` | 2 sem |
| **M3** | Solver nodal + energización + caída de tensión | `REQ-SIM-001`, `002`, `009`, `012` | 2 sem |
| **M4** | Protecciones, fallas, traza causal | `REQ-SIM-003..008`, `010`, `011` | 2–3 sem |
| **M5** | DRC con perfil normativo | `REQ-DRC-001a..011`, INV-06/07 | 2 sem |
| **M6** | Reportes BOM / cuadro / plano | `REQ-REP-001..005` | 1–2 sem |
| **M7** | Cierre MVP: UX, accesibilidad, textos legales, migraciones | `REQ-UX-001..003`, `008`, `009`, `010`, `REQ-NFR-006` | 1–2 sem |
| | **MVP v1.0** | | **~13–17 sem** |
| M8 | v1.1: 3D ligero + glTF | — | 4–6 sem |
| M9 | v1.1: perfil IEC 60364 | — | 3–4 sem |

**Ruta crítica:** M1 → M2 → M3 → M4. Todo lo demás es paralelizable o depende de ellas.

---

## 3. Detalle por hito

### M0 · Esqueleto (1 semana)

**Objetivo:** que `cargo test` y la CI existan antes que la primera línea de dominio. Barato ahora, carísimo después.

**Entregables**
- Workspace con `elec-core`, `elec-profile`, `elec-sim`, `elec-rules`, `elec-report`, `elec-cli` + `xtask`.
- `CONSTITUTION.md` con los 10 invariantes y las reglas de PR.
- `docs/adr/` con plantilla MADR y los 5 ADRs iniciales (stack, netlist, undo por snapshots, dos rutas PDF, un perfil).
- CI: `fmt --check`, `clippy -D warnings`, `test --workspace` en Linux/macOS/Windows.
- Perfiles de compilación (`opt-level = 2` en deps de dev, thin LTO en release).
- Migrador de `schema_version` con corpus vacío y un test que falla si se olvida.
- `cargo xtask coverage-req` que lista REQ sin prueba.

**Puerta:** CI verde en los tres SO; `elec-cli --version`; `xtask coverage-req` informa 0 % y no explota.

**No se hace en M0:** `elec-app` (llega en M1, con su primera pantalla), ningún dato normativo, ninguna regla.

> Nota: el spec lista `elec-app` entre los crates; se crea en M1 para no arrastrar `eframe`/`wgpu` antes de tener algo que dibujar. Queda registrado como ADR.

---

### M1 · Modelo + editor 2D mínimo (2–3 semanas)

**Estado:** completado el 2026-11-26; evidencia y límites en [`docs/hitos/M1.md`](docs/hitos/M1.md).
La demo visual de la puerta queda pendiente de revisión manual.

**Objetivo:** dibujar una vivienda y que sobreviva a guardar, cerrar y reabrir.

**Entregables**
- `elec-core`: `Millimeters`, magnitudes tipadas, `PointMm`, `Wall`, `Opening`, `Room`, `WaterPoint`, `Device`, `Circuit`, `Project`, `Command`, migraciones.
- `elec-app`: ventana `eframe`, lienzo con pan/zoom, snap a grilla/ángulos/extremos, herramientas muro/abertura/habitación/dispositivo, selección.
- Índice espacial `rstar` para snap y hit-test (no recorrido lineal).
- Undo/redo por snapshots inmutables; atajos nativos.
- `elec-cli info <archivo>`: resume muros, aberturas, habitaciones, dispositivos.

**Pruebas de la puerta**
- `REQ-GEO-001`: muro 3000×150 mm ⇒ área exacta 450 000 mm².
- `REQ-GEO-006`: habitación 4000×3000 ⇒ 12,0 m².
- `REQ-IO-004`: guardar dos veces ⇒ bytes idénticos.
- `REQ-IO-002`: 100 comandos + 100 undos ⇒ hash original.
- `REQ-GEO-005`: plano de fondo calibrado a ±1 mm.

**Demo:** dibujar baño + cocina + 2 dormitorios, colocar 8 dispositivos, guardar, cerrar, reabrir y ver todo idéntico.

**Riesgo principal:** el snap con precisión. Se resuelve con geometría entera y `rstar`, nunca con tolerancias de `f64`.

---

### M2 · Netlist lógico (2 semanas)

**Objetivo:** que el plano tenga electricidad, no solo dibujo.

**Entregables**
- `Node`, `Branch`, `Net`, `Edge`; terminales por dispositivo.
- Plantillas deterministas: simple, 3-way, toma, luminaria.
- Edición manual de hilos y conductos (conducto = anotación).
- Etiquetas de hilos en el lienzo y marcas `///` en SVG.
- `elec-cli netlist <archivo>` que imprime el grafo derivado.

**Puerta:** `REQ-NET-001..006`. Demo: interruptor simple y 3-way cableados por plantilla, corregidos a mano, y el netlist impreso coincide con lo dibujado.

---

### M3 · Solver (2 semanas)

**Objetivo:** corriente real, no `I = P/V`.

**Entregables**
- MNA lineal con `nalgebra` LU; estampado de conductancias y fuentes.
- Modelo de carga por dispositivo; estados de interruptor y 3-way.
- Energización, flotantes, caída de tensión por tramo y acumulada.
- `elec-cli simulate <archivo> --scenario <json>`.
- Oráculo de pruebas con `network_analysis` sobre 20 circuitos generados.

**Puerta:** `REQ-SIM-001` (±0,1 % contra analítico **y** contra el oráculo), `002`, `009`, `012`. Demo: encender tres lámparas y ver el flujo resaltado y la caída de tensión por ramal.

---

### M4 · Protecciones y fallas (2–3 semanas)

**Objetivo:** el simulador dice la verdad sobre lo que dispara.

**Entregables**
- Curva térmica calibrada a 1,13 / 1,45 / 2,55·In; magnética por banda B/C/D.
- `SimClock` acelerable; banda de disparo, no instante.
- RCD por desbalance neto con tiempos 0,3 / 0,15 / 0,04 s y zona incierta 0,5–1·IΔn.
- Cortocircuito topológico + impedancia de lazo; retorno invertido; 3-way mal cableado.
- Traza causal en texto.
- UI: panel de estado por circuito y «¿por qué disparó?».

**Puerta:** casos A1–A6 de §18 del spec. **A1 debe NO disparar al instante** a 1,25·In y sí tras el tiempo estimado; A2 debe disparar con 35 mA y no afirmar disparo con 20 mA.

**Riesgo principal:** que la curva quede «bonita» pero no pase los tres puntos. Calibrar con los datos del perfil, no a ojo.

---

### M5 · DRC (2 semanas)

**Objetivo:** el plano se valida solo.

**Entregables**
- `elec-rules` con las 9 familias, severidad y cita; evaluación incremental en Modo Diseño.
- `profiles/nec-2023.json` completo, con `verified: false` donde corresponda.
- Panel de hallazgos clicable; exportación del informe con el aviso legal.
- `REQ-DRC-001a` (ampacidad por temperatura) separada de `001b` (techo 240.4(D)).
- `REQ-DRC-002` con `RoomKind` + `WaterPoint` a 6 ft.

**Puerta:** proyecto con 9 hallazgos inducidos, uno por familia, cada uno con `rule_id`, severidad y cita.

---

### M6 · Reportes (1–2 semanas)

**Objetivo:** material cotizable.

**Entregables**
- BOM con mecanismos, cajas, tubería y cable por función/calibre/color, con metrado vertical por alturas del perfil.
- Cuadro de cargas con las columnas fijas.
- Plano SVG con capas; PDF de plano por `svg2pdf`; PDF de tablas por `printpdf`.
- `elec-cli report <archivo> --out <dir> --all`.

**Puerta:** `REQ-REP-001..005`; el PDF de tablas tiene texto extraíble (`pdftotext` devuelve las filas).

---

### M7 · Cierre MVP (1–2 semanas)

**Objetivo:** que lo pueda usar alguien que no eres tú.

**Entregables**
- Modo Diseño↔Simulación sin pérdida de estado; accesibilidad por color **y** patrón.
- Aviso legal persistente; textos en español centralizados.
- Corpus de migraciones v1→vN con tests.
- Empaquetado por plataforma; binario < 30 MB; arranque < 1 s.
- Demo de §4.4 del spec completa.

**Puerta:** un usuario nuevo completa el recorrido de §4.4 sin ayuda.

---

## 4. Semana 1, día por día

| Día | Trabajo | Verificación |
| --- | --- | --- |
| 1 | Workspace, crates vacíos, perfiles, `.gitignore` de salida | `cargo build --workspace` |
| 1 | `CONSTITUTION.md`, plantilla ADR, ADRs 0001–0005 | revisión |
| 2 | CI de los tres SO; cache; `clippy -D warnings` | Actions en verde |
| 2 | `elec-core`: unidades + ids + geometría entera | tests de área |
| 3 | `Project` + `schema_version` + migrador + test de bytes idénticos | `REQ-IO-004` |
| 3 | `elec-cli` con `--version` e `info` | ejecución real |
| 4 | `xtask coverage-req` | informe legible |
| 4 | Spike S1 (lienzo 5k/50k) y S2 (MNA vs oráculo) | números anotados en `docs/spikes.md` |
| 5 | Cierre de M0, tag `v0.1.0-m0`, retrospectiva escrita | puerta M0 |

---

## 5. Convenciones de trabajo

**Ramas y commits**
- `main` siempre verde. Trabajo en `feat/mN-descripcion`, `fix/...`, `docs/...`.
- Commits pequeños y en imperativo: `add wall snapping to grid`, `fix curve anchor at 2.55 In`.
- Un PR = un requisito o un ADR. Si el PR necesita explicar dos cosas, son dos PRs.

**Definición de terminado (por requisito)**
1. Tiene prueba con su ID en el nombre.
2. `cargo test` y `clippy -D warnings` verdes.
3. Si toca datos normativos: `verified` correcto y cita.
4. Si cambia comportamiento visible: entrada en el CHANGELOG del hito.
5. `cargo xtask coverage-req` sube o mantiene la cobertura.

**ADRs**
- Obligatorio para: cambio de stack, cambio de invariante, cierre de un `verify`, cualquier cosa difícil de revertir.
- Append-only: la decisión nueva supersede a la vieja, nunca la edita.

**Tamaño**
- PR ideal < 400 líneas de diff. Los hitos se parten en tareas de un día.

---

## 6. Tablero de progreso

```bash
cargo xtask coverage-req        # REQ con y sin prueba
cargo test --workspace          # suite
git tag --list 'v0.*'           # hitos cerrados
```

Métrica principal: **REQ MUST con prueba en verde / REQ MUST totales**. No se mide en líneas de código.

Objetivo por hito:

| Hito | Cobertura MUST esperada |
| --- | --- |
| M0 | 0 % (solo infraestructura) |
| M1 | ~20 % |
| M2 | ~32 % |
| M3 | ~45 % |
| M4 | ~62 % |
| M5 | ~78 % |
| M6 | ~92 % |
| M7 | 100 % |

---

## 7. Riesgos de calendario

| Riesgo | Señal temprana | Respuesta |
| --- | --- | --- |
| El editor se come el calendario (como en CADCraft: 70 % del código) | M1 pasa de 3 semanas | Recortar plano de fondo y plantillas; nunca el snap ni el undo |
| La curva no pasa los tres puntos | S3 falla | Ajustar con los datos del perfil; si no, bajar a interpolación monótona documentada |
| Los `verify` no se cierran | M4 llega sin texto oficial | Publicar MVP con perfil marcado «no verificado» y aviso reforzado |
| El 3D presiona antes de tiempo | petición de «ver la casa en 3D» en M2 | M8 ya está planificado; no se adelanta |
| El alcance crece con «una cosita más» | PR sin `REQ` | Se rechaza el PR |

---

## 8. Decisiones que este plan NO reabre

- 3D fuera del MVP.
- Netlist como única verdad eléctrica.
- Un perfil normativo (NEC 2023) primero.
- Undo por snapshots, comandos semánticos.
- Dos rutas de PDF (plano SVG, tablas `printpdf`).
- `egui`/`eframe`, sin Tauri.

Cambiar cualquiera de estas exige un ADR que supersede, no una discusión en un PR.
