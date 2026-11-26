# Imperifex · Plan maestro (2026-11-25)

> Estado: **en curso: bloques 1 a 4 hechos; bloques 5 y 6 revisados por el planificador con los hechos medidos (2026-11-25)**. Siguiente: 5.1. Lo planifica Opus; lo programa Sonnet, tarea a tarea.

## Qué es

La terminal de desarrollo de la familia Fexarium, en Rust: la herramienta principal del autor y la base sobre la que se construye todo lo demás.

## Objetivo de esta fase

Antes de escribir la terminal, **medir Warp** en esta máquina con una herramienta propia, `imperifex-bench`. Esos números son la vara: Imperifex tiene que igualarlos o mejorarlos, medida con la misma herramienta y el mismo protocolo. Al terminar existe `docs/BASELINE.md` con los números de Warp y la herramienta sabe medir cualquier terminal (la siguiente será Imperifex).

---

## Cómo se usa este plan (léelo entero antes de la primera tarea)

Para quien programa (Sonnet):

1. **Lee la skill `fexarium-rust`** y sus referencias `architecture.md`, `style.md`, `testing.md`, `windows.md` y `design-principles.md` antes de la primera tarea. Este plan no repite sus reglas: las da por sabidas.
2. **Una tarea = un commit.** Hazlas en el orden del plan. No empieces la siguiente hasta que la anterior esté subida.
3. **Commit y push solo con** `powershell -File tools/ship.ps1 -Message "<tipo>(<ámbito>): <qué cambió>"`. El script corre formato, clippy (`-D warnings` y sin `unwrap` en producción) y todas las pruebas, y no sube nada si algo falla. Mensajes en inglés, en el estilo de la casa, con la línea `Co-Authored-By` que te dé el sistema. Después mira la CI del commit (`gh run list --limit 1`).
4. **Haz exactamente lo que dice la tarea.** Si un paso es imposible, si un hecho medido contradice el plan o si te falta una decisión: **para**, escribe en «Estado» una línea `Bloqueo (fecha): …` con lo que encontraste y pregunta al autor. No improvises un diseño distinto.
5. **No añadas dependencias** que el plan no nombre. Las features de `windows` se añaden las que la tarea diga, solo esas.
6. **Comprueba las firmas de la API de Windows** en el crate instalado antes de usarlas: `grep -rn "pub unsafe fn NombreDeLaFuncion" ~/.cargo/registry/src/*/windows-0.62.*/src/`.
7. Al terminar una tarea, marca su línea en «Estado» con la fecha y, si midió algo, **el número**.
8. **`PAUSA`** al final de un bloque: no sigas. Avisa al autor con un resumen y espera.
9. Las tareas marcadas **🖥** abren ventanas, mueven el foco o pulsan teclas. **No las corras** sin el permiso explícito del autor en esa sesión, nunca mientras juega, y avísale antes de que no toque el PC durante la prueba.

### Reglas de seguridad de la herramienta (no se rompen nunca)

El autor trabaja **dentro de Warp** (también la sesión de Claude Code). Por eso:

- **S1.** La herramienta solo toca las ventanas **que ella abrió**. Antes de cada medida hace una foto de todas las ventanas. Solo adopta una ventana nueva si es la única que apareció tras su lanzamiento, y nunca cierra, mueve, enfoca ni escribe en otra. Sobre los procesos no hace nada: solo los lee. La ventana nueva de Warp es del mismo proceso que la sesión del autor (1.3), así que no basta con mirar procesos. Lo impone `SafetyGuard` (tarea 5.1), con pruebas.
- **S2.** **Nunca** `TerminateProcess`. Se cierra con `WM_CLOSE` a la ventana propia; si no se cierra en 5 s, la herramienta lo dice y se detiene.
- **S3.** Antes de **cada** pulsación comprueba que la ventana en primer plano es la suya. Si no lo es, aborta la medida: una tecla en la ventana equivocada escribiría en la sesión del autor.
- **S4.** El arranque en frío exige que no haya ningún proceso de esa terminal. Si lo hay, se niega y explica por qué.
- **S5.** Nada de rutas personales en el repo (es público): las rutas se escriben con variables (`%LOCALAPPDATA%`) y se expanden al correr.

---

## Máquina de referencia (medido el 2026-11-25)

| | |
|---|---|
| CPU | AMD Ryzen 5 3600X (6 núcleos, 12 hilos) |
| GPU | NVIDIA GeForce GTX 1660 Ti (controlador 32.0.16.2124) |
| Sistema | Windows 11 Pro 10.0.26200 |
| Warp | v0.2026.11.11.08.17.stable_01, en `%LOCALAPPDATA%\Programs\Warp\warp.exe` |
| Monitores | `\\.\DISPLAY1` principal: 1920×1080 a **144 Hz** (ASUS VG279QR). `\\.\DISPLAY2`: 1920×1080 a 60 Hz (BenQ GW2480). **Todas las medidas de pantalla se hacen en el principal** |

### Primera medida (a mano, 2026-11-25)

Warp abierto 46 minutos, con la sesión de Claude Code trabajando dentro (no es reposo):

| Proceso | Memoria de trabajo | Memoria privada | Hilos | Handles |
|---|---|---|---|---|
| `warp.exe` (principal) | 303 MB | 576 MB | 80 | 1569 |
| `warp.exe minidump-server` | 99 MB | 4 MB | 3 | 201 |

CPU del principal: 4,5 % de un núcleo en 10 s mientras la sesión escribía; 4,8 % de media en 46 minutos.

## Decisiones tomadas

| # | Decisión | Motivo |
|---|---|---|
| D1 | `imperifex-bench` vive en este repo, como app aparte del ecosistema Fexarium | La misma vara para Warp y para Imperifex |
| D2 | Se mide toda la experiencia, con cargas del uso real (Claude Code, `cargo`, `git`, registros largos) | La terminal es la herramienta principal del autor |
| D3 | El arranque en frío lo lanza el autor desde una ventana de PowerShell normal | Esta sesión corre dentro de Warp y no se puede cerrar |
| D4 | Solo se mide Warp | Windows Terminal se va a eliminar |
| D5 | Repo público `github.com/israelazt/imperifex` | — |
| D6 | Las apps se instalan con comandos (Scoop, luego winget), nunca con instaladores | Preferencia del autor; va al final |
| D7 | El banco tiene **sus propios tres crates** (`imperifex-bench-domain`, `-app`, `-platform`) y su binario, separados de los de la terminal | La terminal no carga con el código del banco ni al revés |
| D8 | Sin dependencias nuevas en el banco: estadística, generador pseudoaleatorio y hash propios | Pocas líneas, deterministas y probadas |
| D9 | Los resultados se guardan en `bench/results/<fecha>-<terminal>-<n>/report.json` y se suben al repo | Comparar versiones con el historial |
| D10 | Percentiles por **rango más cercano**: `rango = ceil(p/100 · n)`, contado desde 1 | Simple, sin interpolar, cualquiera lo reproduce |
| D11 | `unsafe` solo para lo que la biblioteca estándar no ofrece: una operación por bloque con su `// SAFETY:` (lo exigen los lints, también en las sondas); prohibido en `domain`, `app` y los binarios. Una dependencia que lo evite se decide con el autor | Si falla la memoria, hay pocos sitios, pequeños y explicados, donde buscar |
| D12 | `SafetyGuard` adopta ventanas concretas y el banco no actúa nunca sobre procesos | La ventana nueva de Warp es del proceso de la sesión del autor (1.3) |
| D13 | El arranque da dos números: hasta la ventana visible y hasta la ventana dibujada (asentada) | «Visible» no prueba que haya contenido (1.3) |
| D14 | Dos latencias de tecla: en el prompt (en Warp, su editor) y en un programa en marcha (`imperifex-bench echo`). **La vara para el motor de Imperifex es la del programa**: hace el mismo camino en cualquier terminal. La del prompt es la de la experiencia y también se compara | Warp dibuja la entrada con su editor, sin pasar por la shell (1.4) |
| D15 | Las órdenes que abren ventanas se lanzan desde una carpeta fija sin Git (`temp/imperifex-bench/home`) y se niegan dentro de un repo | La ventana nueva copia la carpeta y el Git de la pestaña activa (1.4) |
| D16 | Tras cerrar una ventana se esperan sus procesos hasta 90 s, sin forzarlos; en frío, hasta 120 s a que la terminal salga | Quedan entre 40 y 62 s (1.3) |
| D17 | «Asentado» solo cuenta cambios de al menos el 0,5 % de la región | El cursor que parpadea no dejaría asentar nunca |
| D18 | La CPU durante la salida, con el total de CPU entre el principio y el final | El reloj de CPU va en escalones de ~1,55 % por segundo (4.4) |

## Fuera de esta fase

- La terminal (ventana, intérprete de secuencias, shell): fase T, más abajo, se detalla al cerrar esta fase.
- Scoop y winget (D6): al final.
- Medir otras terminales.

## Conceptos nuevos (en `imperifex-bench-domain`)

| Tipo | Qué es |
|---|---|
| `Summary` | `count, min, max, mean, stddev, p50, p95, p99` de unas muestras |
| `Metric` | Qué se mide (arranque frío y caliente, latencia, salida por carga, memoria, CPU, GPU), con su unidad y su nombre en español |
| `Measurement` | Una `Metric` con su `Summary`, sus muestras y notas |
| `Report` | Fecha, máquina, terminal (nombre y versión), versión del banco y las medidas |
| `TerminalProfile` | Cómo se abre una terminal y qué procesos son suyos (dato, en `bench/terminals.json`) |
| `Load` | Una carga de salida: `Ascii`, `LongLines`, `Color`, `Unicode`, `Cursor`, `Cargo` |
| `SafetyGuard` | Lo que existía antes de medir y no se puede tocar (S1) |
| `ChangeDetector` | Decide si un trozo de pantalla cambió, ignorando lo que parpadea |

## Puertos (en `imperifex-bench-domain/src/ports.rs`)

Pequeños, por lo que el servicio necesita. Todos devuelven `Result<_, String>`.

