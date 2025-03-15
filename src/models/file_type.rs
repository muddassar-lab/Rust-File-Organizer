#[derive(Debug)]
pub enum FileType {
    Video,
    Music,
    Document,
    Picture,
    Program,
    Other,
}

impl FileType {
    pub fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "wmv" => FileType::Video,
            "mp3" | "wav" | "flac" | "m4a" | "ogg" => FileType::Music,
            "pdf" | "doc" | "docx" | "txt" | "rtf" => FileType::Document,
            "jpg" | "jpeg" | "png" | "gif" | "bmp" => FileType::Picture,
            "exe" | "msi" | "bat" | "sh" | "app" => FileType::Program,
            _ => FileType::Other,
        }
    }
}