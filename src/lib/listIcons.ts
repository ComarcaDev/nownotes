// Colores e iconos para personalizar las listas (trazos de 24×24, como Icon.svelte).

/** Colores que se leen bien tanto en tema claro como oscuro. */
export const LIST_COLORS = [
  "#0067c0", "#2b88d8", "#038387", "#00a383", "#498205", "#986f0b",
  "#ca5010", "#d13438", "#e3008c", "#c239b3", "#8764b8", "#69797e",
];

export interface ListIcon { id: string; label: string; keywords: string; path: string }

export const LIST_ICONS: ListIcon[] = [
  { id: "briefcase", label: "Trabajo", keywords: "empleo negocio clientes proyecto",
    path: "M4 8h16v11a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V8ZM9 8V6a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2M4 13h16" },
  { id: "building", label: "Oficina", keywords: "empresa edificio reuniones",
    path: "M5 21V5a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v16M15 10h3a1 1 0 0 1 1 1v10M3 21h18M8 8h3M8 12h3M8 16h3" },
  { id: "graduation", label: "Estudio", keywords: "universidad colegio clases curso examen",
    path: "M2 9l10-5 10 5-10 5L2 9ZM6 11v5c0 1.5 2.7 3 6 3s6-1.5 6-3v-5M22 9v6" },
  { id: "book", label: "Lectura", keywords: "libro leer biblioteca",
    path: "M5 4h11a3 3 0 0 1 3 3v13H8a3 3 0 0 1-3-3V4ZM5 17a3 3 0 0 1 3-3h11" },
  { id: "code", label: "Programación", keywords: "codigo software desarrollo",
    path: "M8 8l-4 4 4 4M16 8l4 4-4 4M14 5l-4 14" },
  { id: "bulb", label: "Ideas", keywords: "bombilla inspiracion creatividad",
    path: "M9 18h6M10 21h4M12 3a6 6 0 0 0-4 10.5c.8.7 1 1.5 1 2.5h6c0-1 .2-1.8 1-2.5A6 6 0 0 0 12 3Z" },
  { id: "home", label: "Casa", keywords: "hogar familia limpieza",
    path: "M4 11l8-7 8 7M6 9.5V20h12V9.5M10 20v-5h4v5" },
  { id: "chef", label: "Cocina", keywords: "cocinar recetas comida",
    path: "M7 14a4 4 0 0 1-.6-7.9A5 5 0 0 1 16 5a4 4 0 0 1 1 8.9V20H7v-6ZM7 16.5h10" },
  { id: "coffee", label: "Café", keywords: "descanso cafeteria pausa",
    path: "M5 9h11v5a5 5 0 0 1-5 5h-1a5 5 0 0 1-5-5V9ZM16 10h1.5a2.5 2.5 0 0 1 0 5H16M8 3v3M11 3v3" },
  { id: "cart", label: "Compras", keywords: "supermercado mercado tienda",
    path: "M3 4h2l2.4 11h10.2L20 7H6.2M8.5 20.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2ZM16.5 20.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" },
  { id: "wallet", label: "Dinero", keywords: "finanzas pagos facturas banco",
    path: "M4 7a2 2 0 0 1 2-2h11v4M4 7v11a2 2 0 0 0 2 2h14V9H6a2 2 0 0 1-2-2ZM16 14.5h.01" },
  { id: "heart", label: "Salud", keywords: "medico bienestar corazon",
    path: "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10Z" },
  { id: "dumbbell", label: "Deporte", keywords: "gimnasio ejercicio entrenamiento",
    path: "M6 7v10M18 7v10M3 9.5v5M21 9.5v5M6 12h12" },
  { id: "travel", label: "Viaje", keywords: "vacaciones maleta vuelo",
    path: "M6 8h12a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1ZM9 8V5h6v3M9 8v12M15 8v12" },
  { id: "car", label: "Coche", keywords: "auto transporte taller",
    path: "M4 16v-3.5L6.5 7h11l2.5 5.5V16H4ZM4 16v3h3v-3M17 16v3h3v-3M7.5 12.5h.01M16.5 12.5h.01" },
  { id: "paw", label: "Mascotas", keywords: "perro gato veterinario",
    path: "M12 20c-3 0-5-1.5-5-3.5S9.5 12 12 12s5 2.5 5 4.5-2 3.5-5 3.5ZM4.4 10.5a1.6 1.6 0 1 0 3.2 0 1.6 1.6 0 1 0-3.2 0ZM7.9 6.5a1.6 1.6 0 1 0 3.2 0 1.6 1.6 0 1 0-3.2 0ZM12.9 6.5a1.6 1.6 0 1 0 3.2 0 1.6 1.6 0 1 0-3.2 0ZM16.4 10.5a1.6 1.6 0 1 0 3.2 0 1.6 1.6 0 1 0-3.2 0Z" },
  { id: "users", label: "Familia", keywords: "personas amigos equipo",
    path: "M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7ZM2.5 20a6.5 6.5 0 0 1 13 0M16 4.5a3.5 3.5 0 0 1 0 6.5M18 14.5a6.5 6.5 0 0 1 3.5 5.5" },
  { id: "gift", label: "Regalos", keywords: "cumpleanos celebracion fiesta",
    path: "M4 10h16v3H4zM5 13v7h14v-7M12 10v10M12 10H8.5a2.5 2.5 0 1 1 0-5C11 5 12 10 12 10ZM12 10h3.5a2.5 2.5 0 1 0 0-5C13 5 12 10 12 10Z" },
  { id: "music", label: "Música", keywords: "canciones concierto",
    path: "M9 18V5l11-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM20 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" },
  { id: "game", label: "Ocio", keywords: "juegos videojuegos diversion",
    path: "M7 8h10a4 4 0 0 1 4 4v1a3 3 0 0 1-5.4 1.8L14.5 14h-5l-1.1 1.8A3 3 0 0 1 3 13v-1a4 4 0 0 1 4-4ZM7 11v3M5.5 12.5h3M15.5 11.5h.01M17.5 13.5h.01" },
  { id: "tool", label: "Herramientas", keywords: "reparaciones mantenimiento soporte tecnico",
    path: "M15 4a5 5 0 0 0-4.6 6.9l-6.9 6.9a1.8 1.8 0 0 0 2.6 2.6l6.9-6.9A5 5 0 0 0 20 9l-3 3-3-1-1-3 3-3a5 5 0 0 0-1-1Z" },
  { id: "leaf", label: "Plantas", keywords: "jardin naturaleza ecologia",
    path: "M5 19c0-9 6-14 15-14 0 9-5 15-14 15M5 19l7-7" },
  { id: "phone", label: "Llamadas", keywords: "telefono contactos",
    path: "M5 4h3l2 5-2.5 1.5a11 11 0 0 0 6 6L15 14l5 2v3a2 2 0 0 1-2 2A16 16 0 0 1 3 6a2 2 0 0 1 2-2Z" },
  { id: "star", label: "Importante", keywords: "favoritos estrella prioridad",
    path: "M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9L12 3Z" },
  { id: "flag", label: "Objetivos", keywords: "metas bandera hitos",
    path: "M5 21V4M5 4h11l-2 4 2 4H5" },
];

export const LIST_ICON_PATHS: Record<string, string> = Object.fromEntries(LIST_ICONS.map((i) => [i.id, i.path]));

/** Sin tildes y en minúsculas, para buscar «cafe» y encontrar «Café». */
const fold = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

/** Iconos cuyo nombre o palabras clave contienen el texto buscado. */
export function searchIcons(query: string): ListIcon[] {
  const q = fold(query.trim());
  if (!q) return LIST_ICONS;
  return LIST_ICONS.filter((i) => fold(`${i.label} ${i.keywords}`).includes(q));
}
