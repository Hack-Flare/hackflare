defmodule HackflareWeb.ErrorHTML do
  use HackflareWeb, :html

  embed_templates "error_html/*", suffix: "_html"

  def render(template, assigns) do
    conn = assigns[:conn]
    status = String.trim_trailing(template, ".html")

    defaults = %{
      client_ip: "unknown",
      method: "unknown",
      path: "unknown",
      request_id: "n/a",
      time: "unknown"
    }

    info =
      if conn do
        HackflareWeb.RequestInfo.extract(conn)
      else
        %{}
      end

    assigns =
      assigns
      |> Map.put(:status, status)
      |> Map.merge(defaults)
      |> Map.merge(info)

    template
    |> String.trim_trailing(".html")
    |> Kernel.<>("_html")
    |> String.to_existing_atom()
    |> then(&apply(__MODULE__, &1, [assigns]))
  end
end
