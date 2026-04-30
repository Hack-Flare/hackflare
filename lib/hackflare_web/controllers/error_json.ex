defmodule HackflareWeb.ErrorJSON do
  def render(template, assigns) do
    conn = assigns[:conn]

    info =
      if conn do
        HackflareWeb.RequestInfo.extract(conn)
      else
        %{}
      end

    %{
      error: template,
      details: info
    }
  end
end
