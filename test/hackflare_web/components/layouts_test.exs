defmodule HackflareWeb.LayoutsTest do
  use HackflareWeb.ConnCase, async: true

  import Phoenix.LiveViewTest

  test "domains dashboard shows re-verify action for unverified zones" do
    html =
      render_component(&HackflareWeb.Layouts.dashboard/1,
        current_view: :domains,
        current_user: nil,
        zones: [%{name: "example.com", ns_verified: false}]
      )

    assert html =~ "Re-verify"
    assert html =~ "Waiting for nameserver verification"
  end
end
