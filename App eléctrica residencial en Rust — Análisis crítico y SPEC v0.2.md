# App eléctrica residencial en Rust — Análisis crítico y SPEC v0.2

Nov 25, 2026 · @Aster

## Veredicto

La idea es buena y tiene hueco real, pero el SPEC-v1.0 del PDF mezcla cuatro productos en uno (CAD 2D, simulador eléctrico, motor normativo y visor BIM 3D). Recomiendo un MVP que sea solo **plano 2D + circuito lógico + simulador + DRC con un único perfil normativo + BOM y cuadro de cargas**, y dejar el 3D y el segundo estándar para después.

- **Mantener:** motor en Rust como fuente única de verdad, separación estricta Diseño/Simulación, reglas normativas como datos configurables.
- **Cambiar:** el circuito lógico (netlist) manda, no el dibujo; un perfil normativo primero (NEC o IEC), no ambos a la vez; protecciones con curva tiempo-corriente, no un umbral instantáneo.
- **Corregir:** dos errores físicos en el spec actual (sección «Errores técnicos»).
- **Aplazar:** 3D, auto-cableado completo, edición en 3D, trifásico.

La imagen de referencia que mencionas en tu primer prompt no viene dentro del PDF; adjúntala y ajusto la UX del editor a ella.

## Puntos fuertes

El valor está en que el plano **se valida solo**: los CAD eléctricos dibujan, y casi ninguno de uso residencial te dice «esto dispara» o «esto incumple».

- **Propuesta clara:** diseñar, simular y obtener BOM en un solo flujo. Para instaladores, estudiantes y autoconstructores es un caso de uso que se entiende en una frase.
- **Rust encaja:** modelo de datos tipado (enums para estados, newtypes para unidades), solver determinista, pruebas rápidas y compilación a WASM si luego quieres versión web.
- **Separación Diseño/Simulación:** evita estados mezclados y permite congelar el modelo mientras se simula.
- **Normativa como datos:** perfiles configurables en lugar de reglas fijas en el código.
- **El caso de las fugas y la sobrecarga** que describiste (tomas de 8, 7 y 5 A en un breaker de 16 A; fugas de 15 y 20 mA) es un excelente caso de prueba de aceptación. Lo uso más abajo.
- **Salidas útiles y baratas:** CSV, SVG y PDF de un plano o una tabla son entregables que el usuario aprecia desde el primer día.

## Puntos débiles y riesgos

Los tres riesgos que pueden matar el proyecto son el alcance, la mezcla IEC/NEMA y la responsabilidad por decir que algo «cumple». Ordenados de mayor a menor gravedad:

| # | Riesgo | Por qué importa | Gravedad | Mitigación |
| --- | --- | --- | --- | --- |
| 1 | Alcance del MVP | CAD 2D + solver + DRC + BOM + 3D + exportación en v1 son meses de trabajo cada uno; el editor de planos solo (selección, snap, zoom, deshacer) ya es grande | Alta | Hitos con puerta de salida; 3D y segundo estándar fuera del MVP |
| 2 | «IEC y NEMA» no son dos perfiles equivalentes | NEMA define tomas, clavijas y envolventes; el código de instalación comparable a IEC 60364 es el NEC (NFPA 70). Cambian tensión (120/240 V 60 Hz vs 230 V 50 Hz), AWG vs mm², colores, curvas de breaker, GFCI vs RCD y símbolos | Alta | Un perfil primero; arquitectura con `NormProfile` como dato para añadir el otro después |
| 3 | Responsabilidad legal de «cumple la norma» | Un falso «OK» en un diseño real es un riesgo de seguridad. Además los textos del NEC y de IEC tienen copyright y cada país tiene enmiendas locales | Alta | Decir «reglas implementadas: N», no «cumple»; cada regla con ID, cita y versión; aviso de que no sustituye a un profesional; reglas parafraseadas, no copiadas |
| 4 | Fuente de verdad ambigua | El spec habla de dibujar conductos y de conexión lógica a la vez; si ambos son editables hay dos verdades que se contradicen | Alta | El **netlist lógico** es la verdad; el trazado físico (conductos) es una vista derivada o anotada |
| 5 | Auto-cableado como opción por defecto | Decidir cuántos hilos van por tramo (neutro en el interruptor, retornos de 3-way, relleno de tubo) es un problema de síntesis, no un detalle de UI | Alta | Plantillas deterministas para pocas topologías (simple, 3-way, 4-way, toma); edición manual siempre posible |
| 6 | Simulación con I = P/V por circuito | No detecta caída de tensión real, corriente de cortocircuito esperada ni impedancia del lazo de falla | Media | Solver de red resistiva (análisis nodal) sobre el netlist |
| 7 | Geometría de muros y habitaciones | Detectar habitaciones desde muros con grosor y huecos es geometría frágil con decimales | Media | Coordenadas enteras en mm; habitaciones dibujadas explícitamente en el MVP |
| 8 | 3D embebido «ligero pero bonito» | Es un segundo motor gráfico (wgpu, cámara, picking, capas de alertas, glTF) con poco retorno para validar el diseño | Media | Posponer; si se hace, geometría extruida simple y glTF como exportación |
| 9 | Tablas de producto y de relleno de tubo | El BOM real exige diámetros, calibres y factores de relleno; parte de esas tablas es texto con copyright | Media | Catálogo propio en datos (RON/JSON) con campos editables; derivar el relleno de las dimensiones |
| 10 | Usuario objetivo sin definir | Un instalador, un estudiante y un autoconstructor piden UX y lenguaje legal distintos | Media | Elegir uno para v1 (ver decisiones abiertas) |
| 11 | Formato de archivo sin versionado | Cambiar el modelo rompe los proyectos guardados | Baja | `schema_version` y migraciones desde el primer día |

