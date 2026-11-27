# Imperifex · La terminal y su diseño (plan, 2026-11-25)

> Estado: **T1 en marcha** (T-D10, 2026-11-26). Las demás decisiones abiertas (tabla «Decisiones abiertas») esperan confirmación antes de su bloque.
> Sustituye al resumen «Fase T» de `docs/PLAN.md`. La línea base de Warp **se aplaza hasta T4** (T-D10).
> Las reglas de trabajo son las de `docs/PLAN.md` («Cómo se usa este plan») y D11.

## Objetivo

Una terminal que el autor use a diario en lugar de Warp, con el aspecto de `design/screen.png` (sin lo que no hace falta), tema claro y oscuro, español e inglés, y un panel que diga qué hacen sus agentes de consola.

## Qué se queda y qué se va del diseño

La imagen es la referencia de estilo, no una especificación.

### Se queda

| Parte | Cómo queda |
|---|---|
| Barra de título con pestañas y botones de ventana | Una sola franja, con el botón de pestaña nueva y su menú de perfiles (PowerShell, WSL…) |
| «Search & Palette» (`Ctrl+Shift+P`) | Paleta de órdenes, pestañas y ajustes. `Ctrl+P` no: lo usan las shells |
| Carpeta, botón de copiar, rama y commits por subir | Barra de contexto de la pestaña activa; se oculta lo que no se sabe |
| Bloques por orden: prompt, orden, duración y resultado | Resultado = bien o código de salida |
| Panel de agentes | Con datos reales; oculto por defecto, con atajo y un contador en la barra de estado |
| Paleta oscura con acento cian y fuente monoespaciada | Como tokens de tema, más su versión clara |
| Barra de estado | Mínima: perfil de la pestaña, agentes (trabajando / esperando) y avisos |

### Se va

| Parte | Motivo |
|---|---|
| Tabla de `cargo test`, «4 passed», rendimiento, «LLVM», «Zero-Copy Pool» | La terminal recibe texto, no datos. Habría que interpretar cada herramienta. La salida se muestra tal como llega |
| Sugerencia propia y caja «Tab to complete» | PSReadLine y las shells ya sugieren; `Tab` es de la shell. Su sugerencia se dibuja como la envíen |
| «GPU Accel (Vulkan)» | No es algo que el usuario decida |
| «Create PR» | Es un producto aparte; `gh` ya lo hace |
| Versiones de Rust y Node | Habría que detectarlas por proyecto y aportan poco |
| CPU, MEM, FPS, IPC | Solo en un modo de diagnóstico (T4). No se redibuja para mostrar un contador |
| «runtime», «UTF-8», «LF», «xterm-256color», «Synced» | Ruido, o sin significado definido |
| Icono de usuario | No hay cuentas |
| Versión junto al nombre | Va en Ajustes → Acerca de |
| «Agent Topology Settings», nombres y descripciones de ejemplo | Se cambia por Ajustes → Agentes; el panel muestra los agentes que existan |
| Pestaña «Swarm» | El panel lateral basta |
| Márgenes grandes y tarjeta alrededor de la salida | Quitan filas y columnas |
| Línea blanca sobre la barra de estado | Se toma como un defecto de la imagen |

## Decisiones tomadas (2026-11-25)

| # | Decisión | Motivo |
|---|---|---|
| T-D1 | Los agentes son **los que el autor ya usa** (Claude Code, Codex y otros de consola). El panel observa, avisa y salta a su pestaña. No llama a ninguna IA, no guarda claves y no les escribe | Elegido por el autor |
| T-D2 | Tema: `Sistema` (por defecto), `Oscuro` o `Claro`, desde Ajustes, **sin reiniciar** | Pedido del autor |
| T-D3 | Idioma: `Sistema` (por defecto), `Español (Latinoamérica)` o `English`, desde Ajustes, sin reiniciar. Con `Sistema`: Windows en español → español; cualquier otro → inglés | Pedido del autor; `Sistema` como valor inicial es propuesta de este plan |
| T-D4 | Tema e idioma existen **antes** de la primera pantalla: ningún color ni texto suelto en `ui` | Añadirlos después obliga a tocar todo |
| T-D5 | Las aplicaciones de pantalla alternativa (vim, monitores) usan todo el ancho, sin bloques ni márgenes | Necesitan la cuadrícula exacta |
| T-D6 | La terminal dibuja solo cuando algo cambia | Consumo en reposo |
| T-D7 | El motor funciona sin ventana antes de pintar nada | Se prueba y se confía en él primero |
| T-D8 | El espaciado de la interfaz usa una escala con **base 16**, como el `rem` de CSS: 4, 8, 12, 16, 24 y 32 (ver «Diseño: espaciado») | Pedido del autor: da aire parejo a todo |
| T-D9 | **Todo nativo**, como en Luminifex y Purifex: Rust y las API del sistema. Sin WebView, Electron, HTML, CSS, JavaScript ni motor web. La imagen del diseño es solo una referencia visual | Pedido del autor: rendimiento |
| T-D10 | **La terminal se programa ya; la línea base de Warp se aplaza hasta T4** (2026-11-26). B.1, B.4, la 7.3 y los fallos del banco sin explicar quedan pendientes en `docs/PLAN.md` y se cierran en T4.0 | Pedido del autor. Los números de Warp solo se necesitan para comparar, y no hay nada que comparar hasta que Imperifex se pueda medir |
| T-D11 | El intérprete de secuencias es el crate **`vte`**, detrás de un puerto propio (cierra A1). Antes de usarlo en T1.3 se revisa su `unsafe` | Aprobado por el autor (2026-11-26) |
| T-D12 | El ancho de cada carácter lo da el crate **`unicode-width`**, usado solo en `domain` | Aprobado por el autor (2026-11-26). Prohíbe `unsafe`; mantener tablas propias es más fácil de equivocar |