| Puerto | Métodos | Tarea |
|---|---|---|
| `Clock` | el de `imperifex-domain::ports` (se reutiliza) | — |
| `Output` | `write_all(&mut self, bytes)`, `flush(&mut self)` | 3.3 |
| `FileStore` | `write(path, bytes)`, `read(path) -> Option<Vec<u8>>`, `temp_dir() -> PathBuf`; `exists(path) -> bool`, `make_dir(path)` | 3.3; 5.5 |
| `ProcessProbe` | `processes() -> Vec<ProcessEntry>`, `usage(pid) -> ProcessUsage` | 4.1 |
| `GpuProbe` | `utilization(pids) -> f64` (desde la llamada anterior) | 4.1 |
| `Desktop` | `launch(exe, args, dir) -> pid`, `windows() -> Vec<WindowEntry>` (todas, también ocultas), `place(hwnd, rect)`, `focus(hwnd)`, `foreground() -> Option<u64>`, `close(hwnd)` | 5.1 |
| `ScreenWatch` | `watch(region) -> Box<dyn Watch>`; `Watch::next(timeout) -> Option<ScreenFrame>` | 6.1 |
| `Keyboard` | `type_text(hwnd, text)`, `press(hwnd, Key)` (`Key` incluye combinaciones como `Ctrl+Shift+T`) | 6.1 |
| `KeyInput` | `read(buf) -> usize`: la entrada del propio banco en crudo, para la orden `echo` | 6.6 |
| `MachineProbe` | `machine()`, `product_version(exe)`, `today()`, `env_var(name)` (no estaba prevista; la pide el informe) | 4.4 |

Cada puerto tiene su falso en `imperifex-bench-app/src/testing/`, que **anota lo que le pidieron** y responde lo que la prueba programa.

---

## Bloque 0 · Cimientos del banco

### 0.1 Los crates del banco
1. Crear `crates/imperifex-bench-domain` (deps: `imperifex-domain`, `serde`, `serde_json`), `crates/imperifex-bench-app` (deps: `imperifex-bench-domain`, `imperifex-domain`), `crates/imperifex-bench-platform` (deps: `imperifex-bench-domain`, `imperifex-domain`; `windows` en `[target.'cfg(windows)'.dependencies]` con `Win32_Foundation`) y `apps/imperifex-bench` (deps: los tres, más `imperifex-domain` e `imperifex-platform` para `SystemClock` e `install_stderr_log`). Todos con `[lints] workspace = true`.
2. `#![forbid(unsafe_code)]` en `-domain` y `-app`. Cada `lib.rs` con su `//!` de una línea por capa, como los del generador.
3. Añadir los cuatro a `members` del `Cargo.toml` raíz.
4. En `crates/imperifex-domain/tests/architecture.rs`, ampliar la tabla `allowed` con los cuatro y sus dependencias exactas de arriba (el tamaño del array pasa de 4 a 8).
5. En `.github/workflows/rust.yml`, job `linux`: añadir `-p imperifex-bench-domain -p imperifex-bench-app -p imperifex-bench-platform` a clippy y pruebas, y `cargo build -p imperifex-bench`.
6. `apps/imperifex-bench/src/main.rs` copia la forma de `apps/imperifex/src/main.rs`: `Command::parse` y `run` en `imperifex-bench-app/src/command.rs` con una sola orden, `help`, y `HELP` en español.

**Hecho cuando:** `tools/check.ps1` en verde; `cargo run -p imperifex-bench -- help` imprime la ayuda; la guarda falla si `imperifex-bench-app` depende de `imperifex-bench-platform` (pruébalo a mano y deshazlo).
**Commit:** `chore(bench): the bench's own crates and program, behind the layer guard`

### 0.2 El perfil de Warp
1. `bench/terminals.json` con un perfil:
   ```json
   [{ "name": "warp",
      "exe": "%LOCALAPPDATA%\\Programs\\Warp\\warp.exe",
      "new_window_args": [],
      "own_images": ["warp.exe"],
      "helper_images": ["OpenConsole.exe", "conhost.exe"],
      "new_tab_keys": "Ctrl+Shift+T" }]
   ```
   (`new_window_args` y `new_tab_keys` se confirman en el bloque 1).
2. `domain/profile.rs`: `TerminalProfile` (serde, nombres de arriba), `parse_profiles(json) -> Result<Vec<_>, String>`, `find(&[_], name) -> Result<&_, String>` (error con los nombres que sí hay), `expand_env(text, lookup: &dyn Fn(&str) -> Option<String>) -> Result<String, String>` (reemplaza `%NOMBRE%`; una variable que no existe es error con su nombre; `%%` es un `%`).
3. Pruebas: el JSON del repo se lee; un nombre que no existe dice cuáles hay; `expand_env` con variable, sin variable, desconocida y `%%`.

**Hecho cuando:** pruebas en verde; ninguna ruta personal en el repo (`git grep -i "users\\\\"` vacío).
**Commit:** `feat(bench): terminal profiles as data, with the Warp one`

### 0.3 ARCHITECTURE.md
1. Añadir una sección «El banco de medidas» con la tabla de sus crates (como la de la terminal), los puertos de arriba y las reglas S1-S5.
2. «Estado»: `- [ ] Bloque 1 · Investigación` … `- [ ] Bloque 7 · Línea base`.

**Hecho cuando:** el documento describe los ocho crates y el diagrama tiene las flechas hacia abajo.
**Commit:** `docs: the bench in the architecture`

**PAUSA.**

---

## Bloque 1 · Investigación (sondas de solo lectura)

Cada sonda es un programa en `crates/imperifex-bench-platform/examples/` que imprime lo que encuentra. **No** forma parte del producto. Su resultado se escribe en «Hechos medidos» de este plan, con fecha y números. Si un resultado contradice lo previsto en los bloques 5 y 6, **para y avisa** (regla 4): el planificador ajusta el plan antes de seguir.

### 1.1 Sonda de pantalla: `probe_duplication`
Features: `Win32_Graphics_Dxgi`, `Win32_Graphics_Dxgi_Common`, `Win32_Graphics_Direct3D`, `Win32_Graphics_Direct3D11`, `Win32_System_Performance` (para `QueryPerformanceCounter`/`Frequency`) y `Win32_Graphics_Gdi` (la exige `IDXGIOutput::GetDesc`; añadida con permiso del autor, 2026-11-25).
1. `CreateDXGIFactory1`, recorrer `EnumAdapters1` y `EnumOutputs`; elegir la salida cuyo `DesktopCoordinates` empieza en (0, 0) (el principal). Imprimir nombre, resolución y adaptador.
2. `D3D11CreateDevice` con ese adaptador (`D3D_DRIVER_TYPE_UNKNOWN`). `IDXGIOutput` → `cast::<IDXGIOutput1>()` → `DuplicateOutput(&device)`.
3. Durante 5 s: `AcquireNextFrame(100, …)`; por cada frame con `LastPresentTime != 0`, anotar `LastPresentTime` (unidades de QPC) y el QPC del momento de recibirlo; copiar con `CopySubresourceRegion` una región de 1000×600 a una textura `STAGING` con `CPU_ACCESS_READ`, `Map`, leer filas con `RowPitch`, `Unmap`; `ReleaseFrame`.
4. Imprimir: frames por segundo con el ratón quieto y moviéndolo, retraso entre `LastPresentTime` y la recepción (p50/p95), coste de copiar y leer la región (p50/p95 en ms).

**Hecho cuando:** los cinco números están en «Hechos medidos». Se espera ~144 frames/s moviendo el ratón y el coste de copia por debajo de 2 ms; si no, avisar.

### 1.2 Sonda de GPU: `probe_gpu`
Features: `Win32_System_Performance` y `Win32_System_Diagnostics_ToolHelp` (para el paso 2; añadida con permiso del autor, 2026-11-25).
1. `PdhOpenQueryW`, `PdhAddEnglishCounterW` con `\GPU Engine(*)\Utilization Percentage`, dos `PdhCollectQueryData` separados 1 s, `PdhGetFormattedCounterArrayW(PDH_FMT_DOUBLE)`.
2. Imprimir las instancias que contienen `pid_<pid de warp>` (buscar el pid por nombre con `CreateToolhelp32Snapshot`) y su valor; y el coste de una recolección.

Referencia: `purifex/crates/purifex-platform/src/sampler/cpu_clock.rs` usa PDH igual.
**Hecho cuando:** sabemos el formato del nombre de instancia (se espera `pid_<n>_luid_…_engtype_3D`) y el coste.

### 1.3 🖥 Sonda de ventanas de Warp: `probe_warp_window`
Features: `Win32_UI_WindowsAndMessaging`, `Win32_System_Diagnostics_ToolHelp`, `Win32_System_Threading`, `Win32_Graphics_Dwm`.
1. Foto: procesos (`CreateToolhelp32Snapshot`) y ventanas visibles de primer nivel (`EnumWindows`, `IsWindowVisible`, `GetWindowThreadProcessId`, `GetWindowTextW`).
2. Lanzar `warp.exe` sin argumentos (`std::process::Command`). Durante 10 s, cada 50 ms, comparar con la foto: ¿aparece un proceso nuevo? ¿sigue vivo o sale? ¿aparece una ventana nueva y de qué pid es? ¿qué procesos hijos nuevos aparecen (shell, `OpenConsole.exe`, `conhost.exe`)?
3. Si no aparece ventana nueva, probar `cmd /c start warp://action/new_window` y repetir la observación.
4. A la ventana nueva (solo a ella, S1): `PostMessageW(WM_CLOSE)`. Observar 5 s: ¿se cierra?, ¿aparece un diálogo de confirmación (ventana nueva del mismo pid)? ¿quedan procesos hijos?
5. Imprimir todo.

**Hecho cuando:** sabemos cómo abre Warp una ventana nueva (proceso nuevo o el mismo), qué procesos cuelgan de una ventana y cómo se cierra sin diálogo. Con eso se confirma `new_window_args` en `bench/terminals.json`.

