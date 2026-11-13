fn main() -> libmpv2::Result<()> {
    let player = rod_player::Player::open("testdata/clip-1080p.mp4")?;
    let mut flags = 0;
    for _ in 0..100 {
        flags = player.update();
        if flags != 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("flags = {flags}");
    Ok(())
}
