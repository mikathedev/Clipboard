use std::path::Path;
use glib::home_dir;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label};
use gtk4_layer_shell::{Layer, LayerShell, Edge, KeyboardMode};

fn main() {
    let application = Application::builder()
        .application_id("com.example.layershell")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("GTK4 Layer Shell Example")
            .build();

        // 1. Initialize the window for layer shell use
        window.init_layer_shell();

        // 2. Set the layer (Background, Bottom, Top, or Overlay)
        window.set_layer(Layer::Overlay);
        window.gets_focus_visible();
        window.set_keyboard_mode(KeyboardMode::OnDemand);

        window.connect_is_active_notify(|win|{
            println!("active window");
            if !win.is_active() { win.close(); }
        });



        // 3. Anchor to specific edges (e.g., Top, Left, and Right for a top bar)
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);

        window.set_default_size(350, 600);

        //mouse
        let pos = check_mouse();

        window.set_margin(Edge::Left, pos[0]);
        window.set_margin(Edge::Top, pos[1] - 36);


        let label = Label::new(Some("Hello from the Layer Shell!"));
        window.set_child(Some(&label));
        window.add_css_class("window-class");
        window.present();
    });


    application.run();
}

fn check_mouse() -> Vec<i32> {
    let pos = std::process::Command::new("hyprctl")
        .arg("cursorpos")
        .output();

    match pos {
        Ok(res) => {
            String::from_utf8_lossy(&res.stdout).trim().split(",").map(|x| x.trim().parse::<i32>().unwrap()).collect() }
        _ => vec![0,0],
    }
}

fn load_css() {
    let css = if std::fs::exists(format!("{}.config/clip.css", home_dir().display())).ok().unwrap() {
        std::fs::read_to_string(format!("{}.config/clip.css", home_dir().display())).unwrap()
    } else {
        std::fs::create_dir_all(format!("{}.config/clip.css", home_dir().display())).unwrap();
        std::fs::write(format!("{}.config/clip.css", home_dir().display()), "").expect("failed to write");
        std::fs::read_to_string(format!("{}.config/clip.css", home_dir().display())).unwrap()
    };
}