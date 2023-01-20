# PGPASSWORD='asdf' psql database -U user -h localhost -f main.sql | cat

cargo test --package persistent-democracy-core test_create_constitution_tree -- --show-output
