import { describe, expect, it } from "vitest";
import { appearanceVars, DEFAULT_APPEARANCE, parseAppearance } from "./appearance";
import { accentVars, contrast, hexToRgb, rgbToHex, textOn } from "./colors";
import { LIST_ICONS, searchIcons } from "./listIcons";

describe("colores", () => {
  it("convierte entre hex y RGB", () => {
    expect(hexToRgb("#0067c0")).toEqual([0, 103, 192]);
    expect(hexToRgb("#fff")).toEqual([255, 255, 255]);
    expect(hexToRgb("azul")).toBeNull();
    expect(rgbToHex([0, 103, 192])).toBe("#0067c0");
  });
  it("elige el texto con más contraste sobre el acento", () => {
    expect(textOn([0, 103, 192])).toBe("#ffffff"); // azul oscuro → texto blanco
    expect(textOn([76, 194, 255])).toBe("#000000"); // celeste → texto negro
    expect(textOn([255, 209, 102])).toBe("#000000"); // ámbar → texto negro
    expect(contrast([0, 0, 0], [255, 255, 255])).toBeCloseTo(21, 0);
  });
  it("deriva todas las variables del acento", () => {
    const v = accentVars("#d13438", true)!;
    expect(v["--accent"]).toBe("#d13438");
    expect(v["--accent-soft"]).toBe("rgba(209, 52, 56, 0.14)");
    expect(v["--accent-hover"]).not.toBe("#d13438");
    expect(accentVars("nada", true)).toBeNull();
  });
});

describe("apariencia", () => {
  it("usa los valores por defecto si el ajuste falta o está dañado", () => {
    expect(parseAppearance(undefined)).toEqual(DEFAULT_APPEARANCE);
    expect(parseAppearance("{roto")).toEqual(DEFAULT_APPEARANCE);
    expect(parseAppearance("null")).toEqual(DEFAULT_APPEARANCE);
  });
  it("corrige valores fuera de rango o desconocidos", () => {
    const a = parseAppearance(JSON.stringify({ accent: "rojo", tone: "neón", bg: "video", veil: 300, blur: -5, panels: 10, imagePos: "x" }));
    expect(a).toMatchObject({ accent: "", tone: "neutral", bg: "mica", veil: 85, blur: 0, panels: 30, imagePos: "center" });
  });
  it("conserva una apariencia válida", () => {
    const ok = { ...DEFAULT_APPEARANCE, accent: "#e3008c", tone: "warm", bg: "image", veil: 55, blur: 6, panels: 80, imagePos: "center top", imageRev: 3 };
    expect(parseAppearance(JSON.stringify(ok))).toEqual(ok);
  });
  it("sin personalizar no tiñe nada ni cambia el acento", () => {
    const v = appearanceVars(DEFAULT_APPEARANCE, true);
    expect(v["--tint-amt"]).toBe("0%");
    expect(v["--accent"]).toBeUndefined();
  });
  it("el tono «del acento» usa el color de acento", () => {
    const v = appearanceVars({ ...DEFAULT_APPEARANCE, tone: "accent", accent: "#498205" }, false);
    expect(v["--tint"]).toBe("#498205");
    expect(v["--tint-amt"]).not.toBe("0%");
  });
});

describe("iconos de listas", () => {
  it("busca sin tildes por nombre y palabras clave", () => {
    expect(searchIcons("").length).toBe(LIST_ICONS.length);
    expect(searchIcons("coc").map((i) => i.id)).toContain("chef");
    expect(searchIcons("cafe").map((i) => i.id)).toContain("coffee");
    expect(searchIcons("UNIVERSIDAD").map((i) => i.id)).toEqual(["graduation"]);
    expect(searchIcons("zzz")).toEqual([]);
  });
  it("los identificadores son únicos y válidos para el backend", () => {
    const ids = LIST_ICONS.map((i) => i.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const id of ids) expect(id).toMatch(/^[a-z0-9-]{1,32}$/);
  });
});
