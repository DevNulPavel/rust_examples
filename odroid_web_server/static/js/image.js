"use strict";

let imageRequestIsActive = false;

function blobToImage(blob) {
    return new Promise((resolve) => {
        const url = URL.createObjectURL(blob);
        
        let img = new Image();
        
        img.onload = () => {
            URL.revokeObjectURL(url)
            resolve(img)
        };
        
        img.src = url;
    });
}

async function reloadImage(){
    if(imageRequestIsActive){
        return;
    }

    imageRequestIsActive = true;

    let loadingText = document.querySelector("#status");
    let refreshButton = document.querySelector("#refresh_button");
    let lightOnButton = document.querySelector("#light_on_button");
    let lightOffButton = document.querySelector("#light_off_button");
    let imagesContainer = document.querySelector("#images_container");

    while (imagesContainer.lastElementChild) {
        imagesContainer.removeChild(imagesContainer.lastElementChild);
    }

    refreshButton.disabled = true;
    lightOnButton.disabled = true;
    lightOffButton.disabled = true;

    loadingText.innerHTML = "Loading";

    let camerasCountResponse = await fetch("/cameras_count");
    if (camerasCountResponse.ok) {
        let camerasCountJson = await camerasCountResponse.json();

        // <img id="image" class="image"></img>

        if(camerasCountJson){
            for(let i = 0; i < camerasCountJson.count; i++){
                // https://learn.javascript.ru/fetch
                const path = "/image_from_camera?camera_index=" + i;
                let response = await fetch(path);
                if (response.ok) {
                    let data = await response.blob();

                    const image = await blobToImage(data);
                    imagesContainer.appendChild(image);
                    imagesContainer.appendChild(document.createElement("br"));
                } else {
                    loadingText.innerHTML = "Loading failed";
                    break;
                }
            }
            loadingText.innerHTML = "Loading complete";
        }else{
            loadingText.innerHTML = "Loading failed";    
        }
    } else {
        alert("Ошибка HTTP: " + response.status);
        loadingText.innerHTML = "Loading failed";
    }

    refreshButton.disabled = false;
    lightOnButton.disabled = false;
    lightOffButton.disabled = false;



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