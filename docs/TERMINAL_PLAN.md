# Imperifex · La terminal y su diseño (plan, 2026-11-25)

> Estado: **T1 completo** (2026-11-26). **Pasos de T2 escritos** (2026-11-27); se empieza por T2.1. Las demás decisiones abiertas (tabla «Decisiones abiertas») esperan confirmación antes de su bloque.
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
| T-D13 | Se dibuja con **Direct3D 11**, como Luminifex; no Vulkan (cierra A2) | Elegido por el autor (2026-11-27). Por ahora solo hay Windows |
| T-D14 | El texto lo rasteriza **DirectWrite**, en `platform`; la medida del texto llega a `ui` por un puerto (cierra A3). Se aparta de la skill `fexarium-rust`, que pone `cosmic-text` en `ui` | Elegido por el autor (2026-11-27) |
| T-D15 | La ventana se crea con **`winit`**, tras una sonda que confirme que permite la barra de título propia (cierra A4). Si no lo permite, se vuelve a preguntar | Elegido por el autor (2026-11-27). Menos `unsafe` propio |
| T-D16 | La fuente por defecto es **`Cascadia Mono`** (de `github.com/microsoft/cascadia-code`; sin ligaduras, porque la cuadrícula dibuja celda a celda), no JetBrains Mono (cierra A5). **No va incluida:** se busca por nombre en las fuentes instaladas (Windows 11 la trae en `C:\Windows\Fonts\CascadiaMono.ttf`; DirectWrite ve también las del usuario). Si no está, se usa `Consolas` y se avisa. Se puede cambiar en Ajustes | El autor respondió con el enlace (2026-11-27) y eligió variante y la fuente instalada el mismo día. Al no repartirla, no hay que llevar la licencia OFL |
| T-D17 | **Contraste mínimo en la cuadrícula:** si la letra y el fondo de una celda contrastan menos de **2:1**, el color de la letra se acerca al blanco o al negro (el que más contraste dé con ese fondo) hasta llegar a **3:1**. Por encima de 2:1, los colores son los del programa. Se puede apagar (`Settings`; pantalla en T3.5) | Elegido por el autor (2026-11-27). Ejemplo: la sugerencia de PSReadLine (`ESC[97;2;3m`, blanco brillante atenuado) no se vería sobre el fondo claro |
| T-D18 | **Entrada mínima provisional en T2** (T2.9): el texto tal como lo da Windows, Enter, Retroceso, Tab, Esc y las flechas. Lo demás (`AltGr`, teclas muertas, IME, atajos) sigue en T3.1, que la sustituye | Elegido por el autor (2026-11-27): sin escribir no se puede ver PSReadLine ni Claude Code reales, que A7 necesita |

## Decisiones abiertas (cada una se cierra antes de su bloque)

| # | Pregunta | Recomendación | Antes de |
|---|---|---|---|
| A7 | ¿Qué ancho damos a lo que ConPTY cuenta distinto (acentos sueltos, emojis compuestos, banderas)? Ver «Hechos medidos», T1.6 | Antes de decidir, ver en T2 qué pasa con PSReadLine y Claude Code reales. Candidata: contar como ConPTY, que es quien coloca el cursor en Windows; falta comprobar qué hacen los programas de WSL | T3 |
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

#### Pasos de T1.6 · la sesión
1. Puertos `PtyLauncher` y `Pty` en `domain`: leer con espera, escribir, cambiar tamaño, saber si salió y con qué código, y cerrar la consola. **Nunca** se termina un proceso: cerrar la consola es lo que le pide salir.
2. `domain/command_line.rs`: juntar programa y argumentos en una línea de órdenes de Windows, con sus reglas de comillas y pruebas.
3. `app/session.rs`: `Session` une la consola, el intérprete y el `Terminal`: avanzar, escribir y cambiar el tamaño (de los dos). `app/headless.rs`: correr hasta que el programa salga, esperar 200 ms sin salida, cerrar y leer hasta el final; con un tiempo máximo, tras el cual se cierra la consola y se dice. Si sigue vivo 5 s después, se dice con su número de proceso.
4. `platform/pty`: el adaptador ConPTY, partiendo de la sonda T1.1. Un hilo lee y entrega por un canal con límite; `ClosePseudoConsole` va en otro hilo, para que se pueda seguir leyendo mientras cierra.
5. Orden `imperifex run --headless [--columns N] [--rows N] [--timeout S] [--history] -- <programa> [argumentos]`: escribe la pantalla (con `--history`, también el historial). Sale con 0 solo si el programa salió con 0.
6. Comprobar con programas reales: que 5000 líneas lleguen todas y en orden; **qué ancho da ConPTY a los emojis compuestos** frente al nuestro (`examples/probe_widths.rs`); y si deja pasar los dibujos de `ESC ( 0`.

**Commit:** `feat(terminal): a real shell in a pseudoconsole, and run --headless prints its screen`

