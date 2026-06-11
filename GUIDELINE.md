# 🤖 AI Agent Guidelines for TrashTalk Project

Dokumen ini berisi instruksi teknis, standar coding, dan konteks arsitektur yang harus diikuti oleh AI Agent yang bekerja pada project **TrashTalk (FileDecider)**.

---

# 1. Project Context

| Item             | Value                                                                                                                                         |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| **Project Name** | TrashTalk (FileDecider)                                                                                                                       |
| **Goal**         | Background OS utility untuk mengelola dan membersihkan digital hoarding di folder Downloads menggunakan mekanisme Ghost Folder (Soft Delete). |
| **Platform**     | Windows & Linux                                                                                                                               |
| **Tech Stack**   | Rust (Backend/CLI) + Tauri (Frontend/GUI) + SQLite (`rusqlite`) + TypeScript                                                                  |

---

# 2. Coding Standards & Constraints

## 🦀 Rust (Backend / CLI)

### Error Handling

**Rules**

* ❌ NEVER gunakan `.unwrap()`
* ❌ NEVER gunakan `.expect()` pada production code
* ✅ Gunakan crate `anyhow` untuk error propagation dan handling

### Locked File Handling

Jika menemukan file yang sedang digunakan (`Permission Denied`):

1. Log error
2. Skip file
3. Lanjutkan proses scanning

**Jangan pernah menyebabkan aplikasi crash.**

---

### Performance

Background worker harus memiliki memory footprint seminimal mungkin.

#### Gunakan `async (tokio)` hanya jika diperlukan

Contoh:

* ✅ Menunggu Friday Scheduler
* ✅ Timer
* ✅ Background sleep

#### Gunakan synchronous API

Untuk directory traversal:

```text id="q5p0xw"
walkdir
```

lebih disukai dibanding async karena:

* ukuran binary lebih kecil,
* implementasi lebih sederhana,
* lebih efisien selama scanning tidak memblok UI.

---

### Cross-Platform Compatibility

Selalu gunakan:

```rust id="s2lfgv"
std::path::Path
std::path::PathBuf
```

**Jangan pernah hardcode separator path**

❌

```text id="mjlwm6"
C:\Downloads\
```

atau

```text id="l2o8kc"
/home/user/
```

Biarkan `PathBuf` menangani separator secara otomatis.

---

# 🖥️ Tauri & Frontend (TypeScript + CSS)

## Strict TypeScript

Semua frontend wajib menggunakan **TypeScript**.

### Rules

* ✅ Prioritaskan type safety
* ❌ `any` dilarang digunakan

Jika benar-benar diperlukan:

```text id="lqg6if"
unknown
```

namun tetap lebih disarankan membuat:

* interface
* type alias
* generic type

untuk seluruh:

* IPC payload
* Rust → JS response
* state object

---

## Framework

Prioritas:

1. Vanilla TypeScript
2. Svelte + TypeScript
3. Preact

### Hindari

* React yang berat
* dependency berlebihan

kecuali diminta secara eksplisit oleh user.

---

## Styling

Semua implementasi styling wajib mengikuti:

```text id="38tpvc"
design.md
```

### Requirements

* Implement Glassmorphism CSS Variables
* Gunakan class CSS
* ❌ Jangan menggunakan inline style

---

## IPC Communication

Gunakan:

```text id="dbefc9"
Tauri invoke()
```

dengan wrapper yang strongly typed.

### Principle

Frontend hanya menangani:

* UI
* Event
* State ringan

Semua pekerjaan berat harus dilakukan di Rust:

* file scanning
* database operation
* categorization
* scheduler

---

# 🗄️ Database (SQLite / rusqlite)

## Schema Updates

Jika melakukan perubahan schema:

* pertahankan backward compatibility,
* atau buat migration sederhana.

---

## Queries

Selalu gunakan:

### Parameterized Query

✅

```sql id="gk9x3u"
SELECT * FROM files
WHERE id = ?
```

atau

```sql id="fbkvjn"
INSERT INTO files(name)
VALUES(?)
```

Hal ini mencegah SQL Injection meskipun aplikasi berjalan secara lokal.

---

# 🌐 Localization (i18n)

## CLI / Rust

Gunakan:

```text id="pvjlwm"
rust-i18n
```

Semua output harus dibungkus dengan:

```rust id="ucb21x"
t!("key")
```

Contoh:

* prompt
* error message
* warning
* status

Tidak boleh hardcoded.

---

## Frontend

Gunakan:

```text id="bj3vk8"
i18next
```

dengan TypeScript typings yang benar.

### Rules

❌ Jangan hardcode:

* English
* Indonesian

langsung di component UI.

Semua string harus berasal dari translation file.

---

# 3. Recommended Directory Architecture

```
trashtalk/
├── src-tauri/                  # 🦀 Rust Backend (Tauri Main + Core Engine)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs             # Tauri application entry point
│   │   ├── cli.rs              # CLI argument parser (Phase 1 MVP)
│   │   ├── scanner.rs          # Directory scanning logic (walkdir)
│   │   ├── ghost.rs            # Ghost Folder operations (move, purge, list)
│   │   ├── category.rs         # Smart Categorization logic
│   │   ├── db.rs               # SQLite database handler
│   │   ├── scheduler.rs        # Friday trigger & fallback logic
│   │   └── i18n/               # Localization modules
│   └── locales/                # JSON translation files (EN & ID)
│
├── src/                        # 🖥️ Frontend GUI
│   ├── index.html              # Main window entry
│   ├── style.css               # design.md implementation
│   ├── main.ts                 # Typed Tauri invoke wrappers
│   ├── types/
│   │   └── index.ts            # Shared TS interfaces matching Rust structs
│   └── components/
│       ├── FridayPrompt.ts
│       ├── GhostManager.ts
│       └── IgnoreSettings.ts
│
├── .trashtalkignore            # Default ignore list
└── README.md
```

---

# 4. Execution Workflow for AI

## 1. Understand Intent

Sebelum menulis kode:

* baca `trashtalk_roadmap.md`
* identifikasi Phase yang sedang dikerjakan

Contoh:

```
Phase 1 → CLI Core
Phase 2 → Smart Categories
Phase 3 → GUI
```

---

## 2. Check Design

Jika membangun UI:

Selalu konsultasikan:

```text id="h7afzg"
design.md
```

untuk memastikan:

* class names
* spacing
* glassmorphism variables
* visual consistency

sesuai spesifikasi project.

---

## 3. Iterative Coding

Bangun satu modul hingga selesai sebelum berpindah ke modul berikutnya.

### Contoh Workflow

```
scanner.rs
    ↓
ghost.rs
    ↓
category.rs
    ↓
db.rs
    ↓
scheduler.rs
```

### Principles

* ✅ Small modules
* ✅ Single Responsibility
* ✅ Reusable functions
* ✅ Easy testing

### Hindari

* fungsi monolitik yang sangat panjang,
* mencampur UI, database, dan business logic dalam satu file,
* implementasi beberapa fitur besar sekaligus tanpa menyelesaikan modul sebelumnya.

---

# Guiding Principles

AI Agent harus selalu memprioritaskan:

* 🦀 Rust-first architecture
* ⚡ Lightweight & performant implementation
* 🛡️ Safe error handling (no `unwrap()` / `expect()`)
* 🌐 Full localization support
* 🖥️ Native-feeling cross-platform experience
* 🎨 Consistency dengan `design.md`
* 📦 Modular, maintainable, dan iterative development
