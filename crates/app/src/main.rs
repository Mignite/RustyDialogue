use iced::widget::canvas;

#[derive(Default)]
struct Estado {
    // Add your state fields here
}

struct Lienzo;

enum Message {
    // Define your messages here
}

impl canvas::Program<Message> for Lienzo {
    type State = ();
    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = frame.center();
        let forma = canvas::Path::circle(center, 60.0);
        frame.fill(&forma, iced::Color::from_rgb(0.87, 0.67, 0.0));
        vec![frame.into_geometry()]
    }
}
fn update(_state: &mut Estado, _message: Message) {}
fn view(_state: &Estado) -> iced::Element<'_, Message> {
    canvas(Lienzo)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}
fn main() -> iced::Result {
    iced::application(Estado::default, update, view)
        .title("Rusty OpenDialogue")
        .run()
}
