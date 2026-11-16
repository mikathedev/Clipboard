use gdk_pixbuf::PixbufLoader;
use gdk_pixbuf::prelude::*;
use glib::home_dir;
use std::cell::RefCell;
use std::rc::Rc;
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use hyprland::error::HyprError;
use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActiveOptional;
use rusqlite::{Connection, Result};
use std::io::Write;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use hyprland::data::Monitors;
use log::{debug, error};
use std::time::Instant;


#[derive(Debug)]
struct Clipboard {
    pub content: Option<String>,
    pub data_type: String,
    pub image: Option<Vec<u8>>,
    pub imaget: Option<Vec<u8>>,
    pub id: i64,
}
#[derive(Debug)]
struct Monitor { 
    pub width: u64,
    pub height: u64,
    pub x: u64,
    pub y: u64,
    pub scale: f64,
}
#[derive(Debug)]
struct Ui {
    pub window: ApplicationWindow,
    pub list: gtk4::ListBox,
    pub loaded_items: Vec<i64>
}


#[tokio::main]
async fn main() {
    env_logger::init(); // Initialize the logger at the very top
    let ui: Rc<RefCell<Option<Ui>>> = Rc::new(RefCell::new(None));
    let application = Application::builder()
        .application_id("com.mika.clipboard")
        .build();
    
    application.connect_startup({
        let ui = ui.clone();            // second handle to the same RefCell
        move |app| {
            std::mem::forget(app.hold());
            *ui.borrow_mut() = Some(build_window(app));
        }
    });
    application.connect_activate( move |_app| {
        let t0 = Instant::now();
        let t = Instant::now();
        let mut ui_borrow = ui.borrow_mut();
        let Some(ui) = (*ui_borrow).as_mut() else { return };


        //mouse
        let (pos, res) = get_mouse_and_monitor_res();
        let vheight = (res.height as f64/res.scale) as i32;
        let vwidth = (res.width as f64/res.scale) as i32;
        let left = (pos.x - (res.x as i64 + 200)).clamp(5, vwidth as i64 - 505) as i32;
        let top = (pos.y - (res.y as i64 + 200)).clamp(5, vheight as i64 - 660) as i32;
        debug!("running window at x: {left}, y: {top} vheight: {vheight}, vwidth: {vwidth}");
        ui.window.set_margin(Edge::Left, left);
        ui.window.set_margin(Edge::Top, top);
        debug!("window pos took: {:?}s", t.elapsed());
        let t = Instant::now();

        if let Ok(items) = get_items(5) {
            let window_clone = ui.window.clone();

            for item in items {
                if !ui.loaded_items.contains(&item.id) {
                    ui.loaded_items.push(item.id.to_owned());
                    let button = create_button(&item, &window_clone);
                    if let Some(booger_aids) = button {
                        booger_aids.add_css_class("button");
                        booger_aids.set_has_frame(false);
                        ui.list.add_css_class("list");
                        ui.list.append(&booger_aids);
                    } else {
                        drop(button);
                    }
                }
            }
        }
        
        debug!("loading items took: {:?}", t.elapsed());
        
        let t = Instant::now();
        ui.window.set_visible(true);
        debug!("## TIME ##\nwindow took: {:?} to present itself", t.elapsed());
        
        debug!("\n\n\n### total elapsed time: {:?} ###", t0.elapsed());

        //Load next ones
        if let Ok(items) = get_items(20) {
            let rest = items.into_iter().skip(5);
            let window_clone = ui.window.clone();

            for item in rest {
                if !ui.loaded_items.contains(&item.id) {
                    ui.loaded_items.push(item.id.to_owned());
                    let button = create_button(&item, &window_clone);
                    if let Some(booger_aids) = button {
                        booger_aids.add_css_class("button");
                        booger_aids.set_has_frame(false);
                        ui.list.add_css_class("list");
                        ui.list.append(&booger_aids);
                    } else {
                        drop(button);
                    }
                }
            }
        }
    });

    application.run();
    
}


