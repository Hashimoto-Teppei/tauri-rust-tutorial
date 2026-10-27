use crate::error::{AppError, AppResult};
use crate::notes::{Note, NoteStore};
use serde::Serialize;
use std::time::Duration;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn list_notes(store: State<'_, NoteStore>) -> Vec<Note> {
    store.list()
}

#[tauri::command]
pub fn create_note(app: AppHandle, store: State<'_, NoteStore>, title: String) -> AppResult<Note> {
    let note = store.create(title)?;
    app.emit("notes-changed", ())?;
    Ok(note)
}

#[tauri::command]
pub fn update_note(
    app: AppHandle,
    store: State<'_, NoteStore>,
    id: u64,
    title: String,
    body: String,
) -> AppResult<Note> {
    let note = store.update(id, title, body)?;
    app.emit("notes-changed", ())?;
    Ok(note)
}

#[tauri::command]
pub fn delete_note(app: AppHandle, store: State<'_, NoteStore>, id: u64) -> AppResult<()> {
    store.delete(id)?;
    app.emit("notes-changed", ())?;
    Ok(())
}

/// フロントのダイアログで選ばれたパスへ Markdown として書き出す
#[tauri::command]
pub fn export_note(store: State<'_, NoteStore>, id: u64, path: String) -> AppResult<()> {
    // command は capability の scope 外で動くので、受け取ったパスは自分で検証する
    if !path.ends_with(".md") {
        return Err(AppError::InvalidPath(path));
    }
    let note = store.get(id)?;
    std::fs::write(path, format!("# {}\n\n{}", note.title, note.body))?;
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AnalyzeEvent {
    Progress { done: usize, total: usize },
    Finished { total_chars: usize },
}

/// 全メモの文字数を数える。重い処理のふりをして進捗を Channel で送る
#[tauri::command]
pub async fn analyze_notes(
    store: State<'_, NoteStore>,
    on_event: Channel<AnalyzeEvent>,
) -> AppResult<()> {
    let notes = store.list();
    let total = notes.len();
    let mut total_chars = 0;
    for (i, note) in notes.iter().enumerate() {
        total_chars += note.body.chars().count();
        tokio::time::sleep(Duration::from_millis(300)).await;
        on_event.send(AnalyzeEvent::Progress { done: i + 1, total })?;
    }
    on_event.send(AnalyzeEvent::Finished { total_chars })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_event_json_matches_ts_type() {
        let json = serde_json::to_string(&AnalyzeEvent::Finished { total_chars: 3 }).unwrap();
        assert_eq!(json, r#"{"kind":"finished","totalChars":3}"#);
    }
}
