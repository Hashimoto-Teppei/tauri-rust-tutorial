use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub updated_at: u64,
}

/// アプリ全体で共有するメモの保存先。tauri の State として manage する
pub struct NoteStore {
    path: PathBuf,
    notes: Mutex<Vec<Note>>,
    /// 最後に発行した id。削除済みの id を再利用しないためにメモリ上で単調増加させる
    last_id: AtomicU64,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

impl NoteStore {
    /// JSON ファイルから読み込む。ファイルがなければ空で始める
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let notes = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        let last_id = notes.iter().map(|n: &Note| n.id).max().unwrap_or(0);
        Ok(Self {
            path,
            notes: Mutex::new(notes),
            last_id: AtomicU64::new(last_id),
        })
    }

    /// 一時ファイルに書いてから rename する（書き込み途中で落ちても元のファイルが壊れない）
    fn save(&self, notes: &[Note]) -> AppResult<()> {
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(notes)?)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    /// コピーを変更 → 保存 → 成功したらメモリに反映。保存失敗時にメモリとディスクがずれない
    fn modify<T>(&self, f: impl FnOnce(&mut Vec<Note>) -> AppResult<T>) -> AppResult<T> {
        let mut notes = self.notes.lock().unwrap();
        let mut next = notes.clone();
        let result = f(&mut next)?;
        self.save(&next)?;
        *notes = next;
        Ok(result)
    }

    pub fn list(&self) -> Vec<Note> {
        let mut notes = self.notes.lock().unwrap().clone();
        notes.sort_by_key(|n| std::cmp::Reverse(n.updated_at)); // 新しい順
        notes
    }

    pub fn get(&self, id: u64) -> AppResult<Note> {
        let notes = self.notes.lock().unwrap();
        notes
            .iter()
            .find(|n| n.id == id)
            .cloned()
            .ok_or(AppError::NotFound(id))
    }

    pub fn create(&self, title: String) -> AppResult<Note> {
        self.modify(|notes| {
            let id = self.last_id.fetch_add(1, Ordering::Relaxed) + 1;
            let note = Note {
                id,
                title,
                body: String::new(),
                updated_at: now_ms(),
            };
            notes.push(note.clone());
            Ok(note)
        })
    }

    pub fn update(&self, id: u64, title: String, body: String) -> AppResult<Note> {
        self.modify(|notes| {
            let note = notes
                .iter_mut()
                .find(|n| n.id == id)
                .ok_or(AppError::NotFound(id))?;
            note.title = title;
            note.body = body;
            note.updated_at = now_ms();
            Ok(note.clone())
        })
    }

    pub fn delete(&self, id: u64) -> AppResult<()> {
        self.modify(|notes| {
            let before = notes.len();
            notes.retain(|n| n.id != id);
            if notes.len() == before {
                return Err(AppError::NotFound(id));
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_roundtrip() {
        let path = std::env::temp_dir().join(format!("notes-test-{}.json", now_ms()));
        let store = NoteStore::load(path.clone()).unwrap();
        let a = store.create("a".into()).unwrap();
        let b = store.create("b".into()).unwrap();
        assert!(b.id > a.id);
        store.update(a.id, "a2".into(), "body".into()).unwrap();
        assert_eq!(store.list()[0].id, a.id); // 更新したものが先頭
        assert!(matches!(store.delete(999), Err(AppError::NotFound(999))));

        // 最新のメモを消しても id は再利用されない
        store.delete(b.id).unwrap();
        assert!(store.create("c".into()).unwrap().id > b.id);

        // ファイルから読み直しても残っている
        let reloaded = NoteStore::load(path.clone()).unwrap();
        assert_eq!(reloaded.get(a.id).unwrap().title, "a2");
        assert_eq!(reloaded.list().len(), 2);
        std::fs::remove_file(path).unwrap();
    }
}
