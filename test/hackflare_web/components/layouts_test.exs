defmodule HackflareWeb.LayoutsTest do
  use HackflareWeb.ConnCase, async: true

  import Phoenix.LiveViewTest

  test "domains dashboard shows re-verify action for unverified zones" do
    html =
      render_component(&HackflareWeb.Layouts.dashboard/1,
        flash: %{},
        current_view: :domains,
        current_user: nil,
        zones: [%{name: "example.com", ns_verified: false}],
        zone_records: %{"example.com" => []}
      )

    assert html =~ "Re-verify"
    assert html =~ "Waiting for nameserver verification"
  end
end
