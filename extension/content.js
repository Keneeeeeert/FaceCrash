let useAlternativeImages;
let flipBlacklist;
let blacklistStatus;
const EXTENSION_NAME = chrome.runtime.getManifest().name;

let extensionIsDisabled = false;
let appearChance = 1.00;
let flipChance = 0.25;
let selectedPerson = 0;

let highestImageIndex = 0;

function applyOverlay(thumbnailElement, overlayImageURL, flip = false) {
    const overlayImage = document.createElement("img");
    overlayImage.id = EXTENSION_NAME;
    overlayImage.src = overlayImageURL;
    overlayImage.style.position = "absolute";
    overlayImage.style.top = overlayImage.style.left = "50%";
    overlayImage.style.width = "100%";
    overlayImage.style.transform = `translate(-50%, -50%) ${flip ? 'scaleX(-1)' : ''}`;
    overlayImage.style.zIndex = "0";
    overlayImage.style.pointerEvents = "none";

    const pictureElement = thumbnailElement.parentElement;
    const container = pictureElement.parentElement;

    const computed = getComputedStyle(container);
    if (computed.position === "static") {
        container.style.position = "relative";
    }

    container.insertBefore(overlayImage, pictureElement.nextSibling);
}

function findThumbnails() {
    const imageSelectors = [
        '.bili-video-card__cover img',
        '.bili-video-card__image--wrap img'
    ];

    const allImages = [];
    for (const selector of imageSelectors) {
        allImages.push(...Array.from(document.querySelectorAll(selector)));
    }

    const targetAspectRatio = 16 / 9;
    const errorMargin = 0.03;

    const listAllThumbnails = allImages.filter(image => {
        if (image.height === 0 || image.width === 0) {
            return false;
        }
        const aspectRatio = image.width / image.height;
        return Math.abs(aspectRatio - targetAspectRatio) < errorMargin;
    });

    return listAllThumbnails.filter(image => {
        const pictureElement = image.parentElement;
        if (!pictureElement || pictureElement.tagName !== 'PICTURE') {
            return false;
        }

        const imageWrap = pictureElement.parentElement;
        if (!imageWrap) return false;

        const skeleton = imageWrap.closest('.bili-video-card__skeleton');
        if (skeleton) return false;

        const cardWrap = imageWrap.closest('.bili-video-card__wrap');
        if (!cardWrap) return false;

        const alreadyProcessed = Array.from(cardWrap.querySelectorAll('img')).some(img =>
            img.id && img.id.includes(EXTENSION_NAME)
        );

        return !alreadyProcessed;
    });
}

function applyOverlayToThumbnails() {
    if (highestImageIndex <= 0) return;

    const thumbnailElements = findThumbnails();

    thumbnailElements.forEach((thumbnailElement) => {
        const loops = Math.random() > 0.001 ? 1 : 20;

        for (let i = 0; i < loops; i++) {
            let flip = Math.random() < flipChance;
            let baseImagePath = getImageIndex();

            if (flip && flipBlacklist && flipBlacklist.includes(String(baseImagePath))) {
                if (useAlternativeImages) {
                    let newImagePath = `textFlipped/${baseImagePath}`;
                    checkImageExistence(newImagePath).then(exists => {
                        if (exists) {
                            const overlayImageURL = Math.random() < appearChance
                                ? getImageURL(newImagePath)
                                : "";
                            if (overlayImageURL) {
                                applyOverlay(thumbnailElement, overlayImageURL, false);
                            }
                        }
                    });
                    continue;
                } else {
                    flip = false;
                }
            }

            const overlayImageURL = Math.random() < appearChance
                ? getImageURL(baseImagePath)
                : "";

            applyOverlay(thumbnailElement, overlayImageURL, flip);
        }
    });
}

const sizeOfNonRepeat = 8;
const lastIndexes = Array(sizeOfNonRepeat);

function getImageIndex() {
    if (highestImageIndex <= 0) return 1;

    if (selectedPerson > 0 && selectedPerson <= highestImageIndex) {
        return selectedPerson;
    }

    let randomIndex = -1;

    if (highestImageIndex <= sizeOfNonRepeat) {
        lastIndexes.fill(-1);
    }

    while (lastIndexes.includes(randomIndex) || randomIndex < 0) {
        randomIndex = Math.floor(Math.random() * highestImageIndex) + 1;
    }

    lastIndexes.shift();
    lastIndexes.push(randomIndex);

    return randomIndex;
}

async function getFlipBlocklist() {
    try {
        const response = await fetch(chrome.runtime.getURL(`${IMAGES_PATH}flip_blacklist.json`));
        const data = await response.json();
        useAlternativeImages = data.useAlternativeImages;
        flipBlacklist = data.blacklistedImages;
        blacklistStatus = `Flip blacklist found. ${useAlternativeImages ? "Images will be substituted." : "Images won't be flipped."}`;
    } catch (error) {
        blacklistStatus = "No flip blacklist found. Proceeding without it";
    }
}

async function loadConfig() {
    const defaults = {
        extensionIsDisabled: extensionIsDisabled,
        appearChance: appearChance,
        flipChance: flipChance,
        selectedPerson: selectedPerson
    };

    try {
        const config = await new Promise((resolve, reject) => {
            chrome.storage.local.get({
                extensionIsDisabled,
                appearChance,
                flipChance,
                selectedPerson
            }, (result) => {
                chrome.runtime.lastError
                    ? reject(chrome.runtime.lastError)
                    : resolve(result);
            });
        });

        extensionIsDisabled = config.extensionIsDisabled != null ? config.extensionIsDisabled : defaults.extensionIsDisabled;
        appearChance = config.appearChance != null ? config.appearChance : defaults.appearChance;
        flipChance = config.flipChance != null ? config.flipChance : defaults.flipChance;
        selectedPerson = config.selectedPerson != null ? config.selectedPerson : defaults.selectedPerson;

        if (Object.keys(config).length === 0) {
            await new Promise((resolve, reject) => {
                chrome.storage.local.set(defaults, () => {
                    chrome.runtime.lastError
                        ? reject(chrome.runtime.lastError)
                        : resolve();
                });
            });
        }
    } catch (error) {
        console.error("Error loading configuration:", error);
    }
}

async function main() {
    await loadConfig();

    if (extensionIsDisabled) {
        console.info(`${EXTENSION_NAME} is disabled.`);
        return;
    }

    await getFlipBlocklist();
    console.info(`${EXTENSION_NAME} will now detect the amount of images.`);

    highestImageIndex = await getHighestImageIndex();

    setInterval(applyOverlayToThumbnails, 100);
    console.info(
        `${EXTENSION_NAME} Loaded Successfully. ${highestImageIndex} images detected. ${blacklistStatus}.`
    );
}

main();
