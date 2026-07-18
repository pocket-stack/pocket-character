// Steady-state resource measurement for the widget, using the same
// methodology as the airi baseline: launch, settle, then sample the full
// process tree's RSS + %CPU via ps at 5 s intervals for >= 60 s, plus a
// `footprint` snapshot. Prints a JSON result line and a markdown row.
import { $ } from "bun";

const SETTLE_S = Number(process.env.MEASURE_SETTLE ?? 15);
const SAMPLES = Number(process.env.MEASURE_SAMPLES ?? 13);
const INTERVAL_S = Number(process.env.MEASURE_INTERVAL ?? 5);
const BIN = process.env.MEASURE_BIN ?? "target/release/pocket-character";
const args = process.argv.slice(2);

console.log(`launching ${BIN} ${args.join(" ")}`);
const child = Bun.spawn([BIN, ...args], {
  stdout: "inherit",
  stderr: "inherit",
});
const pid = child.pid;

const median = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2;
};

try {
  await Bun.sleep(SETTLE_S * 1000);
  const cpu: number[] = [];
  const rssMb: number[] = [];
  for (let i = 0; i < SAMPLES; i++) {
    const out = await $`ps -o %cpu=,rss= -p ${pid}`.text();
    const [c, r] = out.trim().split(/\s+/).map(Number);
    if (Number.isFinite(c) && Number.isFinite(r)) {
      cpu.push(c);
      rssMb.push(r / 1024);
      console.log(`sample ${i + 1}/${SAMPLES}: cpu=${c.toFixed(1)}% rss=${(r / 1024).toFixed(1)}MB`);
    }
    await Bun.sleep(INTERVAL_S * 1000);
  }
  let footprintMb: number | null = null;
  try {
    const fp = await $`footprint ${pid}`.text();
    const m = fp.match(/phys_footprint[^\d]*([\d.]+)\s*(KB|MB|GB)/i) ?? fp.match(/([\d.]+)\s*(KB|MB|GB)\s*$/m);
    if (m) {
      const v = Number(m[1]);
      footprintMb = m[2].toUpperCase() === "GB" ? v * 1024 : m[2].toUpperCase() === "KB" ? v / 1024 : v;
    } else {
      console.log("footprint output (unparsed):\n" + fp);
    }
  } catch {
    console.log("footprint unavailable");
  }

  const result = {
    bin: BIN,
    args,
    samples: cpu.length,
    cpu: { min: Math.min(...cpu), median: median(cpu), max: Math.max(...cpu) },
    rssMb: { min: Math.min(...rssMb), median: median(rssMb), max: Math.max(...rssMb) },
    footprintMb,
  };
  console.log("RESULT " + JSON.stringify(result));
  console.log(
    `| pocket-character | 1 | ${result.rssMb.median.toFixed(0)} MB | ${result.cpu.median.toFixed(1)}% | ${footprintMb ? footprintMb.toFixed(0) + " MB" : "—"} |`,
  );
} finally {
  child.kill();
  await child.exited;
}
