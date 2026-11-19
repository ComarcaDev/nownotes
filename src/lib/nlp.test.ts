import { describe, expect, it } from "vitest";
import { parseDate } from "./nlp";

// Miércoles 18 de noviembre de 2026, 10:00 (hora local)
const NOW = new Date(2026, 10, 18, 10, 0, 0);
const at = (y: number, m: number, d: number, h = 0, min = 0) => new Date(y, m - 1, d, h, min).getTime();

describe("parseDate", () => {
  it("día y hora en español", () => {
    const r = parseDate("comprar pan mañana 18:00", NOW)!;
    expect(r.title).toBe("comprar pan");
    expect(r.dueAt).toBe(at(2026, 11, 19));
    expect(r.remindAt).toBe(at(2026, 11, 19, 18));
  });

  it("día de la semana con «a las»", () => {
    const r = parseDate("llamar a Ana el lunes a las 9", NOW)!;
    expect(r.title).toBe("llamar a Ana");
    expect(r.remindAt).toBe(at(2026, 11, 23, 9));
  });

  it("solo fecha: sin aviso", () => {
    const r = parseDate("pagar la luz 25/12", NOW)!;
    expect(r.title).toBe("pagar la luz");
    expect(r.dueAt).toBe(at(2026, 12, 25));
    expect(r.remindAt).toBeNull();
  });

  it("fecha con nombre de mes y hora", () => {
    const r = parseDate("dentista el 25 de diciembre a las 10:30", NOW)!;
    expect(r.title).toBe("dentista");
    expect(r.remindAt).toBe(at(2026, 12, 25, 10, 30));
  });

  it("fecha pasada sin año → el año que viene", () => {
    expect(parseDate("renovar dni 3/2", NOW)!.dueAt).toBe(at(2027, 2, 3));
  });

  it("plazo relativo", () => {
    const r = parseDate("reunión en 2 horas", NOW)!;
    expect(r.title).toBe("reunión");
    expect(r.remindAt).toBe(at(2026, 11, 18, 12));
  });

  it("hora ya pasada hoy → mañana", () => {
    expect(parseDate("revisar correo a las 9:00", NOW)!.remindAt).toBe(at(2026, 11, 19, 9));
  });

  it("«a las 3» sin am/pm es por la tarde", () => {
    expect(parseDate("café a las 3", NOW)!.remindAt).toBe(at(2026, 11, 18, 15));
  });

  it("momentos del día", () => {
    expect(parseDate("cena esta noche", NOW)!.remindAt).toBe(at(2026, 11, 18, 21));
    // «por la mañana» es una hora, no el día de mañana
    expect(parseDate("estudiar por la mañana", NOW)!.remindAt).toBe(at(2026, 11, 19, 9));
    expect(parseDate("estudiar mañana por la mañana", NOW)!.remindAt).toBe(at(2026, 11, 19, 9));
  });

  it("pasado mañana", () => {
    const r = parseDate("enviar informe pasado mañana", NOW)!;
    expect(r.title).toBe("enviar informe");
    expect(r.dueAt).toBe(at(2026, 11, 20));
  });

  it("inglés", () => {
    const r = parseDate("Call John tomorrow at 3pm", NOW)!;
    expect(r.title).toBe("Call John");
    expect(r.remindAt).toBe(at(2026, 11, 19, 15));
    expect(parseDate("Gym next friday 7am", NOW)!.remindAt).toBe(at(2026, 11, 20, 7));
    expect(parseDate("Report in 30 minutes", NOW)!.remindAt).toBe(at(2026, 11, 18, 10, 30));
    expect(parseDate("Party dec 31", NOW)!.dueAt).toBe(at(2026, 12, 31));
  });

  it("no inventa fechas", () => {
    expect(parseDate("comprar 2 kilos de manzanas", NOW)).toBeNull();
    expect(parseDate("leer el capítulo 25", NOW)).toBeNull();
    expect(parseDate("subir el 50% del presupuesto", NOW)).toBeNull();
    expect(parseDate("Mañana", NOW)!.title).toBe("");
  });

  it("devuelve los fragmentos detectados", () => {
    expect(parseDate("comprar pan mañana 18:00", NOW)!.matches).toEqual(["mañana", "18:00"]);
  });
});