#### Pasos de T1.7 · perfiles de shell
1. `domain/shell.rs`: `ShellProfile` como tabla de datos (`pwsh`, `powershell`, `cmd`, `wsl`): programa, lo que va antes de una orden y cómo se le entrecomilla. `cmd` no sigue las reglas comunes: va con `/s /c` y la orden entre comillas, sin escapar.
2. `run --headless --profile <perfil> -- "<orden>"`: una sola orden, escrita para esa shell. Un perfil desconocido dice cuáles hay; una shell que no arranca se nombra.
3. Cómo se abre cada una **sin** orden (la sesión interactiva) se añade en T3.4, que es quien lo usa.
4. Correr cada perfil instalado, con comillas, una tubería y un código de salida.

**Commit:** `feat(terminal): shell profiles as data, and run --headless --profile`
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

Pasos escritos el 2026-11-27 (Opus), con T1 terminado y A2 a A5 cerradas (T-D13 a T-D18).

**Cómo encaja (vale para todo el bloque):**

```
 binario (winit: ventana y bucle; solo traduce eventos y conecta)
   │  eventos de winit → tipos de domain (Pointer, KeyInput, tamaño, escala)
   ▼
 app::window::Core ── tick() / model() / apply(Action)   (servicios, solo puertos)
   │ Model                          ▲ Action
   ▼                                │
 ui::frame(&Model, &Pointer, &dyn TextMeasure) → Frame { draw: DrawList, actions, terminal_area }
   │
   ▼
 platform: D3dRenderer (puerto Renderer): DrawList de la interfaz + la cuadrícula del Terminal
           DirectWriteText (puerto TextMeasure, y rasteriza glifos para el atlas)
```

- `ui` no dibuja la cuadrícula celda a celda: dice **dónde** va (`terminal_area`) y el renderizador la lee del `Terminal`. Una lista de 12 000 textos por frame sería el cuello de botella.
- Los colores de una celda (índice, por defecto, inverso, atenuado, oculto, contraste mínimo) los decide una función pura de `domain`; el renderizador solo la llama.
- La ventana dibuja solo cuando algo cambia (T-D6): `ControlFlow::Wait`, y se pide un redibujo solo por salida de la shell, entrada, tamaño, escala o tema.
- Las tareas **🖥** abren ventanas: regla 9 de `docs/PLAN.md` (permiso del autor en esa sesión y aviso antes).

#### Pasos de T2.1 · `crates/imperifex-domain/src/i18n/`
1. `mod.rs`: `LanguageChoice { System, Spanish, English }` (con `Default` = `System`) y `Language { Spanish, English }`. Regla pura `resolve_language(choice, system_locale: Option<&str>) -> Language`: con `System`, una etiqueta que empiece por `es` (sin distinguir mayúsculas: `es`, `es-MX`, `es-419`, `es-ES`) da español; cualquier otra, o ninguna, inglés.
2. `strings.rs`: la macro `strings!` declara `pub enum Text` y `pub fn text(language, Text) -> &'static str`. Cada entrada es `Clave { es: "…", en: "…" }`: si falta uno de los dos, **no compila** (el patrón de la macro exige los dos). Una prueba `compile_fail` en la documentación lo demuestra.
3. Textos de T2, y solo esos: el nombre de la app; los cuatro botones de la ventana (`Minimizar`, `Maximizar`, `Restaurar`, `Cerrar`); los avisos de ajustes dañados y de fuente no encontrada (T2.3, T2.8); la shell que no arranca (T2.9).
4. `plural.rs`: `count(language, n, Noun) -> String` («1 agente», «2 agentes», «0 agentes»; «1 agent», «2 agents»). En T2 solo existe `Noun::Agent`, para probar la regla.
5. `number.rs`: `decimal(language, value, digits) -> String`: coma en español, punto en inglés (como escribe este plan). Sin separador de miles.
6. Pruebas de cada regla. Una prueba recorre todas las claves y comprueba que ningún texto está vacío.

**Commit:** `feat(terminal): texts in Spanish and English, and the rule that picks one`

#### Pasos de T2.2 · tema, colores y espaciado (`domain`)
1. `theme/mod.rs`: `ThemeChoice { System, Dark, Light }` (`Default` = `System`), `Theme { Dark, Light }` y `resolve_theme(choice, system_dark: Option<bool>) -> Theme`; si no se sabe el del sistema, oscuro.
2. `theme/palette.rs`: `Rgb(u8, u8, u8)` y `Palette` con los diez tokens de «Diseño: tema» y `ansi: [Rgb; 16]`. `palette(theme) -> &'static Palette`, con estos ANSI (propuestos el 2026-11-27; medidos con la fórmula de WCAG: los índices 1 a 6 y 9 a 14 dan al menos **5,0:1** sobre los tres fondos de su tema, y el 8, al menos 3,2:1):

   | Índice | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
   |---|---|---|---|---|---|---|---|---|
   | Oscuro | `#1C1C1F` | `#FF7B72` | `#4ADE9B` | `#F5C451` | `#6CA8FF` | `#C4A7FF` | `#4CD7F6` | `#C8C8CE` |
   | Claro | `#18181B` | `#B3261E` | `#0F6B42` | `#7A5200` | `#1F5FBF` | `#6A3FC0` | `#006781` | `#E4E4E7` |

   | Índice | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
   |---|---|---|---|---|---|---|---|---|
   | Oscuro | `#6B6B75` | `#FFA198` | `#7EEDB8` | `#FFD97A` | `#9CC5FF` | `#DCCBFF` | `#8BE6FA` | `#F4F4F5` |
   | Claro | `#55555E` | `#9E1F18` | `#0B5C38` | `#664400` | `#1A50A3` | `#5A33A8` | `#00576E` | `#FFFFFF` |

   Negro (0) y blanco (7, 15) conservan su sentido en los dos temas: hay programas que pintan «blanco sobre negro» a propósito. Lo ilegible lo arregla el paso 4.
