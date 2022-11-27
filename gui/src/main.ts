import '@brainandbones/skeleton/themes/theme-rocket.css';
import '@brainandbones/skeleton/styles/all.css';
import "./app.postcss";
import App from "./App.svelte";

const app = new App({
  target: document.getElementById("app"),
});

export default app;
