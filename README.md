# Imperifex

The development terminal of the Fexarium family

## Compilar y ejecutar

```
cargo build --release
target\release\imperifex.exe help
```

Antes de cada commit: `powershell -File tools/check.ps1` (formato, clippy sin avisos y tests, guardas incluidas). `tools/ship.ps1 -Message "..."` hace el commit y el push solo si todo está en verde. Para contribuir, lee `CONTRIBUTING.md`.

## Órdenes

| Orden | Hace |
|---|---|
| `imperifex status [--json]` | Ejemplo: dice que la app funciona y cuánto tardó |
| `imperifex help` | La ayuda |

## El banco de medidas

| Orden | Hace |
|---|---|
| `imperifex-bench emit <carga> --out <archivo.json> [--mib N] [--seed S]` | Manda una carga a la terminal en la que corre y mide cuánto tarda en aceptarla |
| `imperifex-bench footprint <terminal> [--seconds N] [--out archivo.json]` | Memoria, CPU y GPU de una terminal ya abierta. Solo lee. Para medir en reposo, no la uses mientras corre |
| `imperifex-bench startup <terminal> [--runs N] [--cold] [--out archivo.json]` | **Abre y cierra ventanas** de la terminal (10 veces) y mide cuánto tarda cada una en verse, en mostrar su primer contenido y en estar lista (3 s sin cambios). Para los dos últimos tiene que abrir entera en el monitor principal. No uses el PC mientras corre |
| `imperifex-bench latency <terminal> [--presses 200] [--mode prompt\|program\|both]` | **Abre una ventana y escribe en ella**: cuánto tarda una tecla en verse, mirando solo su zona, que aprende con 5 pulsaciones previas que no cuentan. Antes de escribir espera a que la ventana esté lista, y para eso tiene que abrir entera en el monitor principal. Probada en el PC real el 2026-11-26, cada modo por separado, todavía sin la zona |
| `imperifex-bench output <terminal> [--load carga\|all] [--repeats 3]` | **Abre ventanas y escribe en ellas**: cuánto tarda en verse una carga, a qué velocidad y con cuánta CPU. Espera igual que `latency`. Probada en el PC real el 2026-11-26 |
| `imperifex-bench memory <terminal>` | **Abre una ventana, escribe y abre pestañas** (tras 90 s sin abrir nada): memoria del historial y por pestaña. Espera igual que `latency`. Probada una vez en el PC real el 2026-11-26 |
| `imperifex-bench echo` | Repite cada tecla hasta la `q`; lo usa `latency` |
| `imperifex-bench all <terminal> [--out-dir carpeta]` | **Toda la línea base** (unos 41 min, sin medir): no toques el PC. Deja un `report.json`; si una parte falla, sigue y lo anota. Sin probar aún |
| `imperifex-bench report <carpeta>... --md docs/BASELINE.md` | Escribe las tablas a partir de los `report.json` de esas carpetas |
| `imperifex-bench help` | La ayuda |

### Órdenes que abren ventanas

Se pueden lanzar desde cualquier carpeta, también desde la del repo. La ventana nueva abre siempre en `$env:TEMP\imperifex-bench\home` (el banco la crea), para que todas las tandas se midan en la misma carpeta, sin Git. El banco solo se niega si esa carpeta queda dentro de un repositorio.

```
.\target\release\imperifex-bench.exe startup warp --out "$env:TEMP\imperifex-bench\startup-warm.json"
```

Mientras corren no se toca el PC. Cada ventana cerrada deja sus procesos cerca de un minuto y el banco los espera (hasta 90 s), así que 10 tandas llevan unos 10 minutos. Nunca fuerza el cierre de nada; si Warp pide confirmar un cierre, se detiene.

### Arranque en frío

Necesita que la terminal no esté abierta, así que no se puede lanzar desde ella:

1. Compila antes (`cargo build --release -p imperifex-bench`) y apunta la ruta completa de `target\release\imperifex-bench.exe`.
2. Guarda lo que tengas a medias y **cierra Warp del todo** (también las sesiones que corran dentro). Comprueba en el Administrador de tareas que no quede `warp.exe`.
3. Abre PowerShell desde el menú Inicio.
4. Corre la orden con la ruta completa y no toques el PC hasta que termine:
   ```
   & "<carpeta del repo>\target\release\imperifex-bench.exe" startup warp --cold --runs 10 --out "$env:TEMP\imperifex-bench\startup-cold.json"
   ```
5. Vuelve a abrir Warp.

La orden se niega si Warp sigue abierto. Si tras cerrar su única ventana Warp no sale, el banco lo espera (hasta 90 s y después 120 s más) y se detiene: no lo fuerza. **El caso en frío aún no se ha probado con Warp.**

## Arquitectura

Ver `ARCHITECTURE.md`. El plan de trabajo está en `docs/PLAN.md`.