3. `theme/contrast.rs`: `luminance(Rgb)` y `contrast(a, b) -> f32` (WCAG 2). Prueba: los siete tokens de texto dan al menos 4,5:1 sobre los tres fondos de su tema; los ANSI, lo del paso 2.
4. `theme/cell_colors.rs`: `cell_colors(style, palette, adjust_contrast: bool) -> (Rgb, Rgb)` (letra, fondo): por defecto = `text` y `surface`; índices 0 a 15 de la paleta, 16 a 255 del cubo de xterm y los grises; inverso cambia letra y fondo; atenuado mezcla la letra al 50 % con el fondo; oculto = letra igual al fondo. Después, **T-D17**: por debajo de 2:1 la letra se acerca al blanco o al negro hasta 3:1 (no con oculto). Pruebas: cada caso, el de PSReadLine en tema claro (`97` atenuado sobre `#FFFFFF` acaba en 3:1 o más) y que un par con 2:1 o más no cambia.
5. `space.rs`: `enum Space { Xs, S, M, L, Xl, Xxl }`, `const BASE: f32 = 16.0` (la única vez que aparece), `Space::rem()` (0,25 a 2) y `Space::px(scale) -> u32`, redondeado. Pruebas a 1,0, 1,25 y 1,5 (`L`: 16, 20, 24; `Xs`: 4, 5, 6).

**Commit:** `feat(terminal): dark and light palettes with their ANSI colours, a minimum contrast, and the spacing scale`

#### Pasos de T2.3 · `Settings`
1. `domain/settings.rs`: `Settings { theme, language, font: FontSettings { family: String, size: f32 }, shell: Option<String>, adjust_contrast: bool }`, `Default` = `System`, `System`, `Cascadia Mono` a **12 pt** (como Windows Terminal), sin shell elegida (T2.9 prueba `pwsh` y luego `powershell`) y contraste mínimo encendido. Serde con `#[serde(default)]` en cada campo, y un campo `version: u32` (hoy 1).
2. Lectura tolerante, pura: `Settings::read(text) -> Read { settings, damaged: bool }`. JSON inválido → `Default` y `damaged`. Un campo de tipo equivocado o un valor fuera de rango (tamaño fuera de 6 a 72, tema desconocido) se cambia por su valor por defecto **y el resto se conserva**: se lee campo a campo desde `serde_json::Value`. Los campos desconocidos se ignoran. `Settings::write() -> String`, con sangría.
3. Puerto `SettingsStore` en `ports.rs`: `load() -> Result<Option<String>, String>` (`None` = no existe), `save(text)`, `keep_damaged(text) -> Result<String, String>` (devuelve dónde la guardó).
4. `app/settings.rs`: `SettingsService::load(store) -> (Settings, Option<Notice>)`: dañado → copia y aviso; ausente → valores por defecto sin escribir nada. `save` solo escribe si algo cambió. Pruebas con un doble que anota lo que le piden.
5. `platform/settings_file.rs`, solo `std`: `%APPDATA%\Imperifex\settings.json` (la carpeta se crea). Escritura atómica: `settings.json.tmp`, `sync_all`, `std::fs::rename` (en Windows es `MoveFileExW` con reemplazo). La copia dañada va a `settings.damaged.json`, o `settings.damaged-2.json` y siguientes, sin pisar ninguna. Pruebas en una carpeta temporal propia, nunca en `%APPDATA%`.

**Commit:** `feat(terminal): settings with defaults, read leniently and written atomically`

#### Pasos de T2.4 · el sistema: tema e idioma
1. Puertos: `SystemAppearance { fn dark(&self) -> Result<bool, String>; fn watch(&self, changed: Wake) -> Result<(), String> }` y `SystemLanguage { fn locale(&self) -> Option<String> }`. `Wake = Arc<dyn Fn() + Send + Sync>` va en `ports.rs` (lo usa también T2.9).
2. `platform/appearance.rs`: lee `AppsUseLightTheme` (DWORD) de `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` con `RegGetValueW`; si no existe, oscuro. `watch`: un hilo con `RegNotifyChangeKeyValue` sobre esa clave (`REG_NOTIFY_CHANGE_LAST_SET`, con evento), que llama a `changed` y vuelve a esperar; la clave se cierra con un `Drop`. No depende de la ventana: winit tiene su `ThemeChanged`, pero solo si no se fija el tema de la ventana, y se probaría tarde.
3. `platform/language.rs`: el primer idioma de `GetUserPreferredUILanguages(MUI_LANGUAGE_NAME)` (el de la interfaz de Windows, no el formato regional). La lista doble terminada en nulo se decodifica en código portátil con pruebas.
4. Features nuevas de `windows`: `Win32_System_Registry` y `Win32_Globalization`. Comprobar las firmas (regla 6).
5. La CLI usa el idioma (`docs/TERMINAL_PLAN.md`, «Diseño: idioma»): `HELP` y los errores de `Command::parse` y `run` pasan a `strings!`; `Command::run` recibe el `Language`, resuelto en el binario con `Settings` (T2.3) y `SystemLanguage`. Las pruebas actuales de la CLI siguen en español pasándole `Language::Spanish`; una nueva comprueba la ayuda en inglés.
6. `examples/watch_appearance.rs` en `platform`: imprime el tema y el idioma, y cada cambio durante 60 s. **A mano:** cambiar de oscuro a claro en Ajustes de Windows y anotar cuánto tarda en llegar.

