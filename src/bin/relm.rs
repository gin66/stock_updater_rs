use std::f64::consts::PI;
use std::fs::File;
//use std::path::PathBuf;

use gdk::{EventMask, RGBA};
use gtk::{
    //BoxExt,
    DrawingArea,
    Inhibit,
    //OrientableExt,
    ContainerExt,
    LabelExt,
    ListBoxExt,
    //ListBoxRowExt,
    WidgetExt,
    WidgetExtManual,
    Window,
    WindowType,
    GtkWindowExt,
};
use gtk::Orientation::Horizontal;
use rand::Rng;
use relm_derive::Msg;
use relm::{
    DrawHandler,
    Relm,
    Widget,
    interval,
};
use relm::*;
//use relm_derive::widget;
use chrono::offset::{Local, TimeZone};
use chrono::Date;
use chrono::{Datelike, NaiveDate};
use ndarray::s;
use ndarray::Array2;
use plotters::prelude::*;

use updater::ohlc::OHLC;

use self::Msg::*;

const SIZE: f64 = 15.0;

struct Circle {
    x: f64,
    y: f64,
    color: RGBA,
    vx: f64,
    vy: f64,
}

impl Circle {
    fn generate() -> Self {
        let mut gen = rand::thread_rng();
        Circle {
            x: gen.gen_range(20.0, 100.0),
            y: gen.gen_range(20.0, 100.0),
            color: RGBA {
                red: gen.gen_range(0.0, 1.0),
                green: gen.gen_range(0.0, 1.0),
                blue: gen.gen_range(0.0, 1.0),
                alpha: 1.0,
            },
            vx: gen.gen_range(1.0, 5.0),
            vy: gen.gen_range(1.0, 5.0),
        }
    }
}

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    circles: Vec<Circle>,
    cursor_pos: (f64, f64),
}

struct Win {
    model: Model,
    window: Window,
    drawing_area: gtk::DrawingArea,
    //isin_list: gtk::ListBox,
}


#[derive(Msg)]
pub enum Msg {
    Generate,
    Move,
    MoveCursor((f64, f64)),
    Quit,
    SelectIsin(String),
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn update(&mut self, event: Msg) {
        match event {
            Generate => self.model.circles.push(Circle::generate()),
            Move => {
                let allocation = self.drawing_area.get_allocation();
                for circle in &mut self.model.circles {
                    if (circle.x + circle.vx + SIZE / 2.0 < allocation.width as f64) &&
                        (circle.x + circle.vx - SIZE / 2.0 > 0.0)
                    {
                        circle.x += circle.vx;
                    }
                    else {
                        circle.vx *= -1.0;
                    }
                    if (circle.y + circle.vy + SIZE / 2.0 < allocation.height as f64) &&
                        (circle.y + circle.vy - SIZE / 2.0 > 0.0)
                    {
                        circle.y += circle.vy;
                    }
                    else {
                        circle.vy *= -1.0;
                    }
                }
            },
            MoveCursor(pos) => self.model.cursor_pos = pos,
            Quit => gtk::main_quit(),
            SelectIsin(isin) => {
                println!("{}",isin);
    let home_path = dirs::home_dir().unwrap();
    let mut fname = home_path.clone();
    fname.push("data");
    fname.push("stock");
    fname.push(isin);
    fname.push("ohlc.csv");
    let f = File::open(fname).unwrap();
    let part = OHLC::load_file(f).unwrap();
    let part = part
        .iter()
        .map(|(d, e)| (chrono::Local.from_utc_date(&d) as Date<Local>, e))
        .collect::<Vec<_>>();

                let context = self.model.draw_handler.get_context();
                let root = CairoBackend::new(&context, (1024, 768)).unwrap().into_drawing_area(); // 1000*1ms
                root.fill(&WHITE).unwrap();
                let from_date = part.first().unwrap().0;
                let to_date = part.last().unwrap().0;
                let from_y = part.iter().map(|e| e.1.low).fold(1. / 0., f32::min);
                let to_y = part.iter().map(|e| e.1.high).fold(0. / 0., f32::max);
                println!("{}", from_date);
                let mut chart = ChartBuilder::on(&root)
                    .x_label_area_size(60)
                    .y_label_area_size(60)
                    .caption("DAX", ("Arial", 50.0).into_font())
                    .build_ranged(from_date..to_date, from_y..to_y)
                    .unwrap();

                chart
                    .configure_mesh()
                    .line_style_2(&WHITE)
                    .x_label_formatter(&|d| d.format("%Y-%m-%d").to_string())
                    .draw()
                    .unwrap();

                chart
                    .draw_series(part.into_iter().map(|(d, x)| {
                        CandleStick::new(
                            d,
                            x.open,
                            x.high,
                            x.low,
                            x.close,
                            &GREEN,
                            &RED,
                            15,
                        )
                    }))
                    .unwrap();
                root.present().unwrap();
            }
        }
    }

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            circles: vec![Circle::generate()],
            cursor_pos: (-1000.0, -1000.0),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 1000, || Generate);
        interval(relm.stream(), 40, || Move);
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.drawing_area);
        self.drawing_area.add_events(EventMask::POINTER_MOTION_MASK);
    }

    //        gtk::Box {
    //            orientation: Horizontal,
    //            #[name="drawing_area"]
    //            gtk::DrawingArea {
    //                child: {
    //                    expand: true,
    //                },
    //                draw(_, _) => (UpdateDrawBuffer, Inhibit(false)),
    //                motion_notify_event(_, event) => (MoveCursor(event.get_position()), Inhibit(false))
    //            },
    //            gtk::ListBox {
    //            }

    // Create the widgets.
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        // GTK+ widgets are used normally within a `Widget`.
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(320, 200);
        window.set_title("Basic example");
        let vbox = gtk::Box::new(Horizontal, 0);

        let drawing_area= gtk::DrawingAreaBuilder::new()
                    .width_request(400)
                    .height_request(400)
                    .build();
        vbox.add(&drawing_area);

        let isin_list = gtk::ListBoxBuilder::new()
                    .width_request(100)
                    .height_request(100)
                    .build();

        let home_path = dirs::home_dir().unwrap();
        let mut path = home_path.clone();
        path.push("data");
        path.push("stock");
        //let mut data = vec![];
        for entry in path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let isin = entry.file_name().into_string().unwrap();
                if entry.metadata().unwrap().is_dir() && isin.len() == 12 {
                    //let isin_label = gtk::LabelBuilder::new()
                    //            .label(&isin)
                    //            .build();
                    let isin_label = gtk::Label::new(None);
                    isin_label.set_markup(&format!("<small>{}</small>",isin));
                    let isin_label = isin_label.upcast::<gtk::Widget>();
                    let isin_entry = gtk::ListBoxRowBuilder::new()
                                .name(&isin)
                                .child(&isin_label)
                                .build();
                    isin_list.add(&isin_entry);
                }
            }
        }
        let stream = relm.stream().clone();
        isin_list.connect_row_activated(move |_lb,entry| {
            let isin = entry.get_name().unwrap().to_string();
            println!("{}",isin);
            stream.emit(Msg::SelectIsin(isin));
        });

        vbox.add(&isin_list);

        window.add(&vbox);

        // Connect the signal `delete_event` to send the `Quit` message.
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));
        // There is also a `connect!()` macro for GTK+ events that do not need a
        // value to be returned in the callback.

        window.show_all();

        Win {
            model,
            window: window,
            drawing_area,
            //isin_list,
        }
    }

}

fn main() {
    Win::run(()).unwrap();
}
