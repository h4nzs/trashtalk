export interface ScanSummary {
  total_stale: number;
  total_size_bytes: number;
  breakdown: Record<string, number>;
}

export interface GhostFile {
  id: number;
  name: string;
  original_path: string;
  ghost_path: string;
}

export type FileCategoryLabel = 
  | "Stale Installers"
  | "Social Media Media"
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
