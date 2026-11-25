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
| `imperifex-bench help` | La ayuda |

## Arquitectura

Ver `ARCHITECTURE.md`. El plan de trabajo está en `docs/PLAN.md`.
