use druid::platform_menus;
use druid::widget::{Align, Button, Flex, Label, TextBox};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Menu, Widget, WidgetExt, WindowDesc};
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

const WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<MultiStream> = LocalizedString::new("Multi-Streamer!");

//Architecture:
//GUI thread, work thread, stream thread.
//Each time the GUI thread gets a stream that it needs it drops it into a queue.
//The work thread waits and watches the queue and spawns stream threads when it gets a job
//The stream threads open streamlink, connect it to a file, then stream the file to somewhere

#[derive(Clone, Data, Lens)]
struct MultiStream {
    status: String,
    url: String,
    tx: Rc<Sender<String>>,
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .menu(|_op, _t, _env| Menu::empty().entry(platform_menus::mac::application::default()))
        .window_size((600.0, 100.0));

    let (tx, rx) = mpsc::channel();

    // create the initial app state
    let initial_state = MultiStream {
        url: "".into(),
        status: "Enter a URL above!".into(),
        tx: Rc::new(tx),
    };

    thread::spawn(move || {
        for msg in rx.into_iter() {
            println!("Second thread: {}", msg);
        }
    });

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<MultiStream> {
    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(MultiStream::url);

    let button = Button::new("Pull Stream").on_click(|_ctx, data: &mut MultiStream, _env| {
        data.status = format!("Pulling '{}'", data.url);
        data.tx.send(data.url.to_owned()).unwrap()
    });
    //https://www.youtube.com/watch?v=qY8_1t8LDWY
    let label = Label::new(|data: &MultiStream, _env: &Env| format!("{}", data.status));

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(Align::centered(
            Flex::row()
                .with_child(textbox)
                .with_spacer(WIDGET_SPACING)
                .with_child(button),
        ))
        .with_spacer(WIDGET_SPACING)
        .with_child(label);

    // center the two widgets in the available space
    Align::centered(layout)
}
