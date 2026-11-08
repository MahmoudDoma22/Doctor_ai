use futures_util::StreamExt;
use std::{path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::mpsc};

#[derive(Clone, Debug)]
pub enum DownloadEvent {
    Progress(u64, Option<u64>),
    Finished(PathBuf),
    Error(String),
    Cancelled,
}

fn content_range_start(value: &str) -> Option<u64> {
    let range = value.strip_prefix("bytes ")?;
    let (span, _) = range.split_once('/')?;
    let (start, _) = span.split_once('-')?;
    start.parse().ok()
}

fn content_range_total(value: &str) -> Option<u64> {
    let (_, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    if total == "*" { None } else { total.parse().ok() }
}

pub fn start_download(
    url: String,
    destination: PathBuf,
    cancel: Arc<AtomicBool>,
    tx: mpsc::UnboundedSender<DownloadEvent>,
) {
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(DownloadEvent::Error(format!("إنشاء بيئة التنزيل: {e}")));
                return;
            }
        };
        runtime.block_on(async move {
            if let Some(parent) = destination.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    let _ = tx.send(DownloadEvent::Error(format!("إنشاء مجلد الموديلات: {e}")));
                    return;
                }
            }
            let partial = destination.with_extension("part");
            let offset = tokio::fs::metadata(&partial).await.map(|m| m.len()).unwrap_or(0);
            let client = match reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(20))
                .pool_max_idle_per_host(2)
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Error(format!("تهيئة اتصال التنزيل: {e}")));
                    return;
                }
            };
            let mut request = client.get(&url).header(reqwest::header::ACCEPT_ENCODING, "identity");
            if offset > 0 {
                request = request.header(reqwest::header::RANGE, format!("bytes={offset}-"));
            }
            let response = match request.send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Error(format!("الاتصال بمصدر الموديل: {e}")));
                    return;
                }
            };

            if response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                let total = response.headers().get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.strip_prefix("bytes */"))
                    .and_then(|v| v.parse::<u64>().ok());
                if total == Some(offset) && offset > 0 {
                    if let Err(e) = tokio::fs::rename(&partial, &destination).await {
                        let _ = tx.send(DownloadEvent::Error(format!("اعتماد الملف المكتمل {}: {e}", partial.display())));
                    } else {
                        let _ = tx.send(DownloadEvent::Finished(destination));
                    }
                } else {
                    let _ = tx.send(DownloadEvent::Error(format!("خادم التنزيل رفض استكمال الملف الجزئي (HTTP 416)، الحجم المحلي {offset} بايت. احذف الملف الجزئي وأعد المحاولة.")));
                }
                return;
            }
            if !response.status().is_success() {
                let _ = tx.send(DownloadEvent::Error(format!("خادم التنزيل رجع HTTP {}", response.status())));
                return;
            }

            let resumed = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
            let range_header = response.headers().get(reqwest::header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok()).map(str::to_owned);
            if resumed && range_header.as_deref().and_then(content_range_start) != Some(offset) {
                let _ = tx.send(DownloadEvent::Error("استجابة Range غير متطابقة مع حجم الملف الجزئي؛ أوقفت التنزيل لمنع تلف الملف.".into()));
                return;
            }
            let total = if resumed {
                range_header.as_deref().and_then(content_range_total)
            } else {
                response.content_length()
            };
            let mut file = match OpenOptions::new()
                .create(true)
                .write(true)
                .append(resumed)
                .truncate(!resumed)
                .open(&partial).await
            {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Error(format!("فتح الملف {}: {e}", partial.display())));
                    return;
                }
            };
            let mut stream = response.bytes_stream();
            let mut received = if resumed { offset } else { 0 };
            while let Some(chunk) = stream.next().await {
                if cancel.load(Ordering::Relaxed) {
                    let _ = file.flush().await;
                    let _ = tx.send(DownloadEvent::Cancelled);
                    return;
                }
                match chunk {
                    Ok(bytes) => {
                        if let Err(e) = file.write_all(&bytes).await {
                            let _ = tx.send(DownloadEvent::Error(format!("كتابة الملف {}: {e}", partial.display())));
                            return;
                        }
                        received += bytes.len() as u64;
                        let _ = tx.send(DownloadEvent::Progress(received, total));
                    }
                    Err(e) => {
                        let _ = file.flush().await;
                        let _ = tx.send(DownloadEvent::Error(format!("انقطع التنزيل؛ الملف الجزئي محفوظ للاستكمال: {e}")));
                        return;
                    }
                }
            }
            if let Err(e) = file.sync_all().await {
                let _ = tx.send(DownloadEvent::Error(format!("حفظ الملف على القرص: {e}")));
                return;
            }
            drop(file);
            if let Some(expected) = total {
                if received != expected {
                    let _ = tx.send(DownloadEvent::Error(format!("اكتمل اتصال التنزيل لكن الحجم غير مكتمل: {received} من {expected} بايت. الملف الجزئي محفوظ للاستكمال.")));
                    return;
                }
            }
            if cancel.load(Ordering::Relaxed) {
                let _ = tx.send(DownloadEvent::Cancelled);
                return;
            }
            if let Err(e) = tokio::fs::rename(&partial, &destination).await {
                let _ = tx.send(DownloadEvent::Error(format!("نقل الملف النهائي: {e}")));
                return;
            }
            let _ = tx.send(DownloadEvent::Finished(destination));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_content_range_correctly() {
        assert_eq!(content_range_start("bytes 1048576-2097151/3145728"), Some(1048576));
        assert_eq!(content_range_total("bytes 1048576-2097151/3145728"), Some(3145728));
        assert_eq!(content_range_total("bytes 0-9/*"), None);
        assert_eq!(content_range_start("invalid"), None);
    }
}
