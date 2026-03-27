defmodule Turxo.NIF do
  use Rustler, otp_app: :turxo, crate: :turxo_nif

  db = [db_open: [:path], db_connect: [:db]]
  conn = [conn_execute: [:conn, :sql, :params], conn_query: [:conn, :sql, :params]]

  nifs = db ++ conn

  for {name, args} <- nifs do
    to_splice = Enum.map(args, fn arg -> Macro.var(arg, __MODULE__) end)
    def unquote(name)(unquote_splicing(to_splice)), do: nif_error()
  end

  defp nif_error, do: :erlang.nif_error(:nif_not_loaded)

  defmodule Wrapped do
    @timeout 5_000

    for {name, args} <- nifs do
      to_splice = Enum.map(args, fn arg -> Macro.var(arg, __MODULE__) end)

      def unquote(name)(unquote_splicing(to_splice)) do
        Turxo.NIF.unquote(name)(unquote_splicing(to_splice)) |> recieve_value
      end
    end

    defp recieve_value(ref) do
      receive do
        {^ref, value} ->
          value
      after
        @timeout ->
          {:error, "response not received"}
      end
    end
  end
end
