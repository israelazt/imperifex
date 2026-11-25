# Imperifex · Plan maestro (2026-11-25)

> Estado: **en curso: bloque 1 hecho; en la PAUSA** (el planificador revisa los bloques 5 y 6). Lo planifica Opus; lo programa Sonnet, tarea a tarea.

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

- **S1.** La herramienta solo toca ventanas y procesos **que ella abrió**. Antes de cada medida hace una foto de los procesos y ventanas existentes y nunca cierra, mueve, enfoca ni escribe en nada de esa foto. Lo impone `SafetyGuard` (tarea 5.1), con pruebas.
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
| `FileStore` | `write(path, bytes)`, `read(path) -> Option<Vec<u8>>`, `temp_dir() -> PathBuf` | 3.3 |
| `ProcessProbe` | `processes() -> Vec<ProcessEntry>`, `usage(pid) -> ProcessUsage` | 4.1 |
| `GpuProbe` | `utilization(pids) -> f64` (desde la llamada anterior) | 4.1 |
| `Desktop` | `launch(exe, args) -> pid`, `windows() -> Vec<WindowEntry>`, `place(hwnd, rect)`, `focus(hwnd)`, `foreground() -> Option<u64>`, `close(hwnd)` | 5.1 |
| `ScreenWatch` | `watch(region) -> Box<dyn Watch>`; `Watch::next(timeout) -> Option<ScreenFrame>` | 6.1 |
| `Keyboard` | `type_text(hwnd, text)`, `press(hwnd, Key)` | 6.1 |

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

Ajustar con lo que dijeron las sondas 1.3 y 1.4 (el planificador lo habrá revisado en la pausa del bloque 1).

### 5.1 Puerto `Desktop` y `SafetyGuard`
1. `ports.rs`: `Desktop` (tabla de puertos). `WindowEntry { hwnd: u64, pid, title, visible, rect: Rect }`, `Rect { x, y, w, h }` en píxeles físicos.
2. `domain/safety.rs`: `SafetyGuard::new(pids, hwnds)` con la foto previa; `may_touch_window(hwnd) -> bool`, `may_touch_process(pid) -> bool`; `check(hwnd) -> Result<(), String>` con un mensaje que dice por qué no.
3. Todo servicio que use `Desktop` o `Keyboard` pasa por `SafetyGuard`. El falso de `Desktop` **hace fallar la prueba** si se le pide tocar algo de la foto previa.
4. Pruebas: una ventana previa no se puede tocar, una nueva sí.

**Commit:** `feat(bench): the bench only touches what it opened`

### 5.2 `domain/windows.rs`
1. `pub fn new_window(before: &[WindowEntry], now: &[WindowEntry], pids_of_profile: &[u32]) -> Option<WindowEntry>`: visible, con título, de un pid de la terminal y no presente antes.
2. Pruebas.

**Commit:** `feat(bench): finding the window a launch opened`

### 5.3 `app/startup.rs`
1. `StartupService::run(profile, runs, cold) -> Result<Measurement, String>`:
   - `cold`: si `terminal_processes` no está vacío, error S4 («cierra Warp: el arranque en frío necesita que no esté abierto»).
   - Por tanda: foto → `SafetyGuard`; `t0`; lanzar; cada 5 ms buscar `new_window` (máximo 20 s); muestra = hasta la ventana visible. Luego `close` (S2): esperar hasta 5 s a que la ventana desaparezca; si no, error y parar. En frío, esperar hasta 10 s a que no quede ningún proceso de la terminal. 2 s entre tandas.
2. Pruebas con falsos: cuenta las tandas, se niega en frío si hay procesos, para si la ventana no se cierra, nunca toca la foto previa.

**Commit:** `feat(bench): timing a terminal's start`

