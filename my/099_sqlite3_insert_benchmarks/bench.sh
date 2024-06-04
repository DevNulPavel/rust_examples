export TZ=":Europe/Moscow"

# rm -rf *.db *.db-shm *.db-wal
# rm -rf sled_db*/
# rm -rf postgres_db*/
# rm -rf mysql_db*/

# Просто запросы вообще без открытой транзакции, очень долго работает
# cargo build --release --quiet --bin basic_no_tr
# echo "$(date)" "[RUST] basic_no_tr.rs (10_000) inserts"
# /usr/bin/time ./target/release/basic_no_tr

# # Бенчмарк периодического открытия и закрытия транзакции SQLite
cargo build --release --quiet --bin basic_raw_tr
echo "$(date)" "[RUST] basic_raw_tr.rs (300_000_000) inserts"
/usr/bin/time ./target/release/basic_raw_tr

# # Просто запросы в пределах транзакции
# cargo build --release --quiet --bin basic
# echo "$(date)" "[RUST] basic.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic

# # Каждый запрос состоит из 50ти строк
# cargo build --release --quiet --bin basic_batched
# echo "$(date)" "[RUST] basic_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_batched

# # Каждый запрос заранее дополнительно был подготовлен и закеширован
# cargo build --release --quiet --bin basic_prep
# echo "$(date)" "[RUST] basic_prep.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_prep

# # Каждый запрос каждый раз кешируется при подготовке + транзакции периодически завершаются для сброса
# cargo build --release --quiet --bin basic_cached_prep_raw_tr
# echo "$(date)" "[RUST] basic_cached_prep_raw_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_cached_prep_raw_tr

# # Каждый запрос заранее дополнительно был подготовлен и закеширован + транзакции периодически завершаются для сброса
# cargo build --release --quiet --bin basic_prep_raw_tr
# echo "$(date)" "[RUST] basic_prep_raw_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_prep_raw_tr

# # Каждый запрос заранее дополнительно был подготовлен и закеширован + транзакции периодически завершаются для сброса
# cargo build --release --quiet --bin basic_deadpool
# echo "$(date)" "[RUST] basic_deadpool.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_deadpool

# # Каждый запрос заранее дополнительно был подготовлен и закеширован + транзакции периодически завершаются для сброса
# cargo build --release --bin basic_async_actor
# echo "$(date)" "[RUST] basic_async_actor.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_async_actor

# # Каждый запрос заранее дополнительно был подготовлен и закеширован + транзакции периодически завершаются для сброса
# cargo build --release --bin basic_async_actor_mp
# echo "$(date)" "[RUST] basic_async_actor_mp.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_async_actor_mp

# Sled insert
# cargo build --release --quiet --bin sled
# echo "$(date)" "[RUST] sled.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/sled

# Sled tr insert
# cargo build --release --quiet --bin sled_tr
# echo "$(date)" "[RUST] sled_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/sled_tr

# # Sled batch insert
# cargo build --release --quiet --bin sled_batch
# echo "$(date)" "[RUST] sled_batch.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/sled_batch

# # Каждый запрос заранее подготовлен, но каждый запрос состоит из 50ти строк на добавление
# cargo build --release --quiet --bin basic_prep_batched
# echo "$(date)" "[RUST] basic_prep_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_prep_batched

# # just like the previous version, so really bad.
# cargo build --release --quiet --bin threaded_str_batched
# echo "$(date)" "[RUST] threaded_str_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/threaded_str_batched

# # previous version but threaded
# cargo build --release --quiet --bin threaded_batched
# echo "$(date)" "[RUST] threaded_batched.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/threaded_batched

# # benching with all prev sqlite optimisations, but on rust with sqlx async
# cargo build --release --quiet --bin basic_async
# echo "$(date)" "[RUST] basic_async.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/basic_async

# Референс для хешмапы
# cargo build --release --quiet --bin hash_map
# echo "$(date)" "[RUST] hash_map.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/hash_map

# Postgres ASYNC insert transaction
# cargo build --release --quiet --bin postgres_async_tr
# echo "$(date)" "[RUST] postgres_async_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/postgres_async_tr

# Postgres SYNC insert transaction
# cargo build --release --quiet --bin postgres_tr
# echo "$(date)" "[RUST] postgres_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/postgres_tr

# MySQL SYNC insert transaction
# cargo build --release --quiet --bin mysql_tr
# echo "$(date)" "[RUST] mysql_tr.rs (1_000_000) inserts"
# /usr/bin/time ./target/release/mysql_tr

du -h -d 0 sled_db*/
# rm -rf sled_db*/

du -h -d 0 postgres_db*/
# rm -rf postgres_db*/

du -h -d 0 mysql_db*/
# rm -rf mysql_db*/