**Commit:** `feat(terminal): the system's light or dark mode and language, and the command line speaks it`

#### Pasos de T2.5 · `crates/imperifex-ui`
1. Crate nuevo, solo con `imperifex-domain`; `#![forbid(unsafe_code)]` y `[lints] workspace = true`. Fila en la guarda: `("imperifex-ui", &["imperifex-domain"])`. El binario podrá depender de él y de `winit` (fila del binario: `+ "imperifex-ui", "winit"`).
2. Tipos que cruzan capas, en `domain`:
   - `draw.rs`: `Rect { x, y, width, height }` en píxeles físicos (`i32`/`u32`), `DrawList { rects: Vec<(Rect, Rgb)>, texts: Vec<TextRun> }` y `TextRun { x, y, text, color, font: UiFont }`, con `UiFont { Text, Icons }`.
   - `view.rs`: `Model { title, language, palette: &'static Palette, scale, window: (u32, u32), maximized, focused }`, `Action { Minimize, ToggleMaximize, Close, Drag, Resize(Edge), SystemMenu }`, `Edge` (ocho bordes) y `Pointer { position: Option<(i32, i32)>, pressed, released, double_click, right_click }`.
   - Puerto `TextMeasure`: `width(&self, text, UiFont, scale) -> u32` y `line_height(UiFont, scale) -> u32`.
3. `ui/src/lib.rs`: `frame(&Model, &Pointer, &dyn TextMeasure) -> Frame { draw, actions, terminal_area: Rect, cursor: CursorIcon }`. En T2.5 solo existe una función de prueba que arma un rectángulo con relleno `Space::L`: la barra de título real es T2.7.
4. `ui/src/layout.rs`: `inset(rect, Space, scale)` y `row(...)`: **toda** función de márgenes y rellenos recibe `Space`, nunca un número, para que lo impida el compilador. Los tamaños que no son espacio (alto de la barra, ancho de un botón, borde de 1 px) son constantes con nombre en `ui/src/metrics.rs`, en píxeles lógicos.
5. Guardas nuevas en `architecture.rs`, cada una probada rompiéndola una vez a mano (y anotado en el commit):
   - `ui_has_no_loose_colours`: en el código de producción de `ui` no aparecen `Rgb(`, `#` seguido de 6 dígitos hexadecimales ni `0x` con 6 u 8.
   - `ui_has_no_loose_visible_text`: en `ui` no hay literales de texto salvo `""`; los textos llegan por `Text` o del `Model`.
6. Pruebas de `layout` y de que `frame` no da acciones sin puntero.

**Commit:** `feat(terminal): the interface crate, with guards against loose colours and texts`

#### Pasos de T2.6 · 🖥 sonda de winit (T-D15)
Sonda desechable en `apps/imperifex/examples/probe_titlebar.rs`, con la técnica de Luminifex (`with_decorations(false)`, `with_undecorated_shadow(true)`, `drag_window`, `drag_resize_window`, `show_window_menu`). Dependencia nueva: `winit = "0.30"` en `[workspace.dependencies]`, usada solo por el binario (`raw-window-handle` llega reexportado por winit: no es una dependencia más). La sonda no dibuja nada: basta con que se vean el borde y la sombra de la ventana.
1. Una ventana sin barra del sistema; una franja de 40 px lógicos arriba hace de barra (arrastrar = `drag_window`, doble clic = maximizar o restaurar, clic derecho = `show_window_menu`) y 8 px en cada borde redimensionan.
2. Lista, con veredicto «sí / no / parcial» y lo que se vio:
   - (a) mover; (b) arrastrar al borde de la pantalla y a una esquina ajusta la ventana (Aero Snap); (c) `Win`+flechas; (d) **menú de diseños de ajuste** al pasar el ratón sobre el botón de maximizar (Windows 11). Se espera **no**: winit 0.30.13 no responde a `WM_NCHITTEST` con `HTMAXBUTTON` (leído en su código el 2026-11-27);
   - (e) maximizada ocupa justo el área de trabajo, sin tapar la barra de tareas ni dejar píxeles fuera; (f) minimizar y restaurar desde la barra de tareas; (g) `Alt`+`Espacio`;
   - (h) pasar a `\\.\DISPLAY2` y volver; si los dos monitores tienen la misma escala, cambiar la de uno al 125 % solo durante la prueba y anotar si llega `ScaleFactorChanged` con el tamaño propuesto;
   - (i) la sombra y las esquinas redondeadas de Windows 11.
