# PGPASSWORD='asdf' psql database -U user -h localhost -f main.sql | cat

# cargo test --package adaptive_democracy_core
cargo test --package adaptive_democracy_core -- --show-output
# cargo check --package adaptive_democracy_core

# cargo run --package adaptive_democracy_server
