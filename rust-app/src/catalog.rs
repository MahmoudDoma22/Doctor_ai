use directories::UserDirs;
use std::{fs, path::{Path, PathBuf}};

pub struct Model {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub ram_gb: u64,
    pub size: &'static str,
    pub url: &'static str,
    pub file_name: &'static str,
    pub tier: &'static str,
}

pub const MODELS: &[Model] = &[
    Model {
        id: "gemma4-e2b",
        name: "Gemma 4 E2B · Q4_K_M",
        description: "الأخف للمحادثة اليومية والأجهزة محدودة الذاكرة.",
        ram_gb: 8,
        size: "حوالي 3 GB",
        url: "https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF/resolve/main/gemma-4-E2B-it-Q4_K_M.gguf?download=true",
        file_name: "gemma-4-E2B-it-Q4_K_M.gguf",
        tier: "خفيف",
    },
    Model {
        id: "gemma4-e4b",
        name: "Gemma 4 E4B · Q4_K_M",
        description: "جودة أعلى مع استهلاك أكبر للذاكرة؛ مناسب للأجهزة ذات 16 GB RAM فأكثر.",
        ram_gb: 16,
        size: "حوالي 5 GB",
        url: "https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF/resolve/main/gemma-4-E4B-it-Q4_K_M.gguf?download=true",
        file_name: "gemma-4-E4B-it-Q4_K_M.gguf",
        tier: "متوازن",
    },
];

pub fn model_dir() -> PathBuf {
    UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".doctor-ai")
        .join("models")
}

pub fn model_path(model: &Model) -> PathBuf {
    model_dir().join(model.file_name)
}

/// Search only known model locations; never crawl the whole disk.
pub fn find_model_path(model: &Model) -> Option<PathBuf> {
    let target = model.file_name.to_ascii_lowercase();
    let mut roots = vec![model_dir()];
    if let Some(home) = UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
        roots.push(home.join(".cache").join("huggingface").join("hub"));
        roots.push(home.join("Downloads").join("DoctorAI"));
        roots.push(home.join("Documents").join("DoctorAI").join("models"));
    }
    for root in roots {
        if let Some(found) = find_named_file(&root, &target, 7) {
            return Some(found);
        }
    }
    None
}

fn find_named_file(root: &Path, target: &str, depth: usize) -> Option<PathBuf> {
    if depth == 0 || !root.is_dir() {
        return None;
    }
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.file_name().and_then(|n| n.to_str())
                .map(|n| n.eq_ignore_ascii_case(target)).unwrap_or(false)
        {
            return Some(path);
        }
    }
    for entry in fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_named_file(&path, target, depth - 1) {
                return Some(found);
            }
        }
    }
    None
}

pub fn installed_model_files() -> Vec<PathBuf> {
    let mut roots = vec![model_dir()];
    if let Some(home) = UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
        roots.push(home.join(".cache").join("huggingface").join("hub"));
        roots.push(home.join("Downloads").join("DoctorAI"));
    }
    let mut found = Vec::new();
    for root in roots {
        collect_gguf(&root, 5, &mut found);
    }
    found.sort();
    found.dedup();
    found
}

fn collect_gguf(root: &Path, depth: usize, output: &mut Vec<PathBuf>) {
    if depth == 0 || !root.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else { return; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("gguf")).unwrap_or(false) {
                output.push(path);
            }
        } else if path.is_dir() {
            collect_gguf(&path, depth - 1, output);
        }
    }
}

pub fn present(model: &Model) -> bool {
    find_model_path(model).is_some()
}

pub fn detect_available_ram_gb() -> u64 {
    let mut s = sysinfo::System::new();
    s.refresh_memory();
    (s.available_memory() / 1024 / 1024 / 1024).max(1)
}

pub fn detect_free_disk_gb() -> u64 {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let home = UserDirs::new().map(|u| u.home_dir().to_path_buf());
    let disk = home.as_ref()
        .and_then(|h| disks.list().iter().filter(|d| h.starts_with(d.mount_point())).max_by_key(|d| d.mount_point().as_os_str().len()))
        .or_else(|| disks.list().iter().max_by_key(|d| d.available_space()));
    disk.map(|d| d.available_space() / 1024 / 1024 / 1024).unwrap_or(0)
}

pub fn recommended(ram: u64, available: u64) -> usize {
    if ram >= 16 && available >= 7 { 1 } else { 0 }
}

pub fn detect_ram_gb() -> u64 {
    let mut s = sysinfo::System::new();
    s.refresh_memory();
    (s.total_memory() / 1024 / 1024 / 1024).max(1)
}

pub fn detect_cpu() -> String {
    let mut s = sysinfo::System::new_all();
    s.refresh_cpu_all();
    let brand = s.cpus().first().map(|c| c.brand()).unwrap_or("Unknown CPU");
    format!("{} · {} logical threads", brand, s.cpus().len())
}

pub fn ensure_model_dir() -> Result<PathBuf, String> {
    let p = model_dir();
    fs::create_dir_all(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(p)
}

pub fn detect_tool(executable: &str, common_paths: &[PathBuf]) -> bool {
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join(executable).is_file() {
                return true;
            }
        }
    }
    common_paths.iter().any(|p| p.is_file() || p.is_dir())
}

pub fn installed_tools() -> Vec<(&'static str, bool)> {
    let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from).unwrap_or_default();
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_default();
    vec![
        ("ComfyUI", detect_tool("ComfyUI.exe", &[local.join("Programs/ComfyUI"), program_files.join("ComfyUI")])),
        ("Blender", detect_tool("blender.exe", &[program_files.join("Blender Foundation/Blender"), local.join("Programs/Blender")])),
        ("Krita", detect_tool("krita.exe", &[program_files.join("Krita (x64)/bin/krita.exe")])),
        ("Whisper.cpp", detect_tool("whisper-cli.exe", &[local.join("Programs/whisper.cpp/whisper-cli.exe")])),
        ("Piper", detect_tool("piper.exe", &[local.join("Programs/Piper/piper.exe")])),
        ("FFmpeg", detect_tool("ffmpeg.exe", &[program_files.join("ffmpeg/bin/ffmpeg.exe")])),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendation_respects_available_memory() {
        assert_eq!(recommended(8, 7), 0);
        assert_eq!(recommended(16, 6), 0);
        assert_eq!(recommended(16, 7), 1);
        assert_eq!(recommended(32, 24), 1);
    }

    #[test]
    fn model_targets_are_unique_and_gguf() {
        assert_ne!(MODELS[0].file_name, MODELS[1].file_name);
        assert!(MODELS.iter().all(|m| m.file_name.ends_with(".gguf")));
    }

    #[test]
    fn model_search_is_scoped_to_known_locations() {
        assert!(model_dir().ends_with(".doctor-ai/models"));
    }
}