3. Anotar en «Hechos medidos».
4. **Si algo de (a) a (i) falla**, parar y preguntar al autor (T-D15). Si solo falla (d), la propuesta es reemplazar el procedimiento de la ventana de winit (`SetWindowLongPtrW` con `GWLP_WNDPROC`, llamando al de winit con `CallWindowProcW`) para responder `HTMAXBUTTON` sobre el botón: unas pocas líneas de `unsafe` en `platform`. La otra opción es vivir sin ese menú.

**Commit:** `docs(plan): what winit allows for our own title bar` (con la sonda).

#### Pasos de T2.7 · 🖥 la ventana con su barra
1. `platform/render/`: `D3dRenderer`, que implementa el puerto `Renderer` (`resize(width, height)`, `draw(&DrawList, Option<TerminalPicture>) -> Result<(), String>`; `TerminalPicture` se usa desde T2.9). Recibe la ventana como `isize` (el `HWND` que el binario saca de `raw-window-handle` vía winit). Partir de `dockview-platform/src/render` y `ui_gpu.rs`: dispositivo D3D11, cadena de intercambio `FLIP_DISCARD` con 2 búferes, `ResizeBuffers` al cambiar el tamaño, `Present(1, 0)`, y sombreadores HLSL compilados al arrancar con `D3DCompile`.
2. Dibujo por instancias: un rectángulo = una instancia (posición, tamaño, color, coordenadas en el atlas y si es glifo); el sombreador de vértices lo abre en 4 esquinas con `SV_VertexID`. Mezcla alfa normal. El atlas (T2.8) ya existe vacío: textura `R8_UNORM` de 2048×2048.
3. Features nuevas de `windows`: `Win32_Graphics_Direct3D`, `Win32_Graphics_Direct3D_Fxc`, `Win32_Graphics_Direct3D11`, `Win32_Graphics_Dxgi`, `Win32_Graphics_Dxgi_Common`, `Win32_Graphics_Dwm` (esquinas y borde, como `set_window_frame` de Luminifex).
4. `ui/src/titlebar.rs`: la franja (tokens `background` y `raised`), el espacio de las pestañas (vacío hasta T3.4) y los tres botones (46 × 40 px lógicos; cerrar se pone rojo `error` al pasar). Devuelve `Action::Drag` al pulsar en la franja, `ToggleMaximize` con doble clic, `SystemMenu` con clic derecho y `Resize(edge)` en los 8 px del borde (sin borde si está maximizada). Los iconos llegan en T2.8; hasta entonces, rectángulos.
5. `app/window.rs`: `Core` con `Settings`, `Theme` y `Language` resueltos, tamaño, escala, `maximized` y `focused`; `model()`, `apply(Action) -> Vec<WindowCommand>` (lo que el binario pide a winit) y `system_theme_changed()`. Pruebas con dobles: un cambio de tema del sistema con `ThemeChoice::System` cambia la paleta del `Model`; con `Dark`, no.
6. Binario: con `imperifex` sin argumentos (o `imperifex window`; hoy sin argumentos es la ayuda: pasa a abrir la ventana, y `help` sigue) se abre la ventana. `src/window/` traduce eventos de winit a `Pointer`, tamaño y escala, llama a `ui::frame` y al renderizador, y ejecuta los `WindowCommand`. `EventLoopProxy` lleva a la ventana el aviso de `SystemAppearance::watch`. Tamaño mínimo: 640 × 400 lógicos.
7. Se dibuja solo con un evento que lo pida; anotar el uso de CPU con la ventana quieta 60 s (Administrador de tareas o `footprint` del banco).
8. Comprobar a mano (a) a (i) de T2.6 con la barra real, más: cambiar el tema de Windows con la ventana abierta y verla cambiar sin reiniciar.

**Commit:** `feat(terminal): a window with its own title bar, drawn with Direct3D 11 only when something changes`

#### Pasos de T2.8 · 🖥 texto con DirectWrite
1. `platform/text/`: `DirectWriteText`, que implementa `TextMeasure` y da al renderizador los glifos. Feature nueva: `Win32_Graphics_DirectWrite`.
   - Fuente: `GetSystemFontCollection` y `FindFamilyName("Cascadia Mono")` (o la de `Settings`); si no está, `Consolas` y el aviso de T2.1, una vez. Negrita y cursiva con `GetFirstMatchingFont` (en Windows 11 es una fuente variable: comprobar que da las caras negrita y cursiva, y anotarlo).
   - Iconos: `Segoe Fluent Icons` (en Windows 11, `SegoeIcons.ttf`), y si no está, `Segoe MDL2 Assets`. Minimizar `U+E921`, maximizar `U+E922`, restaurar `U+E923`, cerrar `U+E8BB`.
   - Carácter que no está en la fuente: `IDWriteFactory2::GetSystemFontFallback` y `MapCharacters`. Los emojis se dibujan **en un color**, el de la letra: el color de los emojis queda fuera de T2 (anotado para T5.5).
