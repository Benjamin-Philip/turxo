defmodule TurxoTest do
  use ExUnit.Case
  doctest Turxo

  test "greets the world" do
    assert Turxo.hello() == :world
  end
end