## Decisiones abiertas (cada una se cierra antes de su bloque)

| # | Pregunta | Recomendación | Antes de |
|---|---|---|---|
| A2 | ¿Dibujo? | Direct3D 11 (como Luminifex), no Vulkan: solo hay Windows por ahora | T2 |
| A3 | ¿Rasterizado de texto? | DirectWrite en `platform`. Revisa lo que dice la skill (`cosmic-text` en `ui`): la medida del texto entraría a `ui` por un puerto | T2 |
| A4 | ¿Ventana con `winit` o Win32 directo? | `winit`: menos código con `unsafe` propio. Confirmar con una sonda que permite la barra de título propia | T2 |
| A5 | ¿Fuente? | Incluir JetBrains Mono (licencia OFL, por verificar al incluirla), con Cascadia Mono de respaldo. Se puede cambiar en Ajustes | T2 |
| A6 | ¿Qué otras funciones de Warp necesitas? | Decidirlo al terminar T3, usando ya Imperifex | T5 |

## Fuera de este plan

- Motor de agentes propio, llamadas a API de modelos, claves y costos.
- Lanzar o definir agentes desde Imperifex, y responderles desde el panel.
- Sincronización, cuentas y telemetría.
- Paneles divididos (se revisa en A6).
- Linux y macOS (la arquitectura no les cierra la puerta). WSL sí entra, como un perfil de shell.
- Temas hechos por el usuario.

## Lo que dijo el bloque 1 del banco sobre el diseño (2026-11-25)

Los datos están en «Hechos medidos» de `docs/PLAN.md`. Esto es lo que toca a la terminal; ninguna decisión cambia todavía.

| Observado | Qué implica |
|---|---|
| La entrada de Warp la dibuja su propio editor, no el prompt de la shell | Con la shell nativa, cada tecla de Imperifex va y vuelve por ConPTY y PSReadLine. Su latencia al escribir en el prompt puede no ser comparable con la de Warp. **Decidir antes de T4 cuál es la vara**: la tecla en el prompt (lo que siente el usuario), la tecla dentro de un programa (compara el dibujo), o las dos (recomiendo las dos) |
| Warp muestra una sugerencia atenuada al escribir | En Imperifex la sugerencia la pone la shell (T-D1 no cambia; la caja propia sigue fuera). T3 comprueba que la de PSReadLine se ve bien en ambos temas |
| Al abrir una ventana, Warp lanzó muchos procesos cortos de `powershell`, `git` y `gh` (se supone que para sus avisos de carpeta y rama; no comprobado) | T5.3: la barra de contexto no lanza procesos al abrir ni en cada prompt sin medir su coste |
| Al cerrar una ventana, su shell siguió viva cerca de un minuto | T3.4: al cerrar una pestaña, Imperifex decide y dice qué pasa con sus procesos; el panel de agentes no muestra los de pestañas cerradas |
| La pantalla principal presenta a 144 Hz y la duplicación entrega los frames en menos de 0,2 ms (p95) | Sirve para medir T-D6 y la latencia de Imperifex con la misma herramienta |

## Conceptos nuevos

| Tipo (en `imperifex-domain`) | Qué es |
|---|---|
| `Grid`, `Cell`, `Scrollback` | La pantalla y su historial |
| `TerminalEvent` | Lo que el motor avisa: título, carpeta, marca de orden, notificación |
| `ShellProfile` | Cómo se abre una shell, como dato |
| `CommandBlock` | Orden, filas que ocupa, duración y código de salida |
| `Settings` | Tema, idioma, fuente, perfiles, atajos; con `Default` |
| `ThemeChoice`, `Palette` | Lo elegido, y los colores resultantes |
| `Space` | Un paso de la escala de espaciado |
| `LanguageChoice`, `Language`, `strings!` | Lo elegido, el idioma resultante y los textos |
| `AgentSession`, `AgentState`, `AgentEvent` | Un agente en una pestaña y lo que le pasa |
| `Model`, `Action` | Lo que dibuja `ui` y lo que `ui` pide |

Crate nuevo: `imperifex-ui` (depende solo de `domain`), añadido a la tabla de la guarda.

## Diseño: tema

