# 🎨 TrashTalk UI/UX Design System & Guidelines

---

# 1. Design Philosophy

## Minimalist & Non-Intrusive

TrashTalk adalah aplikasi utilitas, sehingga antarmuka harus:

* bersih,
* ringan,
* tidak mengganggu workflow pengguna,

dengan memanfaatkan **blur** dan **translucency** daripada border tebal atau blok warna solid.

---

## Glassmorphism Integration

Menggunakan background semi-transparan dengan efek `background-blur` agar menyatu dengan lingkungan desktop OS.

**Inspirasi:**

* Windows 11 Mica Style
* Linux Blur Compositor

---

## Action-Oriented

Fokus utama aplikasi adalah **Friday Cleanup Prompt**.

Button harus menjelaskan aksi yang akan dilakukan.

✅ Contoh yang baik:

```text
Hapus 1.2GB
```

❌ Hindari:

```text
OK
```

---

## Safe & Forgiving

Semua aksi destruktif harus:

* diberi warna merah,
* memiliki warning yang jelas,
* atau melalui mekanisme **Ghost Folder (Soft Delete)** sebelum benar-benar dihapus.

---

# 2. Global Theming

## Theme Base: "Frosted Midnight"

Default menggunakan tema gelap bergaya glassmorphism untuk:

* mengurangi eye strain,
* memberikan tampilan modern,
* terasa menyatu dengan sistem operasi.

---

## Color Palette & Effects (CSS Variables)

```css
:root {
  --bg-glass: rgba(24, 24, 37, 0.65);
  --backdrop-blur: blur(16px) saturate(180%);
  --border-glass: 1px solid rgba(255, 255, 255, 0.08);

  --bg-surface: rgba(30, 30, 46, 0.75);

  --text-primary: #cdd6f4;
  --text-secondary: #a6adc8;

  --accent-primary: rgba(137, 180, 250, 0.9);
  --accent-glow: 0 0 12px rgba(137, 180, 250, 0.4);

  --action-destructive: rgba(243, 139, 168, 0.9);
  --action-warning: rgba(249, 226, 175, 0.9);
}
```

### Variable Reference

| Variable               | Purpose                     |
| ---------------------- | --------------------------- |
| `--bg-glass`           | Deep translucent background |
| `--backdrop-blur`      | Core glass effect           |
| `--border-glass`       | Subtle glass outline        |
| `--bg-surface`         | Cards & submenus            |
| `--text-primary`       | Main text                   |
| `--text-secondary`     | Secondary/path text         |
| `--accent-primary`     | Safe primary actions        |
| `--accent-glow`        | Primary button glow         |
| `--action-destructive` | Hard delete actions         |
| `--action-warning`     | Warning states              |

---

## Typography

### UI Text

Digunakan untuk:

* Headings
* Buttons
* Prompts

**Preferred Fonts**

```
Inter
Roboto
System Sans-serif
```

Contoh:

* Segoe UI (Windows)
* Cantarell (Linux)
* Ubuntu (Linux)

### Font Weight

| Element | Weight    |
| ------- | --------- |
| Heading | Semi Bold |
| Button  | Medium    |

### Readability

Pastikan kontras tinggi terhadap background transparan.

---

## Technical Text

Digunakan untuk:

* File path
* Extensions
* Code snippets

**Preferred Fonts**

```
Fira Code
JetBrains Mono
System Monospace
```

---

# 3. Core Layouts & Interfaces

# A. Friday Prompt Window (Popup/Dialog)

## Visual Style

Floating glass card menggunakan:

* `--bg-glass`
* `--backdrop-blur`
* `--border-glass`

---

## Size

```
Width : ±450px
Height: Auto
Resizable: No
```

---

## Position

* Center Screen
* atau Bottom Right (System Notification Style)

---

## Structure

### Header

Bold hook.

```
Hey, ada [Size] sampah di Downloads.
```

---

### Body

Brief context.

```
File ini udah nggak disentuh lebih dari 2 minggu.
```

---

### Action Stack (Vertical)

```
┌─────────────────────────────────────────┐
│ Hapus Semua Installer (Ghost Folder)   │
├─────────────────────────────────────────┤
│ Pindah Dokumen ke Archive              │
├─────────────────────────────────────────┤
│ Pilih Ekstensi / Rentang Waktu...      │
├─────────────────────────────────────────┤
│ Nanti Aja (Ingatkan Minggu Depan)      │
└─────────────────────────────────────────┘
```

### Button Styles

| Button    | Style           |
| --------- | --------------- |
| Primary   | Accent + Glow   |
| Secondary | Ghost / Outline |
| Tertiary  | Text Only       |

---

# B. Ghost Folder Manager (Main App Window)

## Visual Style

Full glassmorphic window.

---

## Size

```
800 × 600
Resizable: Yes
```

---

## Structure

### Sidebar / Top Navigation

Menggunakan `--bg-surface` agar sedikit berbeda dari area utama.

Tabs:

```
Ghost Folder
Ignore List
```

---

### Main Content (List View)

Table/Grid:

| File Name | Category | Original Path | Days in Ghost |
| --------- | -------- | ------------- | ------------- |

Hover item:

```css
background: rgba(255,255,255,0.05);
```

Setiap baris memiliki tombol kecil:

```
[ Restore ]
```

---

### Footer / Bottom Bar

Dipisahkan dengan:

```
--border-glass
```

#### Storage Indicator

```
Ghost Folder uses 2.1GB / 5GB
```

#### Danger Action

```
[ Empty Ghost Folder Now ]
```

Aksi ini harus memunculkan confirmation modal.

---

# C. Dynamic Sub-Menus

## Extensions & Time Range

Ditampilkan sebagai checklist di dalam modal overlay.

---

## Overlay Behavior

Background utama:

* sedikit redup (*dim*)

Modal:

* solid atau sangat opaque
* memastikan checkbox dan form mudah dibaca

---

## Example

```
[x] Installer (.exe, .msi)

[ ] Video Berat (.mp4, .mkv)

[ ] Dokumen (.pdf, .docx)

[ ] Arsip (.zip, .rar)
```

---

# 4. Component States

## Hover

Button akan:

* sedikit lebih terang,
* atau meningkatkan glow intensity.

---

## Active / Click

Animasi kecil:

```css
transform: scale(0.98);
```

---

## Disabled

```css
opacity: 50%;
cursor: not-allowed;
```

Contoh:

* Ghost Folder sudah kosong
* tombol "Empty Ghost Folder" menjadi disabled

---

## Transitions

Gunakan animasi cepat dan responsif:

```css
transition:
  opacity 150ms ease-in-out,
  transform 150ms ease-in-out;
```

### Performance Note

Efek `background-blur` sebaiknya **statis** untuk menjaga performa dan menghindari render yang berat.

---

# Design Principles Summary

* ✨ Minimalist & Non-Intrusive
* 🧊 Glassmorphism ("Frosted Midnight")
* 🎯 Action-Oriented Interface
* 🛡️ Safe & Forgiving Destructive Actions
* 🌙 Dark Theme First
* ⚡ Fast, Snappy, and Lightweight Animations
* 🖥️ Native Feeling Across Windows & Linux
