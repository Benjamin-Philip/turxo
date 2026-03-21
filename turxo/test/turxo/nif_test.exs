defmodule Turxo.NIFTest do
  use ExUnit.Case, async: true

  alias Turxo.NIF

  describe "build_db/1" do
    test "can build in memory" do
      ref = NIF.build_db(":memory:")
      assert is_reference(ref)

      assert_receive({^ref, {:ok, _db}})
    end

    @tag :tmp_dir
    test "can build a database file", %{tmp_dir: dir} do
      path = "#{dir}/test.db"
      ref = NIF.build_db(path)
      assert is_reference(ref)

      assert_receive({^ref, {:ok, _db}})
      assert File.exists?(path)
    end

    test "can safely handle invalid paths" do
      ref = NIF.build_db("/invalid/path/")
      assert is_reference(ref)

      assert_receive({^ref, {:error, "I/O error (open): entity not found"}})
    end
  end
end