## Errores técnicos en el spec del PDF

Hay seis afirmaciones del PDF que, implementadas tal cual, harían que el simulador diga cosas falsas. Las cifras de normas de esta sección las escribí de memoria (no pude consultar las fuentes en esta sesión): cada una queda marcada `verify` en el spec y debe contrastarse con la edición vigente de tu país antes de codificarla.

| Qué dice el PDF | Problema | Corrección propuesta |
| --- | --- | --- |
| 8 A + 7 A + 5 A = 20 A en un breaker de 16 A, «dispara y corta» | Un termomagnético no dispara al instante por 1,25 × In. Según IEC 60898-1 no debe disparar por debajo de 1,13 × In en 1 h y debe disparar antes de 1 h a 1,45 × In (`verify`); entre ambos valores el resultado es incierto. Solo el disparo magnético es instantáneo (curvas B 3–5, C 5–10, D 10–20 × In, `verify`) | Modelar curva tiempo-corriente con banda mínima/máxima y un reloj de simulación acelerable; mostrar «Sobrecarga: dispara en \~N min» y el disparo cuando se agota el margen térmico |
| Fugas de 15 mA y 20 mA «entre tomas» dispararon el diferencial | La conclusión es correcta (el diferencial mide la suma de todas las fugas aguas abajo: 35 mA), pero una fuga no ocurre «entre tomas»: es una corriente **de un punto a tierra**. Además un RCD de 30 mA debe disparar a partir de 30 mA y no debe disparar por debajo de 15 mA (50 % de IΔn, `verify`); entre 15 y 30 mA es zona incierta | Modelar `EarthLeak { nodo, mA }`; el RCD suma las fugas aguas abajo y aplica banda de disparo y tiempo máximo por múltiplo de IΔn |
| «GFCI a menos de 1,8 m de fregaderos y baños» | Es una regla parcial. El NEC exige protección GFCI en baños a cualquier distancia y en más ubicaciones (cocina, garaje, exteriores, sótanos, lavandería; el alcance cambia por edición, `verify`). IEC 60364 lo plantea como protección adicional con RCD ≤ 30 mA en tomas de hasta 20 A y por zonas en baños | La regla se define por tipo de ambiente y perfil, con la edición de la norma en el ID de la regla |
| 15 A → 14 AWG, 20 A → 12 AWG | Es la regla de conductor pequeño del NEC en cobre, pero ignora factores de corrección por agrupamiento, temperatura y cargas continuas (125 %). En IEC la comprobación es Ib ≤ In ≤ Iz con método de instalación | Tabla de ampacidad en el perfil + factores de corrección como parámetros; mínimo por regla, calculado por circuito |
| Caída de tensión máxima «permitida» | En el NEC el 3 % (ramal) y 5 % (total) son recomendaciones informativas, no obligaciones; IEC deja los límites a cada país (`verify`) | Cada regla tiene severidad `Error`, `Warning` o `Info`; la caída de tensión por defecto es `Warning` |
| Solo L, N y PE | Un servicio residencial NEC típico es de fase partida 120/240 V (dos fases L1/L2, neutro, tierra): hay cargas de 240 V, interruptores bipolares, circuitos multifilares y neutro sobrecargado si hay desbalance | `Conductor.phase` como enum (`L1`, `L2`, `L3`, `N`, `PE`); el perfil decide qué sistema de alimentación existe |

