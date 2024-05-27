//! Демонстрирует как создавать кастомный флоу.
//! Происходит отрытие браузера для пользователя, делая использование InstalledAppFlow более удобным,
//! так как нет нужды выполнении копипасты.
//! При этом откроется браузер, юзер примет запрос. После того, как это произойдет, локальный веб-сервер, запущенный InstalledFlowAuthenticator,
//! получит токен, который отдается oauth2 сервером. Никакой копипасты не требуется для продолжения данной операции.
use std::{
    future::{
        Future
    },
    pin::{
        Pin
    }
};
use yup_oauth2::{
    authenticator_delegate::{
        DefaultInstalledFlowDelegate, InstalledFlowDelegate
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

/// async function to be pinned by the `present_user_url` method of the trait
/// we use the existing `DefaultInstalledFlowDelegate::present_user_url` method as a fallback for
/// when the browser did not open for example, the user still see's the URL.
// TODO: ???
/// Асинхронная функция, которая будет закреплена в present_user_url
async fn browser_user_url(url: &str, need_code: bool) -> Result<String, String> {
    if webbrowser::open(url).is_ok() {
        println!("webbrowser was successfully opened.");
    }
    let def_delegate = DefaultInstalledFlowDelegate{};
    def_delegate.present_user_url(url, need_code).await
}

//////////////////////////////////////////////////////////////////////////////////////////

/// Наш кастомный делегат с помощью которого мы будем реализовать
#[derive(Copy, Clone)]
struct InstalledFlowBrowserDelegate{
}

/// Здесь мы реализуем лишь present_user_url c добавленным открытием урла в браузере, перегрузка чего-то еще нам не нужна сейчас
impl InstalledFlowDelegate for InstalledFlowBrowserDelegate {
    /// Актуальное представление урла и открытия браузера происходит в функции выше, здесь мы лишь пиним и вовращаем футуру
    fn present_user_url<'a>(&'a self, url: &'a str, need_code: bool) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(browser_user_url(url, need_code))
    }
}

#[tokio::main]
async fn main() {
    // Читаем файлик из клиентской директории
    let sec = yup_oauth2::read_application_secret("client_secret.json")
        .await
        .expect("client secret couldn't be read.");
    
    // Создаем объект аутентификации
    // Запускается локальный HTTP сервер, гугл редиректит на локальный адресс
    // с кодом авторизации
    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(sec,
                                                               yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json") // Сохранение токена на диск в файлик
        .flow_delegate(Box::new(InstalledFlowBrowserDelegate{})) // Используем кастомный флоу для наших целей
        .build()
        .await
        .expect("InstalledFlowAuthenticator failed to build");

    // Получаем скоупы для диска
    let scopes = &[
        "https://www.googleapis.com/auth/drive.file"
    ];
    match auth.token(scopes).await {
        Err(e) => println!("error: {:?}", e),
        Ok(t) => println!("The token is {:?}", t),
    }
}
