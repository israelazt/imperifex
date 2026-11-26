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
| `imperifex-bench startup <terminal> [--runs N] [--cold] [--out archivo.json]` | **Abre y cierra ventanas** de la terminal (10 veces) y mide cuánto tarda en verse cada una. No uses el PC mientras corre |
| `imperifex-bench latency <terminal> [--presses 200] [--mode prompt\|program\|both]` | **Abre una ventana y escribe en ella**: cuánto tarda una tecla en verse. Sin probar aún en el PC real |
| `imperifex-bench output <terminal> [--load carga\|all] [--repeats 3]` | **Abre ventanas y escribe en ellas**: cuánto tarda en verse una carga, a qué velocidad y con cuánta CPU. Sin probar aún |
| `imperifex-bench memory <terminal>` | **Abre una ventana, escribe y abre pestañas**: memoria del historial y por pestaña. Sin probar aún |
| `imperifex-bench echo` | Repite cada tecla hasta la `q`; lo usa `latency` |
| `imperifex-bench help` | La ayuda |

### Órdenes que abren ventanas

Se lanzan desde una carpeta sin Git, porque la ventana nueva de Warp copia la carpeta y el Git de la pestaña activa y las tandas no se podrían comparar. El banco se niega dentro de un repo y dice a qué carpeta ir (`$env:TEMP\imperifex-bench\home`, que él crea).

```
cd "$env:TEMP\imperifex-bench\home"
& "<carpeta del repo>\target\release\imperifex-bench.exe" startup warp --out "..\startup-warm.json"
```

Mientras corren no se toca el PC. Cada ventana cerrada deja sus procesos cerca de un minuto y el banco los espera (hasta 90 s), así que 10 tandas llevan unos 10 minutos. Nunca fuerza el cierre de nada; si Warp pide confirmar un cierre, se detiene.

### Arranque en frío

Necesita que la terminal no esté abierta, así que no se puede lanzar desde ella:

1. Compila antes (`cargo build --release -p imperifex-bench`) y apunta la ruta completa de `target\release\imperifex-bench.exe`.
2. Guarda lo que tengas a medias y **cierra Warp del todo** (también las sesiones que corran dentro). Comprueba en el Administrador de tareas que no quede `warp.exe`.
3. Abre PowerShell desde el menú Inicio.
4. Ve a la carpeta de las medidas, corre la orden con la ruta completa y no toques el PC hasta que termine:
   ```
   cd "$env:TEMP\imperifex-bench\home"
   & "<carpeta del repo>\target\release\imperifex-bench.exe" startup warp --cold --runs 10 --out "..\startup-cold.json"
   ```
5. Vuelve a abrir Warp.

La orden se niega si Warp sigue abierto. Si tras cerrar su única ventana Warp no sale, el banco lo espera (hasta 90 s y después 120 s más) y se detiene: no lo fuerza. **El caso en frío aún no se ha probado con Warp.** La carpeta `home` existe después de haber lanzado antes cualquier orden que abra ventanas; si no, créala.

## Arquitectura

Ver `ARCHITECTURE.md`. El plan de trabajo está en `docs/PLAN.md`.
