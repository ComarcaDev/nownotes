import { describe, expect, it } from "vitest";
import type { Item } from "./api";
import { displayTitle, parseTags, quoted, toInput, todayBreakdown } from "./items";
import { renderMarkdown, toggleCheckLine } from "./markdown";
import { isNewer } from "./updates";

describe("isNewer", () => {
  it("compara versiones semánticas", () => {
    expect(isNewer("2.0.1", "2.0.0")).toBe(true);
    expect(isNewer("v2.1.0", "2.0.9")).toBe(true);
    expect(isNewer("10.0.0", "9.9.9")).toBe(true);
    expect(isNewer("2.0.0", "2.0.0")).toBe(false);
    expect(isNewer("1.9.9", "2.0.0")).toBe(false);
  });
  it("tolera partes que faltan o no numéricas", () => {
    expect(isNewer("2.1", "2.0.5")).toBe(true);
    expect(isNewer("2.0.0-beta", "2.0.0")).toBe(false);
  });
});

describe("toggleCheckLine", () => {
  const md = "Compra\n- [ ] pan\n- [x] leche\ntexto";
  it("marca y desmarca la casilla de la línea indicada", () => {
    expect(toggleCheckLine(md, 1)).toBe("Compra\n- [x] pan\n- [x] leche\ntexto");
    expect(toggleCheckLine(md, 2)).toBe("Compra\n- [ ] pan\n- [ ] leche\ntexto");
  });
  it("no cambia nada si la línea no es una casilla o no existe", () => {
    expect(toggleCheckLine(md, 0)).toBe(md);
    expect(toggleCheckLine(md, 3)).toBe(md);
    expect(toggleCheckLine(md, 99)).toBe(md);
  });
  it("normaliza saltos de línea de Windows", () => {
    expect(toggleCheckLine("a\r\n- [ ] b", 1)).toBe("a\n- [x] b");
  });
});

describe("renderMarkdown", () => {
  it("escapa el HTML", () => {
    const html = renderMarkdown('<img src=x onerror="alert(1)">');
    expect(html).not.toContain("<img");
    expect(html).toContain("&lt;img");
  });
});

describe("items", () => {
  const item = {
    id: 5, kind: "task", title: "Comprar pan", body: "", listId: 1, dueAt: null, remindAt: null,
    recur: null, pinned: false, tags: ["casa"],
  } as unknown as Item;

  it("parseTags limpia comas, espacios y #", () => {
    expect(parseTags("trabajo, #casa ,  ,##ideas")).toEqual(["trabajo", "casa", "ideas"]);
    expect(parseTags("")).toEqual([]);
  });
  it("displayTitle usa «Sin título» y recorta", () => {
    expect(displayTitle({ title: "" })).toBe("Sin título");
    expect(displayTitle({ title: "abcdef" }, 3)).toBe("abc…");
    expect(quoted(item)).toBe("«Comprar pan»");
  });
  it("toInput copia los campos y aplica los cambios", () => {
    const input = toInput(item, { title: "Pan integral" });
    expect(input).toMatchObject({ id: 5, kind: "task", title: "Pan integral", listId: 1, tags: ["casa"] });
    expect(toInput(item).title).toBe("Comprar pan");
  });
});

describe("todayBreakdown", () => {
  it("desglosa tareas y notas de Hoy", () => {
    expect(todayBreakdown({ today: 2, todayTasks: 1 })).toBe("1 tarea · 1 nota");
    expect(todayBreakdown({ today: 3, todayTasks: 3 })).toBe("3 tareas");
    expect(todayBreakdown({ today: 2, todayTasks: 0 })).toBe("2 notas");
    expect(todayBreakdown({ today: 0, todayTasks: 0 })).toBe("");
  });
});
