# naga-wasm

[![npm](https://img.shields.io/npm/v/naga-wasm?logo=npm)](https://www.npmjs.com/package/naga-wasm)
[![CI](https://github.com/kekkon-nexus/naga-wasm/actions/workflows/ci.yaml/badge.svg)](https://github.com/kekkon-nexus/naga-wasm/actions/workflows/ci.yaml)
[![node](https://img.shields.io/node/v/naga-wasm?logo=nodedotjs)](https://nodejs.org)

The [naga](https://github.com/gfx-rs/wgpu/tree/trunk/naga) shader translator
from wgpu, compiled to WebAssembly.

Translate between WGSL, GLSL, SPIR-V, HLSL and MSL from JavaScript, in Node or
the browser, without shelling out to the naga CLI.

- Frontends: WGSL, GLSL, SPIR-V
- Backends: WGSL, GLSL, SPIR-V, HLSL, MSL

## Install

```sh
npm install naga-wasm
```

## Usage

```ts
import { translate } from "naga-wasm";

const glsl = translate({
	from: "wgsl",
	to: "glsl",
	source: wgsl,
	options: { version: "300 es", stage: "fragment", entryPoint: "main" },
});
```

In Node the WebAssembly module loads on import. In the browser, call `init()`
once before anything else:

```ts
import { init, translate } from "naga-wasm";

await init();
```

`init()` resolves the binary next to the module, which Vite and webpack 5
handle. To host it yourself, pass a URL, `Response` or bytes:

```ts
await init({ module_or_path: new URL("/naga_bg.wasm", location.href) });
```

The binary is exported as `naga-wasm/naga_bg.wasm`.

### Handles

`translate` is a shortcut for the handle API. Parse once to write several
outputs:

```ts
import { parseWgsl, validate, writeGlsl, writeHlsl } from "naga-wasm";

using module = parseWgsl(source);
using info = validate(module);

const glsl = writeGlsl(module, info, {
	version: "330",
	stage: "vertex",
	entryPoint: "vs_main",
});
const hlsl = writeHlsl(module, info, { shaderModel: "6_0" });
```

Handles hold WebAssembly memory. Release them with `using` or `free()`.

| Function                                                          | Returns       |
| ----------------------------------------------------------------- | ------------- |
| `parseWgsl(source)`                                               | `Module`      |
| `parseGlsl(source, { stage, defines? })`                          | `Module`      |
| `parseSpirv(words)`                                               | `Module`      |
| `validate(module)`                                                | `ModuleInfo`  |
| `writeWgsl(module, info, { flags? })`                             | `string`      |
| `writeGlsl(module, info, { version, stage, entryPoint, flags? })` | `string`      |
| `writeHlsl(module, info, { shaderModel? })`                       | `string`      |
| `writeMsl(module, info, { langVersion? })`                        | `string`      |
| `writeSpirv(module, info, { flags? })`                            | `Uint32Array` |

`parseSpirv` accepts a `Uint8Array` or `Uint32Array`.

`flags` override naga's writer flags one at a time and keep the rest at their
defaults:

```ts
writeGlsl(module, info, {
	version: "300 es",
	stage: "vertex",
	entryPoint: "vs_main",
	flags: { forcePointSize: true },
});
```

### Errors

Failures throw a `NagaError` with:

- `kind`: `"parse"`, `"validation"` or `"write"`
- `message`: a one-line summary
- `formatted`: naga's full report, including source spans

```ts
import { NagaError, parseWgsl } from "naga-wasm";

try {
	parseWgsl("fn main( {");
} catch (error) {
	if (error instanceof NagaError) console.error(error.formatted);
}
```

```
error: expected identifier, found "{"
  ┌─ wgsl:1:10
  │
1 │ fn main( {
  │          ^ expected identifier
```

## Versioning

The major version follows naga's major version. Minor and patch releases
belong to this package. The bundled naga version is exported as `nagaVersion`
and noted in each release.

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
