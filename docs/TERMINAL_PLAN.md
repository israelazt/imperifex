# Imperifex · La terminal y su diseño (plan, 2026-11-25)

> Estado: **solo plan**. Las decisiones abiertas (tabla «Decisiones abiertas») esperan confirmación.
> Sustituye al resumen «Fase T» de `docs/PLAN.md`. Se programa **después** de cerrar la línea base de Warp.
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

## Decisiones abiertas (cada una se cierra antes de su bloque)

| # | Pregunta | Recomendación | Antes de |
|---|---|---|---|
| A1 | ¿Intérprete de secuencias propio o crate `vte`? | `vte`, detrás de un puerto propio. Es una dependencia nueva: necesita tu aprobación, y se revisa su `unsafe` (D11) | T1 |
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
docs/PLAN.md (línea base) → T1 → T2 → T3 → [hito: uso diario] → T4 → T5 → T6 → T7
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

## Estado

- 2026-11-25: plan escrito a partir de `design/screen.png`. Nada programado. Nada de A0 ni de las sondas de este plan verificado.
