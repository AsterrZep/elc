# Investigación v1 — Cómo construir esta app de forma óptima

**Fecha:** 25 nov 2026
**Insumo:** `SPEC-v1.0 — SDD App eléctrica residencial.md`
**Propósito:** decidir con evidencia (no con intuición) las cuatro cosas que más encarecen o hunden un proyecto así: **la arquitectura del editor**, **el motor de simulación**, **los números normativos** y **las herramientas de salida**. Donde la evidencia contradice el SPEC v1.0, se anota el delta y se aplica al documento.

**Método:** búsqueda dirigida a fuentes primarias (textos de normas citados por fabricantes, documentación de crates, informes de rendimiento de proyectos reales en Rust/egui). Cada afirmación lleva fuente. Lo que no pude contrastar con texto oficial queda marcado `verify`, igual que en el spec.

---

## 0. Resumen ejecutivo (lo que cambia y lo que no)

| # | Hallazgo | Impacto |
| --- | --- | --- |
| 1 | egui rinde bien hasta decenas de miles de primitivas; a ~88 000 baja a ~4 fps. Un plano residencial típico está en cientos–pocos miles | **No hace falta motor gráfico propio en el MVP.** Sí hace falta *culling* de viewport desde el primer día |
| 2 | Arquitectura probada en CADs Rust reales: «todo es un comando», motor independiente de egui, índice espacial R-tree, snapshots copy-on-write para undo | Confirma INV-02/INV-05 y añade **R-tree** al diseño |
| 3 | Los crates SPICE en Rust (`spice-rs`, `network_analysis`) resuelven MNA **no lineal con Newton**; nuestro caso es lineal resistivo | Implementar MNA propio (~50 líneas + `nalgebra` LU), usar `network_analysis` como **oráculo de pruebas**, no como dependencia |
| 4 | Curva MCB confirmada con puntos de prueba exactos, incluido **2,55·In** (que el spec no modelaba) | Añadir punto de prueba 2,55·In y la distinción 1 h / 2 h (>63 A) al perfil |
| 5 | NEC 210.8(A) 2023: la distancia de **6 ft (1,83 m)** aplica a fregaderos, bañeras y duchas, **no** a «cerca de agua» en general | Reformular `REQ-DRC-002`: por tipo de ambiente + distancia configurable al punto de agua |
| 6 | Ampacidad 310.16 tiene **columnas por temperatura** (60/75/90 °C) y 240.4(D) impone un techo de protección aparte | El «15 A→14 AWG» es un caso particular, no la regla; el perfil necesita ambas cosas |
| 7 | `svg2pdf`/`usvg` convierte **todo texto a curvas**; `printpdf` genera tablas con texto real | **Dos rutas de PDF**, no una: plano por SVG, tablas por `printpdf` |
| 8 | egui: AccessKit nativo en Windows/macOS, sin menú nativo ni bandeja; WASM viable pero pesa 0,6–2,6 MB | Menú propio, empaquetado por plataforma, i18n centralizada desde el inicio |
| 9 | SDD maduro = *constitución → spec → clarificar → plan → checklist → tareas → analizar → implementar*; ADR append-only (MADR) | Añadir constitución del proyecto y ADRs al proceso, sin adoptar tooling de agente |
| 10 | `rstar`, `slotmap 1.1`, `uuid 1.28` (v7), `rustc-hash 2.1`; `serde_json` ordena claves por defecto (salida determinista) | Confirmar crates y fijar `serde_json` sin `preserve_order` |

**Lo que la investigación descarta:** motor 3D en el MVP, Tauri/webview, SPICE como dependencia, render propio en wgpu, y «una sola ruta de PDF».

---

## 1. El editor 2D: qué aguanta egui de verdad

### 1.1 Datos medidos

