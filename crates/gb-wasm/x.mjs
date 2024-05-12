#!/usr/bin/env node
import { argv } from "node:process";
import { spawnSync } from "node:child_process";

function main() {
  const args = argv.slice(2);
  const command = args[0];

  switch (command) {
    case "build":
      spawnSync(
        "wasm-pack",
        ["build", "--release", "--out-dir", "npm", "--target", "web"],
        {
          stdio: "inherit",
        }
      );
      break;
    case "build-dev":
      spawnSync(
        "wasm-pack",
        [
          "build",
          "--profiling",
          "--out-dir",
          "npm",
          "--target",
          "web",
          "--mode",
          "no-install",
          "--no-opt",
        ].concat(...args.slice(1)),
        {
          stdio: "inherit",
        }
      );
      break;
    default:
      throw new Error(`Unknown command: ${command}`);
  }
}

main();
