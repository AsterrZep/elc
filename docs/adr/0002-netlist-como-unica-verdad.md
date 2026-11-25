# 0002 — El netlist lógico es la única verdad eléctrica

- **Estado:** aceptado
- **Fecha:** 2026-11-25

## Contexto

El spec original permitía editar conductos dibujados y conexiones lógicas a la vez. Dos fuentes
de verdad editables se contradicen: el mismo circuito puede estar conectado de dos formas
distintas según qué vista se mire.

## Opciones consideradas

1. Conductos como verdad y netlist derivado.
2. Netlist como verdad y conductos como anotación.
3. Ambas editables, con reconciliación.

## Decisión

La opción 2: el netlist manda (INV-01). Los conductos guardan diámetro y recorrido para el BOM y
el futuro 3D, pero **el solver nunca los lee para decidir conectividad**.

## Consecuencias

- Borrar un conducto no cambia la simulación; borrar una arista del netlist sí.
- El BOM deriva longitudes de aristas + alturas del perfil, no del dibujo de tubería.
- La reconciliación (opción 3) queda descartada: era el mayor riesgo de coherencia del proyecto.
- `INV-01` se prueba con `inv_01_derivar_bom_no_altera_netlist`.
