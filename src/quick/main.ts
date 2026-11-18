import { mount } from "svelte";
import Quick from "./Quick.svelte";
import "../styles/theme.css";
import { initTheme } from "../lib/theme";

initTheme();
document.addEventListener("contextmenu", (e) => {
  if (!(e.target as HTMLElement).closest("textarea")) e.preventDefault();
});

export default mount(Quick, { target: document.getElementById("app")! });