### 1.4 🖥 Sonda de foco y teclas: `probe_focus`
Features: las de 1.3 más `Win32_UI_Input_KeyboardAndMouse`.
1. Abrir una ventana de Warp nueva como en 1.3.
2. `SetForegroundWindow(nueva)` y leer `GetForegroundWindow()`. Si no la consigue: probar con `AttachThreadInput` entre el hilo de la ventana en primer plano y el de la nueva, y como último recurso pulsar y soltar `Alt` con `SendInput` antes de `SetForegroundWindow`. Anotar qué método funciona.
3. Solo si el primer plano es la nueva (S3): `SendInput` de `a` (`KEYEVENTF_UNICODE`) y de Retroceso.
4. Cerrar como en 1.3.

**Hecho cuando:** sabemos qué método de foco funciona desde un proceso que corre en una pestaña de Warp, y que la tecla llega a la ventana nueva y a ninguna otra.

**PAUSA.** El planificador revisa los hechos y ajusta los bloques 5 y 6 si hace falta.

---

## Bloque 2 · El motor de medidas (dominio puro)

Todo en `imperifex-bench-domain`, sin sistema, con pruebas.

### 2.1 `stats.rs`
1. `pub struct Summary { count: usize, min, max, mean, stddev, p50, p95, p99: f64 }` (serde, `snake_case`).
2. `pub fn summarize(samples: &[f64]) -> Option<Summary>`: descarta NaN e infinitos; `None` si no queda ninguno; `stddev` poblacional; percentiles según D10.
3. `pub fn percentile(sorted: &[f64], p: f64) -> f64` (privada al módulo si nadie más la usa).
4. Pruebas: 1..=100 da p50 50, p95 95, p99 99, media 50,5; una muestra da todo igual a ella; el orden de entrada no importa; NaN no cuenta; vacío es `None`.

**Commit:** `feat(bench): summaries of samples with nearest-rank percentiles`

### 2.2 `metric.rs`
1. `pub enum Load { Ascii, LongLines, Color, Unicode, Cursor, Cargo }` (serde `snake_case`, `ALL`, `name()` en minúsculas para la línea de órdenes, `parse(&str)`).
2. `pub enum Metric { StartupCold, StartupWarm, KeyLatency, OutputConsumed(Load), OutputSeen(Load), MemoryIdlePrivate, MemoryIdleWorkingSet, MemoryScrollbackPrivate, MemoryPerTabPrivate, CpuIdle, GpuIdle, CpuDuringOutput(Load) }`.
3. `pub enum Unit { Ms, MbPerS, Mb, PercentOfCore, Percent }`; `Metric::unit()` y `Metric::label()` (español, p. ej. «Arranque en frío»). **Una sola tabla** (un `match`) para unidad y nombre.
4. Pruebas: cada métrica tiene unidad y nombre no vacío; `Load::parse(name())` vuelve a la misma.

**Commit:** `feat(bench): what is measured, with units and names`

### 2.3 `report.rs`
1. `Machine { cpu, gpu, os, monitors: Vec<Monitor { name, width, height, hz, primary }> }`, `TerminalInfo { name, version }`, `Measurement { metric, summary, samples: Vec<f64>, notes: Vec<String> }`, `Report { date: String /* YYYY-MM-DD */, machine, terminal, bench_version, measurements }`.
2. `Report::to_json() -> String` (bonito) y `Report::from_json(&str) -> Result<Report, String>`.
3. Pruebas: ida y vuelta igual; los nombres JSON de cada campo están fijados (una prueba que lee `value["measurements"][0]["summary"]["p95"]`).

**Commit:** `feat(bench): the report and its JSON`

### 2.4 `markdown.rs`
1. `pub fn baseline_markdown(reports: &[Report]) -> String`: cabecera con máquina y terminal; una tabla por grupo (Arranque, Tecla, Salida, Memoria, CPU y GPU) con columnas `Medida | Tanda | n | p50 | p95 | p99 | Dispersión`; una fila por medida y tanda; números con un decimal y la unidad; dispersión = `stddev / mean` en %.
2. Prueba: con dos informes de ejemplo, el texto completo es igual a uno fijo (prueba de instantánea en la propia prueba).

**Commit:** `feat(bench): the baseline as a Markdown table`

### 2.5 `change.rs`
1. `pub struct ChangeDetector { mask: Vec<bool>, width, height, min_pixels: usize }`.
2. `ChangeDetector::learn(frames: &[&[u8]], width, height, min_pixels) -> Result<Self, String>`: marca en la máscara cada píxel que cambió entre frames consecutivos durante el aprendizaje (el cursor que parpadea). Frames BGRA de `width·height·4` bytes; tamaño distinto es error.
3. `fn changed(&self, before: &[u8], after: &[u8]) -> bool`: cuenta píxeles distintos (cualquier canal difiere más de 8) **fuera** de la máscara; cambió si son `>= min_pixels` (4 por defecto).
4. Pruebas con frames sintéticos de 16×8: un píxel que parpadea durante el aprendizaje no cuenta después; cuatro píxeles nuevos sí; tres no; tamaños distintos dan error.

**Commit:** `feat(bench): telling a real change on screen from a blinking cursor`

### 2.6 `settle.rs`
1. `pub fn settled_at(changes: &[Duration], start: Duration, quiet: Duration, end: Duration) -> Option<Duration>`: instantes en que la pantalla cambió; devuelve el instante del último cambio tras el cual hubo `quiet` sin cambios antes de `end`.
2. Pruebas: sin cambios da `start`; cambios seguidos y luego silencio da el último; si el silencio nunca llega antes de `end`, `None`.

**Commit:** `feat(bench): when the screen has settled`

**PAUSA** corta: el autor solo confirma que las pruebas pasan.

---

## Bloque 3 · Las cargas y la orden `emit`

### 3.1 `domain/loads.rs`
1. Generador propio `XorShift64 { state }` (`next_u64`, semilla 0 se cambia por una constante) y hash `fnv1a64(&[u8]) -> u64`.
2. `pub fn generate(load: Load, bytes: usize, seed: u64) -> Vec<u8>`, siempre exactamente `bytes` bytes y UTF-8 válido (se corta en una frontera de carácter y se rellena con espacios):
   - `Ascii`: líneas de 80 caracteres imprimibles (32..=126) + `\r\n`.
   - `LongLines`: líneas de 10 000 caracteres + `\r\n`.
   - `Color`: cada carácter precedido de `ESC[38;2;R;G;Bm`; `ESC[0m` al final de cada línea de 80.
   - `Unicode`: mezcla por línea de latino con tildes, CJK (doble ancho), emoji (incluidas secuencias con ZWJ, p. ej. 👩‍💻) y letras con acentos combinados; 40 columnas visibles.
   - `Cursor`: barras de progreso: `\r[####    ] 42%` con `ESC[K`, 100 pasos por barra, y cada 10 barras un `ESC[H` + `ESC[2J`.
   - `Cargo`: la grabación de 3.2 repetida hasta llenar.
3. Pruebas: misma semilla, mismos bytes (hash fijo en la prueba para cada carga a 64 KiB); tamaño exacto; UTF-8 válido; `Ascii` solo tiene imprimibles y `\r\n`.

**Commit:** `feat(bench): deterministic output loads`

### 3.2 La grabación de `cargo`
1. Con el repo limpio (`cargo clean -p imperifex-domain`), grabar `cargo build --workspace --color always 2>&1` a `bench/loads/cargo-build.ansi`.
2. `domain/loads.rs`: `pub fn scrub_paths(text: &str) -> String` reemplaza cualquier ruta absoluta de Windows dentro de paréntesis tras `Compiling … (` por `C:\src\imperifex` (S5). Prueba con un ejemplo.
3. Pasar la grabación por `scrub_paths` (con un ejemplo de una sola vez en `examples/`, o a mano) y subirla limpia. `include_bytes!` en `Load::Cargo`.

**Hecho cuando:** `git grep -i "users" bench/` vacío; la carga `Cargo` genera bytes que contienen `Compiling`.
**Commit:** `feat(bench): a recorded cargo build as a load, without personal paths`

### 3.3 `app/emit.rs`
1. Puertos `Output` y `FileStore` en `ports.rs` (tabla de puertos).
2. `pub struct EmitResult { load, bytes, seconds, mb_per_s }` en el dominio (serde).
3. `EmitService::run(&self, out: &mut dyn Output, load, bytes, seed) -> Result<EmitResult, String>`: genera la carga **antes** de empezar a medir; escribe en trozos de 64 KiB; `flush`; mide con `Clock`; al final escribe `ESC[0m` y `\r\nIMPERIFEX-BENCH-DONE <load>\r\n` (fuera del tiempo medido).
4. Falsos `FakeOutput` (anota trozos) y `FakeFiles`.
5. Pruebas: los trozos son de 64 KiB salvo el último; el tiempo sale del reloj falso; la marca final está.

**Commit:** `feat(bench): emitting a load and timing it`

### 3.4 Adaptadores de `emit`
Features: `Win32_System_Console`.
1. `platform/console_out.rs`: `ConsoleOut` implementa `Output` sobre `std::io::stdout().lock()`. Al crearse, `GetConsoleMode` + `SetConsoleMode(… | ENABLE_VIRTUAL_TERMINAL_PROCESSING)`; en `Drop`, el modo anterior.
2. `platform/files.rs`: `DiskFiles` implementa `FileStore` (escritura atómica: `.tmp` y `rename`; `temp_dir` = `std::env::temp_dir().join("imperifex-bench")`).

**Commit:** `feat(bench): console and file adapters`

### 3.5 La orden `emit`
1. `imperifex-bench emit <load> [--mib N] [--seed S] --out <archivo.json>`; por defecto 32 MiB (`ascii`: 100 MiB) y semilla 1. Errores de argumentos con el valor en el mensaje (salida 2).
2. Escribe el `EmitResult` como JSON en `--out`.
3. Prueba de `Command::parse`: valores por defecto, opciones, carga desconocida, número malo.

**Hecho cuando:** el autor la corre a mano en una pestaña de Warp (`imperifex-bench emit ascii --out %TEMP%\a.json`) y el número va a «Estado». Es la primera medida de salida (M3, consumida).
**Commit:** `feat(bench): the emit order`

**PAUSA.**

---

