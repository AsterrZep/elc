# SPEC v1.0 — SDD · App eléctrica residencial en Rust

**Estado:** borrador para aprobación (base para implementación)
**Fecha:** 25 nov 2026
**Linaje:** `PDF (SPEC-v1.0 original, 2D + 3D)` → `Análisis crítico y SPEC v0.2` → **este documento (v1.0 consolidado)**

Este documento sustituye al SPEC del PDF y absorbe el análisis crítico v0.2. La diferencia con ambos es que **aquí no quedan decisiones abiertas que bloqueen el código**: cada punto dudoso está resuelto con una decisión explícita (D-xx), y lo que depende de la norma vigente está marcado `verify` con su procedimiento.

---

## 1. Cómo se usa este documento (el proceso SDD)

El proyecto se desarrolla con **Spec-Driven Development**: la especificación es el artefacto primario y el código es su consecuencia verificable.

### 1.1 Reglas de trabajo (no negociables)

1. **Sin requisito no hay código.** Todo PR referencia al menos un `REQ-*`. Código sin requisito se rechaza.
2. **Sin criterio de aceptación no hay requisito terminado.** Todo `REQ` con MUST tiene al menos un escenario *Given/When/Then*.
3. **El criterio se convierte en prueba antes de la implementación.** La prueba se nombra con el ID: `req_sim_004_sobrecarga_dispara_tras_curva`.
4. **`verify` bloquea.** Ningún número normativo se codifica hasta estar contrastado con la edición oficial; mientras tanto vive en el `NormProfile` como dato marcado `unverified` y el motor lo reporta como tal.
5. **El núcleo no depende de la UI.** Todo comportamiento exigido aquí debe poder ejecutarse desde `elec-cli` sin abrir ventana. Esto es lo que hace el SDD barato: la suite de aceptación corre en CI.
6. **Ningún "cumple la norma".** Las salidas dicen «N reglas implementadas (edición X), M hallazgos». Ver §3.4.

### 1.2 Artefactos del proceso

Inspirado en el flujo de SDD de referencia (constitución → spec → clarificar → plan → checklist → tareas → analizar → implementar), este proyecto mantiene:

| Artefacto | Ubicación | Regla |
| --- | --- | --- |
| **Constitución** | `CONSTITUTION.md` | Los 10 invariantes + las 6 reglas de §1.1. Se cita en cada PR. No se edita sin ADR |
| **Spec** | este documento | Fuente de requisitos y criterios de aceptación |
| **ADRs** | `docs/adr/NNNN-titulo.md` (plantilla MADR) | **Append-only**: una decisión no se reescribe, se supersede. Obligatorio para cerrar un `verify` y para todo cambio de stack |
| **Checklist por hito** | `docs/hitos/Mn.md` | Lista de verificación de la puerta de salida |
| **Cobertura de requisitos** | `cargo xtask coverage-req` | Falla si un `REQ` MUST no tiene prueba que lo declare |
| **Spikes** | `spikes/` | Código desechable; solo pasa a producción vía `REQ` |

La investigación que sustenta las decisiones de stack y los números normativos está en `INVESTIGACIÓN-v1 — Stack, normativa y metodología.md`.

### 1.3 Puertas de salida

Cada hito (§14) termina con **demo + suite de aceptación en verde** antes de abrir el siguiente. Un hito no se "casi termina": o pasa la puerta, o no empieza el siguiente.

### 1.4 Convenciones

| Marca | Significado |
| --- | --- |
| `MUST` / `SHOULD` / `MAY` | Obligatorio / recomendado / opcional (RFC 2119) |
| `verify` | Dato normativo pendiente de contraste con el texto oficial |
| `REQ-<ÁREA>-<nnn>` | Identificador de requisito, estable y trazable a prueba |
| `INV-nn` | Invariante probado en CI |
| `D-nn` | Decisión de diseño cerrada en este documento |

---

## 2. Decisiones

### 2.1 Decisiones cerradas (base del MVP)

