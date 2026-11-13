use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::widget::{column, shader, text};
use iced::{Element, Length};

use rod_app::video;

const W: usize = 1920;
const H: usize = 1080;

struct Lab {
    player: rod_player::Player,
    buf: Vec<u8>,
    frames: u64,
    desde: Instant,
    fps: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

fn main() -> iced::Result {
    iced::application(Lab::nuevo, update, view)
        .title("lab: video")
        .window_size([1280.0, 720.0])
        .subscription(|_| iced::window::frames().map(|_| Message::Tick))
        .run()
}

impl Lab {
    fn nuevo() -> Self {
        let player = rod_player::Player::open("testdata/clip-1080p60.mp4")
            .expect("no se pudo abrir el video");
        player
            .mpv()
            .set_property("loop-file", "inf")
            .expect("loop-file");
        Self {
            player,
            buf: vec![0u8; W * H * 4],
            frames: 0,
            desde: Instant::now(),
            fps: 0.0,
        }
    }
}

fn update(lab: &mut Lab, _mensaje: Message) {
    if lab.player.update() & 1 == 0 {
        return;
    }
    if lab.player.render(&mut lab.buf, W, H).is_err() {
        return;
    }

    lab.frames += 1;

    let transcurrido = lab.desde.elapsed();
    if transcurrido >= Duration::from_secs(1) {
        lab.fps = lab.frames as f32 / transcurrido.as_secs_f32();
        lab.frames = 0;
        lab.desde = Instant::now();
    }
}

fn view(lab: &Lab) -> Element<'_, Message> {
    column![
        text(format!("{:.1} fps", lab.fps)),
        shader(video::Video {
            pixels: Arc::new(lab.buf.clone()),
            width: W as u32,
            height: H as u32,
        })
        .width(Length::Fill)
        .height(Length::Fill),
    ]
    .into()
}