## Cómo mejorarlo

Cinco cambios de diseño resuelven la mayoría de los riesgos anteriores, y un conjunto de funciones nuevas hace que el producto se sienta distinto a un CAD con simulador pegado.

### Cambios de diseño

1. **Netlist lógico como fuente de verdad.** El usuario conecta «interruptor A → lámpara 1»; el motor deriva los hilos por tramo. Los conductos dibujados se asocian a aristas del netlist, no al revés.
2. **Solver nodal sencillo.** Red resistiva con nodos L, N y PE (L1, L2 en fase partida), resistencias por tramo según calibre y longitud. Da corrientes, caída de tensión, cortocircuito y corriente de falla con el mismo código.
3. **Un perfil normativo por versión.** El perfil es un archivo de datos (tensión, frecuencia, tablas, reglas, colores, símbolos). Se añade el segundo cuando el primero pasa todas sus pruebas.
4. **Reglas con ID, severidad y cita.** Ejemplo: `NEC-2023/210.8/A` (`verify`), severidad `Error`. Cada resultado explica qué elemento falla y cómo corregirlo.
5. **Hitos con puerta de salida.** Cada hito termina con una demo y pruebas aceptadas antes de empezar el siguiente (ver plan al final).

### Qué entra y qué no

| Área | MVP (v1.0) | Después (v1.x) | Fuera de alcance |
| --- | --- | --- | --- |
| Plano | Muros con grosor, aberturas, habitaciones dibujadas, plano de fondo calibrado | Detección automática de habitaciones | CAD general |
| Dispositivos | Luminaria, interruptor simple y 3-way, toma, toma GFCI, tablero, breaker, diferencial | 4-way, cargas especiales, motores con arranque | Trifásico, media tensión |
| Conexión | Circuito lógico con hilos derivados; conductos como anotación | Trazado manual de conductos con relleno | Enrutado óptimo automático |
| Simulación | Conectividad, sobrecarga con curva, diferencial, cortocircuito, flotantes, caída de tensión | Corriente de cortocircuito esperada, selectividad | Transitorios, armónicos |
| Normativa | Un perfil (a elegir) | Segundo perfil | Normas por país |
| Reportes | BOM y cuadro de cargas en CSV; plano en SVG y PDF | Lámina PDF con leyenda | IFC |
| 3D | Ninguno | Visor 3D extruido y glTF | Edición en 3D |

### Funciones nuevas que valen la pena

- **¿Por qué disparó?** Traza causal en lenguaje claro («Q3 disparó porque el circuito C2 suma 20 A con los equipos ON; límite 16 A»). Es la función más didáctica y casi gratis una vez existe el solver.
- **Escenarios de carga:** presets «todo encendido», «hora pico de cocina», con factores de uso para no depender de encender cosas a mano.
- **Plano de fondo calibrado:** el usuario importa su imagen o PDF de la planta, marca una cota conocida y traza encima. Resuelve la imagen de referencia que mencionas.
- **Plantillas de circuito:** baño, cocina, A/C, iluminación de sala; cada plantilla ya trae protección y calibre por defecto.
- **Cálculo de demanda y servicio** (factores de demanda): sugiere el tamaño de la acometida y del principal desde el cuadro de cargas.
- **Verificación de disparo por impedancia de lazo:** en cables largos el cortocircuito puede no alcanzar el disparo magnético; es una regla que casi ningún CAD residencial avisa.
- **Peligro de contacto:** si falta PE y hay fuga a carcasa, mostrar tensión de contacto esperada y que el diferencial no dispara hasta que alguien toque.

## SPEC v0.2 · 1. Visión, alcance e invariantes

La app permite dibujar una vivienda en 2D, definir su instalación eléctrica como un circuito lógico, **simular** su comportamiento (cargas, protecciones, fallas) y **verificar** reglas normativas configurables, para entregar BOM y cuadro de cargas exportables. Este spec reemplaza al SPEC-v1.0 del PDF: lo recorta, lo corrige y lo vuelve verificable.

### Convenciones del spec

- Los requisitos llevan ID (`REQ-DOM-001`) y usan MUST / SHOULD / MAY con el sentido habitual.
- Cada requisito importante tiene criterios de aceptación *Given / When / Then* que se convierten en pruebas automáticas.
- `verify` marca un dato normativo escrito de memoria que debe contrastarse con el texto oficial antes de implementarlo.
- Un requisito no está terminado hasta que su prueba pasa; el código sin requisito no se acepta.

### Usuario objetivo (hipótesis a confirmar)

