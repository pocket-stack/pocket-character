// Fetch the airi-parity character assets. They are not committed: the VRoid
// sample-model terms are not MIT, so (like airi) we pull them at setup time.
import { $ } from "bun";

const ASSETS: Array<{ url: string; out: string; bytes: number }> = [
  {
    url: "https://dist.ayaka.moe/vrm-models/VRoid-Hub/AvatarSample-A/AvatarSample_A.vrm",
    out: "assets/AvatarSample_A.vrm",
    bytes: 26_781_812,
  },
  {
    url: "https://raw.githubusercontent.com/moeru-ai/airi/main/packages/stage-ui-three/src/assets/vrm/animations/idle_loop.vrma",
    out: "assets/idle_loop.vrma",
    bytes: 157_664,
  },
];

await $`mkdir -p assets`;
for (const a of ASSETS) {
  const f = Bun.file(a.out);
  if ((await f.exists()) && f.size === a.bytes) {
    console.log(`ok      ${a.out} (${f.size} bytes)`);
    continue;
  }
  console.log(`fetch   ${a.url}`);
  const res = await fetch(a.url);
  if (!res.ok) throw new Error(`${a.url}: HTTP ${res.status}`);
  await Bun.write(a.out, res);
  const got = Bun.file(a.out).size;
  if (got !== a.bytes)
    console.warn(`warn    ${a.out}: expected ${a.bytes} bytes, got ${got} (upstream may have changed)`);
  else console.log(`done    ${a.out} (${got} bytes)`);
}
