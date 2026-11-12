# Slice 0z — bootstrap: workspace de cuatro crates y una ventana

Riesgo que cubre: que la estructura del repo se convierta en un problema
después (un crate mal nombrado, dependencias cruzadas, un binario que no es el
que crees). Es barato ahora y caro en el slice 5.

Criterio de salida: `cargo run -p rod-app` abre una ventana de iced con una
figura dibujada en `Canvas`; `cargo fmt --check`, `cargo clippy --all-targets
-- -D warnings` y `cargo test` en verde. Nada de libmpv ni libass: eso es 0a y
0b.

Decisiones ya tomadas (spec §4):

- Manifiesto virtual en la raíz: `Cargo.toml` solo tiene `[workspace]`; el
  binario vive en `crates/app`.
- Paquetes `rod-core`, `rod-subs`, `rod-player`, `rod-app`; directorios
  `crates/core`, `crates/subs`, `crates/player`, `crates/app`.
- El paquete **no** puede llamarse `core`: tapa el `core` del sysroot y rompe
  `core::time::…` y toda macro derivada que expanda `::core::…` (comprobado con
  `thiserror::Error`; ver spec §4).

Conceptos nuevos, uno por paso: paquete vs workspace, herencia de metadatos,
dependencia por ruta, arquitectura Elm de iced, `canvas::Program`.

## Paso 0 — Árbol limpio

- [ ] La raíz hoy arrastra cambios sin commitear: `Cargo.toml` con `iced`
  añadido, `Cargo.lock` sin seguimiento y `src/main.rs` con el `Hello, world!`
  de la plantilla. Commitea ese estado (o descártalo) antes de reestructurar:
  si no, el diff del slice mezcla lo viejo con lo nuevo y no se puede leer.

  *Verificación*: `git status --short` vacío.

## Paso 1 — Manifiesto virtual y cuatro crates vacíos

- [ ] Borra el `[package]` de la raíz y deja solo `[workspace]` con `members`
  (puedes usar el glob `"crates/*"`) y `resolver = "3"`.
  **Concepto**: un *virtual manifest* no es un paquete, es la lista de miembros.
  **Trampa**: `resolver` no se hereda del `edition` de los miembros; si lo
  omites, un workspace virtual se comporta como resolver 1 y las features se
  unifican como en 2015.
- [ ] Crea los cuatro crates con `cargo new --lib crates/<dir> --name rod-<x>`
  (`--name` es lo que desacopla el nombre del directorio). `crates/app` es el
  único sin `--lib`.
- [ ] Borra `src/main.rs` y el directorio `src/` de la raíz.

*Verificación*: `cargo build` compila los cuatro sin warnings; `cargo
metadata --no-deps --format-version 1` lista exactamente `rod-core`, `rod-subs`,
`rod-player`, `rod-app` con `edition: "2024"`.

## Paso 2 — Una sola fuente para versión y edition

- [ ] Pasa `version`, `edition`, `rust-version` y `license` a
  `[workspace.package]`, y cada crate los hereda con `edition.workspace = true`
  (etc.). Deja `iced` en `[workspace.dependencies]` y en `rod-app` pide
  `iced.workspace = true`.

  **Concepto**: los miembros heredan metadatos; el `.workspace = true` es la
  única forma de compartir versión entre crates que se publican juntos.
  **Trade-off**: si un crate necesita una versión distinta de una dependencia,
  la herencia deja de servir y hay que escribirla explícita. Con cuatro crates
  del mismo repo, compensa.
  **Trampa**: `rust-version = "1.88"` es el mínimo que exige iced 0.14; si
  heredas un MSRV mayor, `cargo build` en la máquina vieja falla con un mensaje
  sobre `edition2024`.

*Verificación*: `cargo build` sigue compilando; cambia a propósito la `version`
del workspace y comprueba con `cargo metadata` que los cuatro se movieron
juntos. Deshaz el cambio.

## Paso 3 — `rod-app` es el único binario

