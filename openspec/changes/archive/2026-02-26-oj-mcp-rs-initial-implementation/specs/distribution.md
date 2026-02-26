# Distribution Specification

Covers npm package structure, JS shim logic, GitHub Actions release workflow, target platforms, and Node.js constraints.

---

## S1: npm Package Structure

### S1.1: Root Package (`oj-mcp-rs`)

The root package contains no native binary. It is a thin JS shim that resolves and spawns the correct platform-specific binary.

```
npm/oj-mcp-rs/
  package.json
  tsconfig.json
  src/
    index.ts
```

**package.json fields:**
- `name`: `"oj-mcp-rs"`
- `version`: injected from git tag (stripped `v` prefix)
- `bin`: `"lib/index.js"` (compiled output of `src/index.ts`)
- `engines`: `{ "node": ">=18.0.0" }`
- `optionalDependencies`: all 6 platform packages at the same version

### S1.2: Platform Packages (6 total)

Each platform package contains a single precompiled binary.

```
{platform-package}/
  package.json
  bin/
    oj-mcp-rs           # (or oj-mcp-rs.exe on windows)
```

**package.json fields:**
- `name`: `"oj-mcp-rs-{os}-{arch}"` (e.g. `"oj-mcp-rs-linux-x64"`)
- `version`: same as root package
- `os`: single-element array matching npm os identifier
- `cpu`: single-element array matching npm cpu identifier

### S1.3: Template

`npm/package.json.tmpl` is a JSON template with placeholders `${node_pkg}`, `${node_version}`, `${node_os}`, `${node_arch}` substituted during CI.

### PBT Properties

```
[INVARIANT] The root package.json optionalDependencies lists exactly 6 platform packages.
[FALSIFICATION] Parse package.json → Object.keys(optionalDependencies).length == 6.

[INVARIANT] All platform package versions match the root package version.
[FALSIFICATION] For each dep in optionalDependencies → dep.version == root.version.

[INVARIANT] The bin field points to a file that exists after TypeScript compilation.
[FALSIFICATION] npm pack → verify lib/index.js exists in tarball.
```

---

## S2: JS Shim Logic

### S2.1: Platform Detection

Map `process.platform` and `process.arch` to package naming:

| `process.platform` | Mapped OS |
|---|---|
| `linux` | `linux` |
| `darwin` | `darwin` |
| `win32` | `windows` |
| `cygwin` | `windows` |

| `process.arch` | Mapped Arch |
|---|---|
| `x64` | `x64` |
| `arm64` | `arm64` |

### S2.2: Binary Resolution

```
packageName = `oj-mcp-rs-${os}-${arch}`
ext = (os === "windows") ? ".exe" : ""
binaryPath = require.resolve(`${packageName}/bin/oj-mcp-rs${ext}`)
```

If `require.resolve` fails (platform package not installed), print an error message listing the current platform and available packages, then exit with code 1.

### S2.3: Execution

```js
const result = spawnSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
process.exit(result.status ?? 0);
```

- `stdio: "inherit"` passes stdin/stdout/stderr directly to the child process (required for MCP stdio transport)
- Exit code mirrors the child process exit code
- `null` status (signal kill) maps to exit code 0

### PBT Properties

```
[INVARIANT] For any (platform, arch) pair in the target matrix,
  the shim resolves a binary path ending with the correct filename.
[FALSIFICATION] Mock process.platform="linux", process.arch="x64" →
               resolved path ends with "oj-mcp-rs-linux-x64/bin/oj-mcp-rs".

[INVARIANT] Windows platforms always append ".exe" to the binary name.
[FALSIFICATION] Mock process.platform="win32" → path ends with "oj-mcp-rs.exe".
               Mock process.platform="linux" → path does NOT end with ".exe".

[INVARIANT] The shim passes all arguments after its own invocation to the binary.
[FALSIFICATION] Invoke shim with ["--base-url", "http://x", "--token", "y"] →
               spawnSync receives exactly those args.

[INVARIANT] The shim's exit code matches the child process exit code.
[FALSIFICATION] Child exits 42 → shim exits 42.
               Child killed by signal (status=null) → shim exits 0.

[INVARIANT] stdio is set to "inherit", ensuring stdin/stdout pass through
  for MCP transport.
[FALSIFICATION] Inspect spawnSync options → stdio === "inherit".
```

