// BP-style A/B/X/Y controller-glyph hint bar, fixed to the bottom.
const GLYPH_CLASS = { A: "bpm-glyph--a", B: "bpm-glyph--b", X: "bpm-glyph--x", Y: "bpm-glyph--y" };

export function Glyph({ k }) {
  return <span className={`bpm-glyph ${GLYPH_CLASS[k] || ""}`}>{k}</span>;
}

export default function Footer({ hints = [], right = null }) {
  return (
    <div className="sdb-footer">
      {hints.map((h, i) => (
        <span className="sdb-hint" key={i}>
          <Glyph k={h.k} /> {h.label}
        </span>
      ))}
      <span className="sdb-footer__spacer" />
      {right}
    </div>
  );
}
