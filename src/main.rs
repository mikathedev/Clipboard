use glib::home_dir;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, STYLE_PROVIDER_PRIORITY_APPLICATION, Button};
use gtk4::gdk::Display;
use gtk4_layer_shell::{Layer, LayerShell, Edge, KeyboardMode};

#[tokio::main]
async fn main() {
    let application = Application::builder()
        .application_id("com.example.layershell")
        .build();

    application.connect_activate(|app|  {
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

        let left = (pos[0] - 100).min(1920 - 400).max(0);
        let top  = (pos[1] - 100).min(1080 - 650).max(0);

        window.set_margin(Edge::Left, left);
        window.set_margin(Edge::Top, top);



        let list = gtk4::ListBox::new();
        list.add_css_class("list");
        for item in get_items() {
            let button = Button::new();
            button.set_label(&item);

            let window_clone = window.clone();
            button.connect_clicked(move |btn| {
                let string = btn.label().unwrap().to_string();

                // Use wl-copy to put text into the Wayland clipboard
                let status = std::process::Command::new("wl-copy")
                    .arg(&string)
                    .status();

                match status {
                    Ok(_) => println!("Successfully copied: {}", string),
                    Err(e) => eprintln!("Failed to execute wl-copy: {}", e),
                }
                window_clone.close();
            });
            button.add_css_class("button");
            list.append(&button);
        }

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&list));
        window.set_child(Some(&scroll));
        window.add_css_class("window-class");
        load_css();
        window.present();

        // shorcut logic

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
    let css = if std::fs::exists(format!("{}/.config/clip.css", home_dir().display())).ok().unwrap() {
        std::fs::read_to_string(format!("{}/.config/clip.css", home_dir().display())).unwrap()
    } else {
        println!("{}/.config/clip.css", home_dir().display());
        std::fs::create_dir_all(format!("{}/.config", home_dir().display())).unwrap();
        std::fs::write(format!("{}/.config/clip.css", home_dir().display()), "").expect("failed to write");
        std::fs::read_to_string(format!("{}/.config/clip.css", home_dir().display())).unwrap()
    };
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&css);
    gtk4::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn get_items() -> Vec<String> {
    if !std::fs::exists(&format!("{}/clip.txt", home_dir().display())).ok().unwrap() {
        std::fs::write(&format!("{}/clip.txt", home_dir().display()), "").expect("failed to write buffer");
    }
    let buffer = std::fs::read_to_string(&format!("{}/clip.txt", home_dir().display())).ok().unwrap_or_default();
    let mut items = buffer.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    items.reverse();
    items
}