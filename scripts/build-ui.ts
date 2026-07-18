// Bundle the guest program (plain TS policy bundle — no JSX/pak needed until
// the widget grows 2D chrome; then this switches to the vendored PocketJS
// build pipeline).
const result = await Bun.build({
  entrypoints: ["app/main.ts"],
  outdir: "dist",
  naming: "character.[ext]",
  format: "iife",
  target: "browser",
  minify: false,
});
if (!result.success) {
  for (const log of result.logs) console.error(log);
  process.exit(1);
}
console.log("dist/character.js built");
