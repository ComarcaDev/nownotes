import { api } from "./api";

export const SOUNDS: { value: string; label: string }[] = [
  { value: "chime", label: "Campanilla" },
  { value: "bell", label: "Timbre" },
  { value: "soft", label: "Suave" },
  { value: "none", label: "Sin sonido" },
  { value: "custom", label: "Archivo propio…" },
];

// [frecuencia Hz, inicio s, duración s]
const PATTERNS: Record<string, [number, number, number][]> = {
  chime: [[880, 0, 0.9], [1318.5, 0.16, 1.2]],
  bell: [[987.8, 0, 0.5], [987.8, 0.28, 0.5], [1318.5, 0.56, 1.0]],
  soft: [[523.25, 0, 0.9], [659.25, 0.22, 0.9], [783.99, 0.44, 1.3]],
};

let ctx: AudioContext | null = null;

function synth(kind: string, volume: number) {
  const pattern = PATTERNS[kind] ?? PATTERNS.chime;
  ctx ??= new AudioContext();
  const master = ctx.createGain();
  master.gain.value = Math.max(0, Math.min(1, volume));
  master.connect(ctx.destination);
  const t0 = ctx.currentTime + 0.02;
  for (const [freq, start, dur] of pattern) {
    const osc = ctx.createOscillator();
    const env = ctx.createGain();
    osc.type = "sine";
    osc.frequency.value = freq;
    env.gain.setValueAtTime(0, t0 + start);
    env.gain.linearRampToValueAtTime(0.5, t0 + start + 0.012);
    env.gain.exponentialRampToValueAtTime(0.0001, t0 + start + dur);
    osc.connect(env).connect(master);
    osc.start(t0 + start);
    osc.stop(t0 + start + dur + 0.05);
  }
}

let current: HTMLAudioElement | null = null;

export async function playSound(kind: string, volume: number, customPath = ""): Promise<void> {
  if (kind === "none") return;
  if (kind === "custom" && customPath) {
    try {
      const buf = await api.readFileBytes(customPath);
      const url = URL.createObjectURL(new Blob([buf]));
      current?.pause();
      current = new Audio(url);
      current.volume = Math.max(0, Math.min(1, volume));
      current.onended = () => URL.revokeObjectURL(url);
      await current.play();
      return;
    } catch (e) {
      console.warn("No se pudo reproducir el sonido propio, se usa el predeterminado", e);
    }
  }
  synth(kind === "custom" ? "chime" : kind, volume);
}

export async function playConfigured(): Promise<void> {
  const s = await api.settings();
  await playSound(s.sound, parseFloat(s.volume || "0.7"), s.sound_path);
}
