// Comprobación de nuevas versiones en GitHub (sin descargar nada: solo se avisa).

import { DAY_MS } from "./dates";

export const REPO = "ComarcaDev/nownotes";
const LAST_CHECK_KEY = "nownotes.lastUpdateCheck";

export interface UpdateInfo { version: string; url: string }

/** ¿`a` es una versión más nueva que `b`? (semver simple "x.y.z") */
export function isNewer(a: string, b: string): boolean {
  const pa = a.replace(/^v/, "").split(/[.-]/).map((n) => parseInt(n, 10) || 0);
  const pb = b.replace(/^v/, "").split(/[.-]/).map((n) => parseInt(n, 10) || 0);
  for (let i = 0; i < 3; i++) {
    if ((pa[i] ?? 0) !== (pb[i] ?? 0)) return (pa[i] ?? 0) > (pb[i] ?? 0);
  }
  return false;
}

/** Devuelve la versión nueva si la hay; null si estás al día. Lanza si no hay conexión. */
export async function checkForUpdate(current: string): Promise<UpdateInfo | null> {
  const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers: { Accept: "application/vnd.github+json" },
  });
  // 404: el repositorio aún no tiene versiones publicadas → no hay nada más nuevo.
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`GitHub respondió ${res.status}`);
  const data = (await res.json()) as { tag_name?: string; html_url?: string };
  const version = String(data.tag_name ?? "").replace(/^v/, "");
  try { localStorage.setItem(LAST_CHECK_KEY, String(Date.now())); } catch { /* sin almacenamiento */ }
  return version && isNewer(version, current)
    ? { version, url: data.html_url ?? `https://github.com/${REPO}/releases/latest` }
    : null;
}

/** ¿Toca comprobar hoy? (como mucho una vez al día) */
export function dueForCheck(): boolean {
  try {
    const last = Number(localStorage.getItem(LAST_CHECK_KEY) || 0);
    return Date.now() - last > DAY_MS;
  } catch {
    return true;
  }
}