Un proyecto CAD en egui hizo públicas sus mediciones con un plano de **88 747 primitivas** ([BIMCraft, plan de aceleración de lienzo](https://github.com/IPedrax/BIMCraft/blob/main/docs/canvas-acceleration-plan.md)):

| Zoom | Crear shapes | Teselar | Primitivas (fuera de pantalla) | FPS aprox. |
| --- | --- | --- | --- | --- |
| 1× | 45 ms | 133 ms | 88 747 (0) | ~4 |
| 16× | 53 ms | 110 ms | 46 578 (44 879) | ~4 |
| 64× | 112 ms | 52 ms | 45 632 (45 456) | ~4 |

El diagnóstico del propio autor: *«el costo no es no saber qué cambió»* — en modo inmediato se reconstruye y tesela todo cada frame, y además se vuelve a subir cada vértice a la GPU (otros ~55 ms). La solución que adoptaron: **teselar una vez y dibujar por `paint_callback` con buffers retenidos**, más *culling* previo a la construcción de shapes.

En la discusión oficial de egui ([#7619](https://github.com/emilk/egui/discussions/7619)) un usuario reporta **40 000 rectángulos** «sorprendentemente bien» sin optimizar, con el límite en un solo hilo de CPU.

### 1.2 Traducción a nuestro caso

Un plano residencial del MVP (vivienda de 150 m², ~25 muros, 4 habitaciones, 8 aberturas, 60 dispositivos, 80 tramos) no pasa de **~2 000–5 000 primitivas**. Estamos **dos órdenes de magnitud** por debajo del umbral donde egui se degrada.

Conclusión operativa:

1. **egui es suficiente para el MVP.** No hay justificación para un viewport propio en wgpu (que sería un segundo motor gráfico, exactamente el riesgo #8 que el v0.2 quería evitar).
2. **Culling de viewport desde el día 1**, aunque hoy no haga falta: es la diferencia entre «funciona» y «funciona cuando el usuario importa un plano de fondo grande».
3. **Presupuesto de rendimiento explícito** en el spec: 60 fps con 5 000 primitivas, 30 fps con 50 000. Medible con `criterion` y un fixture generado.
4. **Vía de escape documentada**: si un proyecto supera ~50 000 primitivas, se activa el camino de teselado retenido por `paint_callback` ([`egui_wgpu::CallbackTrait`](https://docs.rs/egui-wgpu/latest/egui_wgpu/trait.CallbackTrait.html)). No se implementa hasta que exista el problema.

**Delta al spec:** añadir `REQ-NFR-007` (presupuesto de primitivas y fps) y `REQ-UX-009` (culling obligatorio).

---

## 2. Arquitectura del editor: lo que hacen los CAD en Rust que sí funcionan

Tres proyectos abiertos con arquitectura documentada permiten extraer patrones, no opiniones.

### 2.1 CADCraft (AutoCAD-style, Rust + egui)

De su [documentación de arquitectura](https://github.com/storytold/cadcraft/blob/main/docs/architecture.md) y su [artículo](https://cadcraft.org/blog/cad-in-rust):

- **«Everything is a command»**: cada acción visible es un `CommandSpec` con id estable, parámetros JSON y máquina de estados de prompts. La línea de comandos, menús, CLI, scripts y agentes llaman al mismo código.
- **El motor no sabe que egui existe**: por eso los mismos comandos corren headless. Es la validación externa de nuestra INV-02.
- **Índice espacial R-tree** para object snap y selección.
- **Undo por snapshots** porque las colecciones de entidades son copy-on-write: *«undo is a whole-drawing snapshot that costs little»*.
- Reparto de código: **engine 107 600 líneas vs ui-egui 33 932**. La mitad del código es el motor, no el dibujo.

ShapeCraft ([arquitectura](https://github.com/jackharvest/shapecraft/blob/main/docs/architecture.md)) repite el patrón: capas `doc → engine → render → ui-egui`, con `doc` barato de clonar para que undo sea lista de snapshots. MuginCAD usa MVVM con undo/redo en el modelo.

### 2.2 Lecciones aplicables

| Lección | Aplicación concreta |
| --- | --- |
| Comando como unidad universal | Nuestro `Command` no solo da undo: es la **API del CLI y de los tests**. Cada caso de aceptación es una secuencia de comandos |
| Motor sin UI | Ya está en INV-02, pero se refuerza: `elec-core` no debe ni saber que existe `eframe` |
| Índice espacial | Añadir `rstar` para snap y hit-testing; sin él, snap a extremos es O(n) por frame |
| Snapshot vs comando | **Híbrido**: comandos para semántica (y deshacer selectivo), snapshot serializado para el stack de undo. El modelo es pequeño (< 1 MB), así que copiar es más simple y más robusto que implementar `undo()` por comando — y elimina la clase de bugs «undo inconsistente» |
| Proporción de esfuerzo | Esperar **2–3× más código en el núcleo que en la UI**. Planificar los hitos en consecuencia |

**Delta al spec:** INV-05 se reformula a «todo cambio pasa por un `Command` y el stack de undo guarda snapshots inmutables del proyecto» (más simple y probado). Añadir `rstar` al stack.

---

## 3. El solver: por qué no usar SPICE

### 3.1 Lo que existe

| Crate | Qué hace | Por qué no sirve aquí |
| --- | --- | --- |
| [`network_analysis`](https://docs.rs/network_analysis/latest/network_analysis/) | MNA y análisis de mallas, **no lineal** | Dependencia grande para un problema lineal; no controlamos stams ni determinismo |
| [`spice-rs`](https://ferrite-systems.github.io/spice-rs/internals/ch17) | Port fiel de ngspice: Newton-Raphson, paso de tiempo, solver disperso KLU/Markowitz | Es un simulador de transitorios; nuestro régimen es DC resistivo |
| [`general-mna`](https://crates.io/crates/general-mna) | MNA simbólico educativo sobre netlists SPICE | Enfocado a dinámica, no a topología residencial |

Nuestro problema: red **lineal** de resistencias (cables, cargas) con fuentes ideales, decenas de nodos, sin reactancias. Eso es un sistema `A·x = b` de álgebra lineal, no un problema de convergencia.

### 3.2 Decisión

1. **Implementar MNA lineal propio** (estampado de conductancias + fuentes), resolver con `nalgebra::DMatrix` y `LU` con pivoteo parcial. Es código corto, auditable y **determinista**.
2. **Usar `network_analysis` como oráculo en pruebas**: mismo circuito resuelto por dos implementaciones independientes. Si discrepan >0,1 %, hay bug. Esto vale más que cualquier revisión manual.
3. **Dispersión**: con decenas de nodos, la matriz densa es más rápida que cualquier estructura dispersa. `nalgebra-sparse` solo si se superan ~500 nodos (no ocurrirá en residencial).

### 3.3 Determinismo (INV-10)

- Solo operaciones IEEE exactamente definidas (`+ − × ÷`, `sqrt` correctamente redondeado). **Sin `powf`, `exp`, `ln`** en el camino del solver.
- LU con pivoteo determinista (mismo orden de filas ⇒ mismo resultado).
- Comparaciones de igualdad con tolerancia explícita, nunca `==` sobre `f64`.
- `serde_json` ordena las claves de los mapas por defecto (sin la feature `preserve_order`), lo que da archivos byte-idénticos ([docs](https://docs.rs/serde_json/latest/serde_json/map/struct.Map.html)). **No activar `preserve_order`.**

**Delta:** añadir la nota de «prohibido `powf`/`exp` en el solver» a INV-10 y el oráculo a `REQ-SIM-001`.

---

## 4. Física de protecciones: números confirmados (y uno que faltaba)

### 4.1 Interruptor termomagnético (IEC/EN 60898-1)

Confirmado por [ABB, comparativa de curvas](https://library.e.abb.com/public/114371fcc8e0456096db42d614bead67/2CDC400002D0201_view.pdf), [Schneider, curvas de disparo](https://ckm-content.se.com/ckmContent/sfc/servlet.shepherd/document/download/0691H00000FJeeBQAT), [Doepke, manual MCB](https://www.doepke.de/uploads/tx_doepkeproducts/informationsmaterial/web/doepke_5962527_mcb_handbuch_web_en.pdf) y [EFICAM](https://www.apiem.org/wp-content/uploads/2024/04/Manual-Interruptores-Automaticos-EFICAM-2024.pdf):

| Punto de prueba | Requisito | Curvas |
| --- | --- | --- |
| 1,13·In | **No** debe disparar en 1 h (2 h si In > 63 A) | B, C, D |
| 1,45·In | **Debe** disparar en < 1 h | B, C, D |
| **2,55·In** | Debe disparar en > 1 s y < 60 s (In ≤ 32 A); < 120 s (In > 32 A) | B, C, D |
| Magnético inferior | No debe disparar en 0,1 s | B: 3·In · C: 5·In · D: 10·In |
| Magnético superior | Debe disparar en < 0,1 s | B: 5·In · C: 10·In · D: 20·In |

Zonas de disparo magnético: **B 3–5·In, C 5–10·In, D 10–20·In** (confirmado por las cinco fuentes; `K 8–12`, `Z 2–3` existen pero no son del MVP).

**Hallazgo:** el SPEC v1.0 modelaba 1,13·In, 1,45·In y la banda magnética, pero **no el punto 2,55·In**. Ese punto es el que ancla la parte media de la curva térmica y evita que el modelo interpole libremente entre 1,45·In y el magnético. Sin él, «disparo estimado en ~N min» puede desviarse por un factor grande en el rango 1,5–3·In, que es justo donde caen las sobrecargas domésticas reales (un horno + una plancha).

**Delta:** añadir `2,55·In → 1–60 s (In ≤ 32 A)` a la tabla de §8.2 y a `RequestCurve` como tercer punto de anclaje. Con tres puntos (1,13 / 1,45 / 2,55) se ajusta una curva térmica `t = k/(I²−I0²)` con dos parámetros y un grado de libertad para calibrar.

### 4.2 Diferencial (IEC 61008-1 / 61009-1)

Del extracto de [IEC 61008-1:2024, Tabla 11](https://www.iecdown.world/iec/IEC%2061008-1-2024%20PDF.pdf), corroborado por [BEAMA RCD Handbook](https://www.beama.org.uk/static/dd77ff1b-da20-440a-bc338a083127ca46/RCD_GUIDE.pdf) y [ABB](https://library.e.abb.com/public/2ff8d55b149143db8906ce9aae4ac669/B_EPBP_Cat_2019_02_Residual%20current%20devices_LR.pdf):

| Corriente residual | Tiempo máximo de disparo (tipo general) |
| --- | --- |
| 0,5·IΔn | **No debe disparar** (corriente residual no operativa normalizada) |
| 1·IΔn | 0,3 s |
| 2·IΔn | 0,15 s |
| 5·IΔn | 0,04 s |

- Para tipo **A**, los tiempos se multiplican por 1,4 si IΔn > 0,01 A y por 2 si ≤ 0,01 A.
- Un RCD de 30 mA **puede disparar en cualquier punto entre 15 y 30 mA** ([ABB whitepaper](https://library.e.abb.com/public/fc3d92ef8da94d38a3882b0b86afb552/ELAU%20RCD%20Local%20Whitepaper%202022_9AKK108466A8987_RevA.pdf)). Esto confirma la «zona incierta» que el v0.2 introdujo por intuición: **está en la norma**, no es un matiz pedagógico.
- Selectivos (tipo S): 0,13–0,5 s a IΔn, 0,06–0,2 a 2·IΔn, 0,05–0,15 a 5·IΔn.

**Implicación de diseño:** el simulador no debe elegir un punto dentro de la banda; debe reportar el intervalo. `TripCause` lleva `band: (SimTime, SimTime)` en lugar de un instante único. Eso hace que el caso A2 sea defendible incluso si el usuario compara con el catálogo de su fabricante.

**Delta:** `ProtectionState::Tripped` gana `band: (SimTime, SimTime)`; el texto de UI muestra «dispara entre X y Y».

### 4.3 Lo que sigue `verify`

Los valores anteriores están publicados por fabricantes que citan la norma, no por el texto oficial completo. Se mantienen `verify` en el perfil hasta cotejarlos con la edición comprada, pero **ya no son «de memoria»**: tienen fuente secundaria identificable. Eso reduce el riesgo de un error de transcripción, no el de una errata en la fuente.

---

## 5. Normativa NEC: dos correcciones importantes

### 5.1 GFCI por ubicación (210.8(A), edición 2023)

Listado real ([resumen de 210.8](https://electricai.app/nec/210-8)):

| Ubicación | Alcance |
| --- | --- |
| Baños | **Todas** las tomas |
| Cocinas | Tomas que sirven superficies de trabajo (countertop) |
| Fregaderos | Tomas **dentro de 6 ft (1,83 m)** del borde exterior |
| Bañeras y duchas | Tomas dentro de 6 ft |
| Lavandería | Todas |
| Exteriores | Todas |
| Garajes y edificios anexos | Todas ≤ 150 V a tierra, ≤ 20 A |
| Sótanos sin terminar | Todas (salvo alarma contra incendios/seguridad) |
| Espacios de acceso bajo piso (crawl) | A nivel o bajo rasante |
| Boathouses | Todas |

Aplica a tomas de **125 V a 250 V**.

**Corrección al PDF y precisión al v0.2:** el PDF decía «GFCI a menos de 1,8 m de fregaderos y baños». La distancia de 6 ft **solo** aplica a fregaderos, bañeras y duchas; en **baños es a cualquier distancia**. Y hay siete ubicaciones más.

Modelo correcto: regla con dos predicados — `room_kind ∈ {Bathroom, Laundry, Garage, Outdoor, Crawlspace, UnfinishedBasement}` (todas las tomas) **o** `distance(device, water_point) ≤ 6 ft` para fregadero/bañera/ducha. La distancia es parámetro del perfil, no constante de código.

**Delta:** reescribir `REQ-DRC-002` con esa estructura y añadir `WaterPoint` como entidad de geometría (hoy no existe; `Room.has_water_point` es un booleano insuficiente para medir 6 ft).

### 5.2 Ampacidad y protección de conductor pequeño

[Tabla 310.16 (NEC 2023)](https://wiresizes.com/guides/nec-table-310-16), cobre, tres conductores o menos, 30 °C:

| Calibre | 60 °C | 75 °C | 90 °C | Techo 240.4(D) |
| --- | --- | --- | --- | --- |
| 14 AWG | 15 A | 20 A | 25 A | **15 A** |
| 12 AWG | 20 A | 25 A | 30 A | **20 A** |
| 10 AWG | 30 A | 35 A | 40 A | **30 A** |
| 8 AWG | 40 A | 50 A | 55 A | — |

Notas que cambian el modelo:

- La columna aplicable depende de la **temperatura de los terminales** (110.14(C)); en residencial suele gobernar 75 °C, no 60 °C.
- **240.4(D) es un límite separado**: aunque la tabla diga 20 A para 14 AWG en 75 °C, la protección no puede pasar de 15 A. Es decir, hay **dos comprobaciones distintas**: `In ≤ Iz` (ampacidad) y `In ≤ techo 240.4(D)`.
- 18 AWG → 7 A y 16 AWG → 10 A existen como casos especiales.

**Corrección:** el PDF reducía todo a «15 A→14 AWG, 20 A→12 AWG». Eso es la consecuencia de 240.4(D) en la columna de 60 °C, no la regla. Con el modelo anterior, un circuito de 20 A con 12 AWG en 75 °C pasaría con 25 A de ampacidad pero **debe** limitarse a 20 A por 240.4(D); y un 14 AWG con breaker de 20 A fallaría por 240.4(D) aunque la ampacidad lo permita.

**Delta:** `NormProfile.ampacity_table` con eje de temperatura; `RuleDef` separada `REQ-DRC-001a` (ampacidad) y `REQ-DRC-001b` (techo 240.4(D)).

---

## 6. Salidas: por qué el PDF no puede ser una sola ruta

### 6.1 Lo que hacen las librerías

- [`svg2pdf`](https://docs.rs/svg2pdf/latest/svg2pdf): convierte SVG a PDF **sin rasterizar** (salvo filtros), pero usa `usvg` por debajo, y `usvg` **convierte todo texto a path** ([experiencia documentada](https://parsecsreach.org/post/rust_svg_to_pdf)). Resultado: fidelidad perfecta, archivos más grandes, texto no seleccionable ni buscable, sin problemas de fuentes.
- [`printpdf`](https://crates.io/crates/printpdf): escritura nativa de PDF con texto real, fuentes embebidas y subsetting, capas y bookmarks; al importar SVG internamente usa `svg2pdf` (mismo problema de texto). No soporta gradientes ni patterns.

### 6.2 Consecuencia

| Salida | Ruta | Por qué |
| --- | --- | --- |
| **Plano** (líneas, símbolos, cotas) | SVG canónico → `svg2pdf` | El plano es geometría; el texto-a-curvas es aceptable e incluso preferible (sin dependencia de fuentes del sistema) |
| **BOM y cuadro de cargas** (tablas) | `printpdf` directo con ops | Necesitas texto seleccionable, columnas alineadas, paginación y archivos pequeños; convertir 200 filas a curvas sería absurdo |
| **CSV** | `csv` crate | Sin discusión |

Esto **contradice** el «una sola ruta de dibujo para plano, lámina y pruebas» del v0.2 y del SPEC v1.0. La evidencia muestra que la ruta única solo es buena para el plano. Las tablas comparten **datos** (el mismo `Bom`/`LoadSchedule`), no renderizador.

**Delta:** §10 y §12.2 del spec: dos rutas de PDF, una sola fuente de datos. `REQ-REP-004` se ajusta para verificar texto extraíble en PDF de tablas.

---

## 7. egui como base de producto: madurez real

De [análisis del ecosistema GUI Rust](https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/02-egui.md), [comparativa 2026](https://www.pistack.xyz/posts/2026-08-25-rust-gui-frameworks-egui-iced-slint-comparison) y la [doc de eframe](https://crates.io/crates/eframe):

**A favor**
- Usado en producción por **Rerun** y **Nominal Connect** (telemetría en tiempo real; [caso publicado](https://nominal.io/blog/nominal-connect-shipping-realtime-desktop-software-with-rust-bevy-and-egui)).
- AccessKit integrado; en eframe el adaptador nativo viene activado por defecto (Windows UIA, macOS NSAccessibility, Linux AT-SPI).
- IME mantenido activamente (composición visual desde 0.35).
- WASM funciona; mismo código.

**En contra (y cómo se mitiga)**
| Limitación | Mitigación |
| --- | --- |
| No hay **menú nativo** del sistema; egui dibuja el suyo | Aceptable para esta app; menú in-window coherente con el lienzo |
| No hay bandeja de sistema | No la necesitamos |
| Accesibilidad web limitada | MVP es escritorio; WASM queda como «experimental» |
| RTL/bidi débil | Español únicamente (Q-03) |
| WASM pesa 0,6–2,6 MB según versión | No es objetivo del MVP |
| Debug lento (todo se recalcula por frame) | `[profile.dev.package."*"] opt-level = 2`, `release` con `thin` LTO, `codegen-units = 1`, `strip` ([configuración recomendada](https://danpokhrel.github.io/tutorial/ch02-project-setup.html)) |

**Delta:** añadir el bloque de perfiles de compilación al §12.2 y una nota de empaquetado por plataforma en los hitos (M7).

---

## 8. Metodología: SDD con evidencia, no con ceremonia

GitHub Spec Kit define el flujo que hoy se considera referencia ([quickstart](https://github.github.com/spec-kit/quickstart.html)): **constitución (una vez) → specify → clarify → plan → checklist → tasks → analyze → implement → converge**. Fowler lo describe como *«la constitución son los principios inmutables que se aplican a todo cambio»* ([análisis](https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html)).

Nuestro SPEC v1.0 ya cubre `specify` + `plan`. Lo que falta, y la investigación confirma que importa:

1. **Constitución del proyecto** (1 página): los 10 invariantes + «sin requisito no hay código». Va en `CONSTITUTION.md` y se cita en cada PR.
2. **`clarify` explícito**: las 3 preguntas Q-01..Q-03 deben cerrarse antes de M2, no después.
3. **`checklist` por requisito**: los Given/When/Then ya lo son; falta la lista de verificación por hito.
4. **ADRs append-only** con plantilla [MADR](https://adr.github.io/madr/decisions/adr-template.html): contexto, opciones, decisión, consecuencias. Microsoft y Fowler coinciden: un ADR no se edita, se supersede ([Microsoft Learn](https://learn.microsoft.com/en-us/azure/well-architected/architect-role/architecture-decision-record), [Fowler](https://martinfowler.com/bliki/ArchitectureDecisionRecord.html)).
5. **Trazabilidad automática**: no una matriz mantenida a mano, sino una macro de test que declare `#[requirement("REQ-SIM-004")]` y un comando `cargo xtask coverage-req` que falle si un MUST no tiene prueba.

**Lo que NO adoptar:** el tooling de Spec Kit (slash commands, generación de artefactos por agente) no aporta a un proyecto de una persona con spec ya escrito; sí su **estructura de artefactos**, que es lo que estamos replicando.

**Delta:** añadir `CONSTITUTION.md`, `docs/adr/`, y el `xtask` de cobertura de requisitos a M0.

---

## 9. Crates confirmados (fijar en M0)

| Necesidad | Crate | Versión observada | Nota |
| --- | --- | --- | --- |
| Índice espacial | `rstar` | actual | R*-tree n-dimensional, feature `serde` disponible |
| IDs en memoria | `slotmap` | 1.1.x | O(1), claves versionadas (evita usar un id borrado) |
| IDs en archivo | `uuid` (feature `v7`) | 1.28.x | v7 ordenable por tiempo |
| Hashing | `rustc-hash` | 2.1.x | Determinista y rápido, no criptográfico |
| Serialización | `serde` + `serde_json` | 1.x | **Sin** `preserve_order` (claves ordenadas ⇒ bytes idénticos) |
| Álgebra | `nalgebra` | 0.3x | LU densa; `nalgebra-sparse` solo si > 500 nodos |
| UI | `eframe`/`egui` | 0.3x | `default-features = false` + renderer explícito |
| PDF tablas | `printpdf` | 0.12+ | Texto real, subsetting de fuentes |
| SVG→PDF | `svg2pdf` (+`usvg`) | actual | Solo para el plano |
| CSV | `csv` | 1.x | — |
| Property tests | `proptest` | 1.x | Geometría y solver |
| Snapshots | `insta` | 1.x | Informes y hallazgos DRC |
| Casos tabulados | `rstest` | 0.2x | Casos A1–A7 |
| Benchmarks | `criterion` | 0.5 | Presupuesto de rendimiento |

**Nota de disciplina:** el `Cargo.lock` se commitea desde M0; las versiones del spec son «mínimo observado», el lock es la verdad.

---

## 10. Riesgos técnicos nuevos (no estaban en el v0.2)

| # | Riesgo | Por qué aparece ahora | Mitigación |
| --- | --- | --- | --- |
| 13 | El modelo de curva con solo 2 puntos de anclaje extrapola mal en 1,5–3·In | Faltaba el punto 2,55·In | Añadirlo y calibrar `t = k/(I² − I0²)` con 3 puntos |
| 14 | Undo por comando = clase de bugs silenciosos (inversa inconsistente) | CadCraft/ShapeCraft usan snapshots porque el modelo es barato | Híbrido: comando semántico + snapshot en el stack |
| 15 | Divergencia entre simulador y catálogo real del fabricante | La norma da bandas, no curvas exactas | Reportar intervalos, nunca un instante único |
| 16 | PDF de tablas convertido a curvas (ilegible para cotizar) | Ruta única SVG→PDF | Dos rutas, texto real en tablas |
| 17 | El plano se percibe lento al importar fondo grande | Sin culling, se tesela todo | Culling desde el día 1 + presupuesto medible |
| 18 | Snap O(n) por movimiento del ratón | Sin índice espacial | `rstar` + snap como consulta de caja |
| 19 | Requisito normativo mal transcrito desde un resumen web | Fuentes secundarias, no texto oficial | Mantener `verify` y exigir ADR por cada cierre |
| 20 | El editor consume el 70 % del esfuerzo (como en CadCraft) | Subestimado en el plan | Aceptarlo en las estimaciones y recortar features antes que meses |

---

## 11. Experimentos a hacer ANTES de M1 (spikes de 1 día cada uno)

Cada spike produce un número, no una opinión. Si falla, cambia el plan.

| Spike | Qué se mide | Criterio de éxito |
| --- | --- | --- |
| **S1 · Lienzo** | egui dibujando 5 000 y 50 000 primitivas de un plano sintético, con y sin culling | 60 fps a 5 000; 30 fps a 50 000 con culling |
| **S2 · MNA** | Solver propio vs `network_analysis` en 20 circuitos generados | Diferencia < 0,1 % en todas las corrientes |
| **S3 · Curva** | Ajuste de `t = k/(I²−I0²)` a los 3 puntos ancla | Pasa por 1,13/1,45/2,55 dentro de ±5 % y reproduce el caso A1 |
| **S4 · PDF** | Plano SVG→PDF y tabla `printpdf` de 200 filas | Plano vectorial sin pérdida; tabla con texto extraíble (`pdftotext` devuelve las filas) |
| **S5 · Persistencia** | Guardar dos veces el proyecto de A1–A6 | Bytes idénticos; reabrir y re-simular da el mismo hash |
| **S6 · Empaquetado** | Build release en Linux/macOS/Windows con el perfil recomendado | Binario < 30 MB, arranque < 1 s, sin dependencias raras en Linux |

**Regla:** ningún spike se convierte en código de producción sin pasar por su `REQ`. Son de usar y tirar, en `spikes/`.

---

## 12. Deltas aplicados al SPEC v1.0

| Sección del spec | Cambio | Motivo |
| --- | --- | --- |
| §5 INV-05 | Comando + snapshot inmutable en el stack de undo | Evidencia de CADs (riesgo 14) |
| §8.2 | Añadir 2,55·In (1–60 s, In ≤ 32 A), 1 h/2 h > 63 A, `Tripped.band` | IEC 60898-1 / 61008-1 |
| §9 `REQ-DRC-001` | Separar ampacidad (310.16, columnas 60/75/90 °C) de techo 240.4(D) | Dos comprobaciones distintas |
| §9 `REQ-DRC-002` | Por `RoomKind` + `WaterPoint` a 6 ft, no «1,8 m de agua» | 210.8(A) 2023 |
| §6.2 | Añadir `WaterPoint` a geometría | Necesario para medir 6 ft |
| §10 / §12.2 | Dos rutas de PDF (SVG→PDF plano, `printpdf` tablas) | `usvg` convierte texto a curves |
| §12.1 | `rstar` para snap/hit-test; `rustc-hash`; perfil de compilación | Evidencia de rendimiento |
| §12.3 | `REQ-NFR-007` presupuesto fps/primitivas; `REQ-UX-009` culling | Riesgo 17 |
| §13.3 `REQ-SIM-001` | Oráculo `network_analysis` en pruebas | Validación independiente |
| §1 | Añadir constitución, ADRs, cobertura de requisitos y spikes | Metodología SDD |

---

## 13. Fuentes

**Rendimiento / editor**
1. BIMCraft, *canvas-acceleration-plan* — mediciones de 88 747 primitivas en egui. https://github.com/IPedrax/BIMCraft/blob/main/docs/canvas-acceleration-plan.md
2. egui discussion #7619 — 40 000 rectángulos, rendimiento y límites. https://github.com/emilk/egui/discussions/7619
3. `egui_wgpu::CallbackTrait` — render propio dentro de egui. https://docs.rs/egui-wgpu/latest/egui_wgpu/trait.CallbackTrait.html
4. CADCraft, *architecture* — comandos, R-tree, undo por snapshots, reparto engine/UI. https://github.com/storytold/cadcraft/blob/main/docs/architecture.md · https://cadcraft.org/blog/cad-in-rust
5. ShapeCraft, *architecture* — capas doc/engine/render/ui. https://github.com/jackharvest/shapecraft/blob/main/docs/architecture.md
6. MuginCAD — MVVM + undo en modelo, egui. https://github.com/Hakkology/MuginCAD

**Solver**
7. `network_analysis` — MNA/mallas en Rust. https://docs.rs/network_analysis/latest/network_analysis/
8. spice-rs, internals — arquitectura MNA, KLU/Markowitz. https://ferrite-systems.github.io/spice-rs/internals/ch17
9. `nalgebra` — LU, denso/ralo. https://nalgebra.rs/
10. `serde_json::Map` — orden de claves y `preserve_order`. https://docs.rs/serde_json/latest/serde_json/map/struct.Map.html

**Normativa**
11. ABB, *Comparison of tripping characteristics for MCB*. https://library.e.abb.com/public/114371fcc8e0456096db42d614bead67/2CDC400002D0201_view.pdf
12. Schneider, *Curvas de disparo Acti9* (1,13 / 1,45 / 2,55). https://ckm-content.se.com/ckmContent/sfc/servlet.shepherd/document/download/0691H00000FJeeBQAT
13. Doepke, *MCB handbook* — 1,13/1,45 y bandas B/C/D. https://www.doepke.de/uploads/tx_doepkeproducts/informationsmaterial/web/doepke_5962527_mcb_handbuch_web_en.pdf
14. EFICAM, *Manual de interruptores automáticos*. https://www.apiem.org/wp-content/uploads/2024/04/Manual-Interruptores-Automaticos-EFICAM-2024.pdf
15. IEC 61008-1:2024, extracto Tabla 11 (tiempos de disparo RCD). https://www.iecdown.world/iec/IEC%2061008-1-2024%20PDF.pdf
16. BEAMA, *RCD Handbook* — 0,5·IΔn no disparo; 0,3/0,15/0,04 s. https://www.beama.org.uk/static/dd77ff1b-da20-440a-bc338a083127ca46/RCD_GUIDE.pdf
17. ABB, *RCD whitepaper* — «puede disparar entre 15 y 30 mA». https://library.e.abb.com/public/fc3d92ef8da94d38a3882b0b86afb552/ELAU%20RCD%20Local%20Whitepaper%202022_9AKK108466A8987_RevA.pdf
18. NEC 210.8(A) 2023 — ubicaciones con GFCI. https://electricai.app/nec/210-8
19. NEC Tabla 310.16 (2023) — ampacidades por temperatura. https://wiresizes.com/guides/nec-table-310-16
20. NEC 240.4(D) — límites de protección de conductor pequeño. https://www.helukabel.us/HELUKABEL/Publications/Technical-Documents/Allowable-Ampacity-Tables.pdf

**Salidas y UI**
21. `svg2pdf` — conversión vectorial y limitación de texto. https://docs.rs/svg2pdf/latest/svg2pdf
22. Experiencia real SVG→PDF en Rust (texto a paths). https://parsecsreach.org/post/rust_svg_to_pdf
23. `printpdf` — texto real, subsetting, SVG embebido. https://crates.io/crates/printpdf
24. `resvg` / `usvg` — limitaciones y alcance. https://crates.io/crates/resvg
25. Estado del ecosistema GUI Rust (egui: accesibilidad, menú, WASM). https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/02-egui.md
26. Comparativa egui/iced/Slint 2026. https://www.pistack.xyz/posts/2026-08-25-rust-gui-frameworks-egui-iced-slint-comparison
27. Perfiles de compilación recomendados para egui. https://danpokhrel.github.io/tutorial/ch02-project-setup.html
28. Nominal Connect — egui en producción. https://nominal.io/blog/nominal-connect-shipping-realtime-desktop-software-with-rust-bevy-and-egui

**Metodología**
29. GitHub Spec Kit — flujo constitution/specify/clarify/plan/tasks. https://github.github.com/spec-kit/quickstart.html
30. Fowler, *Understanding SDD: Kiro, spec-kit, Tessl*. https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html
31. MADR — plantilla de ADR. https://adr.github.io/madr/decisions/adr-template.html
32. Microsoft Learn — ADRs append-only. https://learn.microsoft.com/en-us/azure/well-architected/architect-role/architecture-decision-record

**Crates**
33. `rstar`. https://docs.rs/rstar/latest/rstar/ · 34. `slotmap`. https://docs.rs/slotmap/latest/slotmap/ · 35. `uuid` (v7). https://docs.rs/uuid/latest/uuid · 36. `rustc-hash`. https://github.com/rust-lang/rustc-hash