Persona que diseña o revisa la instalación de una vivienda (autoconstructor, estudiante o instalador junior) y quiere saber si funciona y es segura antes de comprar material. No es un sustituto del proyecto firmado por un profesional.

### No-objetivos del MVP

- Edición geométrica en 3D, IFC/BIM completo, trifásico, media tensión, cálculo de cortocircuito de red de distribución.
- Certificar cumplimiento legal. El resultado se presenta como «N reglas implementadas evaluadas, M hallazgos».
- Segundo perfil normativo (IEC o NEC, el que no se elija primero).

### Invariantes de diseño

Son propiedades que siempre deben cumplirse y que se prueban en CI.

| ID | Invariante |
| --- | --- |
| INV-01 | El netlist lógico es la única fuente de verdad eléctrica; conductos, BOM y 3D son vistas derivadas |
| INV-02 | El núcleo (`core`, `sim`, `rules`, `reports`) no depende de UI, ficheros ni red, y es determinista: misma entrada, misma salida |
| INV-03 | Magnitudes físicas con tipos propios (`Millimeters`, `Amps`, `Volts`, `Watts`, `Ohms`); no hay `f64` desnudo con significado físico en la API pública |
| INV-04 | Geometría en enteros (milímetros, `i64`); sin comparaciones de decimales para topología |
| INV-05 | Toda mutación del proyecto pasa por un `Command` reversible (deshacer/rehacer ilimitado) |
| INV-06 | Ningún número de una norma está en el código; vive en el `NormProfile` (datos) |
| INV-07 | Cada hallazgo y cada evento de simulación cita su causa: `rule_id` o `fault_id` y los elementos implicados |
| INV-08 | La simulación no modifica el modelo de diseño; su estado vive aparte (`SimState`) |
| INV-09 | Todo archivo guardado lleva `schema_version` y se abre siempre mediante migraciones |

## SPEC v0.2 · 2. Arquitectura y stack

La arquitectura es un *workspace* de Rust donde el núcleo no conoce la interfaz, de modo que todo el comportamiento se puede probar desde una línea de comandos sin abrir la app. Esto es lo que hace viable el desarrollo dirigido por especificación: los criterios de aceptación se ejecutan en CI contra `elec-cli`.

### Crates del workspace

| Crate | Responsabilidad | Depende de |
| --- | --- | --- |
| `elec-core` | Unidades, geometría entera, modelo del proyecto, netlist, `Command`, serialización versionada | — |
| `elec-profile` | `NormProfile`: tablas, reglas, colores, símbolos como datos; cargador y validador de perfiles | `core` |
| `elec-sim` | Solver nodal, curvas de protección, diferencial, fallas, reloj de simulación | `core`, `profile` |
| `elec-rules` | Motor DRC: evalúa reglas del perfil sobre el proyecto y devuelve `Finding` | `core`, `profile`, `sim` |
| `elec-report` | BOM, cuadro de cargas, plano SVG; CSV y PDF | `core`, `profile`, `rules` |
| `elec-cli` | Binario sin UI: `validate`, `simulate`, `report`; ejecuta los casos de aceptación | todos los anteriores |
| `elec-app` | Interfaz: lienzo 2D, paneles, modo Diseño/Simulación | todos los anteriores |
| `elec-3d` (hito posterior) | Escena 3D derivada del modelo y exportación glTF | `core`, `rules` |

### Decisiones de stack

Las versiones de cada librería se fijan al abrir el repo (`verify`); lo que se decide aquí es la categoría.

| Tema | Recomendación | Alternativa | Motivo |
| --- | --- | --- | --- |
| Interfaz | `egui`/`eframe` (nativo y WASM) | Tauri + front web | Un solo lenguaje, lienzo propio con `Painter`, buen punto de partida para herramientas |
| Álgebra del solver | `nalgebra` | `faer` | Sistemas pequeños (decenas de nodos): LU densa es suficiente |
| Serialización | `serde` + JSON legible | RON, SQLite | Se puede versionar con git y revisar diffs |
| IDs | UUID v7 estables en archivo, `slotmap` en memoria | `ulid` | IDs que sobreviven a guardar/abrir; acceso rápido interno |
| Salida vectorial | SVG propio como formato canónico; PDF por conversión del SVG | `printpdf` directo | Una sola ruta de dibujo para plano, lámina y pruebas de instantánea |
| Pruebas | `proptest`, `insta`, `rstest` | — | Propiedades del solver, instantáneas de informes, casos tabulados |
| 3D (posterior) | `wgpu` dentro de `eframe` y `gltf-json` para exportar | `three-d`, Bevy | Mantiene un solo ejecutable y exportación estándar |
