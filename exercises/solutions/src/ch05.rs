//! 第5章: serde / スレッド共有 (Arc<Mutex>) / async

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// フロントエンド (TS) と JSON でやり取りする想定の型。
/// TS 側は camelCase なので rename_all で合わせる
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: u32,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub is_pinned: bool,
}

pub fn to_json(note: &Note) -> Result<String, serde_json::Error> {
    serde_json::to_string(note)
}

pub fn from_json(s: &str) -> Result<Note, serde_json::Error> {
    serde_json::from_str(s)
}

/// n 個のスレッドからそれぞれ per_thread 回カウンタを +1 して最終値を返す
pub fn parallel_count(n: usize, per_thread: usize) -> usize {
    let counter = Arc::new(Mutex::new(0));
    let handles: Vec<_> = (0..n)
        .map(|_| {
            let counter = Arc::clone(&counter);
            std::thread::spawn(move || {
                for _ in 0..per_thread {
                    *counter.lock().unwrap() += 1;
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    *counter.lock().unwrap()
}

/// 重い処理のふり: ms 待ってから id * 10 を返す
pub async fn fake_fetch(id: u32, ms: u64) -> u32 {
    tokio::time::sleep(Duration::from_millis(ms)).await;
    id * 10
}

/// fake_fetch(1, 100), fake_fetch(2, 100), fake_fetch(3, 100) を「並行に」実行して合計（Promise.all 相当）
pub async fn fetch_three() -> u32 {
    let (a, b, c) = tokio::join!(fake_fetch(1, 100), fake_fetch(2, 100), fake_fetch(3, 100));
    a + b + c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_roundtrip() {
        let note = Note { id: 1, title: "買い物".into(), tags: vec!["todo".into()], is_pinned: true };
        let json = to_json(&note).unwrap();
        assert!(json.contains("\"isPinned\":true"));
        assert_eq!(from_json(&json).unwrap(), note);
    }

    #[test]
    fn test_json_default_tags() {
        let note = from_json(r#"{"id":2,"title":"x","isPinned":false}"#).unwrap();
        assert!(note.tags.is_empty());
        assert!(from_json(r#"{"id":"oops"}"#).is_err());
    }

    #[test]
    fn test_parallel_count() {
        assert_eq!(parallel_count(8, 1000), 8000);
    }

    #[tokio::test]
    async fn test_fetch_three() {
        let start = std::time::Instant::now();
        assert_eq!(fetch_three().await, 60);
        // 直列なら 300ms かかる。並行なら ~100ms
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(100), "fake_fetch を指定どおり呼んでいない");
        assert!(elapsed < Duration::from_millis(250), "並行に実行されていない");
    }
}
