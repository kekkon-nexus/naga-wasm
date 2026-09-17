import { $, argv } from "bun";

import { nagaVersion } from "../npm/src/version.ts";

const input = argv[2];
if (!input) {
	throw new Error("usage: bun run bump <version|major|minor|patch>");
}

const status = await $`git status --porcelain`.text();
if (status.trim()) {
	throw new Error("working directory not clean");
}

const output = await $`bun pm version ${input} --no-git-tag-version`
	.cwd("npm")
	.text();
const version = output.trim().slice(1);
const message = `build(release): ${version}\n\nBundles naga ${nagaVersion}.`;

await $`bun install`;
await $`git commit -am ${message}`;
await $`git tag -m ${message} v${version}`;
