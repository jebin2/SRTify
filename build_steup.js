import fs from "fs";
import { execSync } from "child_process";

let sourceDep = "";
const isWindows = process.platform === "win32";
switch (process.platform) {
	case "win32":
		sourceDep = "src-tauri/bin/win32-x64";
		break;
	case "darwin":
		sourceDep = "src-tauri/bin/macos-x64";
		break;
	case "linux":
		sourceDep = "src-tauri/bin/linux-x64";
		break;
}

const targetDep = "src-tauri/bin/dependency";

fs.rmSync(targetDep, { recursive: true, force: true });
fs.cpSync(sourceDep, targetDep, { recursive: true });

console.log(`Copied ${sourceDep} to ${targetDep}`);

if (!isWindows) {
	execSync(`chmod +x ${targetDep}/ffmpeg`);
	console.log("Made ffmpeg executable");
}