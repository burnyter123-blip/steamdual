// Small formatting helpers shared across steps.
export const gib = (n) =>
  n >= 1 ? `${Math.round(n)} GiB` : `${Math.round(n * 1024)} MiB`;

export const pct = (n) => `${Math.round(n)}%`;
