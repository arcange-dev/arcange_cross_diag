import { isTauri } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";

export type AppUpdateMetadata = {
  version: string;
  currentVersion: string;
  date: string | null;
  notes: string;
};

export type AppUpdateProgress = {
  downloadedBytes: number;
  contentLength: number | null;
};

const updaterEnabled = import.meta.env.VITE_UPDATER_ENABLED === "true";
let pendingUpdate: NonNullable<Awaited<ReturnType<typeof check>>> | null = null;

export function isAppUpdaterEnabled(): boolean {
  return updaterEnabled && isTauri();
}

export async function checkForApplicationUpdate(): Promise<AppUpdateMetadata | null> {
  if (!isTauri()) {
    throw new Error("Application updates are available only in the installed CrossDiag desktop application.");
  }
  if (!updaterEnabled) {
    throw new Error("The signed updater is not configured in this development build. It becomes available in signed release builds.");
  }

  if (pendingUpdate) {
    await pendingUpdate.close();
    pendingUpdate = null;
  }
  const update = await check({ timeout: 30_000 });
  pendingUpdate = update;
  if (!update) return null;

  return {
    version: update.version,
    currentVersion: update.currentVersion,
    date: update.date ? String(update.date) : null,
    notes: update.body ?? "No release notes were supplied.",
  };
}

export async function installApplicationUpdate(
  onProgress: (progress: AppUpdateProgress) => void,
): Promise<void> {
  if (!pendingUpdate) {
    throw new Error("There is no verified pending update. Check for updates again first.");
  }

  let downloadedBytes = 0;
  let contentLength: number | null = null;
  await pendingUpdate.downloadAndInstall((event) => {
    if (event.event === "Started") {
      contentLength = event.data.contentLength ?? null;
      downloadedBytes = 0;
      onProgress({ downloadedBytes, contentLength });
    } else if (event.event === "Progress") {
      downloadedBytes += event.data.chunkLength;
      onProgress({ downloadedBytes, contentLength });
    } else if (event.event === "Finished") {
      onProgress({ downloadedBytes, contentLength });
    }
  });

  pendingUpdate = null;
  await relaunch();
}
