#[cfg(test)]
mod agent_test;

use crate::client::ClientTransaction;
use crate::error::*;
use crate::message::*;

use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Instant;

/// Хендлер обрабатывает изменения состояний транзакций
/// Вызывается на изменение состояния транзакции
/// Использование e валидно только во время вызова, пользователь должен 
/// копировать поля явно
pub type Handler = Option<Arc<mpsc::UnboundedSender<Event>>>;

////////////////////////////////////////////////////////////////

/// Создаем пустой обработчик
pub fn noop_handler() -> Handler {
    None
}

////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum EventType {
    Callback(TransactionId),
    Insert(ClientTransaction),
    Remove(TransactionId),
    Close,
}

impl Default for EventType {
    fn default() -> Self {
        EventType::Callback(TransactionId::default())
    }
}

////////////////////////////////////////////////////////////////

/// Ивент передается обработчику описывая ивент транзакции
/// Do not reuse outside Handler.
#[derive(Debug)] //Clone
pub struct Event {
    pub event_type: EventType,
    pub event_body: Result<Message>,
}

impl Default for Event {
    fn default() -> Self {
        Event {
            event_type: EventType::default(),
            event_body: Ok(Message::default()),
        }
    }
}

////////////////////////////////////////////////////////////////

/// Агентская транзакция представляет собой транзакцию в процессе
/// Конкурентный доступ невалидный
pub(crate) struct AgentTransaction {
    id: TransactionId,
    deadline: Instant,
}

////////////////////////////////////////////////////////////////

/// AGENT_COLLECT_CAP - это начальная емкость для Agent.Collect слайсов,
/// достаточное в большинстве случаев, чтобы сделать функцию без аллокаций в большинстве случаев
const AGENT_COLLECT_CAP: usize = 100;

////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, Hash, Copy, Clone, Default, Debug)]
pub struct TransactionId(pub [u8; TRANSACTION_ID_SIZE]);

impl TransactionId {
    /// Создаем новый идентификатор транзакции, заполненный полностью пустыми значениями
    pub fn new() -> Self {
        let mut b = TransactionId([0u8; TRANSACTION_ID_SIZE]);
        rand::thread_rng().fill(&mut b.0);
        b
    }
}

/// Установка идентификатора транзакции для сообщения
impl Setter for TransactionId {
    fn add_to(&self, m: &mut Message) -> Result<()> {
        m.transaction_id = *self;
        m.write_transaction_id();
        Ok(())
    }
}

////////////////////////////////////////////////////////////////

/// ClientAgent - это реализация агента, котрая используется клиентом для
/// обработки транзакций
#[derive(Debug)]
pub enum ClientAgent {
    Process(Message),
    Collect(Instant),
    Start(TransactionId, Instant),
    Stop(TransactionId),
    Close,
}

////////////////////////////////////////////////////////////////

/// Агент - это низкоуровневая абстракция над списком транзакций, 
/// которая обрабатвается конкурентно (все вызовы безопасны с точки зрения корутин)
/// и имеют таймаут?
pub struct Agent {
    // Мапа транзакций, которые сейчас в процессе
    // Обработка событий выполненяется таким образом когда транзакция 
    // незарегистрирована до AgentTransaction,
    // минимизируя блокировку и защиту транзакций от гонок данных
    transactions: HashMap<TransactionId, AgentTransaction>,
    closed: bool,     // Работа завершена
    handler: Handler, // Канал обработки транзакций
}

impl Agent {
    /// Создаем нового агента с каналом событий
    pub fn new(handler: Handler) -> Self {
        Agent {
            transactions: HashMap::new(),
            closed: false,
            handler,
        }
    }

    /// Данный метод удаляет транзакции из списка и передает в начальный канал ошибку.
    /// Может возвращать ErrTransactionNotExists and ErrAgentClosed.
    fn stop_with_error(&mut self, id: TransactionId, error: Error) -> Result<()> {
        if self.closed {
            return Err(Error::ErrAgentClosed);
        }

        // Удаляем элемент из списка транзакций
        let v = self.transactions.remove(&id);
        if let Some(t) = v {
            // Если есть канал - отсылаем туда событие с ошибкой
            if let Some(handler) = &self.handler {
                handler.send(Event {
                    event_type: EventType::Callback(t.id),
                    event_body: Err(error),
                })?;
            }
            Ok(())
        } else {
            Err(Error::ErrTransactionNotExists)
        }
    }

