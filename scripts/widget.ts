// Build + run the widget: guest bundle, release binary, launch.
import { $ } from "bun";

await $`bun scripts/fetch-assets.ts`;
await $`bun scripts/build-ui.ts`;
await $`cargo build --release -p pocket-character`;
await $`target/release/pocket-character ${process.argv.slice(2)}`;
