defmodule TurxoRoot.MixProject do
  use Mix.Project

  # Adapted from https://github.com/elixir-nx/nx/blob/main/mix.exs 

  def project do
    [
      app: :turxo_root,
      version: "0.1.0",
      deps: [{:ecto_turxo, path: "ecto_turxo"}, {:turxo, path: "turxo"}],
      aliases: [
        setup: cmd("deps.get"),
        compile: cmd("compile"),
        test: cmd("test")
      ]
    ]
  end

  defp cmd(command) do
    ansi = IO.ANSI.enabled?()
    base = ["--erl", "-elixir ansi_enabled #{ansi}", "-S", "mix", command]

    for app <- ~w(turxo ecto_turxo) do
      fn args ->
        {_, res} = System.cmd("elixir", base ++ args, into: IO.binstream(:stdio, :line), cd: app)

        if res > 0 do
          System.at_exit(fn _ -> exit({:shutdown, 1}) end)
        end
      end
    end
  end
end