fn build_window(app: &Application) -> Ui {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hyprland Clipboard")
        .build();

    // 1. Initialize the window for layer shell use
    window.init_layer_shell();

    // 2. Set the layer (Background, Bottom, Top, or Overlay)
    window.set_layer(Layer::Overlay);
    window.gets_focus_visible();
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    // 3. Anchor to specific edges (e.g., Top, Left, and Right for a top bar) and set size
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);

    window.set_default_size(500, 600);

    window.connect_is_active_notify(|win| {
        debug!("active window");
        if !win.is_active() {
            sleep(Duration::new(0, 500));
            win.set_visible(false);
                // Prevent GTK from destroying the window after hiding it
            let _ = glib::Propagation::Stop;
        }
    });
    
    let scroll = gtk4::ScrolledWindow::new();
    let list = gtk4::ListBox::new();
    
    list.add_css_class("list");
    list.set_selection_mode(gtk4::SelectionMode::None);
    scroll.set_child(Some(&list));
    window.set_child(Some(&scroll));
    window.set_width_request(-1);
    window.add_css_class("window-class");
    
    load_css();
    Ui {window: window, list: list, loaded_items: Vec::<i64>::new()}
}

fn create_button(item: &Clipboard, window: &ApplicationWindow) -> Option<gtk4::Button> {
    let window_clone = window.clone();
    let button = gtk4::Button::new();
    debug!("data type: {}", item.data_type);
    if &item.data_type == "T" {
        if let Some(content) = &item.content {
                let has_real_content = content.chars().any(|c| !c.is_control() && !c.is_whitespace());
        
                if has_real_content {
                    // debug!("{:#?}", item); 
                    let content_ref = item.content.clone().unwrap();
                    
                    let trimmed = match content_ref.char_indices().nth(300) {
                        Some((idx, _)) => &content_ref[..idx],
                        None => &content_ref,
                    };
                    
                    // debug!("{}", trimmed.len());
                    let label = if trimmed.len() == 300 {
                        gtk4::Label::new(Some(format!("{}...", trimmed).as_str()))
                    } else {
                        gtk4::Label::new(Some(trimmed))
                    };
                    
                    label.set_wrap(true);
                    label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
                    label.set_max_width_chars(30); // Approximate width in characters
                    label.set_width_request(300); // Minimum width in pixels
                    label.set_xalign(0.0); // Left align text
                    button.set_child(Some(&label));
                    button.connect_clicked(move |_| {
                        debug!("{}", label.text());
                        window_clone.hide();
                        copy_text_to_clipboard(content_ref.to_string()).expect("msg");
                    });
                    Some(button)
                } else {
                    None
                }
            } else {
                None
            }
    } else if &item.data_type == "I" {
            debug!("{:?}", item.data_type);

            let pixbuf = match pixbuf_from_encoded_vec(item.imaget.clone().unwrap_or_default()) {
                Ok(pixbuf) => pixbuf,
                Err(e) => {
                    error!("Failed to load pixbuf: {}", e);
                    return None;
                }
            };    
            let aspect_ratio = pixbuf.height() as f64 / pixbuf.width() as f64;
            let target_width = 400;
            let target_height = (target_width as f64 * aspect_ratio) as i32;

            // 1. Use Picture instead of Image
            let picture = gtk4::Picture::for_pixbuf(&pixbuf);

            // 2. Tell the picture how to handle its content
            picture.set_keep_aspect_ratio(true);
            picture.set_can_shrink(true);

            // 3. Set the size on the picture widget
            picture.set_size_request(target_width, target_height);

            // 4. Ensure it fills the button
            picture.set_hexpand(true);
            picture.set_vexpand(true);
            button.set_child(Some(&picture));
            button.add_css_class("image-button");

            let item_image = item.image.clone().unwrap_or_default();
            button.connect_clicked(move |_| {
                // debug!("image clicked");
                window_clone.hide();
                copy_png_to_clipboard(&item_image).expect("msg");
            });
            Some(button)
            
        } else {
            debug!("{:?}", item.data_type);
            None
        }
}

fn get_mouse_and_monitor_res() -> (hyprland::data::CursorPosition, Monitor) {
    let mouse = std::thread::spawn(|| {check_mouse()});
    let monitor = std::thread::spawn(|| {get_monitior_res()});
    (mouse.join().unwrap(), monitor.join().unwrap().unwrap_or(Monitor { width: 0, height: 0, x: 0, y: 0, scale: 0.0 }))
}

