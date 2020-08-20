"use strict";

let imageRequestIsActive = false;

async function reloadImage(){
    if(imageRequestIsActive){
        return;
    }

    imageRequestIsActive = true;

    let loadingText = document.querySelector("#status");
    let refreshButton = document.querySelector("#button");
    let image = document.querySelector("#image");

    refreshButton.disabled = true;

    loadingText.innerHTML = "Loading";

    // https://learn.javascript.ru/fetch

    let response = await fetch("/image_from_camera");

    refreshButton.disabled = false;

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

function reloadImageClick(){
    console.log("Reload image clicked");
    reloadImage();
}

function main(){
    reloadImage();
}

window.onload = main;