Regla pura: `resolve_theme(choice, system) -> Theme`. El sistema llega por un puerto (`SystemAppearance`), que también avisa cuando cambia.

| Token | Oscuro | Claro | Uso |
|---|---|---|---|
| `background` | `#131315` | `#F4F4F5` | Ventana |
| `surface` | `#0F0F10` | `#FFFFFF` | Salida |
| `raised` | `#1C1C1F` | `#E9E9EC` | Pestaña activa, tarjetas |
| `text` | `#E4E4E7` | `#18181B` | Texto |
| `muted` | `#9A9AA3` | `#55555E` | Texto secundario |
| `accent` | `#4CD7F6` | `#006781` | Prompt, cursor, selección |
| `ok` | `#4ADE9B` | `#0F6B42` | Bien, trabajando |
| `warning` | `#F5C451` | `#7A5200` | Aviso |
| `error` | `#FF7B72` | `#B3261E` | Falló |
| `waiting` | `#C4A7FF` | `#6A3FC0` | El agente espera al autor |

Los valores oscuros son una aproximación a ojo de la imagen, no medidos de ella; el gris se aclaró para que se lea. Medido el 2026-11-25: los siete colores de texto dan al menos 5,3:1 sobre los tres fondos de su tema. Una prueba exige 4,5:1.

El estado nunca se dice solo con color: lleva también icono o palabra.

Además, cada tema tiene sus 16 colores ANSI. Los programas que mandan color de 24 bits suelen suponer un fondo: la terminal responde a la consulta del color de fondo (OSC 10/11) para que puedan adaptarse. Qué programas la usan se comprueba en T2.

## Diseño: espaciado

La idea se toma del `rem` de CSS, y **solo la idea** (T-D9): una base, normalmente 16, de la que salen las demás medidas. En Imperifex es código Rust: la base se define **una sola vez**, los pasos salen de ella y al dibujar son píxeles.

| Paso | En `rem` | Valor | Uso orientativo |
|---|---|---|---|
| `xs` | 0,25 | 4 | Entre un icono y su texto |
| `s` | 0,5 | 8 | Entre elementos relacionados |
| `m` | 0,75 | 12 | Relleno de botones y pestañas |
| `l` | 1 | **16 (base)** | Relleno de paneles, tarjetas y salida |
| `xl` | 1,5 | 24 | Entre grupos |
| `xxl` | 2 | 32 | Entre secciones, como en Ajustes |

- **Unidad:** píxeles lógicos, multiplicados por la escala de Windows (a 150 %, la base son 24 píxeles físicos). El resultado se redondea a píxel entero para que nada se vea borroso, y cambia al pasar la ventana a un monitor con otra escala.
- **En el código:** un `enum Space` en `domain`; `ui` no escribe márgenes ni rellenos como números sueltos. Si hace falta un paso más (48, 64), se añade a la escala.
- **No es espaciado:** bordes de 1 px, radios, iconos y tamaño de letra tienen sus propios valores.
- **La cuadrícula queda fuera:** dentro de la salida mandan las celdas, cuyo ancho y alto vienen de la fuente. La escala se aplica alrededor, no entre filas ni columnas. Lo que sobre al dividir el área en celdas queda como margen abajo y a la derecha.
- **Coste en la salida:** 16 a cada lado quita unas 4 columnas (estimado para celdas de unos 8,4 px de ancho; se confirma con la fuente elegida). La pantalla alternativa sigue sin márgenes (T-D5).

## Diseño: idioma

- `domain::i18n` con la macro `strings!`: una clave sin los dos textos **no compila**.
- Regla pura: `resolve_language(choice, system_locale) -> Language`.
- Se traduce la interfaz: menús, ajustes, panel, avisos y errores que ve el usuario. La salida de los programas no se toca. Nombres propios y órdenes, tampoco.
- Plurales y números con función propia y pruebas (1 agente, 2 agentes).
- La CLI de la terminal usa el mismo idioma. El banco de medidas sigue en español.
- La interfaz se prueba con los dos textos: ninguno se corta al ancho mínimo de ventana.

## Diseño: agentes

### Qué muestra el panel

Una tarjeta por agente: nombre, pestaña, carpeta, estado, desde cuándo y su último aviso.

| Estado | Significa |
|---|---|
| `Working` | Está trabajando |
| `Waiting` | Pide una respuesta o un permiso |
| `Done` | Terminó su turno y no se ha visto |
| `Idle` | Abierto, sin nada pendiente |
| `Unknown` | Se detectó el proceso, pero no informa su estado |
| (desaparece) | El proceso salió |

Primero van los que esperan. Un clic lleva a su pestaña. Si la ventana no está a la vista, `Waiting` y `Done` pueden avisar en Windows (se apaga en Ajustes).

### De dónde salen los datos (sin verificar: lo confirma la sonda A0)

