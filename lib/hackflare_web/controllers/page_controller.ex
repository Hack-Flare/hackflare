defmodule HackflareWeb.PageController do
  use HackflareWeb, :controller

  def home(conn, _params) do
    render(conn, :home)
  end
end
