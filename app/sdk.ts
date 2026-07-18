// The `character` surface SDK: typed mirror of the namespace the host
// mounts on globalThis (RUNTIMES.md: facts arrive as per-tick batches via
// __dispatch, intent leaves through queued op calls).

export interface CharacterState {
  /** Sim seconds since boot. */
  t: number;
  /** Current blink expression weight 0..1. */
  blink: number;
  /** Clip currently playing. */
  clip: string;
  hovered: boolean;
  tracking: "none" | "mouse";
  /** Rendered frames per second over the last second. */
  fps: number;
  /** Mean full-frame CPU cost (ms) over the last second. */
  frameMs: number;
}

export type CharacterEvent =
  | { type: "click" }
  | { type: "hoverStart" }
  | { type: "hoverEnd" };

interface CharacterNs {
  __boot: {
    model: string;
    clips: string[];
    expressions: string[];
  };
  setTracking(mode: "none" | "mouse"): void;
  setExpression(name: string, weight: number): void;
  playClip(name: string, loop: boolean): void;
  setMaxFps(fps: number): void;
  quit(): void;
}

declare global {
  // eslint-disable-next-line no-var
  var character: CharacterNs & {
    __dispatch?: (state: CharacterState, events: CharacterEvent[]) => void;
  };
  // eslint-disable-next-line no-var
  var frame: ((buttons: number, analog: number) => void) | undefined;
}

export const character = globalThis.character;

/** Register the per-tick policy callback. */
export function onTick(
  cb: (state: CharacterState, events: CharacterEvent[]) => void,
): void {
  globalThis.character.__dispatch = cb;
}
