defmodule Turxo.NIF do
  use Rustler, otp_app: :turxo, crate: :turxo_nif

  @timeout 5_000

  def db_open(_path), do: nif_error()
  def db_connect(_db), do: nif_error()
  def conn_execute(_conn, _sql, _params), do: nif_error()
  defp nif_error, do: :erlang.nif_error(:nif_not_loaded)

  def wrap(fun, args) do
    ref = apply(__MODULE__, fun, args)

    receive do
      {^ref, value} ->
        value
    after
      @timeout ->
        {:error, "response not received"}
    end
  end
end