## Bloque 4 · Huella: memoria, CPU y GPU

### 4.1 Puertos y reglas
1. `ports.rs`: `ProcessProbe` y `GpuProbe`. Datos: `ProcessEntry { pid, parent, image }`, `ProcessUsage { private_bytes, working_set_bytes, cpu_100ns, threads, handles }`.
2. `domain/tree.rs`: `pub fn terminal_processes(entries, profile) -> Vec<u32>`: los procesos cuya imagen está en `own_images`, más los de `helper_images` que descienden de alguno de ellos. **No** cuenta los shells (pwsh, cmd, bash…): la huella es de la terminal. Comparación de imágenes sin distinguir mayúsculas.
3. `domain/usage.rs`: `cpu_percent_of_core(before_100ns, after_100ns, wall: Duration) -> f64`.
4. Pruebas: árbol con warp, OpenConsole hijo, pwsh nieto y un OpenConsole ajeno: salen los dos primeros; CPU: 1 s de CPU en 10 s es 10 %.

**Commit:** `feat(bench): which processes belong to a terminal, and its CPU share`

### 4.2 `app/footprint.rs`
1. `FootprintService::sample(seconds, interval) -> Result<Vec<Measurement>, String>`: cada `interval` (1 s) recalcula los procesos (pueden nacer y morir), suma `private_bytes` y `working_set_bytes`, y calcula CPU entre muestras y GPU. Devuelve `MemoryIdlePrivate`, `MemoryIdleWorkingSet` (en MB), `CpuIdle`, `GpuIdle`.
2. Si un proceso desaparece entre `processes()` y `usage()`, se ignora en esa muestra y se anota en `notes`.
3. Pruebas con falsos programados.

**Commit:** `feat(bench): sampling a terminal's footprint`

### 4.3 Adaptadores
Features: `Win32_System_Diagnostics_ToolHelp`, `Win32_System_ProcessStatus`, `Win32_System_Threading`, `Win32_System_Performance`.
1. `platform/processes.rs`: `processes()` con `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` + `Process32FirstW`/`Process32NextW` (el handle en una guarda con `Drop`); `usage(pid)`: `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` en guarda, `GetProcessMemoryInfo` con `PROCESS_MEMORY_COUNTERS_EX` (`PrivateUsage`, `WorkingSetSize`), `GetProcessTimes` (kernel + usuario), `GetProcessHandleCount`; hilos del snapshot.
2. `platform/gpu.rs`: PDH como en la sonda 1.2: la consulta se abre una vez y se cierra en `Drop`; suma las instancias `engtype_3D` de los pids pedidos.
3. Medir el coste de una muestra completa y anotarlo.

Referencia: `dockview/crates/dockview-platform/src/perf.rs` (memoria y tiempos del propio proceso).
**Commit:** `feat(bench): process and GPU adapters`

### 4.4 La orden `footprint`
1. `imperifex-bench footprint <terminal> [--seconds 60] [--out archivo.json]`: sobre los procesos que **ya** existen (solo lectura: no abre nada), escribe un `Report` con esas medidas.
2. Datos de la máquina: `platform/machine.rs` con CPU (registro `HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0\ProcessorNameString`), GPU (`IDXGIAdapter1::GetDesc1`), sistema (`RtlGetVersion` o la versión de `kernel32.dll`) y monitores (`EnumDisplayDevicesW` + `EnumDisplaySettingsW` con `ENUM_CURRENT_SETTINGS`). La versión de la terminal: `GetFileVersionInfoW` del exe (`ProductVersion`). Features que hagan falta (`Win32_System_Registry`, `Win32_Storage_FileSystem`, `Win32_Graphics_Gdi`).

**Hecho cuando:** con el PC en reposo (el autor no escribe 60 s), `footprint warp` da memoria, CPU y GPU, y el número va a «Estado».
**Commit:** `feat(bench): the footprint order`

**PAUSA.**

---

## Bloque 5 · Ventanas y arranque

> Revisado por el planificador el 2026-11-25 con los hechos de las sondas 1.3, 1.4 y la huella 4.4. Lo que cambió respecto al plan original y por qué:
>
> - **La ventana nueva es del `warp.exe` que ya estaba abierto** (1.3), el mismo proceso que lleva la sesión del autor. «Solo lo que el banco abrió» no puede decidirse por procesos: `SafetyGuard` **adopta ventanas concretas** (D12).
> - **El banco nunca actúa sobre un proceso**: solo los lee (`ProcessProbe`) y cierra ventanas con `WM_CLOSE` (S2). Por eso no hace falta identificar procesos por pid y hora de creación, como proponía 1.3: un pid reutilizado solo puede hacer que el banco espere de más, nunca que toque algo ajeno.
> - **Si aparecen dos ventanas nuevas a la vez**, el banco no adivina cuál es la suya: para.
> - **El arranque da dos números** (D13): ventana visible (este bloque) y ventana dibujada (6.5, cuando el banco ya ve la pantalla).
> - **Los procesos de una ventana cerrada siguen vivos unos 40 a 62 s** (1.3): se esperan, sin forzarlos (D16).
> - **La ventana nueva depende de la carpeta y del Git de la pestaña activa** (1.4): todo se lanza desde una carpeta fija y vacía (D15).
> - `focus` es solo `SetForegroundWindow` y comprobar (1.4). `AttachThreadInput` y `Alt` no se programan.

### 5.1 Puerto `Desktop` y `SafetyGuard`
1. `ports.rs`: `Desktop` (tabla de puertos). `WindowEntry { hwnd: u64, pid: u32, title: String, class: String, visible: bool, rect: Rect }`, `Rect { x, y, w, h }` (`i32`, píxeles físicos). `windows()` devuelve **todas** las ventanas de primer nivel, también las ocultas.
2. `domain/safety.rs`: `SafetyGuard::new(before: &[WindowEntry])` guarda los `hwnd` de la foto.
   - `adopt(&mut self, hwnd) -> Result<(), String>`: error si el `hwnd` estaba en la foto («la ventana … ya existía: no es del banco»).
   - `check(&self, hwnd) -> Result<(), String>`: `Ok` solo para una ventana adoptada; el error dice si era de la foto o si nunca se adoptó.
   - No hay nada para procesos: el banco no tiene ninguna operación que actúe sobre uno.
3. Todo servicio que use `place`, `focus`, `close` o `Keyboard` llama antes a `check`. El falso de `Desktop` recibe la foto programada y **hace fallar la prueba** (`panic!` con el `hwnd`) si se le pide `place`, `focus` o `close` sobre una ventana de la foto.
4. Pruebas: una ventana de la foto no se adopta ni se toca; una nueva sin adoptar no se toca; una adoptada sí; un `hwnd` de la foto que reaparece (Windows reutiliza números) no se puede adoptar.

**Commit:** `feat(bench): the bench only touches the windows it adopted`

### 5.2 `domain/windows.rs`
1. `pub const MIN_WINDOW: (i32, i32) = (400, 300)`.
2. `pub enum NewWindow { None, One(WindowEntry), Many(Vec<WindowEntry>), Reshown(WindowEntry) }`.
3. `pub fn new_window(before: &[WindowEntry], now: &[WindowEntry], pids: &[u32]) -> NewWindow`. Candidata: está en `now`, visible, con título, de un pid de `pids` y de al menos `MIN_WINDOW`.
   - `One`: una sola candidata que no estaba en `before`.
   - `Many`: más de una.
   - `Reshown`: ninguna nueva, pero una candidata estaba en `before` oculta (la terminal enseñó una ventana que ya tenía). El banco no la adopta: para y lo dice.
   - `None`: nada aún.
4. Pruebas: la `PseudoConsoleWindow` de 0×0 del `powershell.exe` (1.3) no cuenta; sin título no cuenta; dos nuevas dan `Many`; oculta que se vuelve visible da `Reshown`; la ventana del autor, que ya estaba, nunca sale.

**Commit:** `feat(bench): finding the window a launch opened, and only that one`

### 5.3 `app/window_session.rs`
Lo que abren y cierran todas las medidas de los bloques 5 y 6, en un solo sitio.
1. `WindowSession::open(ports, profile, dir: &Path) -> Result<WindowSession, String>`:
   - Foto de ventanas y de procesos de la terminal (`terminal_processes`); `SafetyGuard::new`.
   - `t0` = `Clock::monotonic()`; `launch(exe, new_window_args, dir)`.
   - Cada 5 ms (máximo 20 s): `new_window` con los pids de la terminal **de ese momento**. `One` → `adopt` y listo. `Many` → error («aparecieron N ventanas a la vez; no sé cuál es la mía»). `Reshown` → error con el título. Pasados 20 s → error.
   - Guarda `hwnd`, `t0`, `visible_at` (el instante de la lectura que la encontró) y los pids de la foto.
2. `WindowSession::close(self) -> Result<Closed, String>`:
   - `check`; `close(hwnd)` (`WM_CLOSE`).
   - Cada 50 ms, máximo 5 s: ¿sigue el `hwnd`? Si en ese tiempo aparece una ventana nueva visible del mismo pid, es un diálogo: error («Warp pidió confirmar el cierre; el banco no pulsa nada»). Si el `hwnd` sigue a los 5 s: error (S2).
   - Espera los restos (D16): cada 1 s, `tree::new_since(foto, terminal_processes(ahora))`; termina cuando está vacío o a los 90 s. `Closed { leftovers: usize, waited: Duration }`. Que queden restos **no** es error: va a `notes`.
3. `tree.rs`: `pub fn new_since(before: &[u32], now: &[u32]) -> Vec<u32>`, con prueba.
4. Pruebas con falsos: encuentra la ventana a la tercera lectura y `visible_at` es esa; `Many` y `Reshown` paran sin tocar nada; un diálogo al cerrar para; una ventana que no se cierra para; los restos se esperan y se cuentan; nunca se toca la foto.

**Commit:** `feat(bench): opening and closing one window of a terminal, and waiting for what it leaves`

