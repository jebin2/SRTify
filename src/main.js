const invoke = window.__TAURI__.core.invoke;
const listen = window.__TAURI__.event.listen;

const consoleElement = document.getElementById("console");
const generateSubtitleButton = document.getElementById("generateSubtitle");
const progressBar = generateSubtitleButton.querySelector('.progress-bar');
const buttonTextElement = generateSubtitleButton.querySelector('.button-text');
const modelInput = document.getElementById("model");
const mediaFileInput = document.getElementById("mediaFile");
const outputDirInput = document.getElementById("outputDir");
const modelDropdown = document.getElementById('modelDropdown');

// --- Utility Functions ---
async function invokeAPI(method, ...args) {
    try {
        return await invoke(method, ...args);
    } catch (error) {
        console.error(`Error invoking ${method}:`, error);
        alert(`Error invoking ${method}: ${error}`);
        throw error;
    }
}

function appendConsoleMessage(message) {
    consoleElement.innerHTML += `<p>${message}</p>`;
    autoScrollConsole();
}

function autoScrollConsole() {
    consoleElement.scrollTop = consoleElement.scrollHeight;
}

// --- Event Listeners for Tauri Events ---
listen('info', (event) => {
    appendConsoleMessage(event.payload);
});

listen('transcription_started', () => {
    setGeneratingState(true);
    updateProgress(0, "transcription");
});

listen('transcription_progress', (event) => {
    appendConsoleMessage(event.payload);
    let startTimeString = event.payload.split("start_time: ")[1].split(" end_time:")[0];
    let durationString = event.payload.split("duration: ")[1];
    let startTime = parseFloat(startTimeString);
    let duration = parseFloat(durationString);
    
    if (duration > 0 && startTime >= 0) {
        let percentage = (startTime / duration) * 100;
        percentage = Math.min(Math.max(percentage, 0), 100);
        updateProgress(percentage, "transcription");
    }
});

listen('transcription_complete', () => {
    setGeneratingState(false);
    updateProgress(100, "transcription");
});

listen('transcription_cancelled', (event) => {
    appendConsoleMessage(event.payload);
    setGeneratingState(false);
    updateProgress(0, "transcription");
});

listen('download_progress', (event) => {
    setGeneratingState(true);
    updateProgress(parseInt(event.payload.progress), "download")
});

// --- UI Update Functions ---
function setGeneratingState(isGenerating) {
    generateSubtitleButton.disabled = isGenerating;
    generateSubtitleButton.classList.toggle('generating', isGenerating);
    
    if (!isGenerating) {
        updateProgress(100);
        buttonTextElement.innerText = "Generate Subtitle";
    }
}

// --- File/Folder Selection Logic ---
async function selectFile(isModel) {
    const filePath = await invokeAPI("select_file", { isModel });
    if (filePath) {
        const inputElement = isModel ? modelInput : mediaFileInput;
        inputElement.value = filePath;
        await saveSelection(isModel ? "model" : "file", filePath);
    }
}

async function selectFolder() {
    const folderPath = await invokeAPI("select_folder");
    if (folderPath) {
        outputDirInput.value = folderPath;
        await saveSelection("folder", folderPath);
    }
}

// --- Dropdown Model Selection ---
function showDropdown() {
    modelDropdown.style.display = (modelDropdown.style.display === "block") ? "none" : "block";
}

async function selectModel(model) {
    modelInput.value = model;
    hideDropdown();
    await saveSelection("model", modelInput.value);
}

function hideDropdown() {
    modelDropdown.style.display = "none";
}

// --- Saving Selection ---
async function saveSelection(key, value) {
    try {
        await invokeAPI("save_selection", { data: { key, value } });
        console.log(`Saved ${key}: ${value}`);
    } catch (error) {
        console.error(`Error saving ${key}:`, error);
        alert(`Error saving ${key}: ${error}`);
    }
}

// --- Event Listeners for Buttons ---
document.getElementById("modelFile").addEventListener("click", () => selectFile(true));
document.getElementById("selectFile").addEventListener("click", () => selectFile(false));
document.getElementById("selectFolder").addEventListener("click", () => selectFolder());

generateSubtitleButton.addEventListener("click", async () => {
    await invokeAPI("start_transcription");
});

document.addEventListener('click', function (event) {
    if (!modelInput.contains(event.target) && !modelDropdown.contains(event.target)) {
        hideDropdown();
    }
});
// --- Load Saved Values on DOMContentLoaded ---
window.addEventListener('DOMContentLoaded', async () => {
    try {
        const modelFile = await invokeAPI("load_selection", { key: "model" });
        if (modelFile) {
            modelInput.value = modelFile;
        }

        const mediaFile = await invokeAPI("load_selection", { key: "file" });
        if (mediaFile) {
            mediaFileInput.value = mediaFile;
        }

        const outputDir = await invokeAPI("load_selection", { key: "folder" });
        if (outputDir) {
            outputDirInput.value = outputDir;
        }
    } catch (error) {
        console.error("Error loading saved selections:", error);
    }
});

function updateProgress(progress, type) {
    const clampedProgress = parseInt(Math.max(0, Math.min(100, progress)));
    if (clampedProgress >= 100) {
        clampedProgress = 100;
    }
    progressBar.style.width = `${clampedProgress}%`;
    buttonTextElement.innerText = type == "download" ? "Downloading Model... "+clampedProgress+"%" : "Generating Subtitle... "+clampedProgress+"%";
    
    if (clampedProgress >= 100) {
        console.log(clampedProgress);
        setTimeout(() => {
            progressBar.style.width = '0%';
            generateSubtitleButton.classList.remove('generating');
        }, 1000);
    }
}