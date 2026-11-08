use directories::UserDirs;
use std::{fs, path::PathBuf};

pub struct Model {
    pub id: &'static str, pub name: &'static str, pub description: &'static str,
    pub ram_gb: u64, pub size: &'static str, pub url: &'static str, pub file_name: &'static str, pub tier: &'static str,
}
pub const MODELS: &[Model] = &[
    Model { id:"gemma4-e2b", name:"Gemma 4 E2B · Q4_K_M", description:"الأخف للمحادثة اليومية والأجهزة محدودة الذاكرة.", ram_gb:8, size:"حوالي 3 GB", url:"https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF/resolve/main/gemma-4-E2B-it-Q4_K_M.gguf?download=true", file_name:"gemma-4-E2B-it-Q4_K_M.gguf", tier:"خفيف" },
    Model { id:"gemma4-e4b", name:"Gemma 4 E4B · Q4_K_M", description:"جودة أعلى مع استهلاك أكبر للذاكرة؛ مناسب للأجهزة ذات 16 GB RAM فأكثر.", ram_gb:16, size:"حوالي 5 GB", url:"https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF/resolve/main/gemma-4-E4B-it-Q4_K_M.gguf?download=true", file_name:"gemma-4-E4B-it-Q4_K_M.gguf", tier:"متوازن" },
];
pub fn model_dir() -> PathBuf {
    UserDirs::new().map(|u|u.home_dir().to_path_buf()).unwrap_or_else(||PathBuf::from(".")).join(".doctor-ai").join("models")
}
pub fn model_path(model:&Model)->PathBuf { model_dir().join(model.file_name) }
pub fn present(model:&Model)->bool { model_path(model).is_file() }
pub fn detect_available_ram_gb()->u64 { let mut s=sysinfo::System::new();s.refresh_memory();(s.available_memory()/1024/1024/1024).max(1) }
pub fn detect_free_disk_gb()->u64 { let d=sysinfo::Disks::new_with_refreshed_list();d.list().iter().map(|x|x.available_space()).max().unwrap_or(0)/1024/1024/1024 }
pub fn recommended(ram:u64,available:u64)->usize { if ram>=16 && available>=7 {1} else {0} }
pub fn detect_ram_gb()->u64 { let mut s=sysinfo::System::new();s.refresh_memory();(s.total_memory()/1024/1024/1024).max(1) }
pub fn detect_cpu()->String { let mut s=sysinfo::System::new_all();s.refresh_cpu_all();format!("{} logical threads",s.cpus().len()) }
pub fn ensure_model_dir()->Result<PathBuf,String>{let p=model_dir();fs::create_dir_all(&p).map_err(|e|format!("{}: {e}",p.display()))?;Ok(p)}
