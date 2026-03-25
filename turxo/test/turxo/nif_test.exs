defmodule Turxo.NIFTest do
  use ExUnit.Case, async: true

  alias Turxo.NIF

  describe "build_db/1" do
    test "can build in memory" do
      ref = NIF.build_db(":memory:")
      assert is_reference(ref)

      assert_receive({^ref, {:ok, db}})
      assert is_reference(db)
    end

    @tag :tmp_dir
    test "can build a database file", %{tmp_dir: dir} do
      path = "#{dir}/test.db"
      ref = NIF.build_db(path)
      assert is_reference(ref)

      assert_receive({^ref, {:ok, db}})
      assert is_reference(db)
      assert File.exists?(path)
    end

    test "can safely handle invalid paths" do
      ref = NIF.build_db("/invalid/path/")
      assert is_reference(ref)

      assert_receive({^ref, {:error, "I/O error (open): entity not found"}})
    end
  end

  test "connect_db/1" do
    {:ok, db} = NIF.wrap(:build_db, [":memory:"])
    ref = NIF.connect_db(db)

    assert_receive({^ref, {:ok, conn}})
    assert is_reference(conn)
  end

  describe "conn_execute/3 correctly handles" do
    setup do
      {:ok, db} = NIF.wrap(:build_db, [":memory:"])
      {:ok, conn} = NIF.wrap(:connect_db, [db])

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
    test "build_db/1" do
      assert {:ok, db} = NIF.wrap(:build_db, [":memory:"])
      assert is_reference(db)
    end

    test "connect_db/1" do
      {:ok, db} = NIF.wrap(:build_db, [":memory:"])

      assert {:ok, conn} = NIF.wrap(:connect_db, [db])
      assert is_reference(conn)
    end
  end
end