### 5.4 Adaptador de `Desktop`
Features: `Win32_UI_WindowsAndMessaging`, `Win32_Graphics_Dwm`, `Win32_UI_HiDpi`, `Win32_UI_Input_KeyboardAndMouse`.
1. `launch`: `std::process::Command::new(exe).args(..).spawn()`, devuelve el pid (no se espera al hijo).
2. `windows`: `EnumWindows` + `IsWindowVisible` + `GetWindowThreadProcessId` + `GetWindowTextW` + `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)`.
3. `place`: `SetWindowPos` (`SWP_NOZORDER | SWP_NOACTIVATE`). `close`: `PostMessageW(WM_CLOSE)`. `foreground`: `GetForegroundWindow`.
4. `focus`: el método que funcionó en la sonda 1.4; después comprueba `GetForegroundWindow() == hwnd` y si no, error.
5. En `main` del banco, **lo primero**: `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` (referencia: `purifex/crates/purifex-platform/src/window/mod.rs`), para que todas las coordenadas sean físicas. Va en una función de `platform` que el binario llama.

**Commit:** `feat(bench): the desktop adapter`

### 5.5 🖥 La orden `startup`
1. `imperifex-bench startup <terminal> [--runs 10] [--cold] [--out archivo.json]`.
2. Medir en caliente (Warp abierto: cada tanda abre y cierra una ventana nueva) y anotar en «Estado».
3. Escribir en el README cómo lanza el autor la medida en frío (los pasos de D3: cerrar Warp del todo, PowerShell desde Inicio, la orden con ruta completa, volver a Warp).

**Commit:** `feat(bench): the startup order`

**PAUSA.**

---

## Bloque 6 · Pantalla y teclado: latencia y salida vista

### 6.1 Puertos `ScreenWatch` y `Keyboard`
1. `ScreenFrame { at: Duration /* en el reloj del banco */, width, height, bgra: Vec<u8> }`. `ScreenWatch::watch(region: Rect) -> Result<Box<dyn Watch>, String>`; `Watch::next(timeout) -> Result<Option<ScreenFrame>, String>` (`None` si no hubo frame nuevo).
2. `Keyboard::type_text(hwnd, text)` y `press(hwnd, Key)` con `Key { Enter, Backspace, Char(char) }`; ambos comprueban el primer plano (S3) **en el adaptador** y además el servicio lo comprueba antes.
3. **Un solo reloj:** el banco usa `QpcClock` (en `platform`, implementa `Clock` con `QueryPerformanceCounter`) y la pantalla convierte `LastPresentTime` a ese mismo reloj. Así `t1 - t0` compara lo mismo.

**Commit:** `feat(bench): ports for watching the screen and typing`

### 6.2 Adaptador DXGI
1. `platform/screen/dxgi.rs` según la sonda 1.1: salida del monitor que contiene la región; dispositivo; duplicación; textura `STAGING` del tamaño de la región creada una vez; `next`: `AcquireNextFrame`, ignora frames con `LastPresentTime == 0`, copia, mapea, lee por filas, `ReleaseFrame` siempre (guarda con `Drop`).
2. `DXGI_ERROR_ACCESS_LOST`: recrear la duplicación una vez; si vuelve a fallar, error.
3. Archivo de más de 400 líneas: partirlo (`screen/dxgi.rs`, `screen/staging.rs`).

**Commit:** `feat(bench): watching the screen with desktop duplication`

### 6.3 Adaptador de teclado
1. `SendInput` con `INPUT_KEYBOARD`: texto con `KEYEVENTF_UNICODE` por unidad UTF-16 (pulsar y soltar); teclas con su `VK_*`.
2. Antes de cada lote de eventos: `GetForegroundWindow() == hwnd`, o error.

**Commit:** `feat(bench): the keyboard adapter`

### 6.4 `app/latency.rs`
1. `LatencyService::run(profile, presses) -> Result<Measurement, String>`:
   - Abrir una ventana nueva (reutiliza lo de 5.3), `place` en el monitor principal en `(100, 100, 1280, 720)`, esperar a que se asiente (`settled_at`, 500 ms de silencio, máximo 10 s), `focus`.
   - Región: la ventana sin los 40 px de arriba.
   - Aprender la máscara 2 s (`ChangeDetector::learn`, frames cada vez que llegan).
   - Por pulsación: comprobar foco (S3); `t0`; `press(Char('a'))`; leer frames hasta que `changed(base, frame)`; muestra = `frame.at - t0`. Luego `Backspace`, esperar el cambio de vuelta, nueva base, pausa de 150 a 350 ms (secuencia de `XorShift64` con semilla fija). Sin cambio en 1 s: fallo. Más del 5 % de fallos: abortar con el motivo.
   - Cerrar la ventana.
