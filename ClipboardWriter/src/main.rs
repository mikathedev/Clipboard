use glib::home_dir;
use image::{ImageError, ImageReader};
use rusqlite::{Connection, Result, params};
use std::io::Cursor;
use wayland_clipboard_listener::WlClipboardPasteStream;
use wayland_clipboard_listener::WlListenType;

#[derive(Debug)]
struct Clipboard {
    pub content: Option<String>,
    pub data_type: String,
    pub image: Option<Vec<u8>>,
    pub imaget: Option<Vec<u8>>,
}

fn main() {
    let home = home_dir();
    let clip_txt = format!("{}/clip.txt", home.display());

    if !std::fs::exists(&clip_txt).unwrap_or(false) {
        if let Err(e) = std::fs::write(&clip_txt, "") {
            eprintln!("failed to write buffer: {}", e);
        }
    }

    let stream_init = WlClipboardPasteStream::init(WlListenType::ListenOnCopy);
    let mut stream = match stream_init {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to initialize clipboard stream: {:?}", e);
            return;
        }
    };

    for context in stream.paste_stream().flatten() {
        println!("writing to file:");
        if let Err(e) = append(&context.context.context, &context) {
            eprintln!("Failed to append to clipboard: {}", e);
        }
    }
}

fn append(
    paste: &Vec<u8>,
    format: &wayland_clipboard_listener::ClipBoardListenMessage,
) -> Result<()> {
    let conn = Connection::open("/home/mika/.config/clips.db")?;
    conn.execute_batch(
        "
    CREATE TABLE IF NOT EXISTS clipboard (
        id INTEGER PRIMARY KEY,
        data_type TEXT NOT NULL,
        content TEXT,
        imagep BLOB,
        imaget BLOB
    )
",
    )?;

    if format.mime_types.iter().any(|m| m.contains("image")) {
        println!("image");
        let thumb = dumb_down_image(paste).ok();

        let buffer = Clipboard {
            content: None,
            data_type: "I".to_string(),
            image: Some(paste.clone()),
            imaget: thumb,
        };

        conn.execute(
            "INSERT INTO clipboard (data_type, content, imagep, imaget) VALUES (?1, ?2, ?3, ?4)",
            params![
                buffer.data_type,
                buffer.content,
                buffer.image,
                buffer.imaget
            ],
        )?;
    } else if format.mime_types.iter().any(|m| m.contains("text/plain")) {
        println!("text: {:?}", paste);

        let content = String::from_utf8(paste.clone()).unwrap_or_else(|_| "err".to_string());
        let buffer = Clipboard {
            content: Some(content),
            data_type: "T".to_string(),
            image: None,
            imaget: None,
        };

        conn.execute(
            "INSERT INTO clipboard (data_type, content, imagep, imaget) VALUES (?1, ?2, ?3, ?4)",
            params![
                buffer.data_type,
                buffer.content,
                buffer.image,
                buffer.imaget
            ],
        )?;
    } else {
        println!("{:?}", format.mime_types);
    }

    // Optional: write raw paste to a file for debugging
    let _ = std::fs::write("/home/mika/image.png", paste);

    Ok(())
}

fn dumb_down_image(image_vec: &Vec<u8>) -> Result<Vec<u8>, ImageError> {
    let img = ImageReader::new(Cursor::new(image_vec))
        .with_guessed_format()?
        .decode()?;
    let t = img.resize(400, 400, image::imageops::FilterType::Nearest);
    let mut thumb: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    t.write_to(&mut thumb, image::ImageFormat::Png)?;

    Ok(thumb.into_inner())
}
