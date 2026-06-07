/* =============================================================================
   Steam Big Picture reference — behaviour
   - live clock
   - generated color swatches
   - "gamepad" focus navigation: arrow keys move focus between .bpm-focusable
     elements (spatial), applying the .gpfocus class the same way the client does
   ============================================================================= */

/* ---- live clock ---------------------------------------------------------- */
function tickClock() {
  const el = document.getElementById('clock');
  if (!el) return;
  const d = new Date();
  el.textContent = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}
setInterval(tickClock, 1000); tickClock();

/* ---- color-token swatches (read live from tokens.css) -------------------- */
const TOKENS = [
  '--steam-surface-0', '--steam-surface-3', '--steam-surface-4',
  '--steam-text-muted', '--steam-text-secondary', '--steam-text-bright',
  '--steam-accent', '--steam-accent-bright', '--steam-accent-deep',
  '--steam-online', '--steam-destructive',
];
const swatches = document.getElementById('swatches');
if (swatches) {
  const cs = getComputedStyle(document.documentElement);
  TOKENS.forEach((name) => {
    const val = cs.getPropertyValue(name).trim();
    const el = document.createElement('div');
    el.className = 'bpm-swatch';
    el.innerHTML =
      `<div class="bpm-swatch__chip" style="background:${val}"></div>
       <div class="bpm-swatch__meta">
         <div class="bpm-swatch__name">${name}</div>
         <div class="bpm-swatch__hex">${val}</div>
       </div>`;
    swatches.appendChild(el);
  });
}

/* ---- "gamepad" spatial focus navigation ---------------------------------- */
/* Mirrors the client: focusable elements carry .gpfocus when focused. Arrow
   keys pick the nearest focusable in that direction by screen geometry. */
function focusables() {
  return [...document.querySelectorAll('.bpm-focusable')]
    .filter((el) => el.offsetParent !== null);
}
function center(el) {
  const r = el.getBoundingClientRect();
  return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
}
function nearestInDirection(from, dir) {
  const a = center(from);
  let best = null, bestScore = Infinity;
  for (const el of focusables()) {
    if (el === from) continue;
    const b = center(el);
    const dx = b.x - a.x, dy = b.y - a.y;
    const along = dir === 'left' ? -dx : dir === 'right' ? dx : dir === 'up' ? -dy : dy;
    if (along <= 1) continue;                       // must be in the chosen direction
    const cross = (dir === 'left' || dir === 'right') ? Math.abs(dy) : Math.abs(dx);
    const score = along + cross * 2.5;              // weight cross-axis drift heavily
    if (score < bestScore) { bestScore = score; best = el; }
  }
  return best;
}
const KEYS = { ArrowLeft: 'left', ArrowRight: 'right', ArrowUp: 'up', ArrowDown: 'down' };
document.addEventListener('keydown', (e) => {
  const dir = KEYS[e.key];
  if (!dir) return;
  const cur = document.activeElement?.classList?.contains('bpm-focusable')
    ? document.activeElement : focusables()[0];
  const next = cur ? nearestInDirection(cur, dir) : focusables()[0];
  if (next) {
    e.preventDefault();
    next.focus({ preventScroll: false });
    next.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'smooth' });
  }
});
/* keep .gpfocus in sync with the focused element */
document.addEventListener('focusin', (e) => {
  document.querySelectorAll('.gpfocus').forEach((el) => el.classList.remove('gpfocus'));
  if (e.target.classList?.contains('bpm-focusable')) e.target.classList.add('gpfocus');
});
