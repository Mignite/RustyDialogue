//! Oráculo del slice 0b: el mismo `.ass` y el mismo instante, pero compuesto por
//! el libass que trae libmpv dentro. Su PNG es la referencia contra la que se
//! compara el render propio, así que aquí no se escribe ninguna lógica de
//! subtítulos: solo se le pide a mpv su versión de la verdad.

const W: usize = 1920;
const H: usize = 1080;
const INSTANTE: f64 = 1.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = rod_player::Player::open("testdata/clip-1080p.mp4")?;

    // hr-seek: buscar por tiempo y no por keyframe.
    player.mpv().set_property("hr-seek", "yes")?;
    // No dejar que mpv reinterpreta los estilos del .ass.
    player.mpv().set_property("sub-ass-override", "no")?;
    player.mpv().set_property("pause", "yes")?;
    player.mpv().command(
        "sub-add",
        &["crates/subs/tests/fixtures/basico.ass", "select"],
    )?;
    player
        .mpv()
        .command("seek", &[&INSTANTE.to_string(), "absolute", "exact"])?;

    // Tirar los flags pendientes antes de buscar: si no, el primero que llegue
    // describe el frame anterior al seek y el render devuelve ese.
    while player.update() & 1 != 0 {}

    let mut buf = vec![0u8; W * H * 4];
    let mut ultimo = -1.0f64;

    for _ in 0..500 {
        if player.update() & 1 != 0 {
            player.render(&mut buf, W, H)?;
            ultimo = player.mpv().get_property::<f64>("time-pos").unwrap_or(-1.0);

            if ultimo >= INSTANTE - 0.01 {
                break;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("frame renderizado en time-pos = {ultimo}");

    if ultimo < INSTANTE - 0.01 {
        return Err("mpv no entregó el frame del instante pedido".into());
    }

    let rgb: Vec<u8> = buf
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|px| [px[0], px[1], px[2]])
        .collect();

    image::RgbImage::from_raw(W as u32, H as u32, rgb)
        .expect("el buffer no mide W*H*3")
        .save("testdata/oracle-subs.png")?;

    println!("guardado testdata/oracle-subs.png");

    Ok(())
}