- [ ] `rod-app` tiene `src/main.rs` con un `fn main()` que imprime algo. Añade
  `rod-core` como dependencia por ruta (`rod-core = { path = "../core" }`) y
  llama desde `main` a una función pública de `rod-core` (un `&'static str`
  basta; bórralo en el paso siguiente).

  **Concepto**: `rod-core` → `rod_core` en `use` (guion a guion bajo). El
  grafo de dependencias es explícito: `rod-app` conoce a los otros tres;
  `rod-core` no conoce a nadie.
  **Trampa**: no añadas todavía `rod-subs` ni `rod-player` a `rod-app` "por si
  acaso". Una arista que no se usa cuesta compilación y, en un workspace,
  arrastra las features del otro crate a todas tus builds.

*Verificación*: `cargo run -p rod-app` imprime la cadena que viene de
`rod-core`; `cargo tree -p rod-app` muestra `rod-core` y nada más.

## Paso 4 — La ventana

- [ ] Sustituye el `println!` por una aplicación de iced: `fn main() ->
  iced::Result`, `iced::application(boot, update, view).run()`, un `struct`
  de estado y un `enum Message` (aunque no haya mensajes todavía).

  **Concepto**: arquitectura Elm. `update(&mut State, Message)` es la única
  función que muta el estado; `view(&State) -> Element` es pura. El estado es
  tuyo, no de iced.
  **Trampa**: `Message` debe ser `Send + 'static`; y si `update` devuelve `()`
  en vez de `Task`, no puedes encadenar trabajo asíncrono. Empieza con `()`,
  cámbialo cuando lo necesites (slice 3).

*Verificación*: `cargo run -p rod-app` abre una ventana con el título que
pusiste y se cierra sin error (código de salida 0).

## Paso 5 — `Canvas`

- [ ] Activa la feature `canvas` en la dependencia de iced (en
  `[workspace.dependencies]`: `features = ["canvas"]`). Implementa
  `canvas::Program<Message>` para una struct propia y dibuja una figura
  (un círculo o una línea) dentro de `draw`.

  **Concepto**: `draw` recibe `bounds: Rectangle` y devuelve geometría; el
  tamaño del lienzo llega *por argumento*, no se guarda en el estado.
  **Trampa 1**: `canvas` **no** es una feature por defecto de iced; sin ella el
  import falla con `E0432 unresolved import`.
  **Trampa 2**: la firma de `draw` incluye `renderer`, `theme` y `cursor`
  aunque no los uses; si te sobra alguno, el trait no compila. Prefija con `_`
  y listo.

*Verificación*: la ventana muestra la figura; al redimensionar, la figura se
recalcula con el nuevo tamaño (si se deforma o se sale, estás guardando
coordenadas en el estado).

## Paso 6 — El semáforo del slice

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

*Verificación*: los tres terminan en 0. `--all-targets` incluye tests y
ejemplos: clippy que solo mira la lib deja pasar lo que más se rompe. Sin
`--all-features`: activaría las features experimentales de iced (`debug`, `hot`)
y estarías depurando el toolkit, no tu código.

## Paso 7 — CI

- [ ] `.github/workflows/ci.yml`: matriz `windows-latest` / `ubuntu-latest`,
  pasos `fmt --check`, `clippy -D warnings`, `test`.

  **Aviso**: el repo aún no tiene remoto (`git remote -v` está vacío), así que
  el workflow no corre hasta que exista un repositorio en GitHub. El archivo se
  escribe igual; la primera ejecución es la verificación.
  **Nota para 0a/0b**: cuando entren libmpv y libass, el job de Linux necesitará
  `sudo apt install libmpv-dev libass-dev` (o los paquetes equivalentes) y el de
  Windows las DLLs de `third_party/`. Eso se toca en esos slices, no aquí.

*Verificación*: el archivo es YAML válido y los tres comandos que ejecuta son
exactamente los del paso 6.

## Qué NO hacer en este slice

- No enlaces libmpv ni libass: 0a y 0b.
- No añadas `thiserror`, `serde` ni `anyhow` "para dejarlo listo". Cada crate
  estrena dependencia cuando el paso la necesita.
- No pongas `#![deny(warnings)]` en el código: eso es del CI, donde se puede
  actualizar el toolchain sin romper tu build local.
- No diseñes todavía el `VideoSource` de la spec §4.3: en 0z no hay player.
