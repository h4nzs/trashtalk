# 🗑️ TrashTalk (a.k.a FileDecider)

## Project Roadmap & Architecture

---

# 1. Visi & Konsep Utama

### Masalah

Folder **Downloads** yang penuh sesak karena *digital hoarding* (takut menghapus file karena "siapa tahu butuh").

### Solusi

Sebuah background tool cerdas yang secara proaktif:

* menganalisis umur file,
* mengkategorikannya dengan bahasa manusiawi,
* dan secara interaktif meminta user membersihkannya setiap Jumat sore,

dengan jaring pengaman berupa **Ghost Folder**.

---

# 2. Spesifikasi Stack Teknologi (Cross-Platform: Windows & Linux)

## Core Engine

**Rust 🦀**

Dipilih karena:

* memori rendah,
* eksekusi cepat,
* aman (memory safety).

### Dependencies Utama

* `walkdir` — scanning folder
* `filetime` — manipulasi waktu file
* `chrono` — kalkulasi umur file

## GUI / User Interface

* **Tauri** (HTML/CSS/JS)

## Database / State Management

* **SQLite** (via `rusqlite`)

## Localization (i18n)

* `rust-i18n` (Core/CLI)
* `i18next` (UI Tauri)

Mendukung:

* 🇺🇸 English (EN)
* 🇮🇩 Indonesia (ID)

## Background Scheduler

### Windows

* Windows Task Scheduler
* Tray Icon async loop

### Linux

* systemd user timers

---

# 3. Arsitektur Logika Inti

## A. File Aging Logic & Scanning

Sistem operasi modern sering memanipulasi **Last Accessed Date (`atime`)**, sehingga TrashTalk menggunakan state tracking sendiri.

### Scanning Berkala

* Mencatat file baru yang masuk ke Downloads
* Scan dilakukan secara rekursif termasuk seluruh subfolder

### Ignore List (Pengecualian)

Folder/file yang ada di `.trashtalkignore` akan dilewati (mirip `.gitignore`).

Disediakan fitur interaktif:

* Settings Menu di UI
* Command di CLI

agar user dapat:

* memilih folder melalui file picker,
* atau mengetik path secara manual.

### Tracking Aktivitas

Menggunakan **Modification Time (`mtime`)**.

Contoh:

> Jika sebuah `.exe` diunduh 14 hari lalu dan tidak mengalami perubahan `mtime`, maka file akan ditandai sebagai **"Basi"**.

### Locked File Handling

Jika file sedang digunakan/dikunci oleh OS:

* scanner melakukan **skip**
* tanpa menyebabkan crash (*graceful error handling*)

---

## B. Smart Categorization (Heuristik Nama + Ekstensi)

Mengubah pola nama file dan ekstensi menjadi kategori yang mudah dipahami manusia.

> Nama kategori akan dilokalisasi ke EN/ID.

### 1. Installer Basi / Stale Installers

**Ekstensi**

```text
.exe
.msi
.AppImage
.deb
.dmg
.apk
```

---

### 2. Jejak Sosmed / Social Media Media

**Pola Nama**

```text
WhatsApp Image.*
Screenshot_.*
IMG_.*
Snapchat-.*
```

**Ekstensi**

```text
.png
.jpg
.jpeg
.webp
.heic
```

---

### 3. Video & Rekaman / Heavy Videos

**Pola Nama**

```text
Screen Recording.*
WhatsApp Video.*
Zoom_.*
```

**Ekstensi**

```text
.mp4
.mkv
.mov
.avi
```

---

### 4. Dokumen Kerja / Work Documents

**Pola Nama**

```text
.*Draft.*
.*Final.*
.*Revisi.*
.*Invoice.*
```

**Ekstensi**

```text
.pdf
.docx
.xlsx
.pptx
.csv
```

---

### 5. Arsip Mentahan / Archives

**Ekstensi**

```text
.zip
.rar
.7z
.tar.gz
```

---

### 6. Sisa File Desain / Design Files

**Ekstensi**

```text
.psd
.ai
.svg
.cdr
.fig
```

---

### 7. Meme & Hiburan Singkat / Memes & Gifs

**Ekstensi**

```text
.gif
.webm
```

---

## C. The Friday Afternoon Trigger (Interaktif)

Alih-alih menghapus diam-diam, sistem akan menampilkan **Window Prompt** setiap:

* **Hari:** Jumat
* **Waktu:** 16:00 waktu setempat

### Mekanisme Fallback (Missed Schedule)

Jika komputer mati pada jadwal tersebut:

* sistem mendeteksi jadwal yang terlewat,
* lalu menampilkan prompt pada boot/login berikutnya.

