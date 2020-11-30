
pub trait ResponseWithError {
    /// Пишет сообщение об ошибке в слак + в терминал
    fn slack_response_with_error(self, error_text: String);
}