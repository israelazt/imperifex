# Imperifex: plan (2026-11-25)

> Estado: **Bloque 1 en diseño**. Las decisiones marcadas con **(D)** esperan confirmación del autor.

## Qué es

La terminal de desarrollo de la familia Fexarium, en Rust. Es la base sobre la que se construye todo lo demás: el autor la usará cada día para trabajar en las otras apps.

## Objetivo de la primera fase

Antes de escribir la terminal, **medir Warp** en esta máquina. Esos números son la vara: Imperifex tiene que igualarlos o mejorarlos, y se miden con la misma herramienta y el mismo protocolo. La herramienta de medida es parte del repo, así que luego mide Imperifex sin cambiar nada.

## Máquina de referencia

| | |
|---|---|
| CPU | AMD Ryzen 5 3600X (6 núcleos, 12 hilos) |
| GPU | NVIDIA GeForce GTX 1660 Ti |
| Sistema | Windows 11 Pro 10.0.26200 |
| Warp | v0.2026.11.11.08.17.stable_01 |
| Monitor | _pendiente: resolución y Hz (la latencia depende de ellos)_ |

## Qué se mide

| # | Medida | Cómo | Se informa |
|---|---|---|---|
| M1 | **Arranque** | La herramienta lanza la terminal y espera a su ventana y a que deje de cambiar (captura de pantalla) | ms en frío (sin la app abierta) y en caliente (ventana nueva), mediana de 10 |
| M2 | **Latencia de tecla** | Envía una tecla a la ventana (`SendInput`) y vigila un trozo de pantalla con la duplicación de escritorio (DXGI) hasta que cambia, como Typometer | p50, p95 y p99 de 200 pulsaciones, en ms |
| M3 | **Salida** | `imperifex-bench` dentro de la terminal escribe cargas fijas y mide cuánto tarda en salir todo (la consola frena la escritura si la terminal no da abasto) | MB/s y segundos por carga |
| M4 | **Memoria** | Memoria del proceso: en reposo, tras 1 millón de líneas en el historial y por pestaña más | MB de memoria privada y de trabajo |
| M5 | **CPU y GPU en reposo** | 60 s sin hacer nada, con el cursor parpadeando | % de un núcleo y % de GPU (contadores de Windows) |
| M6 | **CPU durante la salida** | Lo mismo durante M3 | % de núcleo medio y pico |

Cargas de M3: texto ASCII (100 MB), líneas largas sin salto, mucho color (secuencias SGR), Unicode y emojis, movimiento del cursor (lo que hacen las barras de progreso), y una salida real: un `cargo build` de este repo grabado y reproducido.

## Primera medida (sin herramienta, 2026-11-25)

Warp abierto 46 minutos, con esta sesión de Claude Code trabajando dentro (no es reposo):

| Proceso | Memoria de trabajo | Memoria privada | Hilos | Handles |
|---|---|---|---|---|
| `warp.exe` (principal) | 303 MB | 576 MB | 80 | 1569 |
| `warp.exe` (minidump-server) | 99 MB | 4 MB | 3 | 201 |

CPU del proceso principal: 453 ms en 10 s (**4,5 % de un núcleo**) mientras esta sesión escribía; 132 s de CPU en 46 minutos (4,8 % de media). La medida limpia de reposo es M5.

## Decisiones

| # | Decisión | Propuesta |
|---|---|---|
| D1 (D) | La herramienta de medida vive en este repo como `apps/imperifex-bench` y mide cualquier terminal | Sí: la misma vara para Warp y para Imperifex |
| D2 (D) | ¿Qué cargas importan más? | Las del uso real del autor: Claude Code, `cargo build`, `git`, registros largos |
| D3 (D) | M1 en frío necesita Warp cerrado: esta sesión corre dentro de Warp, así que esa medida la lanza el autor desde otra terminal | Sí |
| D4 (D) | ¿Medir también Windows Terminal como segunda referencia? | Sí: cuesta poco y dice si un número de Warp es bueno o malo |
| D5 (D) | ¿Repositorio en GitHub? | Cuando el autor lo cree |
| D6 (D) | Qué funciones de Warp necesita Imperifex (bloques, IA, pestañas, paneles…) | Se decide después de la línea base, en su propio plan |

## Fuera de esta fase

- La terminal en sí: ni ventana, ni intérprete de secuencias, ni shell. Primero la vara.

## Criterios de aceptación

1. `imperifex-bench` mide M1-M6 contra Warp en esta máquina y escribe los números en un archivo (JSON) y en `docs/BASELINE.md`.
2. Dos tandas seguidas dan números parecidos (el ruido se informa: dispersión de cada medida).
3. La herramienta no deja nada abierto ni cambiado: cierra lo que abre y nunca toca la ventana de Warp en la que trabaja el autor.

## Estado

- 2026-11-25: workspace creado con el generador de la casa (capas, guardas, `tools/check.ps1`, `tools/ship.ps1`), pruebas en verde. Primera medida de memoria y CPU anotada arriba.

## Bloque 0 · Cimientos

### 0.1 Quitar el ejemplo
1. Borrar `example.rs` en `domain` y la orden `status` cuando exista la primera orden real.

**Hecho cuando:** `tools/check.ps1` en verde.

## Bloque 1 · La línea base de Warp

| Tarea | Qué | Hecho cuando |
|---|---|---|
| 1.1 | `domain`: el modelo de una medida (muestras, mediana, percentiles, dispersión) y del informe | Pruebas puras |
| 1.2 | M3: las cargas, generadas con semilla fija (siempre los mismos bytes), y su escritura cronometrada | Pruebas: cada carga da el tamaño y el hash esperados |
| 1.3 | M4-M6: muestreo de memoria, CPU y GPU de un proceso por un puerto, con su falso | Pruebas con el falso; medido en real |
| 1.4 | M1: lanzar, encontrar la ventana, esperar a que deje de cambiar | Medido en real con Warp y Windows Terminal |
| 1.5 | M2: tecla y vigilancia de pantalla por DXGI | Medido en real; se contrasta con una tecla a mano grabada a 240 fps si hace falta |
| 1.6 | Correr las dos tandas y escribir `docs/BASELINE.md` | Criterios de aceptación cumplidos |

Pausa: el autor revisa la línea base antes de diseñar la terminal.

## Riesgos

| Riesgo | Mitigación |
|---|---|
| La latencia medida incluye el monitor y la composición de Windows | Igual para todas las terminales: se compara en la misma máquina, no en absoluto |
| Warp se actualiza solo y cambia los números | La versión va en cada informe |
| Medir mientras el autor trabaja en Warp | M1 y M2 se lanzan en una ventana nueva, nunca en la de la sesión |