### Prompt Contoh (ID)

> "Hey, ada 1.2GB sampah di Downloads. Mau diapain?"

### Action Buttons

* `[Hapus Semua Installer]`
* `[Pindahin Dokumen ke 'Archive']`
* `[Pilih & Hapus Berdasarkan Ekstensi]`
* `[Pilih & Hapus Berdasarkan Rentang Waktu]`
* `[Ingatkan 1 Minggu Lagi]`

Semua teks di atas di-load secara dinamis dari file JSON i18n sesuai bahasa sistem atau pilihan user.

---

## D. The "Ghost Folder" (Soft Delete)

Jaring pengaman psikologis agar user tidak takut menekan tombol **Hapus**.

### Soft Delete

File dipindahkan (`mv`) ke:

```text
~/.trashtalk_ghost
```

Status file dicatat di database lokal.

### Auto-Purge (7 Hari)

Background worker akan menghapus permanen file yang berumur lebih dari **7 hari** di Ghost Folder.

### Storage Cap (Limit Kapasitas)

Jika Ghost Folder melebihi batas (misalnya **5GB**):

* sistem memprioritaskan hard delete file tertua,
* sehingga disk tidak penuh.

### Ghost Management

Fitur interaktif melalui CLI maupun UI:

* List
* Restore
* Empty Trash (dengan Warning Prompt Yes/No)

---

# 4. Roadmap Pengembangan (Phases)

## Phase 1 — MVP: CLI Core Engine 🛠️

**Fokus:** Membangun tulang punggung logika di terminal.

* [ ] Inisiasi project Rust (`cargo new`)
* [ ] Setup sistem lokalisasi dasar menggunakan `rust-i18n`

  * `locales/en.json`
  * `locales/id.json`
* [ ] Pastikan output CLI menggunakan variabel bahasa (tidak hardcoded)
* [ ] Buat scanner direktori (`walkdir`) untuk membaca `~/Downloads` beserta subfolder
* [ ] Terapkan graceful error handling untuk `Permission Denied`
* [ ] Implementasi filter umur file (`filetime` + `chrono`)
* [ ] Buat `move_to_ghost_folder()` (Soft Delete)
* [ ] Buat `purge_ghost_folder()` (Hard Delete > 7 hari + batas kapasitas)
* [ ] Implementasi command CLI:

  * `ghost-list`
  * `restore`
  * `ghost-empty`
* [ ] Implementasi command Ignore List:

  * `ignore add`
  * `ignore list`
  * `ignore remove`

---

## Phase 2 — State Tracking & Smart Categories 🧠

**Fokus:** Membuat sistem pintar agar tidak salah menghapus file.

* [ ] Setup `rusqlite` untuk menyimpan log file:

  * Nama
  * Tanggal Download
  * Status
  * Original Path
* [ ] Implementasi pembacaan `.trashtalkignore`
* [ ] Implementasi regex & heuristik Smart Categorization
* [ ] Mapping nama kategori berdasarkan file i18n
* [ ] Buat mode `dry-run` di CLI (tabel rangkuman sampah digital)

---

## Phase 3 — GUI & The Friday Scheduler 🖥️

**Fokus:** Interaksi user yang ramah dan dinamis.

* [ ] Desain UI minimalis utama (Tauri)
* [ ] Implementasi `i18next` pada frontend Tauri
* [ ] Menyesuaikan bahasa dengan OS pengguna
* [ ] Fallback ke English jika `id-ID` tidak tersedia
* [ ] Bangun submenu:

  * Berdasarkan Ekstensi
  * Berdasarkan Rentang Waktu
* [ ] Bangun UI:

  * Ghost Folder Manager
  * Ignore List Manager
* [ ] Sambungkan GUI dengan Core Engine Rust
* [ ] Implementasikan Scheduler:

  * Windows Task Scheduler
  * systemd Linux
  * Fallback Scheduler

---

## Phase 4 — Testing & Packaging 📦

**Fokus:** Siap digunakan oleh user awam.

* [ ] Uji coba cross-platform

  * Windows 10
  * Windows 11
  * Linux distro umum
* [ ] Uji pergantian bahasa (EN/ID)

  * UI
  * Terminal
* [ ] Pastikan prompt & warning berubah sesuai locale
* [ ] Uji integrasi:

  * Ghost Folder Management
  * Ignore List
* [ ] Buat installer siap pakai:

  * `.msi`
  * `.deb`

---

# North Star

Dokumen ini merupakan **North Star** project **TrashTalk (FileDecider)**.

Iterasi, penyempurnaan, dan perubahan dipersilakan seiring berkembangnya proses desain dan implementasi.
