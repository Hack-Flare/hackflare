defmodule HackflareWeb.ErrorJSON do
  def render(template, _assigns) do
    detail =
      case String.trim_trailing(template, ".json") do
        "404" -> "Not Found"
        "500" -> "Internal Server Error"
        other -> other
      end

    %{errors: %{detail: detail}}
  end
end
