use glib::home_dir;
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Clipboard {
    pub content: Option<String>,
    pub data_type: String,
    pub image: Option<Vec<u8>>,
    pub imaget: Option<Vec<u8>>,
}

#[tokio::main]
async fn main() {
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

        window.connect_is_active_notify(|win| {
            println!("active window");
            if !win.is_active() {
                win.close();
            }
        });

        // 3. Anchor to specific edges (e.g., Top, Left, and Right for a top bar)
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);

        window.set_default_size(350, 600);

        //mouse
        let pos = check_mouse();

        let left = (pos[0] - 100).min(1920 - 400).max(0);
        let top = (pos[1] - 100).min(1080 - 650).max(0);

        window.set_margin(Edge::Left, left);
        window.set_margin(Edge::Top, top);

        let list = gtk4::ListBox::new();
        list.add_css_class("list");
        if let Ok(items) = get_items() {
            for item in items {
                println!("{:?}", item);
            }
        } else {
            println!("Failed to get items");
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
        Ok(res) => String::from_utf8_lossy(&res.stdout)
            .trim()
            .split(",")
            .filter_map(|x| x.trim().parse::<i32>().ok())
            .collect(),
        _ => vec![0, 0],
    }
}

fn load_css() {
    let home = home_dir();
    let css_path = format!("{}/.config/clip.css", home.display());

    let css = if std::fs::exists(&css_path).unwrap_or(false) {
        std::fs::read_to_string(&css_path).unwrap_or_default()
    } else {
        println!("{}", css_path);
        let config_dir = format!("{}/.config", home.display());
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            eprintln!("Failed to create config directory: {}", e);
        }
        if let Err(e) = std::fs::write(&css_path, "") {
            eprintln!("Failed to write empty CSS file: {}", e);
        }
        String::new()
    };
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&css);
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    } else {
        eprintln!("Could not connect to a display.");
    }
}

fn get_items() -> Result<Vec<Clipboard>> {
    // 1. Open (or create) a local database file
    let conn = Connection::open("/home/mika/.config/clips.db")?;
    let query = "SELECT * FROM clipboard";
    let rows = conn
        .prepare(query)?
        .query_map([], |row| {
            Ok(Clipboard {
                content: row.get(2)?,
                data_type: row.get(1)?,
                image: row.get(3)?,
                imaget: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<Clipboard>, _>>()?;

    Ok(rows)
}
