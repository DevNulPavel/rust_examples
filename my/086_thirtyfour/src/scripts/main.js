window.generateSignature = function generateSignature(url) {
    if (typeof window.byted_acrawler.sign !== "function") {
        throw "No signature function found";
    }
    return window.byted_acrawler.sign({ url: url });
};

window.generateBogus = function generateBogus(params) {
    if (typeof window._0x32d649 !== "function") {
        throw "No X-Bogus function found";
    }
    return window._0x32d649(params);
};

window.getNavigationInfo = function getNavigationInfo(params) {
    return {
        device_scale_factor: window.devicePixelRatio,
        user_agent: window.navigator.userAgent,
        browser_language: window.navigator.language,
        browser_platform: window.navigator.platform,
        browser_name: window.navigator.appCodeName,
        browser_version: window.navigator.appVersion,
    };
};