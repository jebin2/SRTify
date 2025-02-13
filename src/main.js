const invoke = window.__TAURI__.core.invoke;
const listen = window.__TAURI__.event.listen;

let consoleElement = document.getElementById("console");
listen('info', function (event) {
    consoleElement.innerHTML += `<p>${event.payload}</p>`;
    autoScroll();
});
listen('transcription_started', function (event) {
    consoleElement.innerHTML = "";
    let generateSubtitle = document.getElementById("generateSubtitle");
    generateSubtitle.innerText = "Generating...,"
    generateSubtitle.style.pointerEvents = "none";
    autoScroll();
});
listen('transcription_progress', function (event) {
    consoleElement.innerHTML += `<p>${event.payload}</p>`;
    autoScroll();
});
listen('transcription_complete', function (event) {
    let generateSubtitle = document.getElementById("generateSubtitle");
    generateSubtitle.innerText = "Generate Subtitle"
    generateSubtitle.style.backgroundColor = "#906dee";
    delete generateSubtitle.style.pointerEvents;
    autoScroll();
});


async function invokeAPI(method, ...args) {
    return await invoke(method, ...args);
}

// model Selection Logic
document.getElementById("modelFile").addEventListener("click", async () => {
    let filePath = await invokeAPI("select_file", {isModel: true});
    if (filePath) {
        document.getElementById("model").value = filePath;
        await save_selection("model", filePath);
    }
});

// File Selection Logic
document.getElementById("selectFile").addEventListener("click", async () => {
    let filePath = await invokeAPI("select_file", {isModel: false});
    if (filePath) {
        document.getElementById("mediaFile").value = filePath;
        await save_selection("file", filePath);
    }
});

// Folder Selection Logic
document.getElementById("selectFolder").addEventListener("click", async () => {
    let folderPath = await invokeAPI("select_folder");
    if (folderPath) {
        document.getElementById("outputDir").value = folderPath;
        await save_selection("folder", folderPath);
    }
});

// Generate Subtitle Logic
document.getElementById("generateSubtitle").addEventListener("click", async () => {
    await invokeAPI("start_transcription");
});

// Save Meta Function
async function save_selection(key, value) {
    try {
        await invokeAPI("save_selection", { data: { key: key, value: value } });  // Pass key-value as object
        console.log(`Saved ${key}: ${value}`);
    } catch (error) {
        console.error(`Error saving ${key}:`, error);
        alert(`Error saving ${key}: ${error}`);
    }
}
function showDropdown() {
    const dropdown = document.getElementById('modelDropdown');
    if (dropdown.style.display === "block") {
        hideDropdown();
    } else {
        dropdown.style.display = 'block';  // Show the dropdown when input is clicked
    }
}
async function selectModel(model) {
    const input = document.getElementById('model');
    input.value = model;
    hideDropdown();
    await save_selection("model", input.value);
}
function hideDropdown() {
    const dropdown = document.getElementById('modelDropdown');
    dropdown.style.display = 'none';  // Hide the dropdown after selection
}
document.addEventListener('click', function(event) {
    const dropdown = document.getElementById('modelDropdown');
    const input = document.getElementById('model');
    
    if (!input.contains(event.target) && !dropdown.contains(event.target)) {
        dropdown.style.display = 'none';  // Hide dropdown if clicked outside
    }
});
// Load previously saved values when the page loads (example).  Adapt to your needs.
window.addEventListener('DOMContentLoaded', async () => {
    const modelFile = await invokeAPI("load_selection", { key: "model" });
    if (modelFile) {
        document.getElementById("model").value = modelFile;
    }

    const mediaFile = await invokeAPI("load_selection", { key: "file" });
    if (mediaFile) {
        document.getElementById("mediaFile").value = mediaFile;
    }

    const outputDir = await invokeAPI("load_selection", { key: "folder" });
    if (outputDir) {
        document.getElementById("outputDir").value = outputDir;
    }
});
function autoScroll() {
    const consoleElement = document.getElementById('console');
    consoleElement.scrollTop = consoleElement.scrollHeight;
  }