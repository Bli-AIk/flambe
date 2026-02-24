//! Write AmScene back to .amproj ZIP format.

use bevy_alight_motion::schema::*;
use quick_xml::se::to_string as xml_to_string;
use std::io::{Cursor, Write};
use zip::ZipWriter;
use zip::write::FileOptions;

/// Serialize an AmScene to XML string.
pub fn scene_to_xml(scene: &AmScene) -> Result<String, quick_xml::SeError> {
    let body = xml_to_string(scene)?;
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{body}"
    ))
}

/// Package an AmScene into a .amproj ZIP file (in-memory).
pub fn scene_to_amproj(
    scene: &AmScene,
    embedded_fonts: &std::collections::HashMap<String, Vec<u8>>,
    embedded_images: &std::collections::HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, AmprojWriteError> {
    let xml = scene_to_xml(scene).map_err(AmprojWriteError::Xml)?;

    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // Write project XML
    zip.start_file("project.json", options.clone())
        .map_err(AmprojWriteError::Zip)?;
    zip.write_all(xml.as_bytes())
        .map_err(AmprojWriteError::Io)?;

    // Write embedded fonts
    for (name, data) in embedded_fonts {
        let path = format!("fonts/{name}");
        zip.start_file(&path, options.clone())
            .map_err(AmprojWriteError::Zip)?;
        zip.write_all(data).map_err(AmprojWriteError::Io)?;
    }

    // Write embedded images
    for (name, data) in embedded_images {
        let path = format!("images/{name}");
        zip.start_file(&path, options.clone())
            .map_err(AmprojWriteError::Zip)?;
        zip.write_all(data).map_err(AmprojWriteError::Io)?;
    }

    let cursor = zip.finish().map_err(AmprojWriteError::Zip)?;
    Ok(cursor.into_inner())
}

#[derive(Debug)]
pub enum AmprojWriteError {
    Xml(quick_xml::SeError),
    Zip(zip::result::ZipError),
    Io(std::io::Error),
}

impl std::fmt::Display for AmprojWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "XML serialization error: {e}"),
            Self::Zip(e) => write!(f, "ZIP error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for AmprojWriteError {}
