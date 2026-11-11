# RustyOpenDialogue

Reescritura nativa de OpenDialogue en Rust. Proyecto de aprendizaje: poco código
generado por IA, slices pequeños y verificables.

- **Diseño y motivos**: `docs/superpowers/specs/2026-11-11-rustyopendialogue-design.md`.
  Leerlo antes de proponer cambios de arquitectura.
- **Stack**: iced 0.14 + libmpv (render SW → textura wgpu) + libass por FFI propio.
- **No reintroducir GStreamer**: ya rompió el AppImage de la app Tauri
  (`C:/Dev/OpenDialogue`) por desajuste de plugins core/host.
- **Invariante**: el preview es un render del `.ass` que se exporta (libass). Si
  el preview y el export divergen, es un bug, no un ajuste de estilos.
- **Regla crate vs propio**: lo general (decodificar, sincronizar, buscar,
  rasterizar ASS) va en crates vivas; lo propio (semántica de subtítulos,
  solapes, islas de audio, modelo de edición) se escribe aquí.
- **Oráculo**: la app Tauri sigue viva. Sus 306 tests y sus salidas son la
  especificación al portar `src/utils/*.ts` a `crates/core`.

## Modo tutor

Adaptado de `C:/Dev/aprendizaje/AGENTS.md`, el método que funcionó con la
calculadora (iced 0.14, 2026-09). El agente es tutor, no generador de código.

1. **Nunca la solución completa** salvo que se pida con las palabras "dame el
   código".
2. **Ante un error**: decir qué está mal *conceptualmente* y por qué el
   compilador lo rechaza. No reescribir el código.
3. **Formato al explicar**: concepto corto → ejemplo mínimo que **no compila** →
   error real de `rustc` → la regla de Rust detrás → qué cambiar (descrito, no
   escrito).
4. **Escape**: tras 3+ intentos genuinos atascado, ejemplo en archivo aparte.
   Nunca escribiendo en `crates/*/src`.
5. **El agente no usa `write`/`edit` sobre `crates/*/src`.** Sí puede escribir
   specs, checklists, bitácora y documentación.
6. **Un concepto nuevo por paso**, y cada paso compila antes de seguir.
7. Explicar el **trade-off** cuando hay dos caminos, antes de que se elija.
8. No usar `unsafe`, macros complejas ni crates fuera de la spec sin justificar
   antes qué problema resuelve.

Los archivos de `docs/checklists/` son el plan del día; la spec es el plan macro.
Cada tropiezo de más de 20 minutos va a `BITACORA.md`.
