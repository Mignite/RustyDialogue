use libmpv2::{Mpv, events::Event};
pub fn open(path: &str) -> libmpv2::Result<Mpv> {
    let mpv = Mpv::with_initializer(|init| {
        init.set_option("vo", "null".to_string())?;
        init.set_option("ao", "null".to_string())?;
        init.set_option("config", "no".to_string())?;
        Ok(())
    })?;
    mpv.command("loadfile", &[path, "replace"])?;
    loop {
        match mpv.wait_event(-1.0) {
            Some(Ok(Event::FileLoaded)) => break,
            Some(Ok(Event::EndFile(reason))) => {
                eprintln!("Se acabo antes de cargar: {reason:?}");
                break;
            }
            Some(Ok(otro)) => println!("Evento inesperado: {otro:?}"),
            Some(Err(e)) => {
                eprintln!("Error al esperar evento: {e:?}");
                break;
            }
            None => {
                eprintln!("Se acabo el tiempo de espera antes de cargar");
                break;
            }
        }
    }
    Ok(mpv)
}
