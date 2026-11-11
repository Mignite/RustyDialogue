# Slice 0a — libmpv: un frame en memoria, y cuánto cuesta

Riesgo que cubre: que el enlace con la DLL de libmpv no funcione, o que el
render por software no dé tiempo real. Si esto falla, el plan cambia (spec §7).

Criterio de salida: un binario que abre un video, extrae un frame RGB a un
`Vec<u8>`, lo escribe a PNG, e imprime el costo medio por frame a 1080p.

Conceptos nuevos (uno por paso, no todos juntos): enlace con biblioteca C,
`unsafe`, punteros opacos, contexto de render, `memcpy`/stride.

## Preparación

- [ ] **Windows**: descargar el paquete *libmpv-dev* x86_64 de los builds de mpv
      para Windows (SourceForge `mpv-player-windows/files/libmpv`). Trae
      `libmpv-2.dll`, `mpv.def`, `include/mpv/*.h`.
- [ ] Generar el import library para MSVC:
      `lib /DEF:mpv.def /OUT:mpv.lib /MACHINE:X64`.
- [ ] Dejar `libmpv-2.dll` y `mpv.lib` en `third_party/mpv/` (la DLL va en
      `.gitignore`: pesa decenas de MB).
- [ ] **Linux**: `sudo pacman -S mpv` (da `libmpv.so` y los headers).
- [ ] Verificación de la preparación: `mpv --version` responde, y
      `pkg-config --libs mpv` (Linux) encuentra la biblioteca.

## Pasos

- [ ] **1. Crate vacío.** `crates/player` con `libmpv2 = "7"` (feature `render`)
      y `libmpv2-sys = "4"` como dependencias.
      *Verificación*: `cargo build -p player` termina. Si falla al enlazar,
      el mensaje de `link.exe` o `ld` dice exactamente qué falta — es la primera
      lección del slice.

- [ ] **2. Abrir un video sin dibujar nada.** Configurar mpv con `vo=null`,
      `ao=null`, `config=no`; cargar el archivo y esperar el evento de archivo
      cargado; leer la propiedad `duration`.
      *Verificación*: imprime la duración real del video de prueba (compárala con
      la de `mpv`). Si no coincide, el problema está en la configuración, no en
      el render.

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