    // process incoming message, synchronously passing it to handler.
    fn process(&mut self, message: Message) -> Result<()> {
        if self.closed {
            return Err(Error::ErrAgentClosed);
        }

        self.transactions.remove(&message.transaction_id);

        let e = Event {
            event_type: EventType::Callback(message.transaction_id),
            event_body: Ok(message),
        };

        if let Some(handler) = &self.handler {
            handler.send(e)?;
        }

        Ok(())
    }

    // Close terminates all transactions with ErrAgentClosed and renders Agent to
    // closed state.
    fn close(&mut self) -> Result<()> {
        if self.closed {
            return Err(Error::ErrAgentClosed);
        }

        for id in self.transactions.keys() {
            let e = Event {
                event_type: EventType::Callback(*id),
                event_body: Err(Error::ErrAgentClosed),
            };
            if let Some(handler) = &self.handler {
                handler.send(e)?;
            }
        }
        self.transactions = HashMap::new();
        self.closed = true;
        self.handler = noop_handler();

        Ok(())
    }

    // Start registers transaction with provided id and deadline.
    // Could return ErrAgentClosed, ErrTransactionExists.
    //
    // Agent handler is guaranteed to be eventually called.
    fn start(&mut self, id: TransactionId, deadline: Instant) -> Result<()> {
        if self.closed {
            return Err(Error::ErrAgentClosed);
        }
        if self.transactions.contains_key(&id) {
            return Err(Error::ErrTransactionExists);
        }

        self.transactions
            .insert(id, AgentTransaction { id, deadline });

        Ok(())
    }

    // Stop stops transaction by id with ErrTransactionStopped, blocking
    // until handler returns.
    fn stop(&mut self, id: TransactionId) -> Result<()> {
        self.stop_with_error(id, Error::ErrTransactionStopped)
    }

    // Collect terminates all transactions that have deadline before provided
    // time, blocking until all handlers will process ErrTransactionTimeOut.
    // Will return ErrAgentClosed if agent is already closed.
    //
    // It is safe to call Collect concurrently but makes no sense.
    fn collect(&mut self, deadline: Instant) -> Result<()> {
        if self.closed {
            // Doing nothing if agent is closed.
            // All transactions should be already closed
            // during Close() call.
            return Err(Error::ErrAgentClosed);
        }

        let mut to_remove: Vec<TransactionId> = Vec::with_capacity(AGENT_COLLECT_CAP);

        // Adding all transactions with deadline before gc_time
        // to toCall and to_remove slices.
        // No allocs if there are less than AGENT_COLLECT_CAP
        // timed out transactions.
        for (id, t) in &self.transactions {
            if t.deadline < deadline {
                to_remove.push(*id);
            }
        }
        // Un-registering timed out transactions.
        for id in &to_remove {
            self.transactions.remove(id);
        }

        for id in to_remove {
            let event = Event {
                event_type: EventType::Callback(id),
                event_body: Err(Error::ErrTransactionTimeOut),
            };
            if let Some(handler) = &self.handler {
                handler.send(event)?;
            }
        }

        Ok(())
    }

    // set_handler sets agent handler to h.
    fn set_handler(&mut self, h: Handler) -> Result<()> {
        if self.closed {
            return Err(Error::ErrAgentClosed);
        }
        self.handler = h;

        Ok(())
    }

    pub async fn run(mut agent: Agent, mut rx: mpsc::Receiver<ClientAgent>) {
        while let Some(client_agent) = rx.recv().await {
            let result = match client_agent {
                ClientAgent::Process(message) => agent.process(message),
                ClientAgent::Collect(deadline) => agent.collect(deadline),
                ClientAgent::Start(tid, deadline) => agent.start(tid, deadline),
                ClientAgent::Stop(tid) => agent.stop(tid),
                ClientAgent::Close => agent.close(),
            };

            if let Err(err) = result {
                if Error::ErrAgentClosed == err {
                    break;
                }
            }
        }
    }
}
