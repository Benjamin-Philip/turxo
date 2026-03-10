defmodule Turxo.NIF do
  use Rustler, otp_app: :turxo, crate: :turxo_nif

  def add(_a, _b), do: nif_error()

  defp nif_error, do: :erlang.nif_error(:nif_not_loaded)
end
