fn main() -> libmpv2::Result<()> {
    let player = rod_player::open("testdata/clip-1080p.mp4")?;
    println!(
        "duration = {:.3} s",
        player.get_property::<f64>("duration")?
    );
    // pausa y comprobar que el reloj se congela
    player.set_property("pause", true)?;
    println!("pausa en {:.2} s", player.get_property::<f64>("time-pos")?);
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!(
        "500 ms después: {:.2} s",
        player.get_property::<f64>("time-pos")?
    );

    // salto absoluto a 2 s
    player.command("seek", &["2.0", "absolute"])?;

    // esperar a que se aplique: el seek no es instantáneo
    for _ in 0..50 {
        let t = player.get_property::<f64>("time-pos")?;
        if (1.9..2.2).contains(&t) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!(
        "después del seek: {:.2} s",
        player.get_property::<f64>("time-pos")?
    );

    player.set_property("pause", false)?;
    Ok(())
}
