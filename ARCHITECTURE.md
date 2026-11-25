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

## Reglas

1. Lo que toca hardware, disco, red, reloj o ventana va detrás de un puerto de `domain`, con un nombre por lo que se necesita (`Clock`), no por cómo se hace. El adaptador real vive en `platform`.
2. Receta de una función nueva: reglas en `domain` con tests → puerto → servicio en `app` con un doble → adaptador en `platform` (medido en el PC real) → salida o vista → cableado en el binario.
3. Guardas (`crates/imperifex-domain/tests/architecture.rs`): dependencias permitidas por crate, ninguna fuente de más de 400 líneas (1000 en `tests.rs`), nada de `println!` salvo excepciones con su motivo, y `unsafe` solo en `platform`.
4. Errores: `Result<_, String>` a través de los puertos; lo que el usuario debe saber sale en la salida de la orden; lo demás, a `log_info!`/`log_warn!`/`log_error!`.
5. Portátil: el crate `windows` solo en `platform`; fuera de Windows el binario dice que aún no tiene adaptadores.

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
- [ ] Ver `docs/PLAN.md`.
