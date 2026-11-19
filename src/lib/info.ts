// Datos fijos de la app (versión, carpeta, días de papelera…): se piden una sola vez.
import { api, type Info } from "./api";

let cached: Promise<Info> | null = null;

export function getInfo(): Promise<Info> {
  cached ??= api.info().catch((e) => {
    cached = null; // permite reintentar si falló
    throw e;
  });
  return cached;
}