---

## S3: Target Platforms

### S3.1: Platform Matrix (6 targets)

| npm Package Name | Rust Target | Runner OS | Notes |
|---|---|---|---|
| `oj-mcp-rs-linux-x64` | `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | |
| `oj-mcp-rs-linux-arm64` | `aarch64-unknown-linux-gnu` | `ubuntu-22.04` | `cross` |
| `oj-mcp-rs-darwin-x64` | `x86_64-apple-darwin` | `macos-14` | |
| `oj-mcp-rs-darwin-arm64` | `aarch64-apple-darwin` | `macos-14` | |
| `oj-mcp-rs-windows-x64` | `x86_64-pc-windows-msvc` | `windows-2022` | |
| `oj-mcp-rs-windows-arm64` | `aarch64-pc-windows-msvc` | `windows-2022` | |

### S3.2: Cross-Compilation

- Linux ARM64: use `cross` crate to avoid QEMU overhead
- All other targets: native compilation on the matching runner OS

### S3.3: Binary Naming

- Linux/macOS: `oj-mcp-rs` (no extension)
- Windows: `oj-mcp-rs.exe`

### PBT Properties

```
[INVARIANT] Each platform binary executes "--version" and outputs a string
  matching "oj-mcp-rs {semver}".
[FALSIFICATION] For each target binary → run with --version → parse output.

[INVARIANT] Package name "windows" is used (not "win32") for npm packages.
[FALSIFICATION] Grep all generated package.json files → no "win32" in name field.
```

---

## S4: GitHub Actions Release Workflow

### S4.1: Trigger

Push tag matching `v*` (e.g. `v0.1.0`).

### S4.2: Jobs

**Job 1: `publish-npm-binaries`**
- Strategy: matrix of 6 platform targets
- Steps per target:
  1. Checkout repository
  2. Install Rust toolchain + target
  3. Install `cross` (for Linux ARM64 targets)
  4. Build release binary (`cargo build --release --target {target}` or `cross build`)
  5. Create platform package directory with `package.json` from template
  6. Copy binary to `bin/`
  7. `npm publish` the platform package

**Job 2: `publish-npm-base`**
- `needs: publish-npm-binaries`
- Steps:
  1. Checkout repository
  2. Setup Node.js >= 18
  3. Install dependencies, compile TypeScript
  4. Update `package.json` version from tag
  5. `npm publish` the root package

### S4.3: Version Extraction

Strip the `v` prefix from the git tag: `${GITHUB_REF_NAME#v}`.

### S4.4: npm Authentication

Use `NODE_AUTH_TOKEN` secret for `npm publish`. Token stored as GitHub Actions secret.

### PBT Properties

```
[INVARIANT] The workflow produces exactly 7 npm publishes (6 platform + 1 root).
[FALSIFICATION] Count npm publish steps in workflow YAML → 7 total.

[INVARIANT] Job 2 never starts before all Job 1 matrix entries complete.
[FALSIFICATION] Verify `needs: publish-npm-binaries` in Job 2 definition.

[INVARIANT] The version in all published packages matches the git tag (minus 'v' prefix).
[FALSIFICATION] Tag v1.2.3 → all package.json versions == "1.2.3".

[INVARIANT] Each platform binary in the published package passes --version check.
[FALSIFICATION] Add smoke test step: run binary with --version, assert exit 0.
```

---

## S5: Node.js Constraint

### S5.1: Minimum Version

Node.js >= 18.0.0. Enforced via `engines` field in root `package.json`.

### S5.2: Rationale

Node.js 18 is the oldest active LTS line (as of project inception). The shim uses `spawnSync` and `require.resolve` which are stable across all supported versions.

### PBT Properties

```
[INVARIANT] The engines field specifies "node": ">=18.0.0".
[FALSIFICATION] Parse root package.json → engines.node == ">=18.0.0".

[INVARIANT] The shim uses only Node.js APIs available in v18.0.0:
  child_process.spawnSync, require.resolve, process.platform, process.arch,
  process.argv, process.exit.
[FALSIFICATION] Static analysis of index.ts → no APIs introduced after Node 18.
```