| ID | Decisión | Motivo |
| --- | --- | --- |
| **D-01** | El MVP es **2D + netlist + simulación + DRC con un perfil + BOM/cuadro de cargas**. **Sin 3D.** | El 3D es un segundo motor gráfico con poco retorno para validar el producto (riesgo #8 del v0.2). Se entrega en v1.1 sobre un modelo ya validado. |
| **D-02** | **El netlist lógico es la única verdad eléctrica.** Conductos y trazado físico son vistas derivadas/anotaciones. | Elimina la ambigüedad de "dos verdades editables" (riesgo #4). |
| **D-03** | Primer perfil normativo: **NEC 2023 (NFPA 70)**, sistema residencial 120/240 V, 60 Hz, fase partida. IEC 60364 entra como segundo perfil en v1.1. | Los casos de aceptación del usuario (breaker 16 A, GFCI, AWG 14/12, 1.8 m de fregadero, 120/240 V) son NEC. La arquitectura es genérica desde el día 1 (`NormProfile` como dato). |
| **D-04** | **NEMA ≠ IEC no son perfiles equivalentes.** NEMA define *producto* (tomas, clavijas, envolventes); el código de instalación comparable a IEC 60364 es el **NEC**. El perfil se llama `NEC-2023` y declara aparte qué simbología usa (`NEMA` o `IEC`). | Evita prometer "dos normas" cuando una es catálogo de producto. |
| **D-05** | Unidades con tipos propios; geometría en **milímetros enteros (`i64`)**; habitaciones **dibujadas explícitamente**, sin detección automática. | INV-03/INV-04. La detección de habitaciones desde muros es geometría frágil. |
| **D-06** | UI: **`egui`/`eframe`** (nativo, WASM opcional después). Serialización **JSON legible** con `schema_version`. | Un solo lenguaje, lienzo propio, diffs revisables en git. |
| **D-07** | Protecciones modeladas con **banda tiempo-corriente**, no umbral instantáneo. El diferencial dispara por **desbalance neto**, con zona incierta entre 0,5·IΔn e IΔn. | Corrige los errores físicos del PDF (§1.1 del v0.2). Sin esto el simulador miente. |
| **D-08** | **Auto-cableado (Opción B) por defecto, pero solo por plantillas deterministas** para topologías enumeradas. La edición manual de conductores/hilos siempre está disponible. | La síntesis general de hilos es un problema abierto, no un detalle de UI (riesgo #5). |
| **D-09** | **Cuadro de cargas lo define el usuario** y es la base de asignación circuito↔cargas↔protección. El motor lo valida, no lo inventa. | Pedido explícito del usuario en el PDF. |
| **D-10** | Toda mutación pasa por un `Command`; el stack de undo guarda **snapshots inmutables** del proyecto (undo/redo ilimitado). La simulación **no muta** el diseño. | INV-05 / INV-08. El modelo es pequeño (< 1 MB): el snapshot es más simple y más robusto que una inversa por comando. |
| **D-11** | Reglas como datos con `rule_id`, severidad y cita; severidades `Error` / `Warning` / `Info`. La caída de tensión es `Warning` por defecto. | Riesgo legal #3: un falso "OK" es peligroso. |
| **D-12** | Alcance geográfico/normativo: **el motor no promete cumplimiento legal en ningún país**. Las enmiendas locales son datos del perfil, no código. | Copyright de NEC/IEC y variación por país. |
| **D-13** | MVP **offline-first, un solo usuario, un proyecto por archivo**. Sin nube, sin multiusuario, sin colaboración. | Recorta alcance sin perder valor. |

### 2.2 Decisiones que necesitan tu confirmación (no bloquean M0–M2)

| ID | Pregunta | Valor por defecto que asumiré si no respondes |
| --- | --- | --- |
| **Q-01** | Usuario primario: ¿instalador junior, estudiante o autoconstructor? | **Técnico en formación / instalador junior** (lenguaje didáctico, sin jerga legal). Cambia textos y plantillas, no arquitectura. |
| **Q-02** | ¿NEC 2023 como primer perfil, o prefieres IEC 60364 y cambiar los casos de aceptación a 230 V / mm²? | **NEC 2023** (D-03). |
| **Q-03** | ¿Idioma de la UI en v1.0: solo español, o español + inglés desde el inicio? | **Solo español**, con capa de textos centralizada para traducir en v1.1. |

---

## 3. Visión, usuario y límites del producto

### 3.1 Visión

Permitir que quien diseña la instalación eléctrica de una vivienda **dibuje la planta en 2D, defina el circuito lógico, lo simule y sepa si funciona, si es seguro y cuánto material cuesta**, antes de comprar nada. El plano **se valida solo**: eso es lo que ningún CAD eléctrico residencial hace bien.

### 3.2 Usuario objetivo

Persona que diseña o revisa la instalación de una vivienda (técnico en formación, instalador junior, autoconstructor avanzado) y quiere responder tres preguntas: *¿funciona?, ¿es seguro?, ¿qué material necesito?*

**No es** un sustituto del proyecto firmado por un profesional habilitado. La app no certifica.

### 3.3 Propuesta de valor diferencial (lo que justifica el producto)

1. **Simulación con física honesta:** el breaker no "dispara al instante a 1,25·In"; muestra "sobrecarga 125 %, disparo estimado en ~N min, banda incierta".
2. **¿Por qué disparó?** Traza causal en lenguaje claro, con elementos y valores implicados.
3. **DRC con cita y severidad:** cada hallazgo dice qué regla, de qué edición y cómo corregirlo.
4. **BOM y cuadro de cargas listos para cotizar**, derivados del mismo netlist que se simuló.

### 3.4 Límite legal (texto que la app mostrará siempre)

> «Se evaluaron **N reglas implementadas** del perfil **NEC 2023** (M de ellas pendientes de verificación). Se encontraron **K hallazgos**. Este informe no certifica cumplimiento normativo ni sustituye la revisión de un profesional habilitado.»

---

## 4. Límites del MVP (el corazón del documento)

### 4.1 Dentro del MVP (v1.0)

| Área | Incluido | Requisito |
| --- | --- | --- |
| **Plano 2D** | Muros con espesor real y altura, aberturas (puerta/ventana con antepecho y dintel), habitaciones dibujadas como polígonos con tipo de ambiente, plano de fondo calibrado | `REQ-GEO-001..006` |
| **Dispositivos** | Luminaria, interruptor simple, interruptor 3-way (par), toma estándar, toma GFCI, tablero con barras, breaker, diferencial (RCD/GFCI de circuito) | `REQ-DOM-010..015` |
| **Conexión** | Netlist lógico por terminales; plantillas de auto-cableado (simple, 3-way, toma, luminaria); edición manual de hilos y conductos como anotación | `REQ-NET-001..006` |
| **Simulación** | Conectividad, flujo energizado, sobrecarga con curva tiempo-corriente, diferencial por desbalance, cortocircuito, flotantes, retorno invertido, caída de tensión, reloj acelerable | `REQ-SIM-001..012` |
| **DRC** | 9 familias de reglas del perfil NEC 2023 (§9), con `rule_id`, severidad y cita | `REQ-DRC-001a..009` |
| **Reportes** | BOM (CSV + PDF), cuadro de cargas (CSV + PDF), plano (SVG + PDF) | `REQ-REP-001..005` |
| **Persistencia** | JSON con `schema_version` y migraciones desde el primer commit; undo/redo ilimitado | `REQ-IO-001..004` |
| **UI** | Editor 2D completo + modo Diseño/Simulación + paneles de hallazgos y estado | `REQ-UX-001..008` |

### 4.2 Fuera del MVP (explícitamente NO)

| Excluido | Por qué | Cuándo |
| --- | --- | --- |
| **Vista/edición 3D y exportación glTF** | Segundo motor gráfico (wgpu, cámara, picking, LOD, alertas 3D). No valida el diseño. | v1.1 |
| **Segundo perfil normativo (IEC 60364)** | Duplica tablas, símbolos, colores, curvas y casos de prueba. | v1.1 |
| **Auto-cableado general / enrutado óptimo** | Problema de síntesis NP, no de UI. Solo plantillas. | v1.x |
| **4-way, motores con arranque, cargas especiales trifásicas** | Catálogo crece sin validar el núcleo. | v1.x |
| **Detección automática de habitaciones desde muros** | Geometría frágil con decimales. | v1.x |
| **IFC/BIM completo, cálculo de cortocircuito de red de distribución, transitorios, armónicos, selectividad** | Fuera del caso residencial y del valor diferencial. | Nunca en v1 |
| **Certificación de cumplimiento, firma digital, trámites** | Riesgo legal inasumible. | Nunca |
| **Multiusuario, nube, colaboración, móvil nativo** | No es el problema. | Sin plan |
| **Cálculo de demanda y tamaño de acometida** | Valioso pero derivado; requiere factores de demanda por edición. | v1.1 |

### 4.3 Lista de recorte (qué se cae primero si hay presión de tiempo)

En este orden, y sin tocar el núcleo:

1. Plano de fondo calibrado → se sustituye por rejilla.
2. Plantillas de circuito (baño/cocina) → el usuario conecta a mano.
3. Exportación PDF → queda SVG + CSV.
4. Escenarios de carga predefinidos → se encienden cargas a mano.
5. Conductos como anotación → solo netlist.

**Nunca se recorta:** solver, curva de protección, RCD por desbalance, DRC con cita, `schema_version`, undo/redo.

### 4.4 Definición de Terminado del MVP

El MVP está terminado cuando, sin tocar código, un usuario puede:

1. Dibujar una vivienda de 2 habitaciones, baño y cocina con muros, puertas y ventanas.
2. Colocar tablero, 2 breakers, 1 diferencial, 6 tomas, 2 luminarias con 3-way y 3 interruptores.
3. Crear el cuadro de cargas y asignar cada dispositivo a un circuito.
4. Pasar a Modo Simulación y obtener: flujo energizado correcto, el 3-way funcionando desde ambos interruptores.
5. Provocar la sobrecarga del caso A1 y la fuga del caso A2 (§18) y ver el disparo **con la explicación causal**.
6. Ver los hallazgos DRC de las 9 familias, cada uno con `rule_id`, severidad y cita.
7. Exportar BOM (CSV/PDF), cuadro de cargas (CSV/PDF) y plano (SVG/PDF) coherentes con lo simulado.
8. Cerrar, reabrir el archivo y recuperar exactamente el mismo proyecto (misma simulación, mismo BOM).

Todo lo anterior debe estar cubierto además por la suite de aceptación ejecutada contra `elec-cli` en CI.

---

## 5. Invariantes (probados en CI)

| ID | Invariante | Prueba |
| --- | --- | --- |
| **INV-01** | El netlist lógico es la única fuente de verdad eléctrica; conductos, BOM y (futuro) 3D son vistas derivadas | `inv_01_derivar_bom_no_altera_netlist` |
| **INV-02** | `core`, `sim`, `rules`, `reports` no dependen de UI, ficheros ni red; son deterministas | `inv_02_sin_dependencias_ui` + doble ejecución idéntica |
| **INV-03** | Magnitudes físicas con tipos propios; sin `f64` desnudo con significado físico en API pública | revisión + `clippy` deny en crates núcleo |
| **INV-04** | Geometría en milímetros enteros; topología sin comparaciones de decimales | `proptest` de geometría |
| **INV-05** | Toda mutación pasa por un `Command`; el undo guarda snapshots inmutables del proyecto | `inv_05_toda_mutacion_reversible` (aplicar y deshacer deja hash idéntico) |
| **INV-06** | Ningún número de norma en el código; todo en `NormProfile` | test que falla si `sim`/`rules` contienen literales normativos |
| **INV-07** | Todo hallazgo y evento de simulación cita `rule_id`/`fault_id` y elementos implicados | `insta` sobre estructuras de salida |
| **INV-08** | La simulación no modifica el modelo de diseño | hash del proyecto antes/después de simular |
| **INV-09** | Todo archivo guardado lleva `schema_version` y se abre por migraciones | corpus de archivos v0..vN en tests |
| **INV-10** | El mismo proyecto + mismo escenario ⇒ mismo resultado numérico bit a bit | `proptest` + ejecución repetida |

---

## 6. Modelo de dominio y formato de archivo

### 6.1 Unidades (crate `elec-core`)

```rust
/// Geometría: milímetros enteros (INV-04).
pub struct Millimeters(pub i64);

/// Magnitud física con constructor validado (INV-03).
pub struct Amps(f64);
pub struct Volts(f64);
pub struct Watts(f64);
pub struct Ohms(f64);

/// Identificadores estables en archivo (UUID v7), slotmap en memoria.
pub struct DeviceId(pub Uuid);
pub struct CircuitId(pub Uuid);
pub struct NetId(pub Uuid);
```

Regla: la API pública de `core`/`sim`/`rules` no expone `f64` suelto. `f64` solo dentro de métodos y en serialización.

### 6.2 Geometría

```rust
pub struct Wall { pub id: WallId, pub start: PointMm, pub end: PointMm,
                   pub thickness: Millimeters, pub height: Millimeters, pub base_z: Millimeters }

pub struct Opening { pub id: OpeningId, pub wall: WallId,
                     pub kind: OpeningKind,        // Door | Window
                     pub span: (Millimeters, Millimeters), // desde inicio del muro
                     pub sill_z: Millimeters, pub head_z: Millimeters,
                     pub blocks_conduits: bool }   // zona restringida

pub struct Room { pub id: RoomId, pub polygon: Vec<PointMm>,
                  pub kind: RoomKind,            // Bathroom | Kitchen | Living | Bedroom | Laundry | Garage | Outdoor | Other
                  pub name: String }

/// Punto de agua con posición: necesario para medir distancias de regla
/// (p. ej. GFCI a 6 ft del fregadero). Un booleano por habitación no basta.
pub struct WaterPoint { pub id: WaterPointId, pub kind: WaterKind, // Sink | Tub | Shower
                        pub position: PointMm }
```

`RoomKind` es la clave de la regla GFCI/RCD por ambiente. `blocks_conduits` habilita `REQ-DRC-008`.

### 6.3 Dispositivos y protecciones

```rust
pub enum DeviceKind {
    Luminaire { watts: Watts },
    Switch { kind: SwitchKind },              // Simple | ThreeWay{group} | FourWay{group}
    Receptacle { rating: Amps, gfci: bool, dedicated: bool },
    Panel { main_rating: Amps },
}

pub enum Protection {
    Breaker { nominal: Amps, curve: BreakerCurve },  // B | C | D
    Rcd     { i_delta_n: Milliamps, poles: u8 },     // 10 | 30 | 100 | 300
}

pub struct Circuit {
    pub id: CircuitId, pub number: u16, pub description: String,
    pub voltage: Volts, pub protection: ProtectionKind,
    pub poles: Poles,                     // L1-N | L1-L2 | L1-N-PE ...
}
```

**Conductores:**

```rust
pub enum Phase { L1, L2, L3, N, PE }       // fase partida usa L1/L2 (corrige el PDF, que solo tenía L/N/PE)
pub enum WireFunction { Phase, Neutral, ProtectiveEarth, SwitchedReturn, Traveler }

pub struct Conductor {
    pub phase: Phase, pub function: WireFunction,
    pub size: ConductorSize,               // Awg(u8) | Mm2(f64)
    pub insulation_color: ColorRef,        // por perfil o personalizado
}
```

### 6.4 Netlist (verdad única, INV-01)

```rust
/// Nodo eléctrico: agrupación de terminales equipotenciales.
pub struct Node { pub id: NodeId, pub kind: NodeKind }  // Junction | Box | DeviceTerminal(...)

/// Rama: dispositivo o protección que conecta dos nodos.
pub struct Branch { pub id: BranchId, pub device: DeviceId,
                    pub from: NodeId, pub to: NodeId }

/// Hilo lógico: conjunto de aristas que comparten función y fase.
pub struct Net { pub id: NetId, pub function: WireFunction, pub phase: Phase,
                 pub circuit: Option<CircuitId>, pub edges: Vec<EdgeId> }

pub struct Edge { pub id: EdgeId, pub net: NetId,
                  pub from: NodeId, pub to: NodeId,
                  pub conduit: Option<ConduitId>,   // anotación, no verdad
                  pub length_mm: Millimeters }
```

`Conduit` es **anotación derivada**: guarda diámetro y recorrido para BOM y para el futuro 3D, pero el solver **nunca** lo lee para decidir conectividad.

### 6.5 Proyecto, comandos y persistencia

```rust
pub struct Project {
    pub schema_version: SchemaVersion,   // INV-09
    pub units: UnitSystem,
    pub profile: ProfileRef,             // "NEC-2023@1" + hash del perfil
    pub geometry: Geometry,
    pub devices: Vec<Device>,
    pub circuits: Vec<Circuit>,
    pub netlist: Netlist,
    pub load_schedule: Vec<LoadScheduleEntry>,   // D-09
    pub conduits: Vec<Conduit>,                  // anotación
}

pub trait Command { fn apply(&self, p: &mut Project); fn undo(&self, p: &mut Project); }
```

**Formato de archivo:** `.elec.json` (JSON legible, UTF-8), con `schema_version` en la raíz y migrador `v_n → v_n+1`. Desde el primer commit existe el corpus de archivos antiguos en `tests/fixtures/migrations/`.

---

## 7. Perfil normativo como datos (INV-06)

`elec-profile` carga un `NormProfile` desde JSON incluido en el binario (`include_str!`) o desde un archivo del usuario. Esta es la pieza que hace posible el segundo perfil sin tocar código.

```rust
pub struct NormProfile {
    pub id: String,                  // "NEC-2023"
    pub edition: String,             // "2023"
    pub supply: SupplySystem,        // SplitPhase120_240 | SinglePhase230 | ...
    pub frequency_hz: u16,
    pub ampacity_table: AmpacityTable,          // por material, calibre, método
    pub correction_factors: CorrectionFactors,  // temperatura, agrupamiento
    pub breaker_curves: BTreeMap<BreakerCurve, TimeCurrentBand>,
    pub rcd_behavior: RcdBehavior,              // banda de disparo y tiempos
    pub wire_colors: ColorScheme,
    pub standard_heights_mm: BTreeMap<MountKind, Millimeters>,
    pub rules: Vec<RuleDef>,
}

pub struct RuleDef {
    pub rule_id: String,        // "NEC-2023/210.8(A)/1"
    pub severity: Severity,     // Error | Warning | Info
    pub citation: String,       // referencia parafraseada, no copiada
    pub verified: bool,         // false ⇒ la app lo muestra como "no verificado"
    pub params: serde_json::Value,
}
```

### 7.1 Reglas de oro del perfil

- **Nada de texto copiado.** `citation` es referencia + paráfrasis propia.
- **Todo dato no contrastado nace `verified: false`.** El informe final dice cuántas reglas fueron verificadas.
- **El perfil se versiona con hash.** Un proyecto guardado recuerda el hash del perfil con el que se validó: al reabrir con otro perfil, se avisa y se re-evalúa.
- **Alturas por defecto del perfil** (interruptores 1,20 m; tomas 0,30–0,40 m; tablero 1,70 m) se usan para metrado vertical del BOM y, en v1.1, para la Z del 3D.

### 7.2 Separación NEC / simbología

El perfil declara **dos cosas distintas**: `code` (reglas de instalación) y `symbol_pack` (`NEMA` | `IEC`). El usuario puede usar simbología IEC sobre reglas NEC; es una decisión de presentación, no de cumplimiento.

---

## 8. Simulación (`elec-sim`)

### 8.1 Principio

Una red resistiva con análisis nodal sobre el netlist. Nodos L1, L2, N, PE. Resistencias de tramo según calibre y longitud. De ahí salen, con el mismo código: corrientes, caída de tensión, corriente de cortocircuito y corriente de falla.

Modelo de carga por dispositivo:

| Dispositivo | Modelo |
| --- | --- |
| Luminaria | Resistencia equivalente `R = V²/P` |
| Toma con carga declarada | Carga de potencia o corriente según escenario (`LoadScenario`) |
| Interruptor | Corto (`~0 Ω`) si cerrado, abierto si no |
| 3-way / 4-way | Par de conmutadores con estado `Traveler` |
| Breaker / RCD | Elemento ideal con estado `Closed` / `Tripped` |

### 8.2 Protecciones (D-07)

**Breaker — banda tiempo-corriente, no umbral.** Por curva (valores `verify`):

| Punto de prueba | B | C | D |
| --- | --- | --- | --- |
| 1,13·In | no dispara en 1 h (2 h si In > 63 A) | ídem | ídem |
| 1,45·In | dispara en < 1 h | ídem | ídem |
| **2,55·In** | dispara en 1–60 s (In ≤ 32 A); ≤ 120 s (In > 32 A) | ídem | ídem |
| Magnético inferior | no dispara en 0,1 s a 3·In | a 5·In | a 10·In |
| Magnético superior | dispara en < 0,1 s a 5·In | a 10·In | a 20·In |

Los tres puntos térmicos (1,13 · 1,45 · 2,55·In) son los anclajes de la curva: con ellos se calibra `t = k/(I² − I0²)` y no se interpola libremente en la zona 1,5–3·In, que es donde caen las sobrecargas domésticas reales.

Entre 1,13 y 1,45·In el resultado es **incierto**: el motor simula acumulación térmica con reloj acelerable y reporta «disparo estimado en ~N min (zona incierta)». **Nunca** reporta disparo instantáneo por sobrecarga del 125 %.

**RCD — desbalance neto.** El diferencial compara `Σ I_ida − Σ I_retorno` aguas abajo (no "fugas entre tomas"). Tipo general (IEC 61008-1 / 61009-1): no dispara a 0,5·IΔn; tiempo máximo **0,3 s a IΔn, 0,15 s a 2·IΔn, 0,04 s a 5·IΔn**. Entre 0,5·IΔn e IΔn es **zona incierta** y se reporta como tal: un RCD de 30 mA puede disparar en cualquier punto entre 15 y 30 mA. Por eso el estado de disparo guarda un **intervalo**, nunca un instante único.

**Falla a tierra sin PE:** si falta PE y hay fuga a carcasa, se calcula la tensión de contacto esperada y se advierte que el diferencial **no** disparará hasta que alguien toque.

**Cortocircuito:** detección topológica (L→N o L→PE sin carga intermedia) + corriente de falla estimada por impedancia de lazo; se compara con la banda magnética de la curva. Si la impedancia es demasiado alta para el disparo magnético, es un hallazgo (`REQ-DRC-010`, v1.1: "verificación de disparo por impedancia de lazo").

### 8.3 Estados y eventos

```rust
pub enum Energization { Energized, DeEnergized, Floating }
pub enum ProtectionState {
    Closed,
    Tripped { band: (SimTime, SimTime), cause: TripCause },  // intervalo, no instante: la norma da bandas
}

pub enum FaultKind {
    ShortCircuit { between: (NodeId, NodeId) },
    Overload { circuit: CircuitId, current: Amps, nominal: Amps },
    EarthLeak { node: NodeId, milliamps: Milliamps },
    FloatingNeutral { node: NodeId },
    ReversedReturn { device: DeviceId },
    MiswiredThreeWay { group: u32 },
    MissingPe { device: DeviceId },
}
```

Cada evento cita `fault_id` + elementos (INV-07). La traza causal («¿por qué disparó?») se genera desde estos eventos, no desde la UI.

### 8.4 Reloj de simulación

`SimClock` acelerable (1×, 60×, 3600×) para que una sobrecarga del 125 % sea observable en segundos de usuario sin falsear la física. El reloj es parte de `SimState`, separado del diseño (INV-08).

---

## 9. Motor DRC y catálogo de reglas del MVP

`elec-rules` evalúa el perfil sobre el proyecto y devuelve `Finding { rule_id, severity, message, elements, fix_hint }`. DRC corre en Modo Diseño en tiempo real (sin bloquear el dibujo) y de nuevo completo al exportar.

| ID interno | Regla | Severidad por defecto | Notas |
| --- | --- | --- | --- |
| `REQ-DRC-001a` | **Ampacidad**: `Ib ≤ In ≤ Iz` con tabla por calibre, material, método y **columna de temperatura** (60/75/90 °C), más factores de corrección (temperatura ambiente, agrupamiento, carga continua 125 %) | Error | `Iz` sale de la tabla del perfil; la columna aplicable depende de los terminales (110.14(C)) |
| `REQ-DRC-001b` | **Techo de protección de conductor pequeño** (240.4(D)): 14 AWG ≤ 15 A, 12 AWG ≤ 20 A, 10 AWG ≤ 30 A | Error | Es una comprobación **distinta** de la ampacidad: puede pasar `In ≤ Iz` y fallar el techo |
| `REQ-DRC-002` | **GFCI obligatoria**: (a) por `RoomKind` — baño, lavandería, garaje, exterior, sótano sin terminar, crawl; (b) por distancia a `WaterPoint` (fregadero, bañera, ducha) ≤ 6 ft, parámetro del perfil | Error | La distancia de 6 ft (1,83 m) **solo** aplica a fregaderos/bañeras/duchas; en baño es a cualquier distancia. La edición va en el `rule_id` |
| `REQ-DRC-003` | Máximo de tomas/puntos por circuito | Warning | Parámetro del perfil |
| `REQ-DRC-004` | Caída de tensión por ramal y total | **Warning** | 3 %/5 % son recomendaciones informativas en el perfil |
| `REQ-DRC-005` | Retorno invertido / conmutación mal cableada | Error | Topológico sobre el netlist |
| `REQ-DRC-006` | Neutro flotante o luminaria sin retorno | Error | Topológico |
| `REQ-DRC-007` | Cortocircuito o fase a tierra sin PE | Error | Del solver |
| `REQ-DRC-008` | Conducto/hilo que invade vano de puerta/ventana o zona restringida | Warning | Usa `Opening.blocks_conduits` |
| `REQ-DRC-009` | Circuito sin protección asignada o protección sobredimensionada respecto al conductor | Error | Coherencia cuadro de cargas ↔ netlist |

Reglas **no** incluidas en el MVP, con ID reservado para no reutilizar identificadores:

| ID reservado | Regla | Versión |
| --- | --- | --- |
| `REQ-DRC-010` | Verificación de disparo por impedancia de lazo (cables largos que no alcanzan el disparo magnético) | v1.1 |
| `REQ-DRC-020` | Relleno de tubo derivado de dimensiones | v1.x |
| `REQ-DRC-021` | Selectividad entre protecciones | v1.x |
| `REQ-DRC-022` | Demanda y tamaño de acometida desde el cuadro de cargas | v1.1 |

---

## 10. Reportes (`elec-report`)

**Dos rutas de PDF, una sola fuente de datos.** El plano (line art) usa **SVG como formato canónico** y se convierte a PDF con `svg2pdf`; las **tablas** (BOM y cuadro de cargas) se generan directo con `printpdf` para conservar texto real, seleccionable y buscable. Motivo: `svg2pdf` pasa por `usvg`, que convierte todo texto a curvas; un BOM de 200 filas en curvas sería ilegible de copiar y pesaría de más. Las dos rutas comparten el mismo `Bom`/`LoadSchedule`.

### 10.1 BOM

| Campo | Origen |
| --- | --- |
| Mecanismos por tipo y cantidad | Dispositivos del proyecto |
| Cajas (rectangulares, octagonales) | Derivadas de nodos `Box` |
| Tubería por diámetro (m) | Conductos (horizontal + vertical por altura de montaje) |
| Cable por función, calibre y color (m) | Aristas del netlist × longitud + subidas/bajadas por altura del perfil |

Salida: `bom.csv` (UTF-8, `;` configurable) y `bom.pdf`.

### 10.2 Cuadro de cargas

Columnas fijas: `circuito`, `descripción/zona`, `cargas conectadas`, `potencia (W/VA)`, `corriente (A)`, `tensión (V)`, `FP`, `protección asignada`, `observaciones`. Refleja **lo que definió el usuario** más lo calculado. Salida `cargas.csv` + `cargas.pdf`.

### 10.3 Plano

`plano.svg` con capas separadas: arquitectura, dispositivos, conductos, hilos con marcas (p. ej. `///` para 3 hilos), etiquetas de calibre, simbología del `symbol_pack`. `plano.pdf` por conversión. Sin 3D en v1.0.

---

## 11. UX del editor (`elec-app`)

### 11.1 Modos

- **Diseño:** dibujo, colocación, conexión, cuadro de cargas, DRC en vivo. Sin estados operativos.
- **Simulación:** edición estructural congelada; se accionan interruptores, se activan cargas y se disparan protecciones. Al salir, el diseño queda intacto (INV-08).

### 11.2 Herramientas del MVP

Selección, muro (con snap a grilla, ángulos y extremos), abertura, habitación (polígono), colocación de dispositivo desde paleta, conexión lógica por terminales, conducto (anotación), etiqueta, zoom/pan, deshacer/rehacer ilimitado, capas on/off.

### 11.3 Paneles

Árbol de proyecto · Paleta de dispositivos · Cuadro de cargas editable · Hallazgos DRC (agrupables, con salto al elemento) · Estado de simulación (`Q1-C1` energizado / disparado) · Traza causal del último disparo.

### 11.4 Requisitos UX clave

| ID | Requisito |
| --- | --- |
| `REQ-UX-001` | MUST: cambio Diseño↔Simulación con un clic y sin pérdida de estado |
| `REQ-UX-002` | MUST: todo hallazgo es clicable y centra la vista en el elemento culpable |
| `REQ-UX-003` | MUST: el flujo energizado se distingue por color **y** por patrón (accesibilidad, no solo color) |
| `REQ-UX-004` | MUST: dibujar un muro completo (clic-clic) en < 3 s de interacción, con snap activo |
| `REQ-UX-005` | MUST: undo/redo ilimitado y persistente durante la sesión |
| `REQ-UX-006` | MUST: el panel de simulación muestra por circuito: corriente, % del breaker, estado |
| `REQ-UX-007` | SHOULD: zoom 10 %–800 % sin pérdida de precisión de snap (geometría entera) |
| `REQ-UX-008` | MUST: aviso visible y persistente de que la app no certifica cumplimiento (§3.4) |
| `REQ-UX-009` | MUST: culling de viewport — no se construyen shapes de elementos fuera de pantalla (requisito de rendimiento, no de estética) |
| `REQ-UX-010` | MUST: snap y hit-test resueltos por índice espacial (`rstar`), nunca por recorrido completo |

---

## 12. Arquitectura, crates y stack

### 12.1 Workspace de Rust

| Crate | Responsabilidad | Depende de |
| --- | --- | --- |
| `elec-core` | Unidades, geometría entera, modelo, netlist, `Command`, serialización versionada | — |
| `elec-profile` | `NormProfile`, cargador y validador | `core` |
| `elec-sim` | Solver nodal, curvas de protección, RCD, fallas, reloj | `core`, `profile` |
| `elec-rules` | DRC: reglas del perfil → `Finding` | `core`, `profile`, `sim` |
| `elec-report` | BOM, cuadro de cargas, SVG, CSV, PDF | `core`, `profile`, `rules` |
| `elec-cli` | `validate`, `simulate`, `report`; ejecuta la suite de aceptación | todos |
| `elec-app` | UI egui: lienzo 2D, paneles, modos | todos |
| `elec-3d` | *(v1.1)* escena derivada + glTF | `core`, `rules` |

### 12.2 Stack y herramientas

| Tema | Elección | Alternativa descartada | Motivo |
| --- | --- | --- | --- |
| UI | `egui` + `eframe` | Tauri + web | Un lenguaje, lienzo con `Painter`, WASM futuro gratis |
| Álgebra | `nalgebra` (LU densa) | `faer` | Decenas de nodos |
| Serialización | `serde` + JSON | RON / SQLite | Diffs legibles en git |
| IDs | UUID v7 en archivo, `slotmap` en memoria | `ulid` | Estables al guardar, rápidos en RAM |
| Vectorial (plano) | SVG propio → PDF con `svg2pdf` | `printpdf` directo | Línea con fidelidad vectorial; texto-a-curvas es aceptable en un plano |
| PDF (tablas) | `printpdf` directo con ops | SVG → PDF | Texto real, extraíble y con paginación |
| Índice espacial | `rstar` (R*-tree) | recorrido lineal | Snap y hit-test O(log n) en cada frame |
| Hashing | `rustc-hash` (`FxHashMap`) | `std` SipHash | Determinista y rápido en mapas internos |
| Pruebas | `proptest`, `insta`, `rstest` | — | Propiedades del solver, snapshots de informes, casos tabulados |
| CLI/errores | `clap`, `thiserror` | `anyhow` en núcleo | Errores tipados en librerías |
| CI | GitHub Actions: `fmt`, `clippy -D warnings`, `test`, `insta --check` | — | Puerta automática por hito |

**Versiones exactas:** se fijan al abrir el repo (`cargo add` + `Cargo.lock` commiteado). Lo que este spec decide es la **categoría**, no el número.

**Perfil de compilación** (egui recalcula todo por frame; en `dev` es inusable):

```toml
[profile.dev.package."*"]
opt-level = 2          # dependencias optimizadas, tu código compila rápido

[profile.release]
opt-level = 2
lto = "thin"
codegen-units = 1
strip = "symbols"
```

### 12.3 Requisitos no funcionales

| ID | Requisito |
| --- | --- |
| `REQ-NFR-001` | MUST: `elec-cli validate` sobre un proyecto residencial típico (< 200 dispositivos) en < 200 ms |
| `REQ-NFR-002` | MUST: simulación de un escenario completo en < 50 ms para respuesta interactiva |
| `REQ-NFR-003` | MUST: núcleo 100 % offline; ningún acceso a red en `core/sim/rules/reports` |
| `REQ-NFR-004` | MUST: compila en Linux, macOS y Windows (CI en los tres) |
| `REQ-NFR-005` | MUST: determinismo bit a bit (INV-10) |
| `REQ-NFR-006` | SHOULD: apertura de proyectos de hasta 50 MB en < 1 s |
| `REQ-NFR-007` | MUST: 60 fps con 5 000 primitivas de lienzo y 30 fps con 50 000, con culling de viewport; medido con `criterion` sobre un fixture generado |

---

## 13. Requisitos por área (con criterios de aceptación)

Formato: solo se listan los MUST con escenario; el resto vive en el catálogo completo (`docs/requirements/`).

### 13.1 Geometría

| ID | Requisito | Aceptación |
| --- | --- | --- |
| `REQ-GEO-001` | Muros con espesor y altura reales en mm enteros | Given un muro de 3000 mm × 150 mm, When se mide su área, Then = 450 000 mm² exactos |
| `REQ-GEO-002` | Aberturas con antepecho y dintel que bloquean conductos | Given una ventana `blocks_conduits`, When un conducto la cruza, Then `REQ-DRC-008` |
| `REQ-GEO-003` | Habitaciones explícitas con `RoomKind` | Given un baño, When se evalúa DRC, Then exige GFCI en todas sus tomas (`REQ-DRC-002`) |
| `REQ-GEO-007` | `WaterPoint` con posición (fregadero, bañera, ducha) | Given un fregadero y una toma a 1,5 m, When se evalúa DRC, Then exige GFCI (≤ 6 ft) y a 2,5 m no |
| `REQ-GEO-004` | Snap a grilla, ángulos y extremos | Given dos muros cercanos, Then el extremo se une sin gap decimal |
| `REQ-GEO-005` | Plano de fondo calibrado por cota conocida | Given una imagen y dos puntos con distancia real, Then las coordenadas coinciden ±1 mm |
| `REQ-GEO-006` | Área de habitación calculada del polígono entero | Given un rectángulo 4000×3000, Then 12,0 m² |

### 13.2 Dominio y conexión

| ID | Requisito | Aceptación |
| --- | --- | --- |
| `REQ-DOM-010` | Catálogo MVP: luminaria, interruptor simple/3-way, toma, toma GFCI, tablero, breaker, RCD | Given el catálogo, Then las 7 familias existen y serializan |
| `REQ-DOM-011` | `Phase` incluye L1, L2, L3, N, PE | Given servicio de fase partida, Then existen circuitos L1-N y L1-L2 |
| `REQ-DOM-012` | Tablero con slots numerados, barra de neutro, barra de tierra y breaker principal | Given un tablero, When se asigna un breaker al slot 3, Then el cuadro de cargas lo refleja como Q3 |
| `REQ-DOM-013` | Breaker con nominal, curva, polos y estado `Closed`/`Tripped` | Given un breaker de 16 A curva C, Then su banda térmica y magnética se lee del perfil |
| `REQ-DOM-014` | RCD con `IΔn`, polos y conjunto de circuitos protegidos aguas abajo | Given un RCD de 30 mA, Then el solver suma el desbalance de todos sus circuitos descendientes |
| `REQ-DOM-015` | Toma GFCI como dispositivo con protección local propia | Given una toma GFCI, Then su disparo no desenergiza el resto del circuito |
| `REQ-NET-001` | El netlist es la verdad; conductos no alteran conectividad | Given un conducto borrado, Then la simulación no cambia |
| `REQ-NET-002` | Plantilla de auto-cableado para interruptor simple | Given interruptor S1 y luminaria L1, Then se generan fase, retorno y neutro correctos |
| `REQ-NET-003` | Plantilla 3-way determinista | Given S1, S2 (par) y L1, Then 2 travelers + retorno, y la lámpara enciende desde ambos |
| `REQ-NET-004` | Edición manual de hilos y conductos siempre disponible | Given una conexión automática, Then puede corregirse a mano |
| `REQ-NET-005` | Hilos etiquetados en el plano con marcas de cantidad | Given 3 hilos en un tramo, Then el SVG muestra `///` |
| `REQ-NET-006` | Colores por perfil o personalizados | Given `NEC-2023`, Then se asignan colores por defecto y se pueden sobrescribir |

### 13.3 Simulación

| ID | Requisito | Aceptación |
| --- | --- | --- |
| `REQ-SIM-001` | Análisis nodal (MNA lineal) sobre el netlist, resuelto con LU determinista | Given una red de 10 nodos, Then coincide con la solución analítica ±0,1 % **y** con `network_analysis` usado como oráculo independiente en 20 circuitos generados |
| `REQ-SIM-002` | Conectividad y estado flotante | Given una luminaria sin neutro, Then `FloatingNeutral` |
| `REQ-SIM-003` | Sobrecarga con banda térmica y reloj | Caso A1 (§18) |
| `REQ-SIM-004` | La sobrecarga **no** es disparo instantáneo | Given 20 A en breaker 16 A curva C, Then no hay disparo en t < 1 s y el mensaje indica zona incierta |
| `REQ-SIM-005` | Diferencial por desbalance neto | Caso A2 (§18): 15 + 20 mA → 35 mA > 30 mA → dispara |
| `REQ-SIM-006` | Zona incierta del RCD | Given 20 mA en RCD de 30 mA, Then "zona incierta (0,5–1·IΔn)", sin disparo determinista |
| `REQ-SIM-007` | Cortocircuito L→N / L→PE | Given fase unida a neutro sin carga, Then `ShortCircuit` y disparo magnético con corriente estimada |
| `REQ-SIM-008` | Retorno invertido y 3-way mal cableado | Given un traveler en el nodo de neutro, Then `MiswiredThreeWay` |
| `REQ-SIM-009` | Caída de tensión por tramo y acumulada | Given 30 m de 14 AWG con 12 A, Then la caída coincide con `2·L·I·ρ/S` ±2 % |
| `REQ-SIM-010` | Falla a tierra sin PE → tensión de contacto | Given fuga a carcasa sin PE, Then se reporta la tensión esperada y el aviso |
| `REQ-SIM-011` | Traza causal del disparo | Given un disparo, Then un texto con circuito, corriente, límite y elementos |
| `REQ-SIM-012` | La simulación no muta el diseño | Hash igual antes/después (INV-08) |

### 13.4 DRC, reportes, E/S

| ID | Requisito | Aceptación |
| --- | --- | --- |
| `REQ-DRC-*` | Las 9 familias de reglas de §9, con `rule_id`, severidad y cita | Given cada caso canónico de `tests/acceptance/drc/`, Then el `Finding` esperado (snapshot `insta`) |
| `REQ-DRC-011` | Severidad configurable por perfil | Given caída de tensión al 4 %, Then `Warning`, no `Error` |
| `REQ-REP-001` | BOM con mecanismos, cajas, tubería y cable por función/calibre/color | Given el proyecto de §4.4, Then el BOM cuadra con el conteo manual |
| `REQ-REP-002` | Metrado vertical según alturas del perfil | Given un toma a 0,30 m en muro de 2,70 m, Then +2,40 m de bajada contabilizados |
| `REQ-REP-003` | Cuadro de cargas con las columnas fijas de §10.2 | Given 3 circuitos, Then 3 filas + totales |
| `REQ-REP-004` | Exportación CSV, SVG y PDF | Given `elec-cli report --all`, Then 5 archivos válidos y abribles |
| `REQ-REP-005` | El informe declara reglas evaluadas, verificadas y hallazgos (§3.4) | Given cualquier exportación, Then el encabezado legal aparece |
| `REQ-IO-001` | Guardar/abrir con `schema_version` | Given un archivo v1, Then se abre en vN por migración |
| `REQ-IO-002` | Undo/redo ilimitado | Given 100 comandos, Then 100 undos restauran el hash original |
| `REQ-IO-003` | El proyecto recuerda perfil y hash | Given reapertura con otro perfil, Then aviso y re-evaluación |
| `REQ-IO-004` | Determinismo del archivo (orden estable de claves y colecciones) | Given guardar dos veces, Then bytes idénticos |

---

## 14. Hitos con puerta de salida

Estimaciones orientativas para una persona; sirven para dimensionar el alcance, no como compromiso.

| Hito | Contenido | Demo / puerta de salida | Est. |
| --- | --- | --- | --- |
| **M0 · Esqueleto** | Workspace, CI, `elec-cli` vacío, `schema_version`, migrador, `NormProfile` cargable | `cargo test` verde en 3 SO; `elec-cli --version`; CI bloquea PR sin `REQ` | 1 sem |
| **M1 · Núcleo + editor 2D mínimo** | Unidades, geometría entera, modelo, comandos, JSON; UI con muros, aberturas, habitaciones, dispositivos, undo | Dibujar y guardar/reabrir una vivienda idéntica (hash igual) | 2–3 sem |
| **M2 · Netlist** | Terminales, nets, plantillas simple/3-way/toma, edición manual, etiquetas | Conectar interruptor→luminaria y ver el netlist derivado en CLI | 2 sem |
| **M3 · Solver** | Análisis nodal, energización, caída de tensión | Encender/apagar y ver flujo y caída; `REQ-SIM-001/002/009` en verde | 2 sem |
| **M4 · Protecciones y fallas** | Curvas, RCD, cortocircuito, reloj, traza causal | Casos A1–A6 (§18) en verde; el 125 % **no** dispara al instante | 2–3 sem |
| **M5 · DRC** | 9 familias de reglas, severidades, citas, `verify` | Proyecto con 9 hallazgos inducidos, cada uno con su `rule_id` | 2 sem |
| **M6 · Reportes** | BOM, cuadro, SVG, CSV, PDF, metrado vertical | Los 5 archivos exportados y coherentes con lo simulado | 1–2 sem |
| **M7 · Cierre MVP** | UX pulida, accesibilidad de color, textos legales, corpus de migraciones, documentación | **§4.4 completo de punta a punta** | 1–2 sem |
| **— MVP v1.0 —** | | | **~13–17 sem** |
| **M8 · v1.1: 3D** | Extrusión desde 2D, alturas del perfil, tuberías, hilos, alertas, glTF | Ver la vivienda en 3D con la falla resaltada | 4–6 sem |
| **M9 · v1.1: IEC 60364** | Segundo `NormProfile` completo + suite de aceptación propia | Los mismos casos con 230 V / mm² | 3–4 sem |
| **M10 · v1.x** | Plantillas de circuito, escenarios de carga, demanda de servicio, impedancia de lazo | — | — |

**Regla de puerta:** si M4 no cierra sus casos, no se empieza M5. El MVP nunca se declara "terminado" con un caso de aceptación desactivado.

---

## 15. Riesgos y mitigaciones (heredados y actualizados)

| # | Riesgo | Gravedad | Mitigación en este spec |
| --- | --- | --- | --- |
| 1 | Alcance del MVP | Alta | §4.2/§4.3 con lista de recorte y puertas por hito |
| 2 | Mezcla IEC/NEMA | Alta | D-03/D-04: un perfil, terminología separada, `NormProfile` como dato |
| 3 | Responsabilidad legal | Alta | §3.4, `verified: false`, citas parafraseadas, IDs con edición |
| 4 | Fuente de verdad ambigua | Alta | D-02 + INV-01 |
| 5 | Auto-cableado como default "mágico" | Alta | D-08: solo plantillas deterministas enumeradas |
| 6 | Simulación `I=P/V` | Media | §8.1 solver nodal |
| 7 | Geometría de habitaciones | Media | D-05: habitaciones explícitas, mm enteros |
| 8 | 3D "ligero pero bonito" | Media | D-01: fuera del MVP |
| 9 | Tablas de producto con copyright | Media | Perfil propio en datos, campos editables, sin texto copiado |
| 10 | Usuario sin definir | Media | Q-01 con valor por defecto |
| 11 | Formato sin versionado | Baja | `schema_version` + migraciones desde M0 (INV-09) |
| 12 | Física simplificada presentada como verdad | Alta | Bandas y zonas inciertas explícitas en la salida (D-07) |

---

## 16. Registro `verify` (pendiente de contraste con fuente oficial)

Ninguno de estos datos se implementa como "verificado" hasta tener la edición oficial a mano.

| Dato | Valor usado provisionalmente | Dónde |
| --- | --- | --- |
| Banda térmica de breaker (IEC 60898-1 / UL 489) | 1,13·In no dispara en 1 h; 1,45·In dispara en < 1 h; **2,55·In dispara en 1–60 s** (In ≤ 32 A); 2 h si In > 63 A | §8.2 |
| Bandas magnéticas B/C/D | 3–5 / 5–10 / 10–20·In; no disparo en 0,1 s en el límite inferior | §8.2 |
| Disparo de RCD | no dispara < 0,5·IΔn; **0,3 s a IΔn, 0,15 s a 2·IΔn, 0,04 s a 5·IΔn** (tipo general); tipo A ×1,4 | §8.2 |
| Alcance de GFCI por ambiente (NEC 210.8) | baño (todas), cocina (superficies), 6 ft de fregadero/bañera/ducha, lavandería, exterior, garaje, sótano sin terminar, crawl, boathouse; tomas 125–250 V | §9 / `REQ-DRC-002` |
| Ampacidad y factores de corrección (NEC 310.16 / 310.15) | cobre 60/75/90 °C: 14 AWG 15/20/25; 12 AWG 20/25/30; 10 AWG 30/35/40 | §7 / `REQ-DRC-001a` |
| Techo de protección 240.4(D) | 14 AWG 15 A; 12 AWG 20 A; 10 AWG 30 A; 18 AWG 7 A; 16 AWG 10 A | §9 / `REQ-DRC-001b` |
| Caída de tensión 3 %/5 % | informativo, no obligatorio | §9 / `REQ-DRC-004` |
| Alturas de montaje | interruptor 1,20 m; toma 0,30–0,40 m; tablero 1,70 m | §7.1 |
| Catálogo de productos / símbolos NEMA y IEC | paquete propio simplificado | §7.2 |

**Procedimiento `verify`:** (1) obtener texto oficial de la edición; (2) escribir un ADR corto con la cita parafraseada y el número; (3) añadir el campo al JSON del perfil con `verified: true` y su `rule_id`; (4) añadir el caso de aceptación; (5) el commit cierra el `verify`.

---

## 17. Trazabilidad

| Crate | Requisitos que lo prueban |
| --- | --- |
| `elec-core` | `REQ-GEO-*`, `REQ-DOM-*`, `REQ-IO-*`, INV-03/04/05/09 |
| `elec-profile` | §7, `REQ-DRC-011`, INV-06 |
| `elec-sim` | `REQ-SIM-*`, INV-08/10 |
| `elec-rules` | `REQ-DRC-001..009`, `REQ-DRC-011`, INV-07 |
| `elec-report` | `REQ-REP-*`, INV-01 |
| `elec-app` | `REQ-UX-*` |
| `elec-cli` | toda la suite de aceptación (§18) como caso ejecutable |

Convención de nombres de prueba: `req_<área>_<número>_<descripción_corta>` y etiqueta `#[requirement("REQ-SIM-004")]` (macro propia en `elec-testkit`) para que la suite pueda listar cobertura por requisito.

---

## 18. Casos de aceptación canónicos

Base común: perfil `NEC-2023@1`, servicio 120/240 V, 60 Hz.

**A1 · Sobrecarga con curva (corrige el PDF).**
Given un circuito C2 con breaker 16 A curva C y tres tomas declaradas 8 A, 7 A y 5 A; When se activan las tres en Modo Simulación; Then la corriente es 20 A (125 % de In), **no** hay disparo instantáneo, el estado es «sobrecarga: disparo estimado en ~N min (zona incierta 1,13–1,45·In)» y, al avanzar el reloj más allá del tiempo estimado, el breaker pasa a `Tripped` con `TripCause::Overload`.

**A2 · Fuga a tierra y diferencial (corrige el PDF).**
Given el mismo circuito protegido por un RCD de 30 mA, con `EarthLeak` de 15 mA en la toma 1 y 20 mA en la toma 2; When se simula; Then la suma de fugas aguas abajo es 35 mA > IΔn y el RCD dispara dentro del tiempo máximo; y Given solo 20 mA, Then se reporta «zona incierta (0,5–1·IΔn)» sin afirmar disparo.

**A3 · Cortocircuito.** Given L1 unida a N sin carga; Then `ShortCircuit`, corriente de falla estimada por impedancia de lazo y disparo magnético si supera la banda de la curva; si no, hallazgo de impedancia excesiva (informativo en MVP).

**A4 · Neutro flotante.** Given una luminaria sin neutro; Then `FloatingNeutral` y `REQ-DRC-006`.

**A5 · Caída de tensión.** Given 30 m de 14 AWG con 12 A; Then caída calculada analíticamente ±2 %, severidad `Warning`, y el total ramal+acometida comparado con 3 %/5 %.

**A6 · 3-way.** Given dos interruptores 3-way y una luminaria; Then enciende desde ambos, y el SVG marca 2 travelers; Given un traveler mal conectado a neutro, Then `MiswiredThreeWay`.

**A7 · Persistencia.** Given el proyecto de A1–A6; When se guarda, se cierra, se reabre y se re-simula; Then hash idéntico y mismo BOM (INV-05/09/10).

---

## 19. Qué hay que hacer ahora (arranque concreto)

1. Confirmar **Q-01, Q-02, Q-03** (§2.2). Sin ellas se asume el valor por defecto y se sigue.
2. Escribir `CONSTITUTION.md` con los 10 invariantes y montar `docs/adr/` con la plantilla MADR.
3. Ejecutar los **6 spikes de validación** (`INVESTIGACIÓN-v1` §11). Cada uno entrega un número; si un spike falla, se ajusta el plan antes de escribir el núcleo.
4. Crear el workspace con los 7 crates de §12.1 y CI de M0 (incluido `cargo xtask coverage-req`).
5. Escribir `profiles/nec-2023.json` con los datos de §7 marcados `verified: false` y el registro `verify` de §16.
6. Convertir §18 en `tests/acceptance/` ejecutables contra `elec-cli` **antes** de implementar el solver (las pruebas fallan; ese es el punto).
7. Abrir un ADR por cada `verify` que se cierre.

---

### Anexo · Glosario mínimo

| Término | Significado en este proyecto |
| --- | --- |
| **Netlist** | Grafo lógico de nodos, ramas e hilos; única verdad eléctrica |
| **Conducto** | Anotación física de tubería/canalización; no afecta conectividad |
| **DRC** | Design Rule Check: motor de reglas normativas y de coherencia |
| **NormProfile** | Archivo de datos con tensión, tablas, curvas, colores, alturas y reglas |
| **Finding** | Resultado de una regla: `rule_id`, severidad, elementos, sugerencia |
| **Zona incierta** | Rango donde la norma no garantiza ni prohíbe el disparo; se reporta como tal |
| **`verify`** | Dato normativo no contrastado con la edición oficial |
