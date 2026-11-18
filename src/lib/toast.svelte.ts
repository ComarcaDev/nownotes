// Aviso breve en la parte inferior, con acción opcional (p. ej. «Deshacer»).

export interface ToastData {
  id: number;
  text: string;
  action?: { label: string; run: () => void };
}

export const toast = $state<{ current: ToastData | null }>({ current: null });

let timer: ReturnType<typeof setTimeout> | undefined;
let seq = 0;

export function showToast(text: string, action?: ToastData["action"], ms = 7000) {
  clearTimeout(timer);
  toast.current = { id: ++seq, text, action };
  timer = setTimeout(() => (toast.current = null), ms);
}

/** Mientras el ratón está encima, el aviso no se cierra. */
export function holdToast() {
  clearTimeout(timer);
}

export function releaseToast() {
  clearTimeout(timer);
  timer = setTimeout(() => (toast.current = null), 3000);
}

export function dismissToast() {
  clearTimeout(timer);
  toast.current = null;
}
