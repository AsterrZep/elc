# Constitución de elc

Principios que se aplican a **todo** cambio, sin excepción. Si un cambio los viola, se rechaza o
se supersede el principio con un ADR.

## Reglas de trabajo

1. **Sin requisito no hay código.** Todo PR referencia al menos un `REQ-*` del spec.
2. **Sin criterio de aceptación no hay requisito terminado.** Todo `REQ` MUST tiene escenario
   *Given/When/Then*, y ese escenario es una prueba.
3. **La prueba existe antes que la implementación.** Se nombra con su ID:
   `req_sim_004_sobrecarga_no_dispara_al_instante`.
4. **`verify` bloquea.** Ningún número normativo se codifica hasta contrastarlo con la edición
   oficial; mientras tanto vive en el `NormProfile` con `verified: false` y se reporta como tal.
5. **El núcleo no depende de la UI.** Todo comportamiento exigido se ejecuta desde `elec-cli`
   sin abrir ventana.
6. **Ningún «cumple la norma».** Las salidas dicen «N reglas implementadas (edición X),
   M hallazgos» y llevan el aviso de que no sustituyen a un profesional.

## Invariantes

| ID | Invariante |
| --- | --- |
| INV-01 | El netlist lógico es la única fuente de verdad eléctrica; conductos, BOM y (futuro) 3D son vistas derivadas |
| INV-02 | `core`, `sim`, `rules`, `reports` no dependen de UI, ficheros ni red; son deterministas |
| INV-03 | Magnitudes físicas con tipos propios; no hay `f64` desnudo con significado físico en la API pública |
| INV-04 | Geometría en milímetros enteros (`i64`); la topología no compara decimales |
| INV-05 | Toda mutación pasa por un `Command`; el undo guarda snapshots inmutables del proyecto |
| INV-06 | Ningún número de una norma está en el código; vive en el `NormProfile` (datos) |
| INV-07 | Todo hallazgo y evento de simulación cita `rule_id`/`fault_id` y los elementos implicados |
| INV-08 | La simulación no modifica el modelo de diseño; su estado vive en `SimState` |
| INV-09 | Todo archivo guardado lleva `schema_version` y se abre mediante migraciones |
| INV-10 | Mismo proyecto + mismo escenario ⇒ mismo resultado numérico bit a bit |

## Límites del producto

- El MVP es 2D. El 3D no entra en v1.0.
- Un solo perfil normativo en v1.0 (NEC 2023). El segundo exige un ADR y su propia suite.
- La app **no certifica** cumplimiento legal en ningún país.
- Licencia PolyForm Noncommercial 1.0.0: uso comercial prohibido sin licencia aparte.
