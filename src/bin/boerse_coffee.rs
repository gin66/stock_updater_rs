use std::fs::File;

use chrono::NaiveDate;
use coffee::graphics::{
    self, Frame, HorizontalAlignment, VerticalAlignment, Window, WindowSettings,
};
use coffee::load::loading_screen::ProgressBar;
use coffee::load::Join;
use coffee::load::Task;
use coffee::ui::{
    button, Align, Button, Column, Element, Image, Justify, Radio, Renderer, Row, Text,
    UserInterface,
};
use coffee::{Game, Result, Timer};
use plotters::prelude::*;
use plotters::style::Color;

use updater::ohlc::OHLC;

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
    IsinSelected(usize),
}

struct BoerseApplication {
    opt_image: Option<graphics::Image>,
    load_button: button::State,

    data: Vec<(String, Vec<(NaiveDate, OHLC)>)>,
    selected_isin_id: Option<usize>,
}

impl Game for BoerseApplication {
    type Input = ();
    type LoadingScreen = ProgressBar;

    fn load(_window: &Window) -> Task<BoerseApplication> {
        let loader = Task::new(|| {
            let home_path = dirs::home_dir().unwrap();

            let mut path = home_path.clone();
            path.push("data");
            path.push("stock");
            let mut data = vec![];
            for entry in path.read_dir().expect("read_dir call failed") {
                if let Ok(entry) = entry {
                    let isin = entry.file_name().into_string().unwrap();
                    if entry.metadata().unwrap().is_dir() && isin.len() == 12 {
                        println!("{:?}", isin);
                        let mut p = entry.path();
                        p.push("ohlc.csv");
                        let f = File::open(p).unwrap();
                        let ohlc_data = OHLC::load_file(f).unwrap();
                        data.push((isin, ohlc_data));
                    }
                }
            }

            Ok(data)
        });
        let t2 = Task::succeed(|| {});

        let loader = Task::stage("Load data...", loader);
        let t2 = Task::stage("dummy", t2);

        (loader, t2).join().map(|(data, r2)| BoerseApplication {
            opt_image: None,
            load_button: button::State::new(),
            data,
            selected_isin_id: None,
        })
        //graphics::Image::load("resources/ui.png").map(|image| BoerseApplication { image })
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(graphics::Color {
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
                let w = 500;
                let h = 500;
                let mut buffer = vec![0; (3 * w * h) as usize];
                {
                    let root = plotters::drawing::BitMapBackend::with_buffer(&mut buffer, (w, h))
                        .into_drawing_area();
                    root.fill(&WHITE).unwrap();
                    let mut chart = ChartBuilder::on(&root)
                        .x_label_area_size(35)
                        .y_label_area_size(40)
                        .margin(5)
                        .caption("Histogram Test", ("Arial", 50.0).into_font())
                        .build_ranged(0u32..10u32, 0u32..10u32)
                        .unwrap();

                    chart
                        .configure_mesh()
                        .disable_x_mesh()
                        .line_style_1(&WHITE.mix(0.3))
                        .x_label_offset(30)
                        .y_desc("Count")
                        .x_desc("Bucket")
                        .axis_desc_style(("Arial", 15).into_font())
                        .draw()
                        .unwrap();

                    let data = [
                        0u32, 1, 1, 1, 4, 2, 5, 7, 8, 6, 4, 2, 1, 8, 3, 3, 3, 4, 4, 3, 3, 3,
                    ];

                    chart
                        .draw_series(
                            Histogram::vertical(&chart)
                                .style(plotters::style::colors::RED.mix(0.5).filled())
                                .data(data.iter().map(|x: &u32| (*x, 1))),
                        )
                        .unwrap();
                }
                let im_buffer =
                    image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(w, h, buffer).unwrap();
                let dyn_image = image::DynamicImage::ImageRgb8(im_buffer);
                let gpu = window.gpu();
                let image = graphics::Image::from_image(gpu, dyn_image).unwrap();
                self.opt_image = Some(image);
            }
            Message::IsinSelected(i) => {
                self.selected_isin_id = Some(i);
            }
        }
    }

    fn layout(&mut self, window: &Window) -> Element<Message> {
        let BoerseApplication {
            opt_image,
            load_button,
            data,
            selected_isin_id,
        } = self;

        let mut isins = Column::new();
        for (i, (isin, _)) in data.iter().enumerate() {
            isins = isins.push(Radio::new(i, isin, *selected_isin_id, |x| {
                Message::IsinSelected(x)
            }));
        }

        let mut controls = Row::new();
        controls = controls.push(Button::new(load_button, "Load").on_press(Message::ButtonClicked));
        if let Some(image) = &opt_image {
            controls = controls.push(Image::new(&image).height(250));
        }
        controls = controls.justify_content(Justify::Center);
        Row::new()
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
            .push(isins)
            .into()
    }
}
