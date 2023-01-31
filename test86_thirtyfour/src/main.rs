mod test;

use serde::Deserialize;
use thirtyfour::{error::WebDriverError, DesiredCapabilities, WebDriver};
use tokio::sync::{mpsc, oneshot};
use url::Url;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.109 Safari/537.36";

#[derive(Debug, Deserialize)]
struct NavigationInfo {
    pub device_scale_factor: f32,
    pub user_agent: String,
    pub browser_language: String,
    pub browser_platform: String,
    pub browser_name: String,
    pub browser_version: String,
}

#[derive(Debug)]
enum Task {
    GetNavigationInfo {
        resp: oneshot::Sender<NavigationInfo>,
    },
    CalcSignature {
        url: String,
        resp: oneshot::Sender<String>,
    },
    CalcBogus {
        query_string: String,
        resp: oneshot::Sender<String>,
    },
    Quit,
}

async fn sign_url_actor(user_agent: String, mut receiver: mpsc::Receiver<Task>) {
    // Подключение к драйверу
    let driver = {
        // Параметры для запуска хрома
        let mut caps = DesiredCapabilities::chrome();

        // caps.set_ignore_certificate_errors().unwrap();

        // Не запускаем графический интерфейс
        // caps.set_headless().unwrap();

        caps.add_chrome_arg(format!("--user-agent={user_agent}").as_str())
            .unwrap();

        // caps.add_chrome_arg("--disable-blink-features").unwrap();
        // caps.add_chrome_arg("--disable-blink-features=AutomationControlled")
        //     .unwrap();
        // caps.add_chrome_arg("--disable-infobars").unwrap();
        // caps.add_chrome_arg("--window-size=1920,1080").unwrap();
        // caps.add_chrome_arg("--start-maximized").unwrap();

        WebDriver::new("http://localhost:9515", caps).await.unwrap()
    };

    // Делаем переход на страничку
    driver
        .goto("https://www.tiktok.com/@rihanna?lang=en")
        .await
        .unwrap();

    // TODO: Дождаться завершения загрузки

    let script_result = driver
        .execute(include_str!("scripts/signer.js"), vec![])
        .await
        .unwrap();
    dbg!(script_result);

    let script_result = driver
        .execute(include_str!("scripts/webmssdk.js"), vec![])
        .await
        .unwrap();
    dbg!(script_result);

    let script_result = driver
        .execute(include_str!("scripts/main.js"), vec![])
        .await
        .unwrap();
    dbg!(script_result);

    while let Some(task) = receiver.recv().await {
        match task {
            Task::CalcBogus { query_string, resp } => {
                let script =
                    format!("let res = window.generateBogus('{query_string}'); return res;",);

                let script_result = driver.execute(script.as_str(), vec![]).await.unwrap();

                let bogus: String = script_result.convert().unwrap();

                resp.send(bogus).unwrap();
            }
            Task::CalcSignature { url, resp } => {
                let script = format!("let res = window.generateSignature('{url}'); return res;",);

                let script_result = driver.execute(script.as_str(), vec![]).await.unwrap();

                let signature: String = script_result.convert().unwrap();

                resp.send(signature).unwrap();
            }
            Task::GetNavigationInfo { resp } => {
                let script_result = driver
                    .execute("let res = window.getNavigationInfo(); return res;", vec![])
                    .await
                    .unwrap();

                let info: NavigationInfo = script_result.convert().unwrap();

                resp.send(info).unwrap();
            }
            Task::Quit => {}
        }
    }

    driver.quit().await.unwrap();
}

#[tokio::main]
async fn main() -> Result<(), WebDriverError> {
    let (sender, receiver) = mpsc::channel(1);

    let actor_join = tokio::spawn(sign_url_actor(USER_AGENT.to_string(), receiver));

    let navigation_info = {
        let (send, receive) = oneshot::channel();
        sender
            .send(Task::GetNavigationInfo { resp: send })
            .await
            .unwrap();
        receive.await.unwrap()
    };

    let mut url = {
        let text = format!("https://www.tiktok.com/api/post/item_list/?\
            aid=1988&\
            app_language=en&\
            app_name=tiktok_web&\
            battery_info=0.8&\
            browser_language={}&\
            browser_name={}&\
            browser_online=true&\
            browser_platform={}&\
            browser_version={}&\
            channel=tiktok_web&\
            cookie_enabled=true&\
            device_id=7192603937660667394&\
            device_platform=web_pc&\
            focus_state=true&\
            from_page=user&\
            history_len=3&\
            is_fullscreen=false&\
            is_page_visible=true&\
            os=mac&\
            priority_region=&\
            referer=&\
            region=RU&\
            screen_height=1152&\
            screen_width=2048&\
            tz_name=Europe%2FMoscow&\
            webcast_language=en&\
            msToken=dJOE0Ixc8Hr7wkFh4TD_obyZVikIB11HQJPrc1_lc51SqFa0CK3jhPgwTDzgTYz0PwjC033Stmbs9rbyQQR_Q6_-8CB-zFXQRjkkG25lcNTD2iBxQgOxtw7Yo3b7zgIwRlZi2TFI9ge5I0Hs",
            navigation_info.browser_language,
            navigation_info.browser_name,
            navigation_info.browser_platform,
            navigation_info.browser_version
        );

        Url::parse(&text).unwrap()
    };

    let signature = {
        let (send, receive) = oneshot::channel();
        let url = url.as_str().to_owned();
        sender
            .send(Task::CalcSignature { url, resp: send })
            .await
            .unwrap();
        receive.await.unwrap()
    };
    dbg!(&signature);

    url.query_pairs_mut()
        .append_pair("_signature", signature.as_str());

    let query_str = url.query().unwrap();

    let bogus = {
        let (send, receive) = oneshot::channel();
        sender
            .send(Task::CalcBogus {
                query_string: query_str.to_string(),
                resp: send,
            })
            .await
            .unwrap();
        receive.await.unwrap()
    };
    dbg!(&bogus);

    url.query_pairs_mut().append_pair("X-Bogus", bogus.as_str());

    dbg!(&url);

    sender.send(Task::Quit).await.unwrap();

    actor_join.await.unwrap();

    Ok(())
}