### 5.4 `app/startup.rs`
1. `StartupService::run(profile, runs, cold, dir) -> Result<Measurement, String>`:
   - `cold`: si `terminal_processes` no está vacío, error S4 («cierra Warp del todo: el arranque en frío necesita que no esté abierto»).
   - Por tanda: `WindowSession::open`; muestra = `visible_at − t0`; `close`. En frío, además, esperar hasta 120 s a que no quede **ningún** proceso de la terminal; si sigue alguno, error («Warp no salió al cerrar su única ventana; no se fuerza (S2)»). 2 s entre tandas.
   - `notes`: los restos de cada tanda y cuánto se esperaron.
2. Métricas `StartupCold` y `StartupWarm` («Arranque en frío/caliente, hasta la ventana visible»: ajustar `label`).
3. Pruebas con falsos: cuenta las tandas; se niega en frío si hay procesos; para si Warp no sale en frío; para si la ventana no se cierra; nunca toca la foto.

**Commit:** `feat(bench): timing a terminal's start until its window shows`

### 5.5 La carpeta de las medidas
1. `FileStore`: `exists(path) -> bool` y `make_dir(path) -> Result<(), String>`.
2. `domain/workdir.rs`: `pub fn repo_around(dir: &Path, exists: &dyn Fn(&Path) -> bool) -> Option<PathBuf>`: la primera carpeta, de `dir` hacia arriba, que tiene `.git`. Pruebas: dentro de un repo, en su raíz, fuera.
3. Toda orden que abra ventanas (`startup` y las del bloque 6) se lanza desde `temp_dir()/home`, que se crea vacía, y se **niega** si la carpeta actual del banco está dentro de un repo: «lánzalo desde una carpeta sin Git: `cd "<home>"`; la ventana nueva copia la carpeta y el Git de la pestaña activa y las tandas no se podrían comparar». La carpeta actual la lee el binario y la pasa en la orden; `Desktop::launch` recibe `home`.

**Commit:** `feat(bench): measuring always from the same empty folder`

### 5.6 Adaptador de `Desktop`
Features: `Win32_UI_WindowsAndMessaging`, `Win32_Graphics_Dwm`, `Win32_UI_HiDpi`.
1. `launch`: `std::process::Command::new(exe).args(..).current_dir(dir).spawn()`, devuelve el pid. No se espera al hijo: con Warp abierto, ese proceso sale en ~121 ms y la ventana es de otro (1.3).
2. `windows`: `EnumWindows` sin filtrar por visibilidad; `IsWindowVisible`, `GetWindowThreadProcessId`, `GetWindowTextW`, `GetClassNameW`, `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)`. Lo que ya hacen `examples/common/desktop.rs` y `warp.rs`: pasarlo a `src/` en vez de copiarlo, y que las sondas lo usen.
3. `place`: `SetWindowPos` (`SWP_NOZORDER | SWP_NOACTIVATE`). `close`: `PostMessageW(WM_CLOSE)`. `foreground`: `GetForegroundWindow`.
4. `focus`: `SetForegroundWindow` y luego `GetForegroundWindow() == hwnd`; si no, error. Nada más (1.4).
5. En `main` del banco, **lo primero**: `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` (referencia: `purifex/crates/purifex-platform/src/window/mod.rs`), en una función de `platform` que el binario llama.

**Commit:** `feat(bench): the desktop adapter`

### 5.7 🖥 La orden `startup`
1. `imperifex-bench startup <terminal> [--runs 10] [--cold] [--out archivo.json]`. Antes de empezar, en la consola: «no toques el PC; unos N minutos» (cada tanda puede esperar hasta 90 s los restos).
2. Medir en caliente y anotar en «Estado».
3. **Con el autor delante, una vez:** que mire en qué carpeta abre la ventana nueva lanzada desde `home`. Anotarlo en «Hechos medidos».
4. README: cómo lanza el autor la medida en frío (D3): cerrar Warp del todo; PowerShell desde Inicio; `cd` a la carpeta `home` que imprime el banco; la orden con la ruta completa; esperar; volver a Warp. Y que, si Warp no sale al cerrar su ventana, el banco para y no lo fuerza.

**Hecho cuando:** hay un número en caliente y la carpeta de la ventana está anotada. Si la ventana no abre en `home`, **Bloqueo**: el planificador decide.
**Commit:** `feat(bench): the startup order`

**PAUSA.**

---

## Bloque 6 · Pantalla y teclado: latencia y salida vista

> Revisado por el planificador el 2026-11-25 con los hechos de 1.1, 1.4 y 4.4. Lo que cambió y por qué:
>
> - Los avisos con `LastPresentTime = 0` se ignoran, y cuando DXGI junta presentaciones (`AccumulatedFrames > 1`) se anota (1.1).
> - El instante de un frame es `LastPresentTime`, no cuándo lo recibe el banco. Así no cuenta lo que se tarda en copiarlo, que con el ratón quieto llegó a 6,6 ms en p95 (1.1).
> - **Dos latencias de tecla** (D14). En el prompt, la «a» la dibuja el editor propio de Warp (1.4). En un programa en marcha, la tecla recorre ConPTY ida y vuelta, como en cualquier terminal. Para el segundo número el banco tiene su propio programa, la orden `echo`.
> - **Antes de medir se escribe un prefijo sin sentido** (`zqxj`, sin medir). Así el historial no tiene nada que sugerir, porque con la «a» Warp sugirió `sdasd` (1.4). Además, la tecla solo se pulsa tras 300 ms sin cambios, y un cambio previo rehace la base. Así el primer cambio es el de la tecla.
> - «Asentado» no cuenta el cursor que parpadea (D17). Solo cuenta un cambio de al menos el 0,5 % de la región.
> - **La CPU durante la salida** se calcula con el total de CPU entre el principio y el final (D18), no con muestras de 250 ms: el reloj de CPU va en escalones de ~1,55 % por segundo (4.4).
> - La orden se escribe para **PowerShell**, que es la shell de las ventanas de Warp (1.3): `& '<ruta>' …`.
> - Antes de una medida de memoria se esperan los restos de ventanas anteriores (D16).

### 6.1 Puertos `ScreenWatch` y `Keyboard`, y un solo reloj
1. `ScreenFrame { at: Duration, accumulated: u32, width, height, bgra: Vec<u8> }`. `ScreenWatch::watch(region: Rect) -> Result<Box<dyn Watch>, String>`; `Watch::next(timeout) -> Result<Option<ScreenFrame>, String>` (`None` si no hubo frame nuevo).
2. `domain/keys.rs`: `Key { Enter, Backspace, Char(char), Chord(Chord) }`, `Chord { ctrl, shift, alt: bool, key: char }`, `parse_chord("Ctrl+Shift+T") -> Result<Chord, String>` (sin distinguir mayúsculas; error con lo que no entiende). Pruebas.
3. `Keyboard::type_text(hwnd, text)` y `press(hwnd, Key)`: ambos comprueban el primer plano (S3) **en el adaptador**, y el servicio además llama a `SafetyGuard::check` y a `foreground()` antes.
4. **Un solo reloj:** `domain/qpc.rs`: `pub fn qpc_to_duration(ticks: i64, frequency: i64) -> Duration` con aritmética de `u128` (prueba: 1 tick a 10 MHz son 100 ns; 30 días de ticks no desbordan; negativo o frecuencia 0 da 0). `platform/qpc_clock.rs`: `QpcClock` implementa `Clock` con `QueryPerformanceCounter` desde cero (no desde que arranca el banco), así `LastPresentTime` se convierte con la misma función y `t1 − t0` compara lo mismo. El binario del banco usa `QpcClock` en vez de `SystemClock`.

**Commit:** `feat(bench): ports for watching the screen and typing, on one clock`

### 6.2 Adaptador DXGI
1. `platform/screen/dxgi.rs` según la sonda 1.1 (pasar a `src/` lo que sirva de `examples/probe_duplication/capture.rs`): la salida cuyo escritorio **contiene entera** la región (si no hay ninguna, error con la región); dispositivo; duplicación; textura `STAGING` del tamaño de la región creada una vez.
2. `next`: `AcquireNextFrame`; descarta los frames con `LastPresentTime == 0` (avisos de puntero, cientos por segundo) y los libera; `at` = `qpc_to_duration(LastPresentTime)`; `accumulated` = `AccumulatedFrames`; copia, mapea, lee por filas con `RowPitch`; `ReleaseFrame` siempre (guarda con `Drop`).
3. `DXGI_ERROR_ACCESS_LOST`: recrear la duplicación una vez; si vuelve a fallar, error.
4. Más de 400 líneas: partirlo (`screen/dxgi.rs`, `screen/staging.rs`).

**Commit:** `feat(bench): watching the screen with desktop duplication`

### 6.3 Adaptador de teclado
Features: `Win32_UI_Input_KeyboardAndMouse`.
1. `SendInput` con `INPUT_KEYBOARD`: texto con `KEYEVENTF_UNICODE` por unidad UTF-16 (pulsar y soltar); `Enter` y `Backspace` con su `VK_*`; un `Chord`: modificadores abajo, la tecla, y arriba en orden inverso, en un solo `SendInput`.
2. Antes de cada lote: `GetForegroundWindow() == hwnd`, o error.
3. Pasar a `src/` lo que sirva de `examples/probe_focus/input.rs`.

**Commit:** `feat(bench): the keyboard adapter`

### 6.4 `app/screen_wait.rs`: esperar a que la pantalla se asiente
1. `pub const SETTLE_SHARE: f64 = 0.005` en el dominio: un cambio cuenta para «asentado» si toca al menos el 0,5 % de los píxeles de la región (en 1280×680, 4352). El cursor que parpadea y la letra suelta quedan por debajo; una pantalla de salida, por encima.
2. `settle(watch, clock, start, quiet: Duration, max: Duration) -> Result<Settled, String>`: lee frames, anota los instantes de cambio (`ChangeDetector` sin máscara, `min_pixels` del 0,5 %), y usa `settle::settled_at`. `Settled { at, frame, changes, accumulated }`. Pasado `max` sin silencio: error con cuántos cambios hubo.
3. Pruebas con una pantalla falsa: un cursor que cambia cada 530 ms no impide asentar; una ráfaga grande seguida de silencio da su último instante; sin silencio, error.

