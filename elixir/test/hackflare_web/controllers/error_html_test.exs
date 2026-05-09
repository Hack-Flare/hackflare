defmodule HackflareWeb.ErrorHTMLTest do
  use HackflareWeb.ConnCase, async: true

  # Bring render_to_string/4 for testing custom views
  import Phoenix.Template, only: [render_to_string: 4]

  test "renders 404.html" do
    assert render_to_string(HackflareWeb.ErrorHTML, "404", "html", []) =~ "404"
    assert render_to_string(HackflareWeb.ErrorHTML, "404", "html", []) =~ "Page not found"
  end

  test "renders 500.html" do
    assert render_to_string(HackflareWeb.ErrorHTML, "500", "html", []) =~ "500"
    assert render_to_string(HackflareWeb.ErrorHTML, "500", "html", []) =~ "Internal Server Error"
  end
end
