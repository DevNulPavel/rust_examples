export TZ=":Europe/Moscow"

# Просто запросы в пределах транзакции
rm -rf basic.db basic.db-shm basic.db-wal
cargo build --release --quiet --bin basic
echo "$(date)" "[RUST] basic.rs (1_000_000) inserts"
/usr/bin/time ./target/release/basic

# Каждый запрос заранее дополнительно был подготовлен и закеширован
rm -rf basic_prep.db basic_prep.db-shm basic_prep.db-wal
cargo build --release --quiet --bin basic_prep
echo "$(date)" "[RUST] basic_prep.rs (1_000_000) inserts"
/usr/bin/time ./target/release/basic_prep

# Каждый запрос заранее подготовлен, но каждый запрос состоит из 50ти строк на добавление
rm -rf basic_prep_batched.db basic_prep_batched.db-shm basic_prep_batched.db-wal
cargo build --release --quiet --bin basic_prep_batched
echo "$(date)" "[RUST] basic_prep_batched.rs (1_000_000) inserts"
/usr/bin/time ./target/release/basic_prep_batched

# # benching with all prev sqlite optimisations, but on rust with rusqlite with batched inserts where
# # each batch is a really large ass string
# rm -rf basic_batched_wp.db basic_batched_wp.db-shm basic_batched_wp.db-wal
# cargo build --release --quiet --bin basic_batched_wp
# echo "$(date)" "[RUST] basic_batched_wp.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_batched_wp

# # just like the previous version, so really bad.
# rm -rf threaded_str_batched.db threaded_str_batched.db-shm threaded_str_batched.db-wal
# cargo build --release --quiet --bin threaded_str_batched
# echo "$(date)" "[RUST] threaded_str_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/threaded_str_batched

# # previous version but threaded
# rm -rf threaded_batched.db threaded_batched.db-shm threaded_batched.db-wal
# cargo build --release --quiet --bin threaded_batched
# echo "$(date)" "[RUST] threaded_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/threaded_batched

# # benching with all prev sqlite optimisations, but on rust with sqlx async
# rm -rf basic_async.db basic_async.db-shm basic_async.db-wal
# cargo build --release --quiet --bin basic_async
# echo "$(date)" "[RUST] basic_async.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_async


rm -rf *.db *.db-shm *.db-wal