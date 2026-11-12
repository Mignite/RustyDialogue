# Slice 0a — libmpv: un frame en memoria, y cuánto cuesta

Riesgo que cubre: que el enlace con la DLL de libmpv no funcione, o que el
render por software no dé tiempo real. Si esto falla, el plan cambia (spec §7).

Criterio de salida: un binario que abre un video, extrae un frame RGB a un
`Vec<u8>`, lo escribe a PNG, e imprime el costo medio por frame a 1080p.

Conceptos nuevos (uno por paso, no todos juntos): enlace con biblioteca C,
`unsafe`, punteros opacos, contexto de render, `memcpy`/stride.

## Preparación

- [x] **Windows**: descargar el paquete *libmpv-dev* x86_64 de los builds de mpv
      para Windows (SourceForge `mpv-player-windows/files/libmpv`). Hecho con
      `mpv-dev-x86_64-20261108-git-55d3feb45c.7z`: trae `libmpv-2.dll`,
      `libmpv.dll.a` e `include/mpv/*.h`. **No trae `mpv.def`** (el `.dll.a` es
      de MinGW).
- [x] Generar el import library para MSVC: el `.def` se saca de
      `dumpbin /exports` filtrando solo `mpv_*` (la DLL exporta además
      `opus_*`, `sixel_*` y `Java_java_awt_*` de sus dependencias estáticas), y
      luego `lib /DEF:mpv.def /OUT:mpv.lib /MACHINE:X64`. Comandos exactos en
      `third_party/mpv/README.md`.
- [x] Dejar `libmpv-2.dll` y `mpv.lib` en `third_party/mpv/` (ignorados en
      `.gitignore`); `mpv.def`, los headers y el README sí se versionan.
- [ ] **Linux**: `sudo pacman -S mpv` (da `libmpv.so` y los headers).
- [ ] Verificación de la preparación: `mpv --version` responde, y
      `pkg-config --libs mpv` (Linux) encuentra la biblioteca.

## Pasos

- [x] **1. Crate vacío.** `crates/player` con `libmpv2 = "7"` (feature `render`)
      y `libmpv2-sys = "4"` como dependencias.
      *Hecho*: `cargo build -p rod-player` compila. El enlace se probó con
      `crates/player/examples/link_smoke.rs`, que imprime
      `mpv_client_api_version = 0x20006` (igual al header:
      `MPV_MAKE_VERSION(2, 6)`), cargando `libmpv-2.dll` desde el `PATH`.
      `crates/player/build.rs` emite `rustc-link-search` a `third_party/mpv`
      solo en Windows.
      **Hallazgo (corregido)**: `libmpv2` 7.0.1 **sí** expone eventos, en
      `libmpv2::events` (`wait_event`, `Event::FileLoaded`, `Event::EndFile`,
      `observe_property`), reexportado desde `mpv.rs` con `pub use`. Lo que no
      cubre es el render por software: `libmpv2::render` solo tiene los
      parámetros de OpenGL (`RenderParam`), no `MPV_RENDER_PARAM_SW_*`. El
      contexto SW se escribe con `libmpv2_sys` en un módulo `unsafe` propio.
      SetData solo está implementado para `f64`, `i64`, `bool` y `String` —
      no para `&str`.

- [x] **2. Abrir un video sin dibujar nada.** Configurar mpv con `vo=null`,
      `ao=null`, `config=no`; cargar el archivo y esperar el evento de archivo
      cargado; leer la propiedad `duration`.
      *Hecho*: `rod_player::open(path)` en `crates/player/src/lib.rs` crea el
      handle con `Mpv::with_initializer` (las opciones `vo`/`ao` solo se leen al
      inicializar, de ahí el closure), lanza `command("loadfile", ...)` y espera
      en `wait_event(-1.0)` hasta `Event::FileLoaded`, con `Event::EndFile` como
      salida de error.
      *Verificado*: `duration = 3.000 s` para el clip de prueba; `time-pos`
      avanza 0.10 por cada 100 ms dormidos y el reloj de pared dio 3.04 s para
      3.00 s de medio — reproducción en tiempo real, no *untimed*.
      *Control*: `set_property("pause", true)` congela `time-pos`, y
      `command("seek", &["2.0", "absolute"])` lo lleva a 2.00 s. El segundo
      argumento de `seek` es el modo: sin `"absolute"` el salto es relativo. El
      seek no es instantáneo (hay que decodificar hasta el punto), así que
      conviene sondear `time-pos` en vez de leerlo una vez.
      *Nota*: con `vo=null` no se ve nada; la reproducción se comprueba por
      `time-pos`. La ventana con video real es el slice 0c.

- [ ] **3. Crear el contexto de render SW.** Pasar a `vo=libmpv` y crear el
      contexto con `mpv_render_context_create` y
      `MPV_RENDER_API_TYPE_SW`. Todo el `unsafe` vive en un módulo propio
      (`mpv_render.rs`), con una API segura hacia afuera.
      *Verificación*: la creación no devuelve error y `mpv_render_context_update`
      devuelve flags distintos de cero al menos una vez.

- [ ] **4. Renderizar un frame a memoria.** Llamar
      `mpv_render_context_render` con `SW_SIZE`, `SW_FORMAT` (RGB0 o BGRA),
      `SW_STRIDE` y `SW_POINTER` sobre un buffer propio.
      *Verificación*: el buffer no queda todo en cero y el `pts` que reporta mpv
      cae dentro de la duración del video.

- [ ] **5. Mirarlo con los ojos.** Escribir el frame a PNG con la crate `image`
      desde un `examples/dump_frame.rs`.
      *Verificación*: abres el PNG y es el fotograma correcto, sin franjas ni
      colores invertidos (si hay franjas, el `stride` está mal; si los colores
      están cambiados, es el orden de canales — dos errores distintos).

- [ ] **6. Medir.** 100 frames consecutivos de un video 1080p, con
      `std::time::Instant` alrededor de la llamada de render.
      *Verificación*: tabla con media, p95 y fps teórico. **Criterio de muerte:
      media > 33 ms** (no da 30 fps). Anota el número en `BITACORA.md` aunque
      salga bien.

- [ ] **7. Decidir y cerrar.** ¿Se sostiene el plan? Si sí, el PNG y el número
      van al slice 0c. Si no, se discute el plan B de la spec §7 antes de
      escribir más código.

## Qué NO hacer en este slice

- No abras ventana ni toques iced: aquí solo se demuestra que el frame existe y
  cuánto cuesta.
- No agregues audio todavía. Si suena, es que mpv está haciendo de las suyas con
  la configuración.
- No optimices antes de medir. El paso 6 es el que decide, no la intuición.
