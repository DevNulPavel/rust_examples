mod test;

use thirtyfour::{error::WebDriverError, By, DesiredCapabilities, WebDriver};

pub async fn sign_url() -> Result<(), WebDriverError> {
    // Подключение к драйверу
    let driver = {
        // Параметры для запуска хрома
        let mut caps = DesiredCapabilities::chrome();

        caps.set_ignore_certificate_errors()?;

        // Не запускаем графический интерфейс
        caps.set_headless()?;

        caps.add_chrome_arg(
            "--user-agent=\
                Mozilla/5.0 \
                (Macintosh; Intel Mac OS X 10_15_6) \
                AppleWebKit/537.36 \
                (KHTML, like Gecko) \
                Chrome/98.0.4758.109 \
                Safari/537.36",
        )?;

        caps.add_chrome_arg("--disable-blink-features")?;
        caps.add_chrome_arg("--disable-blink-features=AutomationControlled")?;
        caps.add_chrome_arg("--disable-infobars")?;
        caps.add_chrome_arg("--window-size=1920,1080")?;
        caps.add_chrome_arg("--start-maximized")?;

        WebDriver::new("http://localhost:9515", caps).await?
    };

    // Делаем переход на страничку
    driver
        .goto("https://www.tiktok.com/@rihanna?lang=en")
        .await?;

    // TODO: Дождаться завершения загрузки

    let script_result = driver
        .execute(include_str!("scripts/signer.js"), vec![])
        .await?;
    dbg!(script_result);

    let script_result = driver
        .execute(include_str!("scripts/webmssdk.js"), vec![])
        .await?;
    dbg!(script_result);

    let script_result = driver
        .execute(include_str!("scripts/main.js"), vec![])
        .await?;
    dbg!(script_result);

    let script_result = driver
        .execute("let res = window.getNavigationInfo(); return res;", vec![])
        .await?;
    dbg!(script_result.json());

    let script_result = driver
        .execute("let res = window.generateSignature('https://www.ya.ru'); return res;", vec![])
        .await?;
    dbg!(script_result.json());

    driver.quit().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), WebDriverError> {
    // test_web_driver().await?;
    sign_url().await?;

    Ok(())
}
