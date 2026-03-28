defmodule Turxo.NIFTest do
  use ExUnit.Case, async: true

  alias Turxo.NIF, as: Unwrapped
  alias Turxo.NIF.Wrapped, as: NIF

  setup do
    {:ok, db} = NIF.db_open(":memory:")
    {:ok, conn} = NIF.db_connect(db)

    {:ok, 0} =
      NIF.conn_execute(
        conn,
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
        []
      )

    data = [
      ["Alice", "alice@example.com"],
      ["Bob", nil],
      ["Charlie", "charlie@example.com"]
    ]

    for pair <- data do
      {:ok, 1} =
        NIF.conn_execute(
          conn,
          "INSERT INTO users (name, email) VALUES (:name, :email)",
          pair
        )
    end

    %{db: db, conn: conn, data: data}
  end

  test "Wrapped correctly wraps" do
    # Wrapped version of the unwrapped db_open test below.
    #
    # If the final result of both functions have the same properites,
    # then wrapping is correct.

    assert {:ok, db} = NIF.db_open(":memory:")
    assert is_reference(db)
  end

  describe "db_open/1" do
    test "can build in memory" do
      ref = Unwrapped.db_open(":memory:")
      assert is_reference(ref)

      assert_receive({^ref, {:ok, db}})
      assert is_reference(db)
    end

    @tag :tmp_dir
    test "can build a database file", %{tmp_dir: dir} do
      path = "#{dir}/test.db"

      assert {:ok, db} = NIF.db_open(path)
      assert is_reference(db)
      assert File.exists?(path)
    end

    test "can safely handle invalid paths" do
      assert {:error, "I/O error (open): entity not found"} = NIF.db_open("/invalid/path/")
    end
  end

  test "db_connect/1", %{db: db} do
    assert {:ok, conn} = NIF.db_connect(db)
    assert is_reference(conn)
  end

  describe "conn_execute/3 correctly handles" do
    test "no parameters", %{conn: conn} do
      assert {:ok, 0} =
               NIF.conn_execute(
                 conn,
                 "CREATE TABLE students (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
                 []
               )
    end

    test "positional parameters", %{conn: conn} do
      assert {:ok, 1} =
               NIF.conn_execute(
                 conn,
                 "INSERT INTO users (name, email) VALUES (?1, ?2)",
                 ["Alice", "alice@example.com"]
               )
    end

    test "named parameters", %{conn: conn} do
      assert {:ok, 1} =
               NIF.conn_execute(
                 conn,
                 "INSERT INTO users (name, email) VALUES (:name, :email)",
                 name: "Alice",
                 email: "alice@example.com"
               )

      assert {:ok, 1} =
               NIF.conn_execute(
                 conn,
                 "INSERT INTO users (name, email) VALUES (:name, :email)",
                 name: "Alice"
               )
    end
  end

  describe "conn_query/3 correctly handles" do
    test "no parameters", %{conn: conn, data: data} do
      assert {:ok, [[3]]} = NIF.conn_query(conn, "SELECT COUNT(*) FROM users", [])
      assert {:ok, ^data} = NIF.conn_query(conn, "SELECT name, email FROM users", [])
    end

    test "positional parameters", %{conn: conn} do
      assert {:ok, [[1]]} =
               NIF.conn_query(conn, "SELECT id FROM users WHERE name = (?1)", ["Alice"])

      assert {:ok, [[nil]]} =
               NIF.conn_query(conn, "SELECT email FROM users WHERE name = (?1)", ["Bob"])

      assert {:ok, [["charlie@example.com"]]} =
               NIF.conn_query(conn, "SELECT email FROM users WHERE id = (?1)", [3])
    end

    test "named parameters", %{conn: conn} do
      assert {:ok, [[1]]} =
               NIF.conn_query(
                 conn,
                 "SELECT id FROM users WHERE name = (:name)",
                 name: "Alice"
               )

      assert {:ok, [[nil]]} =
               NIF.conn_query(
                 conn,
                 "SELECT email FROM users WHERE name = (:name)",
                 name: "Bob"
               )

      assert {:ok, [["charlie@example.com"]]} =
               NIF.conn_query(conn, "SELECT email FROM users WHERE id = (:id)", id: 3)
    end
  end

  test "conn_prepare/3", %{conn: conn} do
    assert {:ok, stmt} = NIF.conn_prepare(conn, "SELECT id FROM users WHERE name = (?1)", false)
    assert is_reference(stmt)

    assert {:ok, stmt} = NIF.conn_prepare(conn, "SELECT id FROM users WHERE name = (?1)", true)
    assert is_reference(stmt)
  end
end
