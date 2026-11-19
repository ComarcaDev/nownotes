// Confirmación nativa para acciones destructivas o difíciles de deshacer.
import { ask } from "@tauri-apps/plugin-dialog";

export function confirmDanger(title: string, message: string, okLabel: string, cancelLabel = "Cancelar"): Promise<boolean> {
  return ask(message, { title, kind: "warning", okLabel, cancelLabel });
}
