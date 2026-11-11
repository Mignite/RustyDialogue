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
