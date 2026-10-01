import type { UpdateCheck } from "./types";

const SKIPPED_KEY = "bfm.update.skipped";

function skippedVersion(storage: Storage): string | null {
  try {
    return storage.getItem(SKIPPED_KEY);
  } catch {
    return null;
  }
}

/** Remembers one skipped release tag. Newer releases notify again. */
export function rememberSkippedVersion(storage: Storage, tag: string) {
  try {
    storage.setItem(SKIPPED_KEY, tag);
  } catch {
    // Unavailable storage only means the release can be offered again.
  }
}

export function shouldNotifyUpdate(check: UpdateCheck, storage: Storage) {
  return check.status === "available" && skippedVersion(storage) !== check.tag;
}