**Commit:** `feat(bench): waiting for the screen to settle, past a blinking cursor`

### 6.5 Arranque hasta la ventana dibujada
1. `Metric::StartupColdDrawn` y `StartupWarmDrawn` («… hasta la ventana dibujada»), grupo Arranque.
2. `StartupService` (5.4), tras `visible_at`: sin mover la ventana, `watch(rect de la ventana)` y `settle` (silencio 500 ms, máximo 10 s); muestra dibujada = `settled.at − t0`. Si la ventana no cabe entera en un monitor, no hay muestra dibujada en esa tanda y se anota en `notes`, con el monitor.
3. Pruebas con falsos: la muestra dibujada es el último cambio grande; una ventana entre dos monitores se anota y la visible sigue.

**Commit:** `feat(bench): timing a terminal's start until its window is drawn`

### 6.6 La orden `echo`
Un programa propio que repite cada tecla, para medir la tecla dentro de un programa en marcha igual en todas las terminales.
1. `domain/echo.rs`: `pub const ECHO_PROMPT: &str = "IMPERIFEX-ECHO> "`; `pub fn echo_reply(input: &[u8]) -> Reply { bytes: Vec<u8>, quit: bool }`: imprimibles ASCII, tal cual; `0x7F` y `0x08`, `"\x08 \x08"`; `q`, salir; lo demás se ignora. Pruebas.
2. Puerto `KeyInput { read(&mut self, buf: &mut [u8]) -> Result<usize, String> }`.
3. `platform/console_in.rs`: `ConsoleIn` pone la entrada en crudo (`GetConsoleMode`; quita `ENABLE_LINE_INPUT`, `ENABLE_ECHO_INPUT` y `ENABLE_PROCESSED_INPUT`; pone `ENABLE_VIRTUAL_TERMINAL_INPUT`) y devuelve el modo anterior en `Drop`; `read` sobre `std::io::stdin()`.
4. `app/echo.rs`: escribe `ECHO_PROMPT`, y por cada lectura escribe la respuesta y hace `flush` **en cada tecla**; sale con `q`. Pruebas con falsos.
5. `imperifex-bench echo`.

**Commit:** `feat(bench): an echo program to time keys inside a running program`

### 6.7 `app/latency.rs`
1. `Metric::KeyLatency` pasa a `KeyLatencyPrompt` («Tecla en el prompt») y se añade `KeyLatencyProgram` («Tecla en un programa»).
2. `LatencyService::run(profile, presses, mode: Prompt | Program, dir) -> Result<Measurement, String>`:
   - `WindowSession::open`; `place` en el monitor principal en `(100, 100, 1280, 720)`; `settle` (500 ms, máximo 10 s); `focus`.
   - `Program`: `type_text` de `& '<ruta del banco>' echo` (con `powershell_command`, abajo) y `Enter`; esperar a que se asiente.
   - `Prompt`: `type_text("zqxj")` sin medir; esperar a que se asiente.
   - Región: la ventana sin los 40 px de arriba. Aprender la máscara 2 s (`ChangeDetector::learn` con `DEFAULT_MIN_PIXELS`).
   - Por pulsación: pausa de 150 a 350 ms (`XorShift64`, semilla fija), y además 300 ms seguidos sin cambios (un cambio rehace la base; más de 3 s sin calma es un fallo). Luego `check` + `foreground()` (S3); `t0`; `press(Char('a'))`; leer frames hasta `changed(base, frame)`; muestra = `frame.at − t0`. Luego `Backspace`, esperar el cambio de vuelta y tomar nueva base. Sin cambio en 1 s: fallo. Más del 5 % de fallos: abortar con el motivo.
   - `notes`: fallos, muestras con `accumulated > 1` (pueden llegar hasta un frame tarde, 6,9 ms a 144 Hz) y cuántos píxeles tapó la máscara.
   - `Program`: al final `press(Char('q'))` y esperar a que se asiente. Después `close`.
3. `domain/shell.rs`: `pub fn powershell_command(exe: &str, args: &[&str]) -> String`: `& '<exe>' '<arg>'…` con cada `'` doblado. Pruebas: ruta con espacios y con comilla.
4. Pruebas con falsos: la pantalla cambia N ms después de la tecla y la muestra vale N; un cambio antes de pulsar rehace la base y no da muestra falsa; foco perdido aborta sin pulsar; los fallos se cuentan; en `Program` se escribe la orden `echo` y se sale con `q`.
5. Durante la medida no se toca el ratón: lo dice la consola al empezar (1.1).

**Commit:** `feat(bench): key-to-screen latency, at the prompt and inside a program`

### 6.8 `app/output_seen.rs`
1. `OutputService::run(profile, load, mib, repeats, dir) -> Result<Vec<Measurement>, String>`:
   - Ventana como en 6.7 (abrir, colocar, asentar, foco).
   - Por repetición: `type_text(powershell_command(banco, ["emit", load, "--mib", n, "--out", json]))`. Leer el uso de los procesos de la terminal (`cpu_100ns`). `t0` y `Enter`. Esperar a que exista el JSON (`FileStore::exists`, cada 50 ms, máximo 120 s) y a que la pantalla se asiente (500 ms; el `settle` empieza en `t0`). Volver a leer el uso.
   - Vista = asentado − `t0`; consumida = `seconds` del JSON (MB/s); CPU = `cpu_percent_of_core(antes, después, asentado − t0)` (D18).
   - Entre repeticiones: `cls` + `Enter` y esperar a que se asiente.
   - `close`.
2. Devuelve `OutputSeen(load)` (ms), `OutputConsumed(load)` (MB/s) y `CpuDuringOutput(load)`.
3. Pruebas con falsos: la vista llega al último cambio grande; el JSON que no aparece es error con su ruta; la CPU sale del total.

**Commit:** `feat(bench): output as the terminal shows it`

### 6.9 `app/memory_load.rs`
1. Esperar los restos de ventanas anteriores, máximo 90 s (D16), sin abrir nada.
2. Ventana nueva; huella de 10 s (`FootprintService`); `emit ascii` de 1 millón de líneas (~82 MiB) como en 6.8, y esperar asentado; huella de 10 s → `MemoryScrollbackPrivate` = diferencia de las medianas, en MB.
3. Pestañas, con `parse_chord(new_tab_keys)`. Tres veces: pulsar, y comprobar que en 5 s aparece un proceso auxiliar nuevo de la terminal (`new_since`): eso confirma la pestaña. Luego asentar y huella de 10 s. `MemoryPerTabPrivate` = aumento medio por pestaña. Si el perfil no tiene teclas, o la primera pulsación no abre nada, se omite y se anota.
4. `close`. Si Warp pide confirmar el cierre de varias pestañas, `close` para (5.3). Eso es un **Bloqueo** y no se cierran las pestañas una a una por cuenta propia.
5. Pruebas con falsos: diferencia de memoria; una pestaña que no abre se omite; el diálogo para.

**Commit:** `feat(bench): memory with a long history and per tab`

### 6.10 🖥 Órdenes `latency`, `output`, `memory`
1. `latency <terminal> [--presses 200] [--mode prompt|program|both]` (por defecto `both`), `output <terminal> [--load L|all] [--repeats 3]`, `memory <terminal>`, todas con `--out`; `startup` gana las muestras dibujadas de 6.5.
2. Correr cada una una vez, con el permiso del autor, y anotar los números en «Estado».
3. En «Hechos medidos», lo que el plan da por supuesto y nadie ha visto aún:
   - si el cursor de Warp parpadea (los píxeles de la máscara);
   - si, con `echo` en marcha, las teclas llegan al programa o se las queda el editor de Warp (si se las queda: **Bloqueo**);
   - si `Ctrl+Shift+T` abre pestaña;
   - si cerrar una ventana con pestañas pide confirmar;
   - cuánto duró cada orden.

**Commit:** `feat(bench): the latency, output and memory orders`

**PAUSA.**

---

## Bloque 7 · La línea base de Warp

### 7.1 La orden `all`
1. `imperifex-bench all <terminal> [--out-dir bench/results/<fecha>-<terminal>-<n>]`: en orden, `footprint` 60 s en reposo, `startup` en caliente ×10, `latency` ×200 en los dos modos, `output all` ×3, `memory`. Entre una parte y otra se esperan los restos (D16), así que la duración estimada sale de lo que tardó cada orden en 6.10. Un `report.json` con todo. Si una parte falla, las demás siguen y el informe lo anota.
2. Avisar en la consola, antes de empezar, de que no se toque el PC y cuánto durará (estimado).

**Commit:** `feat(bench): the whole baseline in one order`

### 7.2 La orden `report`
1. `imperifex-bench report <carpeta>... --md docs/BASELINE.md`: lee los `report.json` y escribe la tabla de 2.4.

**Commit:** `feat(bench): the baseline document from the reports`

### 7.3 🖥 Las dos tandas
1. Con el permiso del autor, el PC en reposo y sin jugar: `all warp` dos veces seguidas.
2. El autor corre `startup warp --cold --runs 10 --out …` como dice el README y deja el JSON en la carpeta de la primera tanda.
3. `report` de las dos carpetas → `docs/BASELINE.md`; subir resultados y documento.
4. En «Estado»: si alguna medida difiere más de un 10 % en p50 entre tandas, decirlo y por qué podría ser.

**Hecho cuando:** criterios de aceptación cumplidos.
**Commit:** `docs: the Warp baseline`

**PAUSA.** El planificador revisa la línea base y escribe el plan de la fase T.

---

## Criterios de aceptación de esta fase

1. `docs/BASELINE.md` tiene arranque (frío y caliente), latencia de tecla, salida por carga (consumida y vista), memoria (reposo, historial largo, por pestaña), CPU y GPU de Warp, con dos tandas y su dispersión.
2. Todo número sale de `bench/results/…/report.json` y se puede regenerar con `report`.
3. Ninguna prueba tocó la ventana ni los procesos que ya existían (S1-S3); al terminar no queda ninguna ventana ni proceso abierto por el banco.
4. `tools/check.ps1` y la CI en verde; ninguna ruta personal en el repo.

---

## Fase T · La terminal (resumen; se detalla al cerrar la línea base)

