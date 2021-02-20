watch:
    cargo watch -c

test:
    cargo test

db:
    rm -f reference.db
    cat migrations/* | sqlite3 reference.db
