use glib::home_dir;
use image::{ImageError, ImageReader};
use rusqlite::{Connection, Result, params};
use std::io::Cursor;
use wayland_clipboard_listener::WlClipboardPasteStream;
use wayland_clipboard_listener::WlListenType;
use regex::Regex;
use log::{debug, error};

#[derive(Debug)]
struct Clipboard {
    pub content: Option<String>,
    pub data_type: String,
    pub image: Option<Vec<u8>>,
    pub imaget: Option<Vec<u8>>,
}

#[tokio::main]
async fn main() {
    let home = home_dir();
    let clip_txt = format!("{}/clip.txt", home.display());
    // enable logging
    env_logger::init();
    
    if !std::fs::exists(&clip_txt).unwrap_or(false) {
        if let Err(e) = std::fs::write(&clip_txt, "") {
            error!("failed to write buffer: {}", e);
        }
    }

    let stream_init = WlClipboardPasteStream::init(WlListenType::ListenOnCopy);
    let mut stream = match stream_init {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to initialize clipboard stream: {:?}", e);
            return;
        }
    };

    for context in stream.paste_stream().flatten() {
        debug!("writing to file:");
        if let Err(e) = append(&context.context.context, &context).await {
            error!("Failed to append to clipboard: {}", e);
        }
    }
}

async fn download(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let client = reqwest::ClientBuilder::new()
        .user_agent("ClipboardBot/1.0 (+mikathedev.app)")
        .build()?;
    let img = client.get(url).send().await?;
    debug!("IMAGE: {:?}", img);
    let bytes = img.bytes().await?;
    Ok(bytes.to_vec())

}

async fn append(
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

    debug!("info paste: {:#?} \n\n format: {:#?}", paste, format);

    if format.mime_types.iter().any(|m| m.contains("image")) {
        debug!("image");
        
        let (image, thumb) = if paste.first() <= Some(&60_u8) {
            let html = String::from_utf8_lossy(paste.as_slice());
            let re = Regex::new("src=\"([^\"]*)\"").unwrap();
            
            if let Some(caps) = re.captures(&html) {
                let src_content = &caps[1].replace("&amp;", "&");
                debug!("Found src content: {} \n Downloading...", src_content);
                let img = download(src_content).await.ok();
                    convert_buffer_to_png(img.unwrap()).unwrap()
            } else {
                convert_buffer_to_png(paste.clone()).unwrap()
            }
        } else {convert_buffer_to_png(paste.clone()).unwrap()};

        let buffer = Clipboard {
            content: None,
            data_type: "I".to_string(),
            image: Some(image),
            imaget: Some(thumb),
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
        debug!("inserted image");
    } else if format.mime_types.iter().any(|m| m.contains("text/plain")) && !paste.is_empty() {
        let content = String::from_utf8(paste.clone()).unwrap_or_else(|_| "err".to_string());
        if content.chars().any(|c| !c.is_control() && !c.is_whitespace()) {
            debug!("text: {:?}", paste);
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
            debug!("inserted: {:?}", buffer);
        }
    } else {
        debug!("{:?}", format.mime_types);
    }



    Ok(())
}

fn dumb_down_image(image_vec: &Vec<u8>) -> Result<Vec<u8>, ImageError> {
    let img = ImageReader::new(Cursor::new(image_vec))
        .with_guessed_format()?
        .decode()?;
    let t = img.resize(1500, 1500, image::imageops::FilterType::Nearest);
    let mut thumb: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    t.write_to(&mut thumb, image::ImageFormat::Png)?;

    Ok(thumb.into_inner())
}

fn convert_buffer_to_png(buffer: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>), ImageError> {
    let img = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()?
        .decode()?;
    let mut image: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut image, image::ImageFormat::Png)?;
    let thumb = dumb_down_image(&image.clone().into_inner())?;
    
    Ok((image.into_inner(), thumb))
}