2. Medida de la celda, pura y probada en `domain` (`cell_size(metrics, size_pt, scale) -> CellSize { width, height, baseline }`), con los datos de `DWRITE_FONT_METRICS` y el avance del glifo `0`: tamaño en px = puntos × escala × 96 / 72; ancho, alto (`ascent + descent + lineGap`) y línea base redondeados a píxel entero.
3. Rasterizado: `IDWriteFactory2::CreateGlyphRunAnalysis` con antialias en **escala de grises** (`DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE`, modo `NATURAL_SYMMETRIC`) y `CreateAlphaTexture` con `DWRITE_TEXTURE_ALIASED_1x1`: un byte por píxel, que va al atlas. ClearType queda fuera: depende del fondo, y la sombra y los temas lo complican.
4. Atlas (`domain/atlas.rs`, puro): `GlyphKey { glyph, font_face, bold, italic, size }` → lugar en la textura, por estantes (*shelf packing*). Lleno: se vacía entero y se rehace en el siguiente frame (se anota con `log_info!`). Pruebas del empaquetado y del vaciado.
5. La barra de título usa el texto: nombre de la app e iconos de los botones, cada botón con su texto de ayuda en el idioma elegido (`Text::Minimize`…) dibujado al pasar el ratón 600 ms; si cuesta, la ayuda pasa a T3 y se dice.
6. Al cambiar de escala (otro monitor) se mide de nuevo la celda y se vacía el atlas.

**Commit:** `feat(terminal): text and icons with DirectWrite, in a glyph atlas`

#### Pasos de T2.9 · 🖥 la shell en la ventana
1. **Refactor primero, con las pruebas sin tocar salvo la construcción:** `Session` pasa a tener su propio `Box<dyn SequenceParser>` (no un `&mut` con vida), y `Shells` da una fábrica `new_parser: &dyn Fn() -> Box<dyn SequenceParser>`: cada pestaña de T3.4 tendrá el suyo.
2. `PtyLauncher::start` recibe un `Wake` (T2.4). El adaptador ConPTY lo llama tras dejar salida en el canal, **solo si no hay ya un aviso pendiente** (`AtomicBool` que `read` baja): una ráfaga no inunda la ventana de avisos. `headless` pasa uno que no hace nada.
3. Daños en `Grid`: `take_damage() -> Damage { rows: Vec<bool> }` (o `all`), puesto por cada escritura, borrado, desplazamiento, cambio de tamaño y cambio de pantalla. Prueba al azar (semilla fija): tras cada operación, toda fila cuyo contenido cambió está marcada.
4. `app/window.rs`, `Core::tick(clock)`: lee la salida hasta que no haya más o hasta **8 ms**; si quedó salida, pide otro `tick` justo después del dibujo. El dibujo se pide solo si hubo daños o se movió el cursor.
5. Abrir la shell: `Settings.shell`, o si no hay, `pwsh` y luego `powershell`, con solo su programa (sin orden). Eso es lo mínimo que T2 necesita; argumentos, carpeta y variables de la pestaña siguen en T3.4. Si no arranca ninguna, el aviso de T2.1 dentro de la ventana.
6. Cambio de tamaño: la ventana se redibuja al momento (la cuadrícula vieja recortada, o con fondo `surface` donde sobra), y el `Terminal` y la consola se ajustan cuando el tamaño lleva **100 ms** sin cambiar (regla pura en `domain/resize_wait.rs`, con pruebas; el `Core` pide un `WaitUntil`). Medir cuánto tarda `Terminal::resize` con las 10 000 filas por defecto llenas, y anotarlo.
7. Renderizador: `TerminalPicture { terminal, area, palette, adjust_contrast, cell }`. Una instancia por fondo distinto del de la ventana y una por glifo. Las instancias de cada fila se guardan y solo se rehacen las de las filas dañadas; se sube todo con `Map(WRITE_DISCARD)`. Los anchos dobles ocupan dos celdas. Subrayado y tachado como rectángulos de 1 px por escala. Cursor: bloque `accent` con el carácter encima en `surface`; hueco si la ventana no tiene el foco; nada si `?25` lo ocultó. **Sin parpadeo** (T-D6).
8. La cuadrícula va dentro de `terminal_area` con `Space::L` alrededor; en la pantalla alternativa sin márgenes (T-D5). Lo que sobre al dividir en celdas queda abajo y a la derecha.
9. **Entrada mínima (T-D18)**: el binario traduce `KeyEvent` de winit a `domain::input::KeyInput { text: Option<String>, key: Option<NamedKey> }`, y la regla pura `bytes(KeyInput, Modes) -> Vec<u8>` da: el texto en UTF-8; Enter `\r`; Retroceso `\x7f`; Tab `\t`; Esc `\x1b`; flechas `ESC[A`… o `ESCOA`… con `application_cursor_keys`. Nada más. Pruebas de la tabla.
10. Con la ventana quieta y la shell en reposo, ningún dibujo (contarlos con un contador en `log_info!` al cerrar, o el uso de CPU durante 60 s, anotado).

