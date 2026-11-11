# Slice 0c — iced: componer video y subtítulos en pantalla

Riesgo que cubre: que iced deje subir texturas propias a un `shader::Primitive` y
que el redibujado continuo funcione con la reproducción. Es la prueba de fuego
del slice 0 (spec §8).

Criterio de salida: una ventana que reproduce el video del slice 0a con los
subtítulos del slice 0b encima, y un contador de frames real.

Conceptos nuevos: `shader::Primitive` (`prepare`/`render`), ciclo de vida de una
textura en wgpu, `Subscription` por frame, (re)diseño de `Message`.

## Pasos

- [ ] **1. Ventana y `Canvas`.** Ventana de iced con un `Canvas` que dibuja 1000
      barras verticales de alturas pseudoaleatorias con semilla fija.
      *Verificación*: se ven las barras, y al redimensionar se reescalan sin
      deformarse (el layout del canvas se está midiendo bien).

- [ ] **2. Una textura estática.** Un `shader::Primitive` que dibuja un quad con
      una imagen PNG cargada con `image`.
      *Verificación*: el PNG aparece con las proporciones correctas. Si sale
      volcado, el problema está en las UV, no en la textura.

- [ ] **3. Textura dinámica.** Reemplazar el PNG por un `Vec<u8>` que cambia cada
      50 ms (un degradado que se desplaza), subido en `prepare` con
      `queue.write_texture`.
      *Verificación*: el degradado se mueve y no parpadea. Si parpadea, se está
      recreando la textura en vez de reescribirla.

- [ ] **4. El frame de mpv.** Sustituir el degradado por `player::poll_frame()`.
      *Verificación*: se ve el video. Compara el instante que muestra la ventana
      con `mpv` en el mismo `time-pos`.

- [ ] **5. Los bitmaps de libass.** Añadir N quads con blending sobre el de video,
      usando el render del slice 0b al `time_pos` actual.
      *Verificación*: los subtítulos coinciden con el PNG de referencia del slice
      0b y con el oráculo de mpv. Es el invariante, ahora en pantalla.

- [ ] **6. Play/pause y seek.** `enum Message` con `PlayPause`, `Seek(f64)`,
      `Tick`; el video se detiene y arranca sin reiniciar el reloj.
      *Verificación*: pausar y reanudar no salta de frame; el seek cae en el
      frame pedido.

- [ ] **7. Medir.** Contador de frames en el título de la ventana durante 30 s de
      reproducción a 1080p.
      *Verificación*: los fps sostenidos. Compáralos con la medición del slice 0a:
      si la diferencia es grande, el costo está en la subida de textura, no en el
      decodificado.

- [ ] **8. Cerrar.** Commit con el número de fps y captura de pantalla. Fin del
      slice 0.

## Qué NO hacer en este slice

- No empieces con la UI real (paneles, lista, timeline). Este slice es un
  laboratorio: si metes la app aquí, un fallo del video se confunde con un fallo
  de la UI.
- No armes `crates/app` como la app final: un `main.rs` de laboratorio es
  suficiente hasta el slice 4.
- No optimices el redibujado antes del paso 7.
