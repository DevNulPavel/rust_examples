use mio::{event::Source, Events, Interest, Poll, Registry, Token};
use sharded_slab::Slab;
use std::{
    io::{self, Error},
    sync::Arc,
    task::Waker,
    thread::{self},
};

////////////////////////////////////////////////////////////////////////////////

/// Реактор нужен для того, чтобы отслеживать события от `mio` библиотеки и вызывать
/// waker в соответствии с этими событиями
pub(crate) struct Reactor {
    /// Список разных вейкеров из slab аллокатора для скорости
    wakers: Arc<Slab<Waker>>,

    /// Регистратор через который мы можем подписыаться на какие-то события в сокетах÷
    registry: Registry,
}

impl Reactor {
    /// Создаем новый реактор
    pub(crate) fn new() -> Result<Self, std::io::Error> {
        // Создаем новый полер
        let poll = Poll::new()?;

        // Получаем регистри
        let registry = poll.registry().try_clone()?;

        // Создаем слаб-аллокатор дя наших waker объектов
        let wakers = Arc::new(Slab::new());

        // Стартуем отдельный поток обработки полинга событий
        thread::Builder::new().name("reactor".into()).spawn({
            // Для этого потока шарим Arc вейкера
            let wakers = Arc::clone(&wakers);

            move || {
                // отдельным потоком полим события из сети
                poll_events_routine(wakers, poll)
            }
        })?;

        Ok(Self { registry, wakers })
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Регистрируем интересующие нас событи для конкретного источнка.
    /// Передаем в виде параметра нужный waker, который будет будить футуру.
    pub(crate) fn register<S>(
        &self,
        source: &mut S,
        interests: Interest,
        waker: Waker,
    ) -> io::Result<Token>
    where
        S: Source,
    {
        // Сохраняем waker в slab, в ответ получаем ключ по которому можно будет ссылаться.
        let token = self
            .wakers
            .insert(waker)
            .ok_or(Error::other("slab queue is full"))?;

        // В качестве mio токена как раз у нас будет выступать этот самый ключ
        let token = Token(token);

        // Теперь регистрируем для данного истоника этот токен и его желаемые интереные события
        self.registry.register(source, token, interests)?;

        Ok(token)
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Повторно регистрируем существующий какой-то источник, который уже был зарегистрирован ранее
    pub(crate) fn reregister<S>(
        &self,
        token: Token,
        source: &mut S,
        interests: Interest,
        waker: Waker,
    ) -> io::Result<Token>
    where
        S: Source,
    {
        // Из текущего slab мы удаляем токен
        self.wakers.remove(token.into());

        // Затем мы заново добавляем waker для пробуждения в slab
        // и получаем новый токен для работы
        let new_token = Token(
            self.wakers
                .insert(waker)
                .ok_or(Error::other("slab queue is full"))?,
        );

        // Перерегистрируем теперь уже с новым токеном этот источник и его события
        self.registry.reregister(source, new_token, interests)?;

        Ok(new_token)
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Снимаем регистрацию для указанного источника и его токена
    pub(crate) fn deregister<S>(&self, token: Token, source: &mut S) -> io::Result<()>
    where
        S: Source,
    {
        // Снимаем с регистрации ожидание событий на данном токене
        self.registry.deregister(source)?;
        // Убираем из slab данный токен
        self.wakers.get(token.0);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

fn poll_events_routine(wakers: Arc<Slab<Waker>>, mut poll: Poll) {
    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.into_iter() {
            if let Some(waker) = wakers.get(event.token().into()) {
                // Call waker interested by this event
                waker.wake_by_ref();
            }
        }
    }
}
