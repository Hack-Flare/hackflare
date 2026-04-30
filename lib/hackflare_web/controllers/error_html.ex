defmodule HackflareWeb.ErrorHTML do
  use HackflareWeb, :html

  embed_templates "error_html/*", suffix: "_html"

  def render(template, assigns) do
    conn = assigns[:conn]

    info =
      if conn do
        HackflareWeb.RequestInfo.extract(conn)
      else
        %{}
      end

    assigns = Map.merge(assigns, info)

    template
    |> String.trim_trailing(".html")
    |> Kernel.<>("_html")
    |> String.to_existing_atom()
    |> then(&apply(__MODULE__, &1, [assigns]))
  end
end
