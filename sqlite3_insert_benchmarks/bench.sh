export TZ=":Europe/Moscow"

# # Просто запросы вообще без открытой транзакции, очень долго работает
# rm -rf basic_no_tr.db basic_no_tr.db-shm basic_no_tr.db-wal
# cargo build --release --quiet --bin basic_no_tr
# echo "$(date)" "[RUST] basic_no_tr.rs (10_000) inserts"
# /usr/bin/time ./target/release/basic_no_tr

# # Бенчмарк периодического открытия и закрытия транзакции SQLite
# rm -rf basic_raw_tr.db basic_raw_tr.db-shm basic_raw_tr.db-wal
# cargo build --release --quiet --bin basic_raw_tr
# echo "$(date)" "[RUST] basic_raw_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_raw_tr

# # Просто запросы в пределах транзакции
# rm -rf basic.db basic.db-shm basic.db-wal
# cargo build --release --quiet --bin basic
# echo "$(date)" "[RUST] basic.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic

# # Каждый запрос состоит из 50ти строк
# rm -rf basic_batched.db basic_batched.db-shm basic_batched.db-wal
# cargo build --release --quiet --bin basic_batched
# echo "$(date)" "[RUST] basic_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_batched

# # Каждый запрос заранее дополнительно был подготовлен и закеширован
# rm -rf basic_prep.db basic_prep.db-shm basic_prep.db-wal
# cargo build --release --quiet --bin basic_prep
# echo "$(date)" "[RUST] basic_prep.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_prep

# # Каждый запрос заранее дополнительно был подготовлен и закеширован + транзакции периодически завершаются для сброса
# rm -rf basic_prep_raw_tr.db basic_prep_raw_tr.db-shm basic_prep_raw_tr.db-wal
# cargo build --release --quiet --bin basic_prep_raw_tr
# echo "$(date)" "[RUST] basic_prep_raw_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_prep_raw_tr

# # Каждый запрос заранее дополнительно был подготовлен и закеширован + транзакции периодически завершаются для сброса
# rm -rf basic_async_actor.db basic_async_actor.db-shm basic_async_actor.db-wal
# cargo build --release --bin basic_async_actor
# echo "$(date)" "[RUST] basic_async_actor.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_async_actor

# Каждый запрос заранее дополнительно был подготовлен и закеширован + транзакции периодически завершаются для сброса
rm -rf basic_async_actor_mp.db basic_async_actor_mp.db-shm basic_async_actor_mp.db-wal
cargo build --release --bin basic_async_actor_mp
echo "$(date)" "[RUST] basic_async_actor_mp.rs (1_000_000) inserts"
/usr/bin/time ./target/release/basic_async_actor_mp

# # Sled insert
# cargo build --release --quiet --bin sled
# echo "$(date)" "[RUST] sled.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/sled

# # Sled tr insert
# cargo build --release --quiet --bin sled_tr
# echo "$(date)" "[RUST] sled_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/sled_tr

# # Каждый запрос заранее подготовлен, но каждый запрос состоит из 50ти строк на добавление
# rm -rf basic_prep_batched.db basic_prep_batched.db-shm basic_prep_batched.db-wal
# cargo build --release --quiet --bin basic_prep_batched
# echo "$(date)" "[RUST] basic_prep_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_prep_batched

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
rm -rf sled_db/