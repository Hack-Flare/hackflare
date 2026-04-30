defmodule HackflareWeb.RequestInfo do
  import Plug.Conn

  def extract(conn) do
    %{
      client_ip: format_ip(conn.remote_ip),
      method: conn.method || "unknown",
      path: conn.request_path || "unknown",
      request_id: List.first(get_resp_header(conn, "x-request-id")) || "n/a",
      time: DateTime.utc_now() |> DateTime.to_string()
    }
  end

  defp format_ip(nil), do: "unknown"
  defp format_ip(ip), do: ip |> Tuple.to_list() |> Enum.join(".")
end
