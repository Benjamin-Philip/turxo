# Turxo

Turxo is early-stage Elixir bindings for the Turso database, currently offering
direct access to Turso via Rust-powered NIFs. The package provides low-level,
**raw** functions for opening, connecting, and executing SQL on Turso, along
with statement preparation and reuse.

This is a proof-of-concept release (`v0.1.0`). There are no ergonomic APIs,
documentation, or DBConnection/Ecto integration yet — those are planned for
future releases and in separate repositories.

**Project status:**

- Early alpha, APIs are unstable
- Only low-level NIF wrappers exposed
- Limited documentation

# Supported Features

- Open or create a Turso database, including in-memory databases
- Connect to an open database
- Execute SQL commands or perform queries on a connection
- Prepare reusable SQL statements, then execute or query them

# Installation

Add the package from Hex to your `mix.exs` (since `v0.1.0`):

``` elixir
def deps do
  [
    {:turxo, "~> 0.1.0"}
  ]
end
```

And run:

``` shell
mix deps.get
```

# Example

This example demonstrates all currently supported functions using the
`Turxo.NIF.Wrapped` module.

``` elixir
alias Turxo.NIF.Wrapped, as: Turso

# Open an in-memory database
{:ok, db} = Turso.db_open(":memory:")

# Establish a connection
{:ok, conn} = Turso.db_connect(db)

# Execute SQL (no parameters)
{:ok, 0} =
  Turso.conn_execute(
    conn,
    "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
    []
  )

# Insert using positional parameters
{:ok, 1} =
  Turso.conn_execute(conn, "INSERT INTO users (name, email) VALUES (?1, ?2)", [
    "alice",
    "alice@example.com"
  ])

# Query with positional parameters
{:ok, [["alice@example.com"]]} =
  Turso.conn_query(conn, "SELECT email FROM users WHERE name = (?1)", ["alice"])

# Prepare a statement
{:ok, stmt} =
  Turso.conn_prepare(conn, "INSERT INTO users (name, email) VALUES (:name, :email)", false)

# Execute the prepared statement with named parameters
{:ok, 1} = Turso.stmt_execute(stmt, name: "bob", email: "bob@example.com")

# Query with named parameters
{:ok, [["bob@example.com"]]} =
  Turso.conn_query(conn, "SELECT email FROM users WHERE name = (:name)", name: "bob")

# Prepare a statement for querying
{:ok, qstmt} = Turso.conn_prepare(conn, "SELECT id, name FROM users", false)
{:ok, rows} = Turso.stmt_query(qstmt, [])
# rows = [[1, "alice"], [2, "bob"]]
```

# Roadmap

- Elixir-style API (DBConnection integration)
- Ecto adapter (separate repository)
- Extended documentation and guides

# Copyright

Copyright (c) 2026 Benjamin Philip, MIT License
