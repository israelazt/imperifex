# Arquitectura de Imperifex

Creada el 2026-11-25 con la arquitectura Fexarium (la de Luminifex/dockview y Purifex): núcleo puro y probable, el sistema detrás de puertos, el binario solo conecta, y el compilador y unas guardas impiden romper las reglas.

## Los crates

| Crate | Es | Contiene | No puede |
|---|---|---|---|
| `imperifex-domain` | El **qué** | Datos y reglas puras, los textos, los informes (`to_text`, `to_json`), los **puertos** (traits) y el log | Windows, disco, red, hilos, reloj |
| `imperifex-platform` | El **cómo** en Windows | Adaptadores que implementan los puertos. Lo que decodifica bytes del sistema es código portable con tests | Conocer la app |
| `imperifex-app` | El **cuándo** | Los servicios y las órdenes (`Command`), solo contra puertos; se prueban con dobles | Windows, la plataforma |
| `imperifex` (binario) | La **raíz de composición** | Crea los adaptadores reales, se los da a `app`, imprime o dibuja | Lógica |

```
              imperifex (binario)
              /        |       \
   imperifex-app         |   imperifex-platform
              \        |       /
              imperifex-domain
```

Las flechas solo van hacia abajo. `app` y `platform` no se ven entre sí.

## El banco de medidas

`imperifex-bench` mide una terminal (primero Warp, después Imperifex) con la misma herramienta y el mismo protocolo. Vive en este repo como un programa aparte, con sus propios crates: la terminal no carga con el código del banco ni al revés (`docs/PLAN.md`, D1 y D7).

| Crate | Es | Contiene | No puede |
|---|---|---|---|
| `imperifex-bench-domain` | El **qué** | Las medidas y su estadística, las cargas, el informe, los perfiles de terminal (`bench/terminals.json`), las reglas de seguridad y los **puertos** | Windows, disco, red, hilos, reloj |
| `imperifex-bench-platform` | El **cómo** en Windows | Los adaptadores de los puertos. Las sondas van en `examples/` y no son parte del producto | Conocer los servicios |
| `imperifex-bench-app` | El **cuándo** | Los servicios de cada medida y las órdenes (`Command`), solo contra puertos; se prueban con dobles | Windows, la plataforma |
| `imperifex-bench` (binario) | La **raíz de composición** | Crea los adaptadores reales, se los da a `imperifex-bench-app` e imprime | Lógica |

```
                      imperifex-bench (binario)
                 /               |                \
   imperifex-bench-app           |       imperifex-bench-platform
                 \               |                /
                      imperifex-bench-domain
                                 |
                          imperifex-domain
```

De la terminal, el banco reutiliza `imperifex-domain` (el puerto `Clock` y el log) y, solo en su binario, `imperifex-platform` (`SystemClock` y la salida del log). Los tres crates de biblioteca del banco también pueden usar `imperifex-domain` directamente. La terminal no depende de ningún crate del banco.

### Puertos del banco

En `imperifex-bench-domain` (`ports.rs`). Son pequeños, según lo que cada servicio necesita, y todos devuelven `Result<_, String>`. Se crean en la tarea del plan que los necesita; cada uno tiene un doble en `imperifex-bench-app/src/testing/` que anota lo que le pidieron y responde lo que la prueba programa.

| Puerto | Para | Tarea |
|---|---|---|
| `Clock` | Medir tiempos y esperar (el de `imperifex-domain`) | — |
| `Output` | Escribir la salida que la terminal tiene que consumir | 3.3 |
| `FileStore` | Leer y guardar archivos | 3.3 |
| `ProcessProbe` | Saber qué procesos hay y cuánto gastan | 4.1 |
| `GpuProbe` | Saber cuánta GPU usan unos procesos | 4.1 |
| `Desktop` | Abrir, encontrar, colocar, enfocar y cerrar ventanas | 5.1 |
| `ScreenWatch` | Ver cuándo cambia una región de la pantalla | 6.1 |
| `Keyboard` | Escribir en una ventana | 6.1 |
| `KeyInput` | La entrada del propio banco, tecla a tecla, para `echo` | 6.6 |
| `MachineProbe` | Datos de la máquina, versión de la terminal y fecha del informe | 4.4 |

Las órdenes reciben todos los adaptadores por `Adapters` (`app::command`): cada uno se crea solo cuando una orden lo pide. El binario es quien lo implementa.

### Reglas de seguridad del banco

El autor trabaja **dentro** de la terminal que se mide. Estas reglas no se rompen nunca:

