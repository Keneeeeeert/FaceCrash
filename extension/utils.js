const IMAGES_PATH = "images/";

function getImageURL(index) {
    return chrome.runtime.getURL(`${IMAGES_PATH}${index}.png`);
}

async function checkImageExistence(index) {
    try {
        await fetch(getImageURL(index));
        return true;
    } catch {
        return false;
    }
}

async function getHighestImageIndex() {
    const INITIAL_INDEX = 4;
    let i = INITIAL_INDEX;

    while (await checkImageExistence(i)) {
        i *= 2;
    }

    let min = i <= INITIAL_INDEX ? 1 : i / 2;
    let max = i;

    while (min <= max) {
        let mid = Math.floor((min + max) / 2);
        if (await checkImageExistence(mid)) {
            min = mid + 1;
        } else {
            max = mid - 1;
        }
    }

    return max;
}
