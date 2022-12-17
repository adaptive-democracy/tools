# PGPASSWORD='asdf' psql database -U user -h localhost -f main.sql | cat

cargo test --package persistent-democracy-core play_tree -- --show-output