- **S1.** La herramienta solo toca las ventanas **que ella abrió**. Antes de cada medida hace una foto de todas las ventanas. Solo **adopta** una ventana nueva si es la única que apareció tras su lanzamiento, y nunca cierra, mueve, enfoca ni escribe en otra. Sobre los procesos no hace nada: solo los lee. Lo impone `SafetyGuard`, con pruebas.
- **S2.** **Nunca** `TerminateProcess`. Se cierra con `WM_CLOSE` a la ventana propia; si no se cierra en 5 s, la herramienta lo dice y se detiene.
- **S3.** Antes de **cada** pulsación comprueba que la ventana en primer plano es la suya. Si no lo es, aborta la medida: una tecla en la ventana equivocada escribiría en la sesión del autor.
- **S4.** El arranque en frío exige que no haya ningún proceso de esa terminal. Si lo hay, se niega y explica por qué.
- **S5.** Nada de rutas personales en el repo (es público): las rutas se escriben con variables (`%LOCALAPPDATA%`) y se expanden al correr.

## Reglas

1. Lo que toca hardware, disco, red, reloj o ventana va detrás de un puerto de `domain`, con un nombre por lo que se necesita (`Clock`), no por cómo se hace. El adaptador real vive en `platform`.
2. Receta de una función nueva: reglas en `domain` con tests → puerto → servicio en `app` con un doble → adaptador en `platform` (medido en el PC real) → salida o vista → cableado en el binario.
3. Guardas (`crates/imperifex-domain/tests/architecture.rs`): dependencias permitidas por crate, ninguna fuente de más de 400 líneas (1000 en `tests.rs`), nada de `println!` salvo excepciones con su motivo, y `unsafe` solo en los dos crates de plataforma (`imperifex-platform` e `imperifex-bench-platform`).
   `unsafe` es solo para lo que la biblioteca estándar no ofrece (llamar al sistema): `domain`, `app` y los binarios lo prohíben (`#![forbid(unsafe_code)]`), y en la plataforma los lints exigen una sola operación por bloque, cada uno con su `// SAFETY:`, también dentro de una `unsafe fn`. Una dependencia que evite `unsafe` propio se decide con el autor, caso por caso.
4. Errores: `Result<_, String>` a través de los puertos; lo que el usuario debe saber sale en la salida de la orden; lo demás, a `log_info!`/`log_warn!`/`log_error!`.
5. Portátil: el crate `windows` solo en los crates de plataforma; fuera de Windows el binario dice que aún no tiene adaptadores.

## Dónde va cada cosa

| Quiero… | Va en |
|---|---|
| Una regla de decisión | `domain`, función pura con tests |
| Un tipo que cruza capas (informe, evento, puerto) | `domain` |
| Leer la línea de órdenes | `app::command` |
| Leer un dispositivo, un archivo, el sistema | `platform`, tras un puerto |
| Coordinar varias cosas con estado | un servicio en `app` |
| Crear un objeto real y pasárselo a otro | el binario |

## Estado

- [x] Esqueleto, guardas, log, puerto de ejemplo (`Clock`) y orden de ejemplo (`status`).
- [x] Bloque 0 · Cimientos del banco: sus crates, la orden `help` y el perfil de Warp.
- [x] Bloque 1 · Investigación: sondas de pantalla, GPU, ventanas y foco (hechos en `docs/PLAN.md`).
- [x] Bloque 2 · El motor de medidas: estadística, métricas, informe y su tabla, cambio en pantalla y asentamiento (dominio puro, sin probar todavía con datos reales).
- [x] Bloque 3 · Las cargas y la orden `emit`: seis cargas fijas, el servicio, los adaptadores de consola y archivos, y una primera medida real en Warp.
- [x] Bloque 4 · Huella: memoria, CPU y GPU, con la orden `footprint` y una primera medida de Warp en reposo (no es un Warp limpio).
- [ ] Bloque 5 · Ventanas y arranque: la guarda que solo toca ventanas adoptadas, la apertura y el cierre, la carpeta fija, el adaptador y la orden `startup`. **Falta la medida (5.7).**
- [ ] Bloque 6 · Pantalla y teclado: latencia y salida vista. Programado y probado con dobles; **falta correrlo en el PC real** (6.10), incluida la comprobación de que la captura de pantalla entrega imágenes.
- [ ] Bloque 7 · Línea base. Las órdenes `all` y `report` están programadas y probadas con dobles; **las dos tandas (7.3) no se han corrido**.

- [ ] **La terminal** (`docs/TERMINAL_PLAN.md`), bloque T1, el motor sin ventana: hechos la sonda de ConPTY, la cuadrícula con estilos, historial y reajuste (`domain::grid`), el intérprete (puerto `SequenceParser` con `vte`, y `domain::interpret`), el `Terminal`, y la sesión sobre ConPTY (puertos `PtyLauncher` y `Pty`) con `run --headless` y los perfiles de shell como datos. **T1 completo** (2026-11-26); sigue T2 (ventana, dibujo, tema e idioma). La línea base se aplaza hasta T4.

El detalle de cada bloque del banco está en `docs/PLAN.md`.
