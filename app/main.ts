// airi-parity widget behavior. The native core owns the continuous state
// (idle clip, blink envelope, saccades, springs); this bundle is the policy
// layer, and parity with airi's out-of-the-box VRM stage is deliberately
// almost-empty policy: idle loop + autonomous eyes, no reactions.
import { character, onTick } from "./sdk";

console.log(
  `pocket-character: model=${character.__boot.model}`,
  `clips=[${character.__boot.clips.join(", ")}]`,
  `expressions=${character.__boot.expressions.length}`,
);

// airi VRM defaults: tracking mode "none", idle_loop.vrma looped.
character.setTracking("none");
character.playClip("idle_loop", true);

let lastStatsLog = 0;
onTick((state, events) => {
  for (const ev of events) {
    // Parity note: airi's default stage ignores taps too (its tap handler is
    // dead code out of the box). Keep the hook so a non-parity personality
    // can react.
    if (ev.type === "click") console.log("character: click at t =", state.t.toFixed(2));
  }
  // Heartbeat once a minute so long measurement runs show guest liveness.
  if (state.t - lastStatsLog >= 60) {
    lastStatsLog = state.t;
    console.log(
      `character: t=${state.t.toFixed(0)}s fps=${state.fps.toFixed(1)} frameMs=${state.frameMs.toFixed(2)}`,
    );
  }
});
