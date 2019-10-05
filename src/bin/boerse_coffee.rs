use coffee::graphics::{
    self, Color, Frame, HorizontalAlignment, VerticalAlignment, Window, WindowSettings,
};
use coffee::load::Task;
use coffee::ui::{Align, button, Button, Column, Row, Element, Image, Justify, Renderer, Text, UserInterface};
use coffee::{Game, Result, Timer};

pub fn main() -> Result<()> {
    <BoerseApplication as UserInterface>::run(WindowSettings {
        title: String::from("Boerse"),
        size: (1280, 1024),
        resizable: false,
        fullscreen: false,
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ButtonClicked,
}

struct BoerseApplication {
    opt_image: Option<graphics::Image>,
    load_button: button::State,
}

impl Game for BoerseApplication {
    type Input = ();
    type LoadingScreen = ();

    fn load(_window: &Window) -> Task<BoerseApplication> {
        //graphics::Image::load("resources/ui.png").map(|image| BoerseApplication { image })
        Task::succeed(|| BoerseApplication { opt_image: None,
        load_button: button::State::new(),})
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        });
    }
}

impl UserInterface for BoerseApplication {
    type Message = Message;
    type Renderer = Renderer;

    fn react(&mut self, event: Message, window: &mut Window) {
        match event {
            Message::ButtonClicked => {
                let gpu = window.gpu();
                let image = graphics::Image::new(gpu, "resources/ui.png").unwrap();
                self.opt_image = Some(image); 
            }
        }
    }

    fn layout(&mut self, window: &Window) -> Element<Message> {
        let BoerseApplication {
            opt_image,
            load_button,
        } = self;

        let mut controls = Row::new();
        controls = controls.push(
            Button::new(load_button, "Load")
                .on_press(Message::ButtonClicked));
        if let Some(image) = &opt_image {
            controls = controls.push(Image::new(&image).height(250));
        }
        controls = controls.justify_content(Justify::Center);
        Column::new()
            .width(window.width() as u32)
            .height(window.height() as u32)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .spacing(20)
            .push(
                Text::new("This is an image")
                    .size(50)
                    .height(60)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            )
            .push(controls)
            .into()
    }
}
