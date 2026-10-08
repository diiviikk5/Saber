const root = document.documentElement;
const $ = (s) => document.querySelector(s);
const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
const store = (k, v) => { try { localStorage.setItem(k, v); } catch {} };

function syncThemeColor() {
  const meta = $('meta[name="theme-color"]');
  if (meta) meta.content = getComputedStyle(document.body).getPropertyValue("--ib-paper").trim();
}

// ---------------------------------------------------------------- light / dark
$("#mode").addEventListener("click", () => {
  const dark = root.dataset.theme !== "dark";
  if (dark) root.dataset.theme = "dark";
  else delete root.dataset.theme;
  store("saber-mode", dark ? "dark" : "light");
  syncThemeColor();
});

// ---------------------------------------------------------------- moods: keys latch down
const moods = document.querySelectorAll(".mood");
function setMood(mood) {
  root.dataset.mood = mood;
  moods.forEach((m) => {
    const on = m.dataset.mood === mood;
    m.setAttribute("aria-checked", String(on));
    m.tabIndex = on ? 0 : -1;
  });
}
moods.forEach((m, i) => {
  m.addEventListener("click", () => {
    setMood(m.dataset.mood);
    store("saber-mood", m.dataset.mood);
  });
  m.addEventListener("keydown", (e) => {
    const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    const next = moods[(i + step + moods.length) % moods.length];
    next.click();
    next.focus();
  });
});
setMood(root.dataset.mood || "ember");
syncThemeColor();

// ---------------------------------------------------------------- the app preview
const hero = $("#app-hero");
const posters = document.querySelectorAll(".poster");
function select(p) {
  posters.forEach((q) => q.classList.toggle("is-on", q === p));
  hero.style.setProperty("--a", p.style.getPropertyValue("--a"));
  hero.style.setProperty("--b", p.style.getPropertyValue("--b"));
  $("#app-title").textContent = p.textContent;
  $("#app-stats").textContent = p.dataset.hours;
}
posters.forEach((p) => p.addEventListener("click", () => select(p)));

// session timers tick for real
const timers = document.querySelectorAll("[data-timer]");
let elapsed = 42 * 60 + 17;
const pad = (n) => String(n).padStart(2, "0");
setInterval(() => {
  elapsed++;
  const t = `${pad(Math.floor(elapsed / 3600))}:${pad(Math.floor(elapsed / 60) % 60)}:${pad(elapsed % 60)}`;
  timers.forEach((el) => (el.textContent = t));
}, 1000);

// Ctrl K (on the page or the big keycaps) types a search into the preview
const keys = $("#keys");
const search = $("#app-search");
const query = $("#app-query");
let typing;
function demoSearch() {
  keys.classList.add("is-down");
  setTimeout(() => keys.classList.remove("is-down"), 160);
  clearInterval(typing);
  const target = posters[Math.floor(Math.random() * posters.length)];
  const word = target.textContent;
  search.classList.add("is-typing");
  let n = 0;
  query.textContent = "";
  typing = setInterval(() => {
    query.textContent = word.slice(0, ++n);
    if (n >= word.length) {
      clearInterval(typing);
      select(target);
      setTimeout(() => {
        search.classList.remove("is-typing");
        query.textContent = "Search your library";
      }, 1400);
    }
  }, reduced ? 0 : 70);
}
$("#find").addEventListener("click", demoSearch);
addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    $("#app-search").scrollIntoView({ behavior: reduced ? "auto" : "smooth", block: "center" });
    demoSearch();
  }
});

// ---------------------------------------------------------------- the blade
const blade = $("#blade");
const hilt = $("#hilt");
function ignite(on) {
  blade.dataset.lit = String(on);
  hilt.setAttribute("aria-pressed", String(on));
  hilt.setAttribute("aria-label", on ? "Retract the saber" : "Ignite the saber");
}
hilt.addEventListener("click", () => ignite(blade.dataset.lit !== "true"));
// ignite once the first time it scrolls into view
new IntersectionObserver((entries, obs) => {
  if (entries[0].isIntersecting) {
    setTimeout(() => ignite(true), reduced ? 0 : 250);
    obs.disconnect();
  }
}, { threshold: 1 }).observe(blade);

// ---------------------------------------------------------------- the stack: explodes on click
const stack = $("#stack");
const stage = $("#stage");
const stackBtn = $("#stack-btn");
function toggleStack() {
  const open = stack.classList.toggle("open");
  stage.setAttribute("aria-pressed", String(open));
  stackBtn.textContent = open ? "Put it back together" : "Take it apart";
}
stage.addEventListener("click", toggleStack);
stackBtn.addEventListener("click", toggleStack);
stage.addEventListener("keydown", (e) => {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    toggleStack();
  }
});
document.querySelectorAll(".steps .ib-row").forEach((row) => {
  row.addEventListener("mouseenter", () => {
    stack.dataset.focus = row.dataset.layer;
    if (!stack.classList.contains("open")) toggleStack();
  });
  row.addEventListener("mouseleave", () => delete stack.dataset.focus);
});