> El plan de la terminal está en `docs/TERMINAL_PLAN.md` (2026-11-25) y sustituye a este resumen.

Bloques previstos, en este orden. Cada uno se planifica en detalle **antes** de empezarlo, con los números de la línea base delante.

| Bloque | Qué | Decisión abierta (con recomendación) |
|---|---|---|
| T1 · Motor sin ventana | ConPTY (crear la consola, leer y escribir, cambiar tamaño), intérprete de secuencias VT, la cuadrícula (celdas, atributos, historial, pantalla alternativa, ajuste de líneas). `imperifex run --headless` vuelca la pantalla como texto: se confía en el motor antes de pintarlo | Intérprete: **recomiendo empezar con el crate `vte`** (el de Alacritty, pequeño y probado) detrás de un puerto propio, y cambiarlo solo si la línea base lo pide |
| T2 · Ventana y dibujo | Ventana, atlas de glifos en la GPU (Direct3D 11, como Luminifex), dibujo por celdas con daños, cursor, colores | Texto: **recomiendo DirectWrite** para rasterizar (las fuentes de Windows y ClearType); cosmic-text si se quiere portabilidad desde el principio |
| T3 · Entrada | Teclas → secuencias VT, ratón, selección, portapapeles, IME | — |
| T4 · Medir y optimizar | `imperifex-bench all imperifex`: igualar o mejorar a Warp en cada medida | — |
| T5 · Lo que hace cómoda la terminal | Pestañas, paneles, búsqueda en el historial, bloques por orden (integración con la shell, OSC 133), enlaces, temas | **Qué funciones de Warp hacen falta**: se decide con el autor al llegar |
| T6 · Distribución | Zip en Releases, bucket de Scoop, terminal predeterminada de Windows como orden que se deshace | — |

## Orden y dependencias

```
0.1 → 0.2 → 0.3 → [PAUSA]
1.1, 1.2 (sin ventanas) → 1.3 🖥 → 1.4 🖥 → [PAUSA: el planificador ajusta 5 y 6]
2.1 → 2.2 → 2.3 → 2.4 ; 2.5 → 2.6        (el bloque 2 se puede hacer antes que el 1)
3.1 → 3.2 → 3.3 → 3.4 → 3.5 → [PAUSA]     (3.5 da la primera medida de salida)
4.1 → 4.2 → 4.3 → 4.4 → [PAUSA]           (4.4 da la primera huella limpia)
5.1 → 5.2 → 5.3 → 5.4 → 5.5 → 5.6 → 5.7 🖥 → [PAUSA]
6.1 → 6.2 → 6.3 → 6.4 → 6.5 → 6.6 → 6.7 → 6.8 → 6.9 → 6.10 🖥 → [PAUSA]
7.1 → 7.2 → 7.3 🖥 → [PAUSA: plan de la fase T]
```

El camino más corto a un número útil: bloque 2 → 3 (salida) → 4 (huella). Las tareas sin 🖥 se pueden hacer mientras el autor trabaja.

## Riesgos

| Riesgo | Mitigación |
|---|---|
| Una tecla acaba en la sesión del autor | S1 y S3, comprobados en el adaptador y en el servicio; el falso hace fallar la prueba si se intenta |
| Warp abre ventanas nuevas en el mismo proceso, o pide confirmación al cerrar | Confirmado que sí, el mismo proceso (1.3): `SafetyGuard` adopta ventanas (D12). Un diálogo al cerrar para la medida (5.3); con varias pestañas no se ha probado (6.10) |
| Las teclas con `echo` en marcha se las queda el editor de Warp | Se comprueba en 6.10; si pasa, Bloqueo y el planificador busca otra forma de medir la tecla en un programa |
| El foco no se puede tomar desde un proceso de fondo | La sonda 1.4 prueba tres métodos; si ninguno sirve, se para y se replantea |
| La latencia medida incluye el monitor y la composición de Windows | Igual para todas las terminales: la comparación es en esta máquina, a 144 Hz |
| Las sugerencias o la IA de Warp cambian la pantalla al escribir `a` | La máscara solo quita lo que parpadea; si aparecen cambios ajenos, se anota y se prueba con otra tecla |
| Warp se actualiza solo y cambia los números | La versión va en cada informe |
| La carga `Cargo` filtra rutas personales | `scrub_paths` con prueba y `git grep` en la tarea |

## Hechos medidos

_(Los escribe cada sonda del bloque 1, con fecha y números.)_

### 1.1 Pantalla (2026-11-25, `probe_duplication` en release, 5 s por tanda)

`\\.\DISPLAY1` 1920×1080 en la GTX 1660 Ti. La duplicación funciona.

| | Ratón quieto | Ratón en movimiento |
|---|---|---|
| Frames recibidos con imagen | 110 (21,9/s) | 666 (133,2/s) |
| Presentaciones acumuladas según DXGI (`AccumulatedFrames`) | 110 (21,9/s) | 719 (143,8/s) |
| Avisos solo de puntero (`LastPresentTime = 0`) | 1 | 4492 (898,2/s) |
| Retraso de `LastPresentTime` a recibirlo: p50 / p95 / máx (ms) | 0,062 / 0,090 / 6,25 | 0,061 / 0,106 / 2,70 |
| Copiar y leer 1000×600: p50 / p95 / máx (ms) | 0,886 / 6,61 / 11,15 | 0,907 / 1,38 / 1,91 |

- Se cumple lo previsto: ~144/s moviendo el ratón y mediana de copia por debajo de 2 ms.
- Con el ratón quieto el p95 de la copia subió a 6,6 ms. La causa no se ha comprobado (una hipótesis: la GPU baja de reloj). No se controló qué más cambiaba en pantalla.
- **Para el bloque 6:** (a) ignorar los avisos con `LastPresentTime = 0`, que llegan a cientos por segundo; (b) moviendo el ratón se juntaron 53 de 719 presentaciones: cuando `AccumulatedFrames > 1`, `LastPresentTime` es el de la última, así que esa muestra puede salir hasta un frame (6,9 ms) tarde. `LatencyService` debe anotar en `notes` cuántas muestras venían juntas; durante la latencia no se toca el ratón.

### 1.2 GPU (2026-11-25, `probe_gpu` en release, 10 lecturas de 1 s)

- Instancia: `pid_<pid>_luid_0x00000000_0x00014AB0_phys_0_eng_<n>_engtype_<tipo>`, como se esperaba. Tipos vistos en Warp: `3D`, `Copy`, `LegacyOverlay`, `VideoDecode`, `VideoEncode`, `VR`, `Security`, `Security_1`. Hubo 331 instancias en total.
- Hay dos `warp.exe`: el principal y un hijo suyo (el `minidump-server`). Solo el principal tenía instancias de GPU.
- Suma `3D` de Warp con la sesión de Claude Code trabajando dentro (no es reposo): entre 3,6 % y 6,4 %.
- Coste en dos ejecuciones: `PdhCollectQueryData` p50 0,27 y 0,32 ms (máximo 1,85 ms); leer el arreglo p50 0,31 y 0,35 ms (máximo 0,65 ms). Con 10 muestras, el p95 es el máximo.
- **Para el bloque 4:** muestrear la GPU cada segundo cuesta del orden de 1 ms; no hace falta cambiar el plan.

### 1.3 Ventanas de Warp (2026-11-25, `probe_warp_window`, una ejecución, con Warp ya abierto)

- **Abrir:** `warp.exe` sin argumentos arranca un proceso que sale en unos 121 ms; la ventana nueva es **del `warp.exe` que ya estaba abierto**. `new_window_args: []` queda confirmado. No hizo falta `warp://` (sin probar).
- **La ventana:** clase `Window Class`, título «Warp», 1280×800, vista en la lectura de los 121 ms (las lecturas salían cada ~62 ms). «Visible» no demuestra que ya tuviera contenido dibujado: no se midió.
- **También aparece** una ventana visible de 0×0, clase `PseudoConsoleWindow`, del `powershell.exe` de la pestaña (no es de un proceso de Warp).
- **Procesos:** quedan con la ventana un `OpenConsole.exe` y un `powershell.exe`, los dos hijos del `warp.exe` principal. Al abrir, Warp lanza además muchos procesos de vida corta (`powershell.exe`, `git.exe`, `gh.exe` y sus `conhost.exe`); los vistos terminaron durante la observación de 10 s, y la sonda siguió viendo nacer algunos hasta los ~9,7 s.
- **Cerrar:** con `WM_CLOSE` la ventana desapareció dentro de los 5 s observados (el `hwnd` dejó de existir), sin otra ventana de confirmación. La shell tenía solo el prompt; no se probó con un programa en marcha.
- **El `OpenConsole.exe` y el `powershell.exe` de la ventana cerrada siguieron vivos** y terminaron solos entre 40 y 62 s después, aproximadamente (instantes calculados con la hora de creación y lecturas posteriores a mano). Hipótesis sin comprobar: Warp conserva la sesión cerrada para poder reabrirla.

**Para los bloques 4 a 7 (lo revisa el planificador en la PAUSA):**
- 5.2: «visible, con título y de un proceso del perfil» basta para esta ventana; la `PseudoConsoleWindow` queda fuera. Conviene exigir además un tamaño mínimo (la sonda usa 400×300) y la foto de **todas** las ventanas, también las ocultas.
- 5.3: decidir si «ventana visible» es el final del arranque o si hay que esperar al primer contenido en pantalla.
- Criterio de aceptación 3 y cualquier medida de memoria: tras cerrar una ventana quedan sus procesos cerca de un minuto. El banco debe esperarlos (máximo propuesto: 90 s) antes de medir huella o de dar por terminada una tanda `all`. No se fuerzan (S2).
- 4.1: los `conhost.exe` de las shells de vida corta descienden de Warp y entrarían en la huella como `helper_images`.
- 5.1: la lista de procesos puede dar como padre un pid que ya salió (aquí, el del lanzador). Como Windows reutiliza pids, `SafetyGuard` debería identificar un proceso por pid **y hora de creación**.
- Cerrar la única ventana (arranque en frío) no se probó.
- `new_tab_keys` sigue sin confirmar: ninguna sonda del bloque lo prueba; lo hará la 6.6.