**Commit:** `feat(terminal): a shell in the window: damaged rows, cursor, colours, a minimal keyboard and a resize that waits`

#### Pasos de T2.10 · 🖥 programas reales y la respuesta del color de fondo
1. OSC 10 y 11 con `?`: `Terminal` guarda respuestas pendientes (`take_replies() -> Vec<u8>`), `interpret` responde `ESC]11;rgb:rrrr/gggg/bbbb ESC\` con el `surface` del tema (y `text` para el 10), y `Core` las escribe en la consola. El `Terminal` recibe los dos colores al crearse y al cambiar el tema. **Antes**, comprobar con `run --headless` y un `printf` de la consulta si ConPTY la deja pasar hasta nosotros; si la contesta él o la borra, no se programa y se anota.
2. Mirar con el autor, en los dos temas: el prompt y la sugerencia de PSReadLine, `git log --color`, `Get-ChildItem` en color, una aplicación de pantalla completa (`edit`, o `vim` si está) y **Claude Code**. Anotar en «Hechos medidos» lo que se ve mal, y para A7 si el cursor se descuadra con textos de la tabla de T1.6 (emojis, banderas, acentos sueltos).
3. Medidas anotadas, una ejecución cada una: CPU y memoria en reposo 60 s; frame con la pantalla entera cambiando (`cmd /c dir /s C:\Windows` unos segundos); `Terminal::resize` de T2.9.
4. `ARCHITECTURE.md`: `imperifex-ui` en la tabla y el dibujo de capas; «Estado».

**Commit:** `feat(terminal): answer the background colour query, and what real programs look like`

| # | Tarea | Hecho cuando |
|---|---|---|
| T2.1 | `i18n` con `strings!` y `resolve_language` | Pruebas de la regla y de plurales |
| T2.2 | `Palette`, tokens, ANSI, `resolve_theme`, colores de una celda con contraste mínimo y la escala `Space` | Pruebas, incluidas la de contraste y la de la escala a 100, 125 y 150 % |
| T2.3 | `Settings` con `Default`, lectura tolerante y escritura atómica en `%APPDATA%\Imperifex\settings.json` | Un archivo dañado o antiguo no impide arrancar; se conserva copia |
| T2.4 | Puertos y adaptadores de apariencia e idioma del sistema, con aviso de cambios; la CLI en el idioma | Cambiar el tema de Windows llega como evento (probado a mano) |
| T2.5 | Crate `imperifex-ui` y su fila en las guardas | Una guarda falla si `ui` tiene un color o un texto visible suelto (probada rompiéndola). Los márgenes y rellenos reciben `Space`, no números, para que lo impida el compilador |
| T2.6 🖥 | Sonda de winit para la barra de título propia (T-D15) | Veredicto de (a) a (i); si falla algo, el autor decide antes de T2.7 |
| T2.7 🖥 | Ventana con barra de título propia: mover, maximizar, ajuste de Windows, varios monitores y DPI | Lista de comprobación en el PC real; el tema de Windows cambia la ventana sin reiniciar |
| T2.8 🖥 | Texto e iconos con DirectWrite y el atlas de glifos | La barra muestra su texto e iconos a 100 % y a otra escala |
| T2.9 🖥 | La cuadrícula en la ventana, con daños, cursor, fuente, entrada mínima y reajuste con espera | Se ve y se usa una shell; sin cambios no se dibuja (número anotado) |
| T2.10 🖥 | OSC 10/11 y revisión con programas reales, Claude Code incluido | Hechos anotados, también para A7 |

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
| La barra de título propia pierde funciones de Windows | Sonda T2.6 antes de programar la ventana; lista de comprobación en T2.7 |
| El teclado en español (`AltGr`, teclas muertas) falla | Tarea propia con pruebas (T3.1) |
| En tema claro, programas que suponen fondo oscuro se leen mal | OSC 10/11, ANSI propio por tema; evaluar contraste mínimo en T2 |
| El diseño y los agentes retrasan una terminal usable | Hito de uso diario antes de T5 y T6 |

## Hechos medidos

### T1.7 · Perfiles (2026-11-26, en release, una ejecución)

| Perfil | Resultado |
|---|---|
| `powershell` | Instalado. Orden simple, comillas dobles con tubería y `exit 5`: bien |
| `cmd` | Instalado. Orden simple, comillas con tubería y `exit 6`: bien |
| `pwsh` | **No instalado, sin probar.** El error dice «PowerShell 7 (¿está instalado?)» |
| `wsl` | **Sin probar:** está `wsl.exe` pero no el subsistema. No se ejecutó el perfil, para no arrancar una instalación (sin comprobar que la ofrezca) |

### T1.6 · Una shell real (2026-11-26, `imperifex run --headless` y `probe_widths` en release, una ejecución, Windows 10.0.26200, `cmd` y Windows PowerShell)

- **5000 líneas** escritas por PowerShell llegaron **las 5000, iguales y en orden** (pantalla más historial).
- Al agotarse el tiempo con `cmd` abierto, cerrar la consola bastó para que saliera. Un programa que no existe da el error de Windows. El código de salida 7 llega.
- **`ESC ( 0`** (dibujos de líneas): ConPTY los entrega ya dibujados (`┌──┐`). No hace falta en el motor, al menos para lo que escribe PowerShell.
- **Anchos.** «Consola» es la columna que Windows PowerShell lee con `[Console]::CursorLeft` tras escribir el texto desde la columna 0; «nuestra», la de la cuadrícula:

| Texto | Consola | Nuestra |
|---|---|---|
| `e` + `U+0301` | 2 | 1 |
| `漢字` | 4 | 4 |
| `🚀` | 2 | 2 |
| Familia unida con ZWJ (3 personas) | 8 | 6 |
| `❤` + `U+FE0F` / sin él | 2 / 1 | 1 / 1 |
| Bandera (2 indicadores) | 4 | 2 |
| Pulgar con tono de piel | 4 | 4 |
| `1` + `U+FE0F` + `U+20E3` | 3 | 1 |
| `→±…│` | 4 | 4 |

- Por los números, la consola parece dar columna propia a lo que para `unicode-width` mide 0 (inferencia). Tras un texto de los que difieren, lo que la consola coloque por posición caería en otra columna que en nuestra cuadrícula (sin comprobar con un programa real). Decisión abierta **A7**.

### T1.5 · Memoria y tiempo con un millón de líneas (2026-11-26, `measure_history` en release, una ejecución por caso, 80×25)

| | 40 caracteres sin color | 86 caracteres con 4 colores (2 filas de 80) |
|---|---|---|
| Filas en el historial | 1 000 000 | 1 000 000 (unas 500 000 líneas) |
| Memoria privada de más | 89 MB | 128 MB |
| Reajustar a 100, 60 y 80 columnas | 596, 576 y 610 ms (otra corrida: 646, 633 y 618) | 826, 937 y 994 ms (otra corrida: 835, 952 y 1225) |
| Memoria privada de más tras cada reajuste | 91, 91 y 91 MB | 104, 154 y 138 MB |
| Pico de memoria privada del proceso | 192 MB | 239 MB |
| Llenarlo (incluye generar las líneas) | 1,59 s | 3,12 s |

- MB = 1 048 576 bytes. A 100 columnas el caso con color queda en 499 988 filas: cada línea ya cabe en una.
- El llenado incluye generar cada línea en la propia sonda y no dibuja nada: **no se compara** con las medidas de Warp.
- **Para T2:** reajustar un millón de filas tarda entre 0,6 y 1,2 s; no se puede hacer en cada paso de un arrastre de la ventana. Con el límite por defecto (10 000) no se midió.

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

- [x] T2.1 (2026-11-27): `domain::i18n` (regla del idioma, `strings!`, plurales y números). La prueba `compile_fail` y su gemela que sí compila pasan.
- [x] T1.7 (2026-11-26): perfiles como datos; `powershell` y `cmd` probados; `pwsh` y `wsl` sin probar. **T1 completo. PAUSA:** antes de T2 se cierran A2 a A5 y el planificador escribe sus pasos.
- 2026-11-27: A2 a A5 cerradas (T-D13 a T-D16). Faltan los pasos de T2.
- 2026-11-27: pasos de T2 escritos (Opus). T-D16 confirmada (`Cascadia Mono`, la instalada, `Consolas` si falta); nuevas T-D17 (contraste mínimo) y T-D18 (entrada mínima en T2). T2 pasa de 7 a 10 tareas: la sonda de winit (T2.6), el texto (T2.8) y los programas reales (T2.10) van aparte. Sigue T2.1.
- [x] T1.6 (2026-11-26): ConPTY, sesión y `run --headless`, probados con `cmd` y Windows PowerShell. Los anchos difieren en 5 de 10 casos: A7.
- [x] T1.5 (2026-11-26): historial con límite y reajuste. Un millón de filas: 89 MB sin color y 128 MB con color; reajustarlo, de 0,6 a 1,2 s. La prueba al azar encontró que se perdía texto al reajustar dos veces un carácter doble cortado; corregido, con su prueba.
- [x] T1.4 (2026-11-26): estilos, SGR, pantalla alternativa, regiones, insertar y borrar y los modos, en `domain`. Probado con pruebas.
- [x] T1.3 (2026-11-26): `vte` 0.15.0 sin `std` tras el puerto `SequenceParser`; el significado, en `domain::interpret`. Probado con pruebas; todavía no le llega ninguna shell. La prueba del límite de OSC se vio fallar con `std` activado (guardó 1 048 577 bytes).
- [x] T1.2 (2026-11-26): la cuadrícula en `imperifex-domain`, probada solo con pruebas; ningún programa real escribe aún en ella.
- [x] T1.1 (2026-11-26): sonda ConPTY; resultados en «Hechos medidos».
- 2026-11-26: el autor aplaza la línea base hasta T4 y aprueba `vte` (T-D10, T-D11). Empieza T1.
- 2026-11-25: plan escrito a partir de `design/screen.png`. Nada programado. Nada de A0 ni de las sondas de este plan verificado.
