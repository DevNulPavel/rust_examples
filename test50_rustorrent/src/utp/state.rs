
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum State {
    /// Еще не подключен
	None,
	/// Отправка sync пакет,не получили никакого ack
    SynSent,
    /// Пакет syn-ack был получен, теперь нормальное состояние для отправки и получения пакетов
    Connected,
    /// FIN пакет отправлен, но все пакеты до fin пакета пока не получили ack.
    /// Нужно продолжить ожидать FIN пакет с обратной стороны.
	FinSent,
    ///
    MustConnect
	// /// ====== states beyond this point =====
	// /// === are considered closing states ===
	// /// === and will cause the socket to ====
	// /// ============ be deleted =============
	// /// the socket has been gracefully disconnected
	// /// and is waiting for the client to make a
	// /// socket call so that we can communicate this
	// /// fact and actually delete all the state, or
	// /// there is an error on this socket and we're
	// /// waiting to communicate this to the client in
	// /// a callback. The error in either case is stored
	// /// in m_error. If the socket has gracefully shut
	// /// down, the error is error::eof.
	// ErrorWait,
	// /// there are no more references to this socket
	// /// and we can delete it
	// Delete
}

/// Конвертация из u8 в состояние
impl From<u8> for State {
    fn from(n: u8) -> State {
        match n {
            0 => State::None,
            1 => State::SynSent,
            2 => State::Connected,
            3 => State::FinSent,
            4 => State::MustConnect,
            _ => unreachable!()
        }
    }
}

/// Конвертация из состояния в u8
impl From<State> for u8 {
    fn from(s: State) -> u8 {
        match s {
            State::None => 0,
            State::SynSent => 1,
            State::Connected => 2,
            State::FinSent => 3,
            State::MustConnect => 4,
        }
    }
}
