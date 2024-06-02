//! Поддержка для создания футур, которые представляют таймауты.
//!
//! Данный модуль содержит объект `Delay`, который является футурой, которая резолвится в определенный момент времени.

////////////////////////////////////////////////////////////////////////////////

use super::{arc_list::Node, AtomicWaker, ScheduledTimer, TimerHandle};
use std::{
    fmt,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc, Mutex,
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};

////////////////////////////////////////////////////////////////////////////////

/// Данная футура представляет собой уведомление, что определенная длительность была завершена.
///
/// Не обеспечивает идеальную точность времени исполнения.
pub struct Delay {
    /// Текущее состояние
    state: Option<Arc<Node<ScheduledTimer>>>,
}

impl Delay {
    /// Возвращаемый объект будет привязан к стандартному таймеру для данного потока.
    /// Таймер будет запущен во вспомогательном потоке при первом использовании.
    #[inline]
    pub fn new(dur: Duration) -> Delay {
        Delay::new_handle(Instant::now() + dur, Default::default())
    }

    /// Создаем футуру, которая будет исполнена в какой-то момент времени.
    ///
    /// Возвращаемый инстанс будет привязан к таймеру, указанному в виде handle аргумента.
    pub(crate) fn new_handle(at: Instant, handle: TimerHandle) -> Delay {
        // Пробуем получить реальнй хендл
        let inner = match handle.inner.upgrade() {
            Some(i) => i,
            None => return Delay { state: None },
        };

        // Создаем новый таймер
        let state = Arc::new(Node::new(ScheduledTimer {
            // Время запуска
            at: Mutex::new(Some(at)),
            // Текущее состояние
            state: AtomicUsize::new(0),
            // Пробуждалка атомарная
            waker: AtomicWaker::new(),
            // Сохраняем weak ссылку
            inner: handle.inner,
            // Слот
            slot: Mutex::new(None),
        }));

        // If we fail to actually push our node then we've become an inert
        // timer, meaning that we'll want to immediately return an error from
        // `poll`.
        //
        // Пробуем сохранить теперь этот элемент в ArcList
        if inner.list.push(&state).is_err() {
            return Delay { state: None };
        }

        // Помечаем внутренний waker для пробуждения сразу же
        inner.waker.wake();

        Delay { state: Some(state) }
    }

    /// Resets this timeout to an new timeout which will fire at the time
    /// specified by `at`.
    #[inline]
    pub fn reset(&mut self, dur: Duration) {
        if self._reset(dur).is_err() {
            self.state = None
        }
    }

    fn _reset(&mut self, dur: Duration) -> Result<(), ()> {
        let state = match self.state {
            Some(ref state) => state,
            None => return Err(()),
        };
        if let Some(timeouts) = state.inner.upgrade() {
            let mut bits = state.state.load(SeqCst);
            loop {
                // If we've been invalidated, cancel this reset
                if bits & 0b10 != 0 {
                    return Err(());
                }
                let new = bits.wrapping_add(0b100) & !0b11;
                match state.state.compare_exchange(bits, new, SeqCst, SeqCst) {
                    Ok(_) => break,
                    Err(s) => bits = s,
                }
            }
            *state.at.lock().unwrap() = Some(Instant::now() + dur);
            // If we fail to push our node then we've become an inert timer, so
            // we'll want to clear our `state` field accordingly
            timeouts.list.push(state)?;
            timeouts.waker.wake();
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = match self.state {
            Some(ref state) => state,
            None => panic!("timer has gone away"),
        };

        if state.state.load(SeqCst) & 1 != 0 {
            return Poll::Ready(());
        }

        state.waker.register(cx.waker());

        // Now that we've registered, do the full check of our own internal
        // state. If we've fired the first bit is set, and if we've been
        // invalidated the second bit is set.
        match state.state.load(SeqCst) {
            n if n & 0b01 != 0 => Poll::Ready(()),
            n if n & 0b10 != 0 => panic!("timer has gone away"),
            _ => Poll::Pending,
        }
    }
}

impl Drop for Delay {
    fn drop(&mut self) {
        let state = match self.state {
            Some(ref s) => s,
            None => return,
        };
        if let Some(timeouts) = state.inner.upgrade() {
            *state.at.lock().unwrap() = None;
            if timeouts.list.push(state).is_ok() {
                timeouts.waker.wake();
            }
        }
    }
}

impl fmt::Debug for Delay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Delay").finish()
    }
}
