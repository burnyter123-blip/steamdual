// Spatial "gamepad" focus navigation, ported from SteamBPClone/app.js into a
// React hook. Arrow keys / d-pad / left-stick move focus to the nearest
// `.bpm-focusable` in that direction; the focused element carries `.gpfocus`
// (Steam's class) so the BP white-inversion / lift treatment applies. Enter /
// A activates; Esc / B is left to the app.
import { useEffect } from "react";

function focusables() {
  return [...document.querySelectorAll(".bpm-focusable")].filter(
    (el) => el.offsetParent !== null && !el.hasAttribute("disabled")
  );
}
function center(el) {
  const r = el.getBoundingClientRect();
  return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
}
function nearestInDirection(from, dir) {
  const a = center(from);
  let best = null,
    bestScore = Infinity;
  for (const el of focusables()) {
    if (el === from) continue;
    const b = center(el);
    const dx = b.x - a.x,
      dy = b.y - a.y;
    const along = dir === "left" ? -dx : dir === "right" ? dx : dir === "up" ? -dy : dy;
    if (along <= 1) continue;
    const cross = dir === "left" || dir === "right" ? Math.abs(dy) : Math.abs(dx);
    const score = along + cross * 2.5;
    if (score < bestScore) {
      bestScore = score;
      best = el;
    }
  }
  return best;
}

const KEYS = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" };

export function useGamepadFocus() {
  useEffect(() => {
    function move(dir) {
      const cur = document.activeElement?.classList?.contains("bpm-focusable")
        ? document.activeElement
        : focusables()[0];
      const next = cur ? nearestInDirection(cur, dir) : focusables()[0];
      if (next) {
        next.focus({ preventScroll: false });
        next.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" });
      }
      return !!next;
    }

    function onKey(e) {
      const dir = KEYS[e.key];
      if (dir) {
        if (move(dir)) e.preventDefault();
      } else if (e.key === "Enter" || e.key === " ") {
        if (document.activeElement?.classList?.contains("bpm-focusable")) {
          // let the element's own click handler run
        }
      }
    }

    function onFocusIn(e) {
      document.querySelectorAll(".gpfocus").forEach((el) => el.classList.remove("gpfocus"));
      if (e.target.classList?.contains("bpm-focusable")) e.target.classList.add("gpfocus");
    }

    document.addEventListener("keydown", onKey);
    document.addEventListener("focusin", onFocusIn);

    // --- Gamepad API polling (real controller / Deck sticks in the webview) ---
    let raf = 0;
    const cooldown = { t: 0 };
    function pollPads(ts) {
      const pads = navigator.getGamepads ? navigator.getGamepads() : [];
      for (const p of pads) {
        if (!p) continue;
        const lx = p.axes[0] || 0;
        const ly = p.axes[1] || 0;
        const dpadL = p.buttons[14]?.pressed;
        const dpadR = p.buttons[15]?.pressed;
        const dpadU = p.buttons[12]?.pressed;
        const dpadD = p.buttons[13]?.pressed;
        if (ts - cooldown.t > 180) {
          let dir = null;
          if (lx < -0.5 || dpadL) dir = "left";
          else if (lx > 0.5 || dpadR) dir = "right";
          else if (ly < -0.5 || dpadU) dir = "up";
          else if (ly > 0.5 || dpadD) dir = "down";
          if (dir) {
            move(dir);
            cooldown.t = ts;
          }
        }
        if (p.buttons[0]?.pressed && ts - cooldown.t > 250) {
          document.activeElement?.click?.();
          cooldown.t = ts;
        }
      }
      raf = requestAnimationFrame(pollPads);
    }
    raf = requestAnimationFrame(pollPads);

    // Seed focus on the first focusable.
    setTimeout(() => focusables()[0]?.focus(), 50);

    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("focusin", onFocusIn);
      cancelAnimationFrame(raf);
    };
  }, []);
}