### 1.4 Foco y teclas (2026-11-25, `probe_focus`, una ejecución desde la sesión de Claude Code en Warp)

Para no tocar ninguna ventana ajena (S1), la sonda abre dos ventanas propias, A y B, y pasa el foco de B a A.

- Cada ventana nueva **tomó el primer plano sola**. Se vieron en la primera lectura (56 y 57 ms; en la 1.3, 121 ms).
- **`SetForegroundWindow` solo bastó** para volver de B a A. Por eso `AttachThreadInput` y `Alt` **no se probaron**. Tampoco se probó con una ventana de otro programa en primer plano.
- **La tecla llegó:** tres fotos de A (antes, con «a» y tras Retroceso) muestran la letra y luego el campo vacío. Cada envío comprobó antes que A estaba en primer plano (S3). Las fotos no se suben al repo.
- Al cerrar las dos, el foco volvió a la ventana que lo tenía al empezar. Sus procesos terminaron solos (comprobado menos de dos minutos después).
- **Warp muestra una sugerencia al escribir:** tras la «a» apareció el texto atenuado `sdasd` y un botón al lado. Es el riesgo previsto en la tabla de riesgos.
- **La entrada la dibuja el editor propio de Warp**, en la parte baja de la ventana, no el prompt de la shell.
- La ventana nueva abrió en la carpeta de este repo, con avisos de carpeta, rama y cambios de Git, y un panel «New terminal session».

**Para los bloques 5 y 6 (lo revisa el planificador en la PAUSA):**
- 5.4 `focus`: `SetForegroundWindow` y comprobar `GetForegroundWindow`; si falla, error. No implementar `AttachThreadInput` ni `Alt` mientras no hagan falta.
- 6.4: el primer cambio tras la «a» puede incluir la sugerencia; no se sabe si llega en el mismo frame que la letra. La muestra mide el **primer** cambio, así que hay que comprobar que ese primer cambio ya contiene la letra. La base se toma después de que la pantalla se asiente.
- 6.4: esta latencia es la del editor de Warp, que presumiblemente dibuja sin esperar a la shell. Una terminal que manda cada tecla a la shell recorre además ConPTY y PSReadLine. **Para comparar lo mismo, medir también la tecla dentro de un programa en marcha** (una orden del banco que repita lo que recibe), y decidir cuál es la vara.
- 5.5, 6 y 7: la ventana nueva depende de la carpeta y del estado de Git de la pestaña activa. Fijar desde dónde se lanza el banco (una carpeta sin repo) para que las tandas sean comparables.

## Estado

- 2026-11-25: workspace creado con el generador de la casa; repo en GitHub con la CI en verde; primera medida a mano de memoria y CPU (arriba); plan maestro escrito.
- [x] 0.1 (2026-11-25; además, la guarda de `unsafe` admite `imperifex-bench-platform` y `main.rs` del banco puede imprimir su salida) · [x] 0.2 (2026-11-25) · [x] 0.3 (2026-11-25)
- [x] 1.1 (2026-11-25: 143,8 presentaciones/s moviendo el ratón; copia p95 1,38 ms) · [x] 1.2 (2026-11-25: `pid_<n>_luid_…_engtype_3D`; recolectar y leer, ~0,6 ms en p50) · [x] 1.3 (2026-11-25: la ventana la abre el Warp ya abierto; cierra sin diálogo; su shell vive ~1 min más) · [x] 1.4 (2026-11-25: basta `SetForegroundWindow`; la tecla llega a la ventana propia; Warp sugiere al escribir)
- [x] 2.1 (2026-11-25) · [x] 2.2 (2026-11-25) · [x] 2.3 (2026-11-25) · [x] 2.4 (2026-11-25) · [x] 2.5 (2026-11-25) · [x] 2.6 (2026-11-25)
- [x] 3.1 (2026-11-25) · [x] 3.2 (2026-11-25) · [x] 3.3 (2026-11-25) · [x] 3.4 (2026-11-25) · [x] 3.5 (2026-11-25: Warp consume `ascii` a **19,1 MB/s**, 100 MiB en 5,23 s; una sola muestra, ver abajo)
- [x] 4.1 (2026-11-25) · [x] 4.2 (2026-11-25) · [x] 4.3 (2026-11-25) · [x] 4.4 (2026-11-25: Warp en reposo, **689 MB privados, 462 MB de trabajo, CPU 1,1 % de un núcleo de media, GPU 0 %**; ver abajo)
- 2026-11-25: el planificador revisó los bloques 5 y 6 con los hechos de 1.1-1.4 y 4.4 (decisiones D12-D18; el bloque 5 pasa a 7 tareas y el 6 a 10).
- 2026-11-25, corrección: los commits `6e09964`, `01917ea`, `2e25b25` y `f385ae8` programaron el bloque 5 **como estaba antes de la revisión** (`11485b4`): quien programaba no releyó el plan, y anotó aquí, en falso, que la revisión no se había hecho. Ninguna de las tareas 5.1-5.4 revisadas estaba hecha; se rehacen en orden, aprovechando ese código.
- [x] 5.1 (2026-11-25) · [x] 5.2 (2026-11-25) · [x] 5.3 (2026-11-25) · [x] 5.4 (2026-11-25) · [x] 5.5 (2026-11-25) · [x] 5.6 (2026-11-25) · [ ] 5.7
- Pendiente de la 5.7 (2026-11-25): la orden `startup` está programada y subida, **sin medir**: falta correrla en caliente con el permiso del autor y que mire en qué carpeta abre la ventana. Una medida anterior (137 ms) se hizo con el diseño viejo y lanzada desde el repo: no vale. Por indicación del autor se programa el bloque 6 sin esperar a ese número.
- [x] 6.1 (2026-11-25) · [ ] 6.2 · [ ] 6.3 · [ ] 6.4 · [ ] 6.5 · [ ] 6.6 · [ ] 6.7 · [ ] 6.8 · [ ] 6.9 · [ ] 6.10
- [ ] 7.1 · [ ] 7.2 · [ ] 7.3

**Primera medida de salida (M3, consumida), 2026-11-25:** el autor corrió `emit ascii` (release, 100 MiB, semilla 1) en una pestaña de Warp: 5,229 s, 19,1 MB/s (1 MB = 1 048 576 bytes). Es **una sola muestra**, no es la línea base: no se anotó el tamaño de la ventana ni qué más hacía el PC (había una sesión de Claude Code en el mismo Warp). «Consumida» es el tiempo hasta que la consola aceptó los bytes, no hasta que se vieron (eso lo medirá la 6.5). Sin comprobar: en qué trozos entrega la biblioteca estándar de Rust esos bytes a la consola y cuánto cuesta; es igual para todas las terminales que se midan con esta herramienta.

**Coste de una muestra de huella (4.3), 2026-11-25:** prueba `measure_a_whole_footprint_sample` en release, 10 muestras separadas 1 s, con 269 procesos en el sistema: en total entre 8,6 y 12,6 ms; listar los procesos 7,0 a 10,7 ms, leer 15 procesos 0,4 a 1,3 ms, GPU 0,6 a 1,1 ms. La prueba toma todos los `conhost.exe` del sistema (no solo los de Warp) y uno de 16 no se dejó leer. Con una muestra por segundo es cerca del 1 % de un núcleo, gastado por el banco y no por la terminal medida.


### Primera huella de Warp (4.4), 2026-11-25

`footprint warp`, 60 muestras de 1 s, en release, con el autor lejos del PC. La lanzó la sesión de Claude Code en segundo plano, con 90 s de espera antes de la primera tanda y 30 s entre tandas. Warp v0.2026.11.11.08.17.stable_01.

| | Tanda 1 (alterada) | Tanda 2 |
|---|---|---|
| Memoria privada, p50 / máx (MB) | 688,8 / 757,9 | 688,8 / 740,7 |
| Memoria de trabajo, p50 / máx (MB) | 464,3 / 494,6 | 462,2 / 484,7 |
| CPU, media / p50 / p95 / máx (% de un núcleo) | 2,7 / 1,5 / 10,8 / 15,5 | 1,1 / 0,0 / 4,6 / 15,5 |
| GPU, media / p95 / máx (%) | 0,46 / 3,0 / 4,0 | 0,00 / 0,0 / 0,1 |
| Procesos contados | 12 a 16 | 12 a 15 |

- **La tanda 1 no es reposo:** el autor escribió en la sesión mientras corría y Claude respondió; la GPU solo subió en ese tramo (muestras 25 a 35 aprox.). **La medida es la tanda 2.**
- **No es un Warp limpio:** tenía la sesión de Claude Code abierta, con su historial. Poco antes se vieron 7 `OpenConsole.exe` bajo Warp (parecen 7 sesiones; no se comprobó), 2 `warp.exe` (el principal con unos 663 MB privados) y 3 `conhost.exe` de programas lanzados por Claude Code, unos 4 MB, que la regla de 4.1 cuenta. No se controló si la ventana estaba visible.
- La memoria de trabajo bajó respecto a una prueba de 3 s con Claude trabajando (516 MB); la privada casi no cambió (689 MB).

**Para la línea base (lo revisa el planificador):**
- Fijar el estado en que se mide Warp (cuántas pestañas, con qué dentro, ventana visible o no): Imperifex tendrá que medirse igual.
- La CPU de cada muestra sale en escalones de ~1,55 % (probablemente la resolución del reloj de CPU de Windows; sin comprobar). En reposo el p50 puede dar 0: **la media dice más**, y la tabla de la 2.4 no la muestra.
- Nada impide por sí solo que alguien use el PC durante una tanda: la alteración de la tanda 1 solo se vio mirando las muestras.

**Coste de leer las ventanas (5.4), 2026-11-25:** `measure_reading_the_windows` en release, 195 ventanas (28 visibles), 10 lecturas seguidas: la primera 1,1 ms, las demás 0,25 a 0,50 ms. El arranque (5.3) solo lista los procesos (unos 10 ms, ver 4.3) cuando ya hay una ventana candidata, y el instante de la muestra se toma antes de listarlos.
