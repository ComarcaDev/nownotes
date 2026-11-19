import { mount } from "svelte";
import App from "./App.svelte";
import "../styles/theme.css";
import { initTheme } from "../lib/theme";

initTheme({ backgroundImage: true });
// Sin menú contextual del navegador (salvo en campos de texto, para copiar/pegar).
document.addEventListener("contextmenu", (e) => {
  const t = e.target as HTMLElement;
  if (!t.closest("input, textarea, .markdown")) e.preventDefault();
});

export default mount(App, { target: document.getElementById("app")! });
