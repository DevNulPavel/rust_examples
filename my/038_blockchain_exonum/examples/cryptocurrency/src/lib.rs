// Copyright 2020 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Demo [Exonum][exonum] service implementing a simple cryptocurrency.
//! See [the documentation][docs] for a detailed step-by-step guide how to approach this demo,
//! and [the repository README][readme] on how to use, test, and contribute to it.
//!
//! **Note.** The service in this crate is intended for demo purposes only. It is not intended
//! for use in production.
//!
//! [exonum]: https://github.com/exonum/exonum
//! [docs]: https://exonum.com/doc/version/latest/get-started/create-service
//! [readme]: https://github.com/exonum/cryptocurrency#readme

#![deny(
    missing_debug_implementations,
    missing_docs,
    unsafe_code,
    bare_trait_objects
)]

#[macro_use]
extern crate serde_derive; // Required for Protobuf.

pub mod proto;

/// Постоянные данные
pub mod schema;

/// Транзакции
pub mod transactions;

/// Ошибки контрактов
pub mod errors;

/// Контракты
pub mod contracts;

/// Криптографическое API
pub mod api;

/// Юнит-тесты
#[cfg(test)]
mod tx_tests;