1. **Detección:** en el árbol de procesos de cada pestaña, las imágenes conocidas (lista como dato). Da «hay un agente», con estado `Unknown`. Los agentes que corren como `node` necesitan además su línea de órdenes; los de WSL pueden no verse.
2. **Avisos del propio agente:** sus ganchos (*hooks* de Claude Code; la notificación de Codex) ejecutan `imperifex agent-event`, que manda el evento a la ventana por una tubería con nombre. Cada pestaña define `IMPERIFEX_TAB` para saber de dónde viene.
3. **Respaldo:** campana y notificaciones por secuencias de la terminal, para agentes sin ganchos.

**Nunca** se adivina el estado leyendo el texto de la pantalla.

### Seguridad

- Activar la integración cambia la configuración del agente: se muestra el cambio exacto antes, se guarda copia, se lee de vuelta y se puede deshacer sin pisar cambios hechos después.
- El gancho no frena ni rompe al agente: si Imperifex no está, sale bien y en silencio.
- La tubería es solo para el usuario actual. Lo que llega es dato no confiable: tamaño máximo, formato estricto; solo cambia el estado de una tarjeta y nunca ejecuta nada.
- Del evento solo se toma tipo, pestaña y un aviso corto. No se guardan conversaciones.

---

## Bloques

Aquí quedan las tareas y su criterio. Antes de empezar cada bloque, el planificador añade sus pasos exactos (archivos, tipos y API) con lo que digan las sondas y la línea base. Una tarea = un commit; cada bloque termina en **PAUSA**.

### A0 · Sondas de agentes (se pueden adelantar; solo lectura)
| # | Tarea | Hecho cuando |
|---|---|---|
| A0.1 | Qué eventos dan los ganchos de la versión instalada de Claude Code y la notificación de Codex, qué datos traen y cuánto tardan. Se prueba con configuración temporal de un proyecto de prueba, sin tocar la del usuario | Tabla «evento → `AgentState`» con lo observado |
| A0.2 | Cómo se ve cada agente en el árbol de procesos (nativo, `node`, WSL) | Lista de reglas de detección, con lo que no se puede detectar |

Si no se puede saber cuándo un agente espera, **parar** y replantear el panel.

### T1 · Motor sin ventana