fn get_monitior_res() -> hyprland::Result<Monitor> {

    let monitors = Monitors::get()?;
    
        for monitor in monitors.clone() {
            debug!("Monitor Name: {}", monitor.name);
            debug!("Resolution: {}x{} @ {}Hz", monitor.width, monitor.height, monitor.refresh_rate);
            debug!("Focused: {}", monitor.focused);
            debug!("Active Workspace: {}", monitor.active_workspace.name);
            debug!("-----------------------------------");
            if monitor.focused {
                return Ok(Monitor { 
                    width: monitor.width as u64,
                    height: monitor.height as u64,
                    x: monitor.x as u64,
                    y: monitor.y as u64,
                    scale: monitor.scale as f64 
                });
            }
        }

    Ok(Monitor { width: 0, height: 0, x: 0, y: 0, scale: 0.0 })
}

fn check_mouse() -> hyprland::data::CursorPosition {
    debug!("{:?}", hyprland::data::CursorPosition::get());
    hyprland::data::CursorPosition::get().unwrap()
}

fn load_css() {
    let home = home_dir();
    let css_path = format!("{}/.config/clip.css", home.display());

    let css = if std::fs::exists(&css_path).unwrap_or(false) {
        std::fs::read_to_string(&css_path).unwrap_or_default()
    } else {
        debug!("{}", css_path);
        let config_dir = format!("{}/.config", home.display());
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            debug!("Failed to create config directory: {}", e);
        }
        if let Err(e) = std::fs::write(&css_path, "") {
            debug!("Failed to write empty CSS file: {}", e);
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
        debug!("Could not connect to a display.");
    }
}

fn get_items(amount: u16) -> Result<Vec<Clipboard>> {
    // 1. Open (or create) a local database file
    let conn = Connection::open("/home/mika/.config/clips.db")?;
    let query = &format!("SELECT * FROM clipboard ORDER BY id DESC LIMIT {}", amount);
    let rows = conn
        .prepare(query)?
        .query_map([], |row| {
            Ok(Clipboard {
                content: row.get(2)?,
                data_type: row.get(1)?,
                image: row.get(3)?,
                imaget: row.get(4)?,
                id: row.get(0)?,
            })
        })?
        .collect::<Result<Vec<Clipboard>, _>>()?;

    Ok(rows)
}

fn pixbuf_from_encoded_vec(data: Vec<u8>) -> Result<gdk_pixbuf::Pixbuf, Box<dyn std::error::Error>> {
    let loader = PixbufLoader::new();
    loader.write(&data)?;
    loader.close()?;
    Ok(loader.pixbuf().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to get pixbuf"))?)
}

fn active_window_terminal() -> Result<bool, HyprError> {
    let terminals: &[&str] = &[
        "kitty", "alacritty", "foot", "footclient", "org.wezfurlong.wezterm",
        "com.mitchellh.ghostty", "org.kde.konsole", "konsole", "xterm",
        "org.gnome.terminal", "gnome-terminal-server", "org.gnome.ptyxis",
    ];
    
    if let Some(active) = hyprland::data::Client::get_active()? {
        let window = active.class;
        Ok(terminals.contains(&window.as_str()))
    } else {
        debug!("NO DATA, assuming no terminal");
        Ok(false)
    }
}

fn copy_png_to_clipboard(png_bytes: &Vec<u8>) -> std::io::Result<()> {
    let mut child = std::process::Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write_all(&png_bytes)?;
    drop(stdin); // Close stdin so wl-copy knows we are done

    child.wait()?;
    trigger_paste();
    Ok(())
}

fn copy_text_to_clipboard(text: String) -> std::io::Result<()> {
    std::process::Command::new("wl-copy")
        .arg(text)
        .status()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    trigger_paste();
    Ok(())
}

fn trigger_paste()  {
    glib::timeout_add_local_once(Duration::from_millis(50), ||
     {if active_window_terminal().unwrap_or(false) {
    Command::new("wtype")
        .args(["-M", "ctrl", "-M", "shift", "v", "-m", "shift", "-m", "ctrl"])
        .status()
        .expect("Failed to execute paste command");
    } else {
    Command::new("wtype")
        .args(["-M", "ctrl", "v", "-m", "ctrl"])
        .status()
        .expect("Failed to execute paste command");
    }
});
}

fn throw_error(err_msg: String) {
    let _ = std::process::Command::new("notify-send")
        .arg("Clipboard Error")
        .arg(format!("Failed to paste: {err_msg}"))
        .arg("-u")
        .arg("critical")
        .arg("-i")
        .arg("edit-paste")
        .spawn();
}
