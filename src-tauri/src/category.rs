use regex::Regex;
use std::sync::LazyLock;

use serde::Serialize;

/// Represents the human-friendly categories for files.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
pub enum FileCategory {
    StaleInstallers,
    SocialMedia,
    HeavyVideos,
    WorkDocuments,
    Archives,
    DesignFiles,
    MemesAndGifs,
    Unknown,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileCategory::StaleInstallers => "Stale Installers",
            FileCategory::SocialMedia => "Social Media Media",
            FileCategory::HeavyVideos => "Heavy Videos",
            FileCategory::WorkDocuments => "Work Documents",
            FileCategory::Archives => "Archives",
            FileCategory::DesignFiles => "Design Files",
            FileCategory::MemesAndGifs => "Memes & Gifs",
            FileCategory::Unknown => "Others",
        }
    }
}

/// Static Regex patterns for performance
static RE_SOCIAL: LazyLock<Regex> = LazyLock::new(|| 
    Regex::new(r"(?i)(WhatsApp Image|Screenshot_|IMG_|Snapchat-)").unwrap()
);
static RE_VIDEO: LazyLock<Regex> = LazyLock::new(|| 
    Regex::new(r"(?i)(Screen Recording|WhatsApp Video|Zoom_)").unwrap()
);
static RE_DOCS: LazyLock<Regex> = LazyLock::new(|| 
    Regex::new(r"(?i)(Draft|Final|Revisi|Invoice)").unwrap()
);

/// Categorizes a file based on its name and extension using heuristics.
pub fn categorize_file(file_name: &str, extension: &str) -> FileCategory {
    let ext = extension.to_lowercase();
    let ext_str = ext.as_str();

    // 1. Heuristic: Extension-based priority (Installers, Archives, Design, Memes)
    match ext_str {
        "exe" | "msi" | "appimage" | "deb" | "dmg" | "apk" => return FileCategory::StaleInstallers,
        "zip" | "rar" | "7z" | "tar.gz" => return FileCategory::Archives,
        "psd" | "ai" | "svg" | "cdr" | "fig" => return FileCategory::DesignFiles,
        "gif" | "webm" => return FileCategory::MemesAndGifs,
        _ => {}
    }

    // 2. Heuristic: Name Pattern + Extension (Social Media)
    if RE_SOCIAL.is_match(file_name) || matches!(ext_str, "png" | "jpg" | "jpeg" | "webp" | "heic") {
        return FileCategory::SocialMedia;
    }

    // 3. Heuristic: Name Pattern + Extension (Videos)
    if RE_VIDEO.is_match(file_name) || matches!(ext_str, "mp4" | "mkv" | "mov" | "avi") {
        return FileCategory::HeavyVideos;
    }

    // 4. Heuristic: Name Pattern + Extension (Work Docs)
    if RE_DOCS.is_match(file_name) || matches!(ext_str, "pdf" | "docx" | "xlsx" | "pptx" | "csv") {
        return FileCategory::WorkDocuments;
    }

    FileCategory::Unknown
}