#### Pasos de T1.1 · `crates/imperifex-platform/examples/probe_conpty.rs`
Sonda desechable: no es código del producto y no abre ventanas.
1. Features nuevas de `windows` en `imperifex-platform`, solo estas: `Win32_System_Console`, `Win32_System_Pipes`, `Win32_System_Threading`, `Win32_Security` (la pide la firma de `CreatePipe` y `CreateProcessW`).
2. Crear: dos tuberías, `CreatePseudoConsole` de 80×25, lista de atributos con `PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE` y `CreateProcessW` con `EXTENDED_STARTUPINFO_PRESENT`. Se pone `STARTF_USESTDHANDLES` con manejadores vacíos: sin eso, una shell lanzada desde un proceso con la salida redirigida puede escribir en la del padre y no en la consola falsa (sin comprobar: lo dirá la sonda).
3. Leer en un hilo aparte desde antes de lanzar y hasta el fin de la tubería.
4. Casos, cada uno con su veredicto: (a) `cmd` y Windows PowerShell escriben una marca y se lee; (b) escribir en la tubería de entrada una orden y leer su resultado; (c) cambiar el tamaño y que la shell vea el nuevo; (d) OSC 7, OSC 133 (A, B, C y D con código de salida) y OSC 9, con final BEL y con final ESC `\`, cada una entre marcas de texto: ¿llegan intactas y en su orden respecto al texto?; (e) cerrar: la shell sale sola, cuánto tarda y que la lectura termina.
5. Imprimir también lo que ConPTY manda por su cuenta al arrancar (por si pregunta algo y espera respuesta).
6. **Nunca** `TerminateProcess`. Si una shell no sale al cerrar la consola en 5 s, la sonda lo dice con su pid.
7. Anotar los resultados en «Hechos medidos» de este plan.

#### Pasos de T1.2 · `crates/imperifex-domain/src/grid/`
Solo la pantalla visible, sin colores. Atributos, pantalla alternativa, regiones, insertar y borrar líneas o caracteres: T1.4. Historial, cambiar de tamaño y recordar qué filas se cortaron solas: T1.5.
1. `cell.rs`: `Cell`: un carácter, las marcas que no ocupan columna (sin reservar memoria si no hay) y si ocupa una columna, dos, o es la segunda de uno doble. Vacía = un espacio.
2. `mod.rs`: `Grid::new(columns, rows)`, con cada medida entre 1 y 1000. Cursor desde 0.
3. `print(c)`:
   - ancho según `unicode-width`; los caracteres de control no hacen nada;
   - ancho 0: se une a la celda recién escrita (si es la segunda de un doble, a la primera); sin ninguna a su izquierda en la fila, se descarta. Como mucho 8 marcas por celda: la entrada no es de fiar;
   - al escribir en la última columna el cursor no salta: queda el salto **pendiente** y lo hace el siguiente carácter. Mover el cursor lo anula;
   - un doble que no cabe en lo que queda de la fila salta antes y deja esa columna vacía; en una cuadrícula de 1 columna se descarta;
   - escribir sobre media letra doble borra también la otra mitad.
4. Movimientos: `carriage_return`, `line_feed` (baja; en la última fila desplaza todo y la de arriba se pierde), `backspace`, `tab` (cada 8, sin pasar de la última columna), `move_to(row, column)` recortado a la cuadrícula.
5. `erase_in_line` y `erase_in_display` con `Erase::{ToEnd, ToStart, All}`, incluida la celda del cursor. No mueven el cursor y nunca dejan media letra doble.
6. Lectura: `cell(row, column)`, `row_text(row)` sin los espacios del final y `text()`.
7. Pruebas de cada punto, y una con miles de operaciones al azar (semilla fija) tras las cuales ninguna fila tiene media letra doble y el cursor está dentro.

**Sin comprobar, para T1.6:** el ancho se calcula carácter a carácter. Los emojis compuestos (unidos con ZWJ, o con el selector `U+FE0F`) pueden ocupar en ConPTY otras columnas que aquí y descuadrar el cursor.

**Commit:** `feat(terminal): the grid: writing, cursor, scrolling and wide and combining characters`

#### Pasos de T1.3 · el intérprete
`vte` solo **parte** los bytes en trozos con forma. **Qué significa** cada uno lo decide `domain`, con pruebas que no necesitan `vte`.
1. Dependencia: `vte = { version = "0.15", default-features = false }`, usada solo en `imperifex-platform`. **Sin `std` a propósito:** con ella, una secuencia OSC sin final crece en memoria sin límite; sin ella se corta a 1024 bytes.
2. `domain/sequence.rs`: `Sequence`: carácter, control, CSI (parámetros, marcas intermedias y letra final), ESC y OSC. Puerto `SequenceParser` en `ports.rs`: los bytes llegan cortados por cualquier sitio y nada es un error.
3. `domain/interpret.rs`: `apply(grid, sequence)`:
   - controles: BS, HT, LF (VT y FF igual) y CR;
   - CSI sin marcas: `A B C D E F G d H f` (mover; un 0 o nada cuenta como 1 y las posiciones empiezan en 1), `J K` (borrar, 0 a 2) y `X` (borrar caracteres);
   - ESC `D` (bajar) y `E` (nueva línea);
   - **todo lo demás se ignora sin fallar**, incluidos color, `?9001`, `?1004` y `?25`. Color, modos, regiones, insertar, borrar y desplazar son T1.4; los eventos OSC, T5 y T6.
4. `platform/parser.rs`: `VteParser`, portátil. Una secuencia que `vte` marca como incompleta no se entrega. DCS se descarta.
5. Pruebas: bytes → cuadrícula, con el arranque de `cmd` visto en T1.1 (sin rutas personales); cortar la entrada en cualquier byte da lo mismo; bytes al azar no hacen fallar nada; una OSC de 1 MB llega recortada.

**Revisión de `unsafe` de `vte` 0.15.0 (2026-11-26):** cinco usos en `lib.rs`, correctos: un arreglo de `MaybeUninit` sin iniciar; su lectura solo hasta donde se llenó (16 como máximo); y tres conversiones de texto hasta donde la validación UTF-8 dijo que era válido. No se revisó el `unsafe` de sus dependencias `arrayvec` y `memchr`.

**Commit:** `feat(terminal): the bytes of a shell reach the grid, through vte behind our own port`

#### Pasos de T1.4 · estilos, pantalla alternativa, regiones y modos
1. `grid/style.rs`: `Color` (por defecto, índice 0 a 255 o RGB) y `Style` (color de letra, de fondo y atributos: negrita, tenue, cursiva, subrayado, parpadeo, inverso, oculto y tachado). Cada `Cell` guarda el suyo; la cuadrícula tiene el estilo con el que escribe. **Lo que se vacía queda con el color de fondo en uso**, como en xterm.
2. `Sequence::Csi` conserva los sub-parámetros (`38:2::r:g:b`) en un tipo `Params`.
3. `interpret/sgr.rs`: SGR (`m`), con los colores de 16, 256 y 24 bits en sus dos escrituras (`;` y `:`). El color del subrayado (58) se lee y se descarta.
4. Región de desplazamiento (`r`), con el desplazamiento al bajar y al subir (`ESC M`) y las órdenes `S T L M`; subir y bajar el cursor se detiene en sus bordes. Insertar y borrar caracteres (`@ P`), sin dejar media letra doble.
5. Guardar y recuperar el cursor con su estilo (`ESC 7`, `ESC 8`, `CSI s`, `CSI u`). Reinicio total (`ESC c`).
6. `domain/terminal.rs`: `Terminal`, la pantalla activa, la principal guardada mientras se usa la alternativa y los modos que lee la ventana. `interpret::apply` pasa a actuar sobre él.
7. Modos: `?1` (teclas de cursor de aplicación), `?6` (origen), `?7` (salto automático), `?25` (cursor visible), `?47`, `?1047`, `?1048`, `?1049` (pantalla alternativa y cursor), `?2004` (pegado delimitado) y `4` (insertar). La pantalla alternativa empieza vacía cada vez. Ratón y foco: T3.
8. Fuera: tabuladores configurables, juegos de caracteres de dibujo (`ESC ( 0`; sin comprobar que ConPTY los deje pasar, se verá en T1.6) y las respuestas a consultas (las contesta ConPTY; no se vio ninguna dirigida a la terminal).

**Commit:** `feat(terminal): colours and styles, the alternate screen, scroll regions and modes`

#### Pasos de T1.5 · historial y reajuste
1. Cada fila recuerda si su línea **sigue** en la siguiente porque no cupo, y si dejó vacía la última celda porque un carácter doble no entraba.
2. `grid/history.rs`: el historial **no guarda celdas**: cada línea es su texto, sus tramos de estilo y si sigue; una línea que termina pierde sus celdas vacías del final. Tiene límite (10 000 filas por defecto) y olvida lo más viejo.
3. Entra en el historial lo que sale **por arriba de la pantalla** al desplazarse (también con una región que empieza en la primera fila); no lo borrado con `ESC[M` ni lo que sale de una región más baja. `ESC[3J` lo vacía. La pantalla alternativa no tiene.
4. Reajuste (`resize`): une las líneas que siguen y las corta al ancho nuevo, historial incluido, sin tener nunca todo el historial como celdas. El cursor se queda en su texto; las filas vacías bajo el cursor no empujan texto al historial. La pantalla alternativa solo se recorta o se amplía.
5. Pruebas, una de ellas con texto al azar por una cadena de anchos que debe volver igual.
6. `examples/measure_history.rs` en `platform` (feature `Win32_System_ProcessStatus`): memoria privada y tiempos con un millón de líneas.

**Commit:** `feat(terminal): a history with a limit, and long lines that are cut again when the width changes`

Los pasos de T1.6 y T1.7 se escriben antes de cada una.
| # | Tarea | Hecho cuando |
|---|---|---|
| T1.1 | Sonda ConPTY: crear, leer, escribir, cambiar tamaño, cerrar; ¿pasan intactas y en orden las secuencias OSC 7, 133 y 9? | Hechos anotados. Si OSC 133 no sirve, se replantea T5 **antes** de seguir |
| T1.2 | `Grid` y `Cell`: escritura, cursor, desplazamiento, anchos dobles y combinados | Pruebas |
| T1.3 | Puerto del intérprete y su adaptador (A1) | Secuencias de ejemplo dan la cuadrícula esperada; entradas basura no hacen caer el programa |
| T1.4 | Atributos y colores, pantalla alternativa, regiones y modos | Pruebas |
| T1.5 | Historial con límite y reajuste de líneas | Pruebas; memoria medida con 1 millón de líneas |
| T1.6 | Adaptador ConPTY y servicio de sesión | `imperifex run --headless` vuelca la pantalla de una orden real |
| T1.7 | Perfiles de shell como datos: PowerShell 7, Windows PowerShell, cmd, WSL | Cada uno que esté instalado corre en `--headless` |

### T2 · Ventana, dibujo, tema e idioma
| # | Tarea | Hecho cuando |
|---|---|---|
| T2.1 | `i18n` con `strings!` y `resolve_language` | Pruebas de la regla y de plurales |
| T2.2 | `Palette`, tokens, ANSI, `resolve_theme` y la escala `Space` | Pruebas, incluidas la de contraste y la de la escala a 100, 125 y 150 % |
| T2.3 | `Settings` con `Default`, lectura tolerante y escritura atómica en `%APPDATA%\Imperifex\settings.json` | Un archivo dañado o antiguo no impide arrancar; se conserva copia |
| T2.4 | Puertos y adaptadores de apariencia e idioma del sistema, con aviso de cambios | Cambiar el tema de Windows llega como evento (probado a mano) |
| T2.5 | Crate `imperifex-ui` y su fila en las guardas | Una guarda falla si `ui` tiene un color o un texto visible suelto (probada rompiéndola). Los márgenes y rellenos reciben `Space`, no números, para que lo impida el compilador |
| T2.6 | Ventana con barra de título propia: mover, maximizar, ajuste de Windows, varios monitores y DPI | Lista de comprobación en el PC real |
| T2.7 | Atlas de glifos y dibujo de la cuadrícula con daños, cursor y fuente | Se ve la salida de una shell; sin cambios no se dibuja |

### T3 · Uso diario
| # | Tarea | Hecho cuando |
|---|---|---|
| T3.1 | Teclado a secuencias, incluidas `AltGr` y teclas muertas (teclado en español) | Pruebas de la tabla; á, ñ, @, `\` en el PC real |
| T3.2 | IME y pegado seguro (pegado delimitado, aviso con varias líneas) | Probado a mano |
| T3.3 | Ratón, selección, copiar y pegar | Probado en shell y en pantalla completa |
| T3.4 | Pestañas y menú de perfiles; confirmar al cerrar con procesos vivos | Pruebas del servicio |
| T3.5 | Pantalla de Ajustes: tema, idioma, fuente, tamaño y shell por defecto | **Tema e idioma cambian sin reiniciar** y se recuerdan |
| T3.6 | Búsqueda en el historial y enlaces | Pruebas y uso real |

**Hito:** el autor trabaja un día completo en Imperifex, incluido Claude Code, y anota lo que falta (A6).

### T4 · Medir y optimizar
| # | Tarea | Hecho cuando |
|---|---|---|
| T4.0 | La línea base de Warp aplazada (T-D10): los fallos del banco sin explicar, B.1, B.4 y 7.3 de `docs/PLAN.md` | `docs/BASELINE.md`, con la versión de Warp medida |
| T4.1 | Perfil `imperifex` en `bench/terminals.json` y `all imperifex` | Informe junto al de Warp |
| T4.2+ | Una tarea por cada medida peor que Warp | Iguala o mejora, o el autor acepta la diferencia con su número |
| T4.final | Modo de diagnóstico (CPU, memoria, tiempo por frame), apagado por defecto | Apagado no cuesta redibujos |

### T5 · El diseño: contexto, bloques y paleta
| # | Tarea | Hecho cuando |
|---|---|---|
| T5.1 | Integración con la shell (OSC 133 y 7) para PowerShell y bash de WSL, cargada al abrir la pestaña **sin editar el perfil del usuario** | Convive con su perfil y su prompt; se apaga en Ajustes |
| T5.2 | `CommandBlock`: bloques con duración y código de salida; saltar entre ellos y copiar su salida | Sin integración, la terminal se ve y funciona normal |
| T5.3 | Barra de contexto: carpeta, copiar, rama y commits por subir, sin frenar el prompt | Un repo grande no retrasa la escritura; fuera de un repo no muestra rama |
| T5.4 | Paleta `Ctrl+Shift+P` | Órdenes, pestañas y ajustes en el idioma elegido |
| T5.5 | Repaso visual contra `design/screen.png` en ambos temas e idiomas | Cuatro capturas aprobadas por el autor |

### T6 · Agentes (requiere A0)
| # | Tarea | Hecho cuando |
|---|---|---|
| T6.1 | `AgentSession` y la regla `next_state(state, event)` | Pruebas de cada paso y de eventos tardíos, repetidos o de pestañas cerradas |
| T6.2 | Detección por árbol de procesos | Aparece y desaparece con el agente real |
| T6.3 | Tubería y orden `imperifex agent-event` | Pruebas con mensajes hostiles; sin Imperifex sale en silencio |
| T6.4 | Panel, contador en la barra de estado, atajo y salto a la pestaña | Con tres agentes se distingue cuál espera |
| T6.5 | Activar y desactivar la integración desde Ajustes → Agentes | Vista previa, copia y deshacer; pruebas solo en carpetas de prueba |
| T6.6 | Avisos de Windows | Solo con la ventana fuera de la vista; se apagan |
| T6.7 | Medir el coste del panel | Número anotado; comparado con la línea base sin panel |

### T7 · Distribución
Zip en Releases, bucket de Scoop, y terminal predeterminada de Windows como una orden que se puede deshacer. README en español e inglés.

---

## Orden y dependencias

```
T1 → T2 → T3 → [hito: uso diario] → T4 → T5 → T6 → T7
A0 (en cualquier momento) ───────────────────────────────────────────→ T6
```

Camino más corto a algo que se vea: T1 → T2. A algo que se use: hasta T3. Si prefieres los agentes antes que el diseño completo, T6 puede ir antes de T5: no depende de los bloques por orden.

## Criterios de aceptación

1. El autor usa Imperifex como terminal principal durante una semana.
2. Tema e idioma siguen al sistema, se cambian en Ajustes sin reiniciar y se recuerdan. Las cuatro combinaciones se revisan a ojo.
3. Con tres agentes reales, el panel muestra su estado correcto y lleva a la pestaña; sin integración dice «sin datos», no inventa.
4. Activar y desactivar la integración deja la configuración del agente como estaba.
5. Cada medida de la línea base: igual o mejor que Warp, o diferencia aceptada con su número.
6. `tools/check.ps1` y la CI en verde; ninguna ruta personal en el repo.

## Riesgos

| Riesgo | Mitigación |
|---|---|
| El motor VT es el trabajo más grande y sus errores se notan en programas reales | `vte` (A1), motor sin ventana, casos grabados de programas reales sin datos personales |
| ConPTY altera u omite secuencias | Sonda T1.1 antes de depender de ellas |
| Los ganchos de un agente cambian con una actualización | Tabla de eventos en un solo lugar; un evento desconocido no cambia el estado; queda la detección |
| La integración rompe la configuración del agente | Vista previa, copia, lectura de vuelta y deshacer |
| La barra de título propia pierde funciones de Windows | Lista de comprobación en T2.6; sonda en A4 |
| El teclado en español (`AltGr`, teclas muertas) falla | Tarea propia con pruebas (T3.1) |
| En tema claro, programas que suponen fondo oscuro se leen mal | OSC 10/11, ANSI propio por tema; evaluar contraste mínimo en T2 |
| El diseño y los agentes retrasan una terminal usable | Hito de uso diario antes de T5 y T6 |

## Hechos medidos

### T1.5 · Memoria y tiempo con un millón de líneas (2026-11-26, `measure_history` en release, una ejecución por caso, 80×25)

| | 40 caracteres sin color | 86 caracteres con 4 colores (2 filas de 80) |
|---|---|---|
| Filas en el historial | 1 000 000 | 1 000 000 (unas 500 000 líneas) |
| Memoria privada de más | 89 MB | 128 MB |
| Reajustar a 100, 60 y 80 columnas | 596, 576 y 610 ms | 826, 937 y 994 ms |
| Memoria privada de más tras cada reajuste | 91, 91 y 91 MB | 104, 154 y 138 MB |
| Pico de memoria privada del proceso | 192 MB | 239 MB |
| Llenarlo (incluye generar las líneas) | 1,59 s | 3,12 s |

- MB = 1 048 576 bytes. A 100 columnas el caso con color queda en 499 988 filas: cada línea ya cabe en una.
- El llenado incluye generar cada línea en la propia sonda y no dibuja nada: **no se compara** con las medidas de Warp.
- **Para T2:** reajustar un millón de filas tarda entre 0,6 y 1 s; no se puede hacer en cada paso de un arrastre de la ventana. Con el límite por defecto (10 000) no se midió.

### T1.1 · ConPTY (2026-11-26, `probe_conpty` en release, una ejecución, Windows 10.0.26200; `cmd` y Windows PowerShell; PowerShell 7 no está instalado y WSL no se probó)

| Caso | Visto |
|---|---|
| Crear, escribir y leer | Las dos shells escriben su marca y salen solas (778 y 983 ms, desde antes de crear la consola). Con la shell abierta, una orden escrita en la tubería de entrada da su resultado en 21 ms (`cmd`) y 60 ms (PowerShell); una sola muestra cada una |
| Cambiar tamaño | PowerShell ve 80 columnas antes y 100 después |
| Cerrar | Con la shell abierta, al cerrar la consola sale sola en 5 ms (`cmd`) y 16 a 17 ms (PowerShell), y la lectura termina. No hizo falta terminar nada |
| OSC 133 (A, B, C, D con código) | **Intactas y en su sitio** entre las marcas de texto, con final BEL y con final ESC `\` |
| 200 líneas con desplazamiento, cada una entre 133;A y 133;D | **200 de 200** en orden |
| OSC 7, OSC 9 y OSC 9;9 | Intactas, con los dos finales, pero **adelantadas** a texto escrito antes que ellas. En esta prueba, OSC 7 llegó antes incluso del borrado inicial de pantalla |
| Lo que ConPTY manda por su cuenta | `ESC[?9001h` (entrada en modo Win32) y `ESC[?1004h` (avisos de foco) al empezar, y los apaga al salir; borra la pantalla y manda el título (OSC 0). No se vio ninguna consulta que esperara respuesta |

**Qué implica:**
- OSC 133 sirve: T5 no se replantea.
- **La posición de OSC 7 y OSC 9 no es de fiar.** OSC 7 se trata como «la carpeta actual» y OSC 9 como un aviso, sin ligarlos a una fila. Un bloque toma la carpeta al llegar su 133;A (T5). Sin comprobar: si se adelantan también a texto que la consola ya había entregado.
- El motor (T1.3, T1.4) tiene que aceptar `?9001` y `?1004` sin romperse. Escribir texto normal y `` en la entrada funcionó con `?9001` activo.
- La sonda usa `STARTF_USESTDHANDLES` con manejadores vacíos, lanzada con la salida redirigida, y todo llegó por la consola falsa. No se probó sin eso.

## Estado

- [x] T1.5 (2026-11-26): historial con límite y reajuste. Un millón de filas: 89 MB sin color y 128 MB con color; reajustarlo, de 0,6 a 1 s. La prueba al azar encontró que se perdía texto al reajustar dos veces un carácter doble cortado; corregido, con su prueba.
- [x] T1.4 (2026-11-26): estilos, SGR, pantalla alternativa, regiones, insertar y borrar y los modos, en `domain`. Probado con pruebas.
- [x] T1.3 (2026-11-26): `vte` 0.15.0 sin `std` tras el puerto `SequenceParser`; el significado, en `domain::interpret`. Probado con pruebas; todavía no le llega ninguna shell. La prueba del límite de OSC se vio fallar con `std` activado (guardó 1 048 577 bytes).
- [x] T1.2 (2026-11-26): la cuadrícula en `imperifex-domain`, probada solo con pruebas; ningún programa real escribe aún en ella.
- [x] T1.1 (2026-11-26): sonda ConPTY; resultados en «Hechos medidos».
- 2026-11-26: el autor aplaza la línea base hasta T4 y aprueba `vte` (T-D10, T-D11). Empieza T1.
- 2026-11-25: plan escrito a partir de `design/screen.png`. Nada programado. Nada de A0 ni de las sondas de este plan verificado.
