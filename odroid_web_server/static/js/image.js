"use strict";

let imageRequestIsActive = false;

async function reloadImage(){
    if(imageRequestIsActive){
        return;
    }

    imageRequestIsActive = true;

    let loadingText = document.querySelector("#status");
    let refreshButton = document.querySelector("#refresh_button");
    let lightOnButton = document.querySelector("#light_on_button");
    let lightOffButton = document.querySelector("#light_off_button");
    let image = document.querySelector("#image");

    refreshButton.disabled = true;
    lightOnButton.disabled = true;
    lightOffButton.disabled = true;

    loadingText.innerHTML = "Loading";

    // https://learn.javascript.ru/fetch

    let response = await fetch("/image_from_camera");

    refreshButton.disabled = false;
    lightOnButton.disabled = false;
    lightOffButton.disabled = false;

    if (response.ok) {
        let data = await response.blob();

        loadingText.innerHTML = "Loading complete";
        
        image.src = URL.createObjectURL(data);
    } else {
        alert("Ошибка HTTP: " + response.status);
        loadingText.innerHTML = "Loading failed";
    }

    imageRequestIsActive = false;
}

async function reloadImageClick(){
    console.log("Reload image clicked");
    reloadImage();
}

async function postRequest(addr, data){
    // https://developer.mozilla.org/ru/docs/Web/API/Fetch_API/Using_Fetch
    await fetch(addr, {
        method: "POST",
        cache: "no-cache",
        mode: "cors",
        credentials: "same-origin",
        redirect: "follow",
        referrerPolicy: "no-referrer", 
        headers: {
            "Content-Type": "application/json"
            //"Content-Type": "application/x-www-form-urlencoded",
        },
        body: JSON.stringify(data)
    });
}

async function lightOn(){
    console.log("Light on clicked");
    await postRequest("/light", {
        status: true
    });
}

async function lightOff(){
    console.log("Light off clicked");
    await postRequest("/light", {
        status: false
    });
}

function main(){
    reloadImage();
}

window.onload = main;