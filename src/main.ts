import { invoke } from "@tauri-apps/api/core";
import i18n from "./i18n";
import { ScanSummary, GhostFile, AppSchedule } from "./types";

document.addEventListener("DOMContentLoaded", async () => {
  // Translate static UI elements
  function translateUI() {
    document.querySelectorAll("[data-i18n]").forEach(el => {
      const key = el.getAttribute("data-i18n");
      if (key) {
        el.textContent = i18n.t(key);
      }
    });
  }

  translateUI();

  // --- Navigation & Tabs ---
  const navItems = document.querySelectorAll(".nav-item");
  const tabs = document.querySelectorAll(".tab-content");

  navItems.forEach((item) => {
    item.addEventListener("click", () => {
      const targetTab = item.getAttribute("data-tab");
      navItems.forEach(ni => ni.classList.remove("active"));
      tabs.forEach(t => t.classList.remove("active"));
      item.classList.add("active");
      document.getElementById(`tab-${targetTab}`)?.classList.add("active");

      if (targetTab === "ghost") loadGhostFiles();
      if (targetTab === "settings") loadSettings();
      if (targetTab === "prompt") runInitialScan();
    });
  });

  // --- Elements ---
  const promptTitle = document.getElementById("prompt-title");
  const promptBody = document.getElementById("prompt-body");
  const detectedFilesList = document.getElementById("detected-files-list");
  const btnPurgeAll = document.getElementById("btn-purge-all");
  const btnPickCustom = document.getElementById("btn-pick-custom");

  const ghostListBody = document.getElementById("ghost-list-body");
  const btnEmptyGhost = document.getElementById("btn-empty-ghost");

  const scheduleDay = document.getElementById("schedule-day") as HTMLSelectElement;
  const scheduleTime = document.getElementById("schedule-time") as HTMLInputElement;
  const btnSaveSchedule = document.getElementById("btn-save-schedule");
  const ignoreListItems = document.getElementById("ignore-list-items");
  const ignoreInput = document.getElementById("ignore-input") as HTMLInputElement;
  const btnAddIgnore = document.getElementById("btn-add-ignore");

  const customModal = document.getElementById("custom-modal");
  const modalCancel = document.getElementById("modal-cancel");
  const modalApply = document.getElementById("modal-apply");

  const resultModal = document.getElementById("result-modal");
  const resultHeader = document.getElementById("result-header");
  const resultBody = document.getElementById("result-body");
  const btnResultOk = document.getElementById("btn-result-ok");

  // --- Logic ---

  async function runInitialScan() {
    try {
      const summary = await invoke<ScanSummary>("run_scan");
      const sizeFormatted = formatBytes(summary.total_size_bytes);
      
      if (promptTitle) promptTitle.innerText = i18n.t("prompt_title", { size: sizeFormatted });
      if (promptBody) promptBody.innerText = i18n.t("prompt_body");
      
      if (detectedFilesList) {
        detectedFilesList.innerHTML = summary.files.map(file => `
          <li>
            <span class="file-name-dim" title="${file.path}">${file.name}</span>
            <span class="file-size-dim">${formatBytes(file.size_bytes)}</span>
          </li>
        `).join("");
      }
    } catch (error) { 
      // Handle error silently or show minimal feedback
    }
  }

  async function loadGhostFiles() {
    if (!ghostListBody) return;
    try {
      const files = await invoke<GhostFile[]>("get_ghost_files");
      if (files.length === 0) {
        ghostListBody.innerHTML = '<tr><td colspan="3" class="text-center">No files in Ghost Folder.</td></tr>';
        return;
      }
      ghostListBody.innerHTML = files.map(file => `
        <tr>
          <td><span class="monospace">${file.name}</span></td>
          <td title="${file.original_path}"><small>${truncate(file.original_path, 40)}</small></td>
          <td>
            <button class="btn-secondary btn-small" onclick="window.restoreFile(${file.id}, '${file.ghost_path.replace(/\\/g, '/')}', '${file.original_path.replace(/\\/g, '/')}')">
              Restore
            </button>
          </td>
        </tr>
      `).join("");
    } catch (error) {
      // Handle error
    }
  }

  async function loadSettings() {
    try {
      const schedule = await invoke<AppSchedule>("get_schedule");
      if (scheduleDay) scheduleDay.value = schedule.day;
      if (scheduleTime) scheduleTime.value = schedule.time;
    } catch (error) { }
    loadIgnoreList();
  }

  async function loadIgnoreList() {
    if (!ignoreListItems) return;
    try {
      const list = await invoke<string[]>("get_ignore_list");
      ignoreListItems.innerHTML = list.map(item => `
        <li class="ignore-item">
          <span class="monospace">${item}</span>
          <button class="btn-remove-small" onclick="window.removeIgnore('${item}')">Remove</button>
        </li>
      `).join("");
    } catch (error) { }
  }

  function showResult(count: number) {
    if (resultHeader) resultHeader.textContent = i18n.t("cleanup_complete");
    if (resultBody) resultBody.textContent = i18n.t("cleanup_result_body", { count });
    resultModal?.classList.add("active");
  }

  // Window-exposed handlers (Global scope for inline onclick)
  (window as any).restoreFile = async (id: number, ghost: string, original: string) => {
    try {
      await invoke("restore_ghost_file", { id, ghostPath: ghost, originalPath: original });
      loadGhostFiles();
    } catch (error) { 
      alert(error); 
    }
  };

  (window as any).removeIgnore = async (path: string) => {
    try {
      const currentList = await invoke<string[]>("get_ignore_list");
      const newList = currentList.filter(i => i !== path);
      await invoke("save_ignore_list", { list: newList });
      loadIgnoreList();
    } catch (error) { }
  };

  // Event Listeners
  btnPurgeAll?.addEventListener("click", async () => {
    try {
      const count = await invoke<number>("run_purge");
      showResult(count);
      runInitialScan();
    } catch (error) { 
      alert(error); 
    }
  });

  btnEmptyGhost?.addEventListener("click", async () => {
    if (confirm("Delete everything in Ghost Folder?")) {
      try { 
        await invoke("empty_ghost"); 
        loadGhostFiles(); 
      } catch (error) { 
        alert(error); 
      }
    }
  });

  btnSaveSchedule?.addEventListener("click", async () => {
    const schedule: AppSchedule = { day: scheduleDay.value, time: scheduleTime.value };
    try {
      await invoke("save_schedule", { schedule });
      alert("Schedule saved!");
    } catch (error) { 
      alert(error); 
    }
  });

  btnAddIgnore?.addEventListener("click", async () => {
    const val = ignoreInput?.value.trim();
    if (!val) return;
    try {
      const currentList = await invoke<string[]>("get_ignore_list");
      if (!currentList.includes(val)) {
        currentList.push(val);
        await invoke("save_ignore_list", { list: currentList });
        ignoreInput.value = "";
        loadIgnoreList();
      }
    } catch (error) { }
  });

  btnPickCustom?.addEventListener("click", () => customModal?.classList.add("active"));
  modalCancel?.addEventListener("click", () => customModal?.classList.remove("active"));
  
  modalApply?.addEventListener("click", async () => {
    const selectedCategories = Array.from(document.querySelectorAll("#extensions-list input[type='checkbox']:checked"))
      .map(el => (el as HTMLInputElement).value);
    const timeRange = (document.getElementById("purge-time-range") as HTMLSelectElement).value;

    if (selectedCategories.length === 0) {
      alert("Please select at least one category.");
      return;
    }

    try {
      const timeRangeDays = parseInt(timeRange, 10);
      const count = await invoke<number>("run_custom_purge", { timeRangeDays, categories: selectedCategories });
      
      customModal?.classList.remove("active");
      showResult(count);
      runInitialScan();
    } catch (error) {
      alert(error);
    }
  });

  btnResultOk?.addEventListener("click", () => resultModal?.classList.remove("active"));

  function formatBytes(bytes: number, decimals = 2) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(decimals)) + ' ' + sizes[i];
  }

  function truncate(s: string, max: number) {
    return s.length > max ? s.substring(0, max - 3) + "..." : s;
  }

  runInitialScan();
});
