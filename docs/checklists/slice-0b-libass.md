# Slice 0b — libass: bitmaps de subtítulo y fidelidad con mpv

Riesgo que cubre: construir/enlazar libass en Windows, que resuelva fuentes, y
que el render coincida con mpv. Es el slice donde nace el invariante del
proyecto (spec §3.3).

Criterio de salida: un `.ass` de fixture rasterizado a PNG desde Rust, y una
comparación numérica contra el mismo frame renderizado por mpv.

Conceptos nuevos: FFI declarado a mano, `#[repr(C)]`, punteros opacos, strings C,
libertad de memoria (`ass_free_track`), composición alfa.

## Preparación

- [ ] **Windows**: `vcpkg install libass:x64-windows`. (El port existe: libass
      0.17.5, con freetype, fribidi y harfbuzz como dependencias.)
- [ ] **Linux**: `sudo pacman -S libass`, y comprobar con
      `pkg-config --modversion libass`.
- [x] **El oráculo sin `mpv.exe`.** No hace falta instalar mpv: la DLL de libmpv
      que ya enlazamos trae su propio libass y compone los subtítulos en el
      frame. `crates/player/examples/oracle_subs.rs` carga el fixture con
      `sub-add`, busca el instante con `hr-seek` y guarda
      `testdata/oracle-subs.png`.
      *Ojo con el orden*: hay que **drenar** los flags de
      `mpv_render_context_update` antes del `seek`. El flag pendiente de la
      carga describe el frame anterior, y si se consume primero el render
      devuelve el frame 0 con el `time-pos` ya en 1.0. Cuesta 15 minutos y el
      player real va a repetirlo.
- [ ] **Verificar el proveedor de fuentes**, que es el punto delicado en Windows:
      inicializar libass con `ASS_FONTPROVIDER_AUTODETECT` y leer el log. Debe
      decir que usa *directwrite* en Windows y *fontconfig* en Linux.
      *Verificación*: aparece el nombre del proveedor en el log. Si no aparece,
      libass se compiló sin ese proveedor y hay que resolverlo **antes** de
      seguir (es el motivo por el que existen los avisos de "fuente no
      instalada").

## Pasos

- [ ] **1. FFI a mano, sin bindgen.** Declarar solo lo necesario: cuatro tipos
      opacos (`AssLibrary`, `AssRenderer`, `AssTrack`, `AssFontProvider`) y unas
      quince funciones (`ass_library_init`, `ass_renderer_init`,
      `ass_set_frame_size`, `ass_set_storage_size`, `ass_set_pixel_aspect`,
      `ass_set_margins`, `ass_set_use_margins`, `ass_set_fonts`,
      `ass_read_memory`, `ass_free_track`, `ass_render_frame`,
      `ass_renderer_done`, `ass_library_done`, `ass_set_message_cb`).
      *Verificación*: compila y `ass_library_init` no devuelve nulo. Bindgen
      queda como plan B solo si el API crece.

- [ ] **2. Leer un `.ass` desde memoria.** Un fixture de tres líneas con dos
      estilos distintos.
      *Verificación*: `ass_read_memory` devuelve un track y el log de libass no
      reporta error de parseo.

- [ ] **3. Rasterizar un instante.** `ass_render_frame(track, 1000 ms, &changed)`
      y recorrer la lista de `ASS_Image`.
      *Verificación*: imprime cuántas imágenes hay, y para cada una `w`, `h`,
      `stride`, `dst_x`, `dst_y`. Compara esas coordenadas con lo que esperas de
      tu fixture: ahí se ve si el tamaño de frame está bien configurado.

- [ ] **4. Componer en CPU.** Volcar los bitmaps sobre un buffer RGBA con
      `dst = rgb·(a·A/255) + dst·(1 − a·A/255)` y escribirlo a PNG.
      *Verificación*: se lee el texto, con su borde y su color, en la posición
      correcta. Sin wgpu todavía: primero la matemática, después la GPU.

- [ ] **5. El oráculo.** El mismo `.ass`, el mismo instante, renderizado por mpv:
      `mpv --no-config --vo=image --sub-file=fixture.ass --sub-ass-override=no
      --start=1 --frames=1`. Comparar los dos PNG.
      *Verificación*: porcentaje de píxeles distintos. Anotar el número, la
      versión de libass y la fuente usada. Si difiere en más de un 1 %, averiguar
      si es escala, fuente o versión **antes** de dar el paso por bueno.

- [ ] **6. Cerrar.** Escribir en `BITACORA.md` la versión de libass, el proveedor
      de fuentes y el porcentaje de coincidencia. Ese número es la línea base
      del proyecto.

## Qué NO hacer en este slice

- No dibujes en la ventana: el PNG es suficiente para decidir.
- No implementes caché de bitmaps ni `detect_change` todavía. La primera versión
  vuelve a rasterizar cada vez; la optimización se mide en el slice 4.
- No adaptes el `.ass` para que "se vea igual". Si difiere de mpv, la diferencia
  es información, no un estorbo.
