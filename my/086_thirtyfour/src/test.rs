use thirtyfour::{error::WebDriverError, By, DesiredCapabilities, WebDriver};

pub async fn test_web_driver() -> Result<(), WebDriverError> {
    // Подключение к драйверу
    let driver = {
        // Параметры для запуска хрома
        let caps = DesiredCapabilities::chrome();

        // Не запускаем графический интерфейс
        // caps.set_headless()?;

        // Не активируем GPU?
        // caps.set_disable_gpu()?;

        WebDriver::new("http://localhost:9515", caps).await?
    };

    // Делаем переход на википедию
    driver.goto("https://wikipedia.org").await?;

    // Ищем какой-то элемент там
    let elem_form = driver.find(By::Id("search-form")).await?;

    // Находим что-то внутри этого самого элемента
    let elem_text = elem_form.find(By::Id("searchInput")).await?;

    // Печатаем текст в элемент
    elem_text.send_keys("selenium").await?;

    // Находим кнопку поиска и нажимаем
    let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
    elem_button.click().await?;

    // Ищем заголовок для ожидания подгрузки странички
    driver.find(By::ClassName("firstHeading")).await?;
    assert_eq!(driver.title().await?, "Selenium - Wikipedia");

    // Always explicitly close the browser.
    driver.quit().await?;

    Ok(())
}
