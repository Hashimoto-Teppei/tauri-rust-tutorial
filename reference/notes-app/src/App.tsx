import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import * as api from "./api";
import type { Note } from "./api";
import "./App.css";

function App() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selected, setSelected] = useState<Note | null>(null);
  const [status, setStatus] = useState("");

  // invoke の reject（Rust の AppError の Display 文字列）を画面に出す
  const run = (action: () => Promise<unknown>) =>
    action().catch((e) => setStatus(`エラー: ${e}`));

  async function reload() {
    const list = await api.listNotes();
    setNotes(list);
    // 別ウィンドウ等で削除されたメモを開いたままにしない
    setSelected((s) => (s && list.some((n) => n.id === s.id) ? s : null));
  }

  async function addNote() {
    setSelected(await api.createNote("無題のメモ"));
  }

  // 第8章: Rust 側の emit を購読。アンマウント時に解除する
  useEffect(() => {
    run(reload);
    const unlisteners = [
      listen("notes-changed", () => run(reload)),
      listen("menu-new-note", () => run(addNote)),
    ];
    return () => {
      unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  async function saveNote() {
    if (!selected) return;
    // 戻り値で selected を置き換えない（保存中に入力した内容を消さないため）
    await api.updateNote(selected.id, selected.title, selected.body);
    setStatus("保存しました");
  }

  async function removeNote() {
    if (!selected) return;
    await api.deleteNote(selected.id);
    setSelected(null);
  }

  // 第9章: dialog プラグインで保存先を選び、書き込みは Rust の command で行う
  async function exportNote() {
    if (!selected) return;
    const path = await save({
      defaultPath: `${selected.title}.md`,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (!path) return; // キャンセル
    await api.exportNote(selected.id, path);
    setStatus(`書き出しました: ${path}`);
  }

  // 第8章: Channel で進捗を受け取る
  async function analyze() {
    await api.analyzeNotes((e) => {
      if (e.kind === "progress") setStatus(`解析中… ${e.done}/${e.total}`);
      else setStatus(`合計 ${e.totalChars} 文字`);
    });
  }

  return (
    <main className="layout">
      <aside className="sidebar">
        <button onClick={() => run(addNote)}>＋ 新規</button>
        <button onClick={() => run(analyze)}>文字数を解析</button>
        <ul>
          {notes.map((n) => (
            <li
              key={n.id}
              className={n.id === selected?.id ? "active" : ""}
              onClick={() => setSelected(n)}
            >
              {n.title}
            </li>
          ))}
        </ul>
      </aside>

      <section className="editor">
        {selected ? (
          <>
            <input
              value={selected.title}
              onChange={(e) => setSelected({ ...selected, title: e.target.value })}
            />
            <textarea
              value={selected.body}
              onChange={(e) => setSelected({ ...selected, body: e.target.value })}
            />
            <div className="actions">
              <button onClick={() => run(saveNote)}>保存</button>
              <button onClick={() => run(exportNote)}>書き出し</button>
              <button onClick={() => run(removeNote)}>削除</button>
            </div>
          </>
        ) : (
          <p>メモを選択してください</p>
        )}
        <p className="status">{status}</p>
      </section>
    </main>
  );
}

export default App;
