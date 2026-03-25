defmodule Turxo.NIFTest do
  use ExUnit.Case, async: true

  alias Turxo.NIF

  describe "db_open/1" do
    test "can build in memory" do
      ref = NIF.db_open(":memory:")
      assert is_reference(ref)

      assert_receive({^ref, {:ok, db}})
      assert is_reference(db)
    end

    @tag :tmp_dir
    test "can build a database file", %{tmp_dir: dir} do
      path = "#{dir}/test.db"
      ref = NIF.db_open(path)
      assert is_reference(ref)

      assert_receive({^ref, {:ok, db}})
      assert is_reference(db)
      assert File.exists?(path)
    end

    test "can safely handle invalid paths" do
      ref = NIF.db_open("/invalid/path/")
      assert is_reference(ref)

      assert_receive({^ref, {:error, "I/O error (open): entity not found"}})
    end
  end

  test "db_connect/1" do
    {:ok, db} = NIF.wrap(:db_open, [":memory:"])
    ref = NIF.db_connect(db)

    assert_receive({^ref, {:ok, conn}})
    assert is_reference(conn)
  end

  describe "conn_execute/3 correctly handles" do
    setup do
      {:ok, db} = NIF.wrap(:db_open, [":memory:"])
      {:ok, conn} = NIF.wrap(:db_connect, [db])

      {:ok, 0} =
        NIF.wrap(:conn_execute, [
          conn,
          "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
          []
        ])

      %{db: db, conn: conn}
    end

    test "no parameters", %{conn: conn} do
      ref =
        NIF.conn_execute(
          conn,
          "CREATE TABLE students (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
          []
        )

      assert_receive({^ref, {:ok, 0}})
    end

    test "positional parameters", %{conn: conn} do
      ref =
        NIF.conn_execute(
          conn,
          "INSERT INTO users (name, email) VALUES (?1, ?2)",
          ["Alice", "alice@example.com"]
        )

      assert_receive({^ref, {:ok, 1}})
    end

    test "named parameters", %{conn: conn} do
      ref =
        NIF.conn_execute(
          conn,
          "INSERT INTO users (name, email) VALUES (:name, :email)",
          name: "Alice",
          email: "alice@example.com"
        )

      assert_receive({^ref, {:ok, 1}})

      ref =
        NIF.conn_execute(
          conn,
          "INSERT INTO users (name, email) VALUES (:name, :email)",
          name: "Alice"
        )

      assert_receive({^ref, {:ok, 1}})
    end
  end

  describe "wrap/2 correctly wraps" do
    test "db_open/1" do
      assert {:ok, db} = NIF.wrap(:db_open, [":memory:"])
      assert is_reference(db)
    end

    test "db_connect/1" do
      {:ok, db} = NIF.wrap(:db_open, [":memory:"])

      assert {:ok, conn} = NIF.wrap(:db_connect, [db])
      assert is_reference(conn)
    end
  end
end
