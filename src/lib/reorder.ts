// Acción de Svelte para reordenar con el ratón (eventos de puntero, sin el drag & drop del sistema,
// que Tauri intercepta en Windows). Los hijos ordenables llevan `data-drag-id` y un asa `data-drag-handle`.

export interface ReorderOptions {
  enabled: boolean;
  /** "y" = lista vertical; "grid" = cuadrícula (se decide antes/después por la X). */
  axis: "y" | "grid";
  onreorder: (ids: number[]) => void;
}

export function reorderable(node: HTMLElement, initial: ReorderOptions) {
  let opts = initial;
  let dragging: HTMLElement | null = null;
  let target: { el: HTMLElement; after: boolean } | null = null;

  const children = () => Array.from(node.querySelectorAll<HTMLElement>(":scope > [data-drag-id]"));

  function clearTarget() {
    target?.el.classList.remove("drop-before", "drop-after");
    target = null;
  }

  function stop() {
    dragging?.classList.remove("dragging");
    node.classList.remove("sorting");
    document.body.style.cursor = "";
    dragging = null;
    clearTarget();
  }

  function onDown(e: PointerEvent) {
    if (!opts.enabled || e.button !== 0) return;
    const handle = (e.target as HTMLElement).closest<HTMLElement>("[data-drag-handle]");
    if (!handle || !node.contains(handle)) return;
    const el = handle.closest<HTMLElement>("[data-drag-id]");
    if (!el || el.parentElement !== node) return;
    e.preventDefault();
    handle.setPointerCapture(e.pointerId);
    dragging = el;
    el.classList.add("dragging");
    node.classList.add("sorting");
    document.body.style.cursor = "grabbing";
  }

  function onMove(e: PointerEvent) {
    if (!dragging) return;
    const under = document.elementFromPoint(e.clientX, e.clientY)?.closest<HTMLElement>("[data-drag-id]");
    if (!under || under === dragging || under.parentElement !== node) {
      if (!under) clearTarget();
      return;
    }
    const r = under.getBoundingClientRect();
    const after = opts.axis === "grid" ? e.clientX > r.left + r.width / 2 : e.clientY > r.top + r.height / 2;
    if (target?.el === under && target.after === after) return;
    clearTarget();
    target = { el: under, after };
    under.classList.add(after ? "drop-after" : "drop-before");
  }

  function onUp() {
    if (dragging && target) {
      const ids = children().map((c) => Number(c.dataset.dragId));
      const from = Number(dragging.dataset.dragId);
      const to = Number(target.el.dataset.dragId);
      const rest = ids.filter((id) => id !== from);
      const idx = rest.indexOf(to) + (target.after ? 1 : 0);
      rest.splice(idx, 0, from);
      if (rest.join() !== ids.join()) opts.onreorder(rest);
    }
    stop();
  }

  node.addEventListener("pointerdown", onDown);
  node.addEventListener("pointermove", onMove);
  node.addEventListener("pointerup", onUp);
  node.addEventListener("pointercancel", stop);

  return {
    update(next: ReorderOptions) {
      opts = next;
    },
    destroy() {
      node.removeEventListener("pointerdown", onDown);
      node.removeEventListener("pointermove", onMove);
      node.removeEventListener("pointerup", onUp);
      node.removeEventListener("pointercancel", stop);
    },
  };
}
