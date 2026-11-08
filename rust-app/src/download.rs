use futures_util::StreamExt;
use std::{path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::mpsc};

#[derive(Clone, Debug)]
pub enum DownloadEvent { Progress(u64, Option<u64>), Finished(PathBuf), Error(String), Cancelled }

pub fn start_download(url: String, destination: PathBuf, cancel: Arc<AtomicBool>, tx: mpsc::UnboundedSender<DownloadEvent>) {
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
            Ok(rt) => rt, Err(e) => { let _ = tx.send(DownloadEvent::Error(format!("إنشاء بيئة التنزيل: {e}"))); return; }
        };
        runtime.block_on(async move {
            if let Some(parent) = destination.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await { let _ = tx.send(DownloadEvent::Error(format!("إنشاء مجلد الموديلات: {e}"))); return; }
            }
            let partial = destination.with_extension("part");
            let offset = tokio::fs::metadata(&partial).await.map(|m| m.len()).unwrap_or(0);
            let client = reqwest::Client::builder().connect_timeout(std::time::Duration::from_secs(20)).build().unwrap();
            let mut request = client.get(&url);
            if offset > 0 { request = request.header(reqwest::header::RANGE, format!("bytes={offset}-")); }
            let response = match request.send().await {
                Ok(r) => r, Err(e) => { let _ = tx.send(DownloadEvent::Error(format!("الاتصال بمصدر الموديل: {e}"))); return; }
            };
            if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                let _ = tx.send(DownloadEvent::Error(format!("خادم التنزيل رجع HTTP {}", response.status()))); return;
            }
            let resumed = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
            let total = response.content_length().map(|n| n + if resumed { offset } else { 0 });
            let mut file = match OpenOptions::new().create(true).write(true).append(resumed).truncate(!resumed).open(&partial).await {
                Ok(f) => f, Err(e) => { let _ = tx.send(DownloadEvent::Error(format!("فتح الملف {}: {e}", partial.display()))); return; }
            };
            let mut stream = response.bytes_stream();
            let mut received = if resumed { offset } else { 0 };
            while let Some(chunk) = stream.next().await {
                if cancel.load(Ordering::Relaxed) { let _ = tx.send(DownloadEvent::Cancelled); return; }
                match chunk {
                    Ok(bytes) => {
                        if let Err(e) = file.write_all(&bytes).await { let _ = tx.send(DownloadEvent::Error(format!("كتابة الملف {}: {e}", partial.display()))); return; }
                        received += bytes.len() as u64;
                        let _ = tx.send(DownloadEvent::Progress(received, total));
                    }
                    Err(e) => { let _ = tx.send(DownloadEvent::Error(format!("انقطع التنزيل؛ الملف الجزئي محفوظ للاستكمال: {e}"))); return; }
                }
            }
            if let Err(e) = file.flush().await { let _ = tx.send(DownloadEvent::Error(format!("حفظ الملف: {e}"))); return; }
            drop(file);
            if let Err(e) = tokio::fs::rename(&partial, &destination).await { let _ = tx.send(DownloadEvent::Error(format!("نقل الملف النهائي: {e}"))); return; }
            let _ = tx.send(DownloadEvent::Finished(destination));
        });
    });
}
