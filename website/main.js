// Theme swatches: repaint the page and remember the choice.
const root = document.documentElement;
const swatches = document.querySelectorAll(".swatch");

function apply(theme) {
  if (theme === "ember") delete root.dataset.theme;
  else root.dataset.theme = theme;
  swatches.forEach((s) => s.setAttribute("aria-checked", String(s.dataset.theme === theme)));
  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) meta.content = getComputedStyle(root).getPropertyValue("--bg").trim();
}

swatches.forEach((s) =>
  s.addEventListener("click", () => {
    apply(s.dataset.theme);
    try {
      localStorage.setItem("saber-theme", s.dataset.theme);
    } catch {}
  })
);

try {
  const saved = localStorage.getItem("saber-theme");
  if (saved) apply(saved);
} catch {}
