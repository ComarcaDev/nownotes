// Ejecuta una acción y, si falla, avisa con un toast en lugar de fallar en silencio.
import { showToast } from "./toast.svelte";

export async function safe<T>(action: () => Promise<T>, what = "No se pudo guardar el cambio"): Promise<T | undefined> {
  try {
    return await action();
  } catch (e) {
    console.error(what, e);
    showToast(`${what}. ${String(e)}`.trim(), undefined, 6000);
    return undefined;
  }
}
