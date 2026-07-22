defmodule Turxo.MixProject do
  use Mix.Project

  @source "https://github.com/Benjamin-Philip/turxo"

  def project do
    [
      app: :turxo,
      version: "0.1.0",
      elixir: "~> 1.18",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      name: "Turxo",
      description: "Elixir driver for the Turso database",
      source_url: @source,
      homepage_url: @source,
      docs: &docs/0,
      package: &package/0
    ]
  end

  defp docs do
    [
      main: "readme",
      extras: ["README.md", "CHANGELOG.md", "LICENSE"]
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger]
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      # {:dep_from_hexpm, "~> 0.3.0"},
      # {:dep_from_git, git: "https://github.com/elixir-lang/my_dep.git", tag: "0.1.0"},
      # {:sibling_app_in_umbrella, in_umbrella: true}
      {:rustler, "~> 0.38.0"},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false, warn_if_outdated: true}
    ]
  end

  defp package() do
    [
      name: "turxo",
      files: ~w(lib native priv .formatter.exs mix.exs README.md LICENSE
          CHANGELOG.md src),
      licenses: ["MIT"],
      links: %{"GitHub" => @source}
    ]
  end
end