2. Pruebas con falsos: la pantalla falsa cambia N ms después de la tecla y la muestra vale N; foco perdido aborta sin pulsar; los fallos se cuentan.

**Commit:** `feat(bench): key-to-screen latency`

### 6.5 `app/output_seen.rs`
1. `OutputService::run(profile, load, mib, repeats) -> Result<Vec<Measurement>, String>`: ventana nueva como en 6.4; escribir la orden `"<ruta del banco>" emit <load> --mib <n> --out "<temp>\<load>-<i>.json"` con `type_text` y `Enter` (`t0` al pulsar Enter); muestrear CPU de la terminal cada 250 ms en paralelo (`ProcessProbe`); esperar a que exista el JSON y la pantalla se asiente (500 ms); muestra vista = asentado − `t0`; muestra consumida = `seconds` del JSON. Entre repeticiones, escribir `cls` + Enter y esperar asentado.
2. Devuelve `OutputSeen(load)`, `OutputConsumed(load)` (MB/s) y `CpuDuringOutput(load)`.
3. Pruebas con falsos.

**Commit:** `feat(bench): output as the terminal shows it`

### 6.6 `app/memory_load.rs`
1. Ventana nueva; huella; `emit ascii` de 1 millón de líneas (~82 MiB) y esperar asentado; huella → `MemoryScrollbackPrivate` (la diferencia en MB).
2. Pestañas: pulsar `new_tab_keys` tres veces (asentado entre cada una) → `MemoryPerTabPrivate` = aumento medio por pestaña. Si el perfil no tiene `new_tab_keys`, se omite y se anota.
3. Pruebas con falsos.

**Commit:** `feat(bench): memory with a long history and per tab`

### 6.7 🖥 Órdenes `latency`, `output`, `memory`
1. `latency <terminal> [--presses 200]`, `output <terminal> [--load L|all] [--repeats 3]`, `memory <terminal>`, todas con `--out`.
2. Correr cada una una vez y anotar los números en «Estado».

**Commit:** `feat(bench): the latency, output and memory orders`

**PAUSA.**

---

## Bloque 7 · La línea base de Warp

### 7.1 La orden `all`
1. `imperifex-bench all <terminal> [--out-dir bench/results/<fecha>-<terminal>-<n>]`: en orden, `footprint` 60 s en reposo, `startup` en caliente ×10, `latency` ×200, `output all` ×3, `memory`. Un `report.json` con todo. Si una parte falla, las demás siguen y el informe lo anota.
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
5.1 → 5.2 → 5.3 → 5.4 → 5.5 🖥 → [PAUSA]
6.1 → 6.2 → 6.3 → 6.4 → 6.5 → 6.6 → 6.7 🖥 → [PAUSA]
7.1 → 7.2 → 7.3 🖥 → [PAUSA: plan de la fase T]
```

El camino más corto a un número útil: bloque 2 → 3 (salida) → 4 (huella). Las tareas sin 🖥 se pueden hacer mientras el autor trabaja.

## Riesgos

| Riesgo | Mitigación |
|---|---|
| Una tecla acaba en la sesión del autor | S1 y S3, comprobados en el adaptador y en el servicio; el falso hace fallar la prueba si se intenta |
| Warp abre ventanas nuevas en el mismo proceso, o pide confirmación al cerrar | Lo aclara la sonda 1.3 antes de escribir el bloque 5 |
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
- [x] 2.1 (2026-11-25) · [x] 2.2 (2026-11-25) · [x] 2.3 (2026-11-25) · [ ] 2.4 · [ ] 2.5 · [ ] 2.6
- [ ] 3.1 · [ ] 3.2 · [ ] 3.3 · [ ] 3.4 · [ ] 3.5
- [ ] 4.1 · [ ] 4.2 · [ ] 4.3 · [ ] 4.4
- [ ] 5.1 · [ ] 5.2 · [ ] 5.3 · [ ] 5.4 · [ ] 5.5
- [ ] 6.1 · [ ] 6.2 · [ ] 6.3 · [ ] 6.4 · [ ] 6.5 · [ ] 6.6 · [ ] 6.7
- [ ] 7.1 · [ ] 7.2 · [ ] 7.3
