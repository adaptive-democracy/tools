# PGPASSWORD='asdf' psql database -U user -h localhost -f main.sql | cat

# cargo test --package persistent_democracy_core
# cargo test --package persistent_democracy_core -- --show-output
cargo check --package persistent_democracy_core

# cargo run --package persistent_democracy_server
