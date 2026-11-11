# Bitácora

Una entrada por tropiezo que costó más de 20 minutos. El valor está en el
diagnóstico, no en la solución: el mismo malentendido suele repetirse con otra
cara.

## Formato

```md
## AAAA-MM-DD — título corto

- **Intentaba:**
- **Error literal:**
  ```
  (pegar el mensaje completo, no un resumen)
  ```
- **Qué estaba mal** (concepto, no síntoma):
- **Qué aprendí:**
- **Dónde quedó** (archivo:línea, commit):
```

---

## 2026-11-11 — libmpv2 no expone el render por software

- **Intentaba:** usar `libmpv2` como fachada única para video, incluyendo el
  render a memoria.
- **Error:** ninguno de compilación; el problema es de API. El enum
  `libmpv2::render::RenderParam` solo tiene variantes para OpenGL
  (`ApiType`, `InitParams`, `FBO`, …). No existen `SW_SIZE`, `SW_FORMAT`,
  `SW_STRIDE` ni `SW_POINTER`.
- **Qué estaba mal:** asumí que la crate cubría la API de render completa porque
  el módulo se llama `render`. La cubre a medias: OpenGL sí, software no.
- **Qué aprendí:** `libmpv2-sys` 4.0.1 sí trae los parámetros `SW_*` (bindings
  pregenerados, sin libclang) y enlaza con `cargo:rustc-link-lib=mpv`. El
  contexto de render por software se escribe a mano sobre el crate `-sys`, con
  una API segura encima. De paso: `libmpv-sys` en crates.io es de otro autor y
  está muerto (2020); el bueno es `libmpv2-sys`.
- **Dónde quedó:** spec §3.2 y §4.3; checklist `slice-0a-mpv.md`, paso 3.
