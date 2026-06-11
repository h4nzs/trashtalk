export interface StaleFile {
  name: string;
  path: string;
  size_bytes: number;
  category: string;
}

export interface ScanSummary {
  total_stale: number;
  total_size_bytes: number;
  breakdown: Record<string, number>;
  files: StaleFile[];
}

export interface GhostFile {
  id: number;
  name: string;
  original_path: string;
  ghost_path: string;
}

export type FileCategoryLabel = 
  | "Stale Installers"
  | "Social Media"
  | "Heavy Videos"
  | "Work Documents"
  | "Archives"
  | "Design Files"
  | "Memes & Gifs"
  | "Others";

export interface AppSchedule {
  day: string;
  time: string;
}
