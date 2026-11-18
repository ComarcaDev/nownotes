import { mount } from "svelte";
import Reminder from "./Reminder.svelte";
import "../styles/theme.css";
import { initTheme } from "../lib/theme";

initTheme();
document.addEventListener("contextmenu", (e) => e.preventDefault());

export default mount(Reminder, { target: document.getElementById("app")! });
