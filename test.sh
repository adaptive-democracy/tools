# PGPASSWORD='asdf' psql database -U user -h localhost -f main.sql | cat

cargo test --package persistent_democracy_core test_create_constitution_tree_detach -- --show-output
