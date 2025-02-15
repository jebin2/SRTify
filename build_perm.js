import { execSync } from "child_process";

let targetDep = "";
const isWindows = process.platform === "win32";
switch (process.platform) {
	case "win32":
		targetDep = "src-tauri/bin/dependency/ffmpeg.exe";
		break;
	case "darwin":
		targetDep = "src-tauri/bin/dependency/ffmpeg";
		break;
	case "linux":
		targetDep = "src-tauri/bin/dependency/ffmpeg";
		break;
}

if (!isWindows) {
	execSync(`chmod +x ${targetDep}`);
	console.log("Made ffmpeg executable");
}