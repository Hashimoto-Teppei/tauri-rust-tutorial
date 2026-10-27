// Rust の command を型付きで呼ぶ薄いラッパー（Electron の preload + contextBridge の役割）
import { Channel, invoke } from "@tauri-apps/api/core";

export type Note = {
  id: number;
  title: string;
  body: string;
  updatedAt: number;
};

export type AnalyzeEvent =
  | { kind: "progress"; done: number; total: number }
  | { kind: "finished"; totalChars: number };

export const listNotes = () => invoke<Note[]>("list_notes");
export const createNote = (title: string) => invoke<Note>("create_note", { title });
export const updateNote = (id: number, title: string, body: string) =>
  invoke<Note>("update_note", { id, title, body });
export const deleteNote = (id: number) => invoke<void>("delete_note", { id });
export const exportNote = (id: number, path: string) => invoke<void>("export_note", { id, path });

export function analyzeNotes(onEvent: (e: AnalyzeEvent) => void) {
  const channel = new Channel<AnalyzeEvent>();
  channel.onmessage = onEvent;
  return invoke<void>("analyze_notes", { onEvent: channel });
}
