let extensionIsDisabled;
let appearChance;
let flipChance;
let selectedPerson;

async function populatePersonSelect() {
    const select = document.getElementById('personSelect');
    const maxIndex = await getHighestImageIndex();

    for (let i = 1; i <= maxIndex; i++) {
        const option = document.createElement('option');
        option.value = i;
        option.textContent = '人物 ' + i;
        select.appendChild(option);
    }

    if (selectedPerson !== undefined) {
        select.value = selectedPerson;
    }
}

function loadSettings() {
    chrome.storage.local.get({
        extensionIsDisabled: false,
        appearChance: 1.00,
        flipChance: 0.25,
        selectedPerson: 0
    }, function (data) {
        extensionIsDisabled = data.extensionIsDisabled;
        appearChance = data.appearChance;
        flipChance = data.flipChance;
        selectedPerson = data.selectedPerson;

        document.getElementById('disableExtension').checked = !data.extensionIsDisabled;
        document.getElementById('appearChance').value = data.appearChance * 100;
        document.getElementById('flipChance').value = data.flipChance * 100;

        populatePersonSelect();
    });
}

function saveSettings() {
    const data = {
        extensionIsDisabled: !document.getElementById('disableExtension').checked,
        appearChance: parseInt(document.getElementById('appearChance').value) / 100,
        flipChance: parseInt(document.getElementById('flipChance').value) / 100,
        selectedPerson: parseInt(document.getElementById('personSelect').value)
    };

    chrome.storage.local.set(data, () => {
        if (chrome.runtime.lastError) {
            console.error("Error saving settings:", chrome.runtime.lastError);
        }
    });
}

function setExtensionTitle() {
    const titleElement = document.getElementById('extension-title');
    titleElement.textContent = chrome.runtime.getManifest().name;
}

document.addEventListener('DOMContentLoaded', () => {
    loadSettings();
    setExtensionTitle();
});

document.getElementById('disableExtension').addEventListener('input', saveSettings);
document.getElementById('appearChance').addEventListener('input', saveSettings);
document.getElementById('flipChance').addEventListener('input', saveSettings);
document.getElementById('personSelect').addEventListener('change', saveSettings);
