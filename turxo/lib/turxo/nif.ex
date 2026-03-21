defmodule Turxo.NIF do
  use Rustler, otp_app: :turxo, crate: :turxo_nif

  def build_db(_path), do: nif_error()
  defp nif_error, do: :erlang.nif_error(:nif_not_loaded)
end
