use failure::Fail;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::result::Result as StdResult;

// Интерфейс, который должен реализовать системный компонент
pub trait System<'context>: Sized + 'context {
    // Типы, которые должна описать унаследованная реализация
    type Dependencies;
    type Error: Fail;

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Методы, которые надо реализовать
    ////////////////////////////////////////////////////////////////////////////////////////////
    fn debug_name() -> &'static str;
    fn create(dependencies: Self::Dependencies) -> StdResult<Self, Self::Error>;

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Стандартные реализации методов
    ////////////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    fn setup(&mut self, _dependencies: Self::Dependencies) -> StdResult<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn update(&mut self, _dependencies: Self::Dependencies) -> StdResult<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn teardown(&mut self, _dependencies: Self::Dependencies) -> StdResult<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn destroy(self, _dependencies: Self::Dependencies) -> StdResult<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn bind() -> BoundSystem<Self> {
        BoundSystem(PhantomData)
    }
}

// Трейт надежной системы
pub trait InfallibleSystem<'context>: Sized + 'context {
    type Dependencies;

    // Имя
    fn debug_name() -> &'static str;

    // Создание
    fn create(dependencies: Self::Dependencies) -> Self;

    // Настройка
    #[inline]
    fn setup(&mut self, _dependencies: Self::Dependencies) {}

    #[inline]
    fn update(&mut self, _dependencies: Self::Dependencies) {}

    #[inline]
    fn teardown(&mut self, _dependencies: Self::Dependencies) {}

    #[inline]
    fn destroy(self, _dependencies: Self::Dependencies) {}
}

pub type AlwaysOk<T> = StdResult<T, NoError>;

#[derive(Debug)]
pub enum NoError {}

impl Display for NoError {
    fn fmt(&self, _: &mut Formatter) -> FmtResult {
        unreachable!();
    }
}

impl Fail for NoError {}

impl<'context, SystemT> System<'context> for SystemT
where
    Self: InfallibleSystem<'context>,
{
    type Dependencies = <Self as InfallibleSystem<'context>>::Dependencies;
    type Error = NoError;

    #[inline]
    fn debug_name() -> &'static str {
        <Self as InfallibleSystem>::debug_name()
    }

    #[inline]
    fn create(dependencies: Self::Dependencies) -> AlwaysOk<Self> {
        Ok(<Self as InfallibleSystem>::create(dependencies))
    }

    #[inline]
    fn setup(&mut self, dependencies: Self::Dependencies) -> AlwaysOk<()> {
        <Self as InfallibleSystem>::setup(self, dependencies);
        Ok(())
    }

    #[inline]
    fn update(&mut self, dependencies: Self::Dependencies) -> AlwaysOk<()> {
        <Self as InfallibleSystem>::update(self, dependencies);
        Ok(())
    }

    #[inline]
    fn teardown(&mut self, dependencies: Self::Dependencies) -> AlwaysOk<()> {
        <Self as InfallibleSystem>::teardown(self, dependencies);
        Ok(())
    }

    #[inline]
    fn destroy(self, dependencies: Self::Dependencies) -> AlwaysOk<()> {
        <Self as InfallibleSystem>::destroy(self, dependencies);
        Ok(())
    }
}

// Вспомогательная структура, которая описывает связанный компонент системы
// PhantomData нужна для того, чтобы описать некоторый объект, который ведет себя так же, как параметр в шаблоне,
// но при этом не содержит сам объект. То есть - некторая заглушка
pub struct BoundSystem<SystemT>(PhantomData<SystemT>);
