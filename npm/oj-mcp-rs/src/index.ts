import { spawnSync } from "child_process";

const PLATFORM_MAP: Record<string, string> = {
  linux: "linux",
  darwin: "darwin",
  win32: "windows",
  cygwin: "windows",
};

const ARCH_MAP: Record<string, string> = {
  x64: "x64",
  arm64: "arm64",
};

const os = PLATFORM_MAP[process.platform];
const arch = ARCH_MAP[process.arch];

if (!os || !arch) {
  console.error(
    `Unsupported platform: ${process.platform}-${process.arch}. ` +
      `Supported: linux-x64, linux-arm64, darwin-x64, darwin-arm64, windows-x64, windows-arm64.`
  );
  process.exit(1);
}

const ext = os === "windows" ? ".exe" : "";
const pkg = `oj-mcp-rs-${os}-${arch}`;
let binaryPath: string;

try {
  binaryPath = require.resolve(`${pkg}/bin/oj-mcp-rs${ext}`);
} catch {
  console.error(
    `Could not find binary package '${pkg}'. ` +
      `Ensure the correct platform-specific package is installed.`
  );
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
});

process.exit(result.status ?? 0);
