use druid::platform_menus;
use druid::widget::{Align, Button, Flex, Label, TextBox};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Menu, Widget, WidgetExt, WindowDesc};
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::fs;

const WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 300.0;
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
    name: String,
    tx: Rc<Sender<String>>,
    rx: Rc<Receiver<String>>,
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .menu(|_op, _t, _env| Menu::empty().entry(platform_menus::mac::application::default()))
        .window_size((600.0, 200.0));

    let (tx_gui, rx_gui) = mpsc::channel();
    let (tx_worker, rx_worker) = mpsc::channel();

    // create the initial app state
    let initial_state = MultiStream {
        url: "".into(),
        name: "".into(),
        status: "Enter a URL above!".into(),
        tx: Rc::new(tx_gui),
        rx: Rc::new(rx_worker),
    };

    thread::spawn(move || {
        for msg in rx_gui.into_iter() {
            //-j / -json for outputing json shaped text (maybe useful?)
            // --url can be used to specify the url
            // --logfile useful for activity, can be set to default file via '-'

            let msg_parts = msg.split(",").collect::<Vec<&str>>();
            Command::new("streamlink")
                // .current_dir(home)
                .arg("-r")
                .arg(format!("{}.ts", msg_parts[1]))
                .arg(msg_parts[0])
                .arg("best")
                .spawn()
                .expect("failed to execute command");
        }
    });

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<MultiStream> {
    // a urlbox that modifies `name`.
    let urlbox = TextBox::new()
        .with_placeholder("URL?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(MultiStream::url);

    let filebox = TextBox::new()
        .with_placeholder("name?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(MultiStream::name);

    let button = Button::new("Pull Stream").on_click(|_ctx, data: &mut MultiStream, _env| {
        data.status = format!("Pulling '{}'", data.url);
        let msg = data.url.to_owned() + "," + &data.name.to_owned();
        data.tx.send(msg).unwrap();
    });
    let label = Label::new(|data: &MultiStream, _env: &Env| format!("{}", data.status));

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(
            Flex::row()
                .with_child(Align::centered(
                    Flex::column()
                        .with_child(urlbox)
                        .with_spacer(WIDGET_SPACING)
                        .with_child(filebox),
                ))
                .with_spacer(WIDGET_SPACING)
                .with_child(button),
        )
        .with_spacer(WIDGET_SPACING)
        .with_child(label);

    // center the two widgets in the available space
    Align::centered(layout)
}
