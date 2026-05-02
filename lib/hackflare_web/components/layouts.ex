defmodule HackflareWeb.Layouts do
  @moduledoc """
  Shared HTML components and helpers for the web UI.
  """

  use HackflareWeb, :html

  embed_templates "layouts/*"

  @doc """
  Shows the flash group with standard titles and content.

  ## Examples

      <.flash_group flash={@flash} />
  """
  def flash_group(assigns) do
    assigns = assign_new(assigns, :id, fn -> "flash-group" end)

    ~H"""
    <div id={@id} aria-live="polite">
      <.flash kind={:info} flash={@flash} />
      <.flash kind={:error} flash={@flash} />

      <.flash
        id="client-error"
        kind={:error}
        title={gettext("We can't find the internet")}
        phx-disconnected={show(".phx-client-error #client-error") |> JS.remove_attribute("hidden")}
        phx-connected={hide("#client-error") |> JS.set_attribute({"hidden", ""})}
        hidden
      >
        {gettext("Attempting to reconnect")}
        <.icon name="hero-arrow-path" class="ml-1 size-3 motion-safe:animate-spin" />
      </.flash>

      <.flash
        id="server-error"
        kind={:error}
        title={gettext("Something went wrong!")}
        phx-disconnected={show(".phx-server-error #server-error") |> JS.remove_attribute("hidden")}
        phx-connected={hide("#server-error") |> JS.set_attribute({"hidden", ""})}
        hidden
      >
        {gettext("Attempting to reconnect")}
        <.icon name="hero-arrow-path" class="ml-1 size-3 motion-safe:animate-spin" />
      </.flash>
    </div>
    """
  end

  @doc """
  Provides dark vs light theme toggle based on themes defined in app.css.

  See <head> in root.html.heex which applies the theme before page load.
  """
  def theme_toggle(assigns) do
    ~H"""
    <div class="card relative flex flex-row items-center rounded-full border-2 border-base-300 bg-base-300">
      <div class="absolute left-0 h-full w-1/3 rounded-full border-1 border-base-200 bg-base-100 brightness-200 transition-[left] [[data-theme=light]_&]:left-1/3 [[data-theme=dark]_&]:left-2/3" />

      <button
        class="flex w-1/3 cursor-pointer p-2"
        phx-click={JS.dispatch("phx:set-theme")}
        data-phx-theme="system"
      >
        <.icon name="hero-computer-desktop-micro" class="size-4 opacity-75 hover:opacity-100" />
      </button>

      <button
        class="flex w-1/3 cursor-pointer p-2"
        phx-click={JS.dispatch("phx:set-theme")}
        data-phx-theme="light"
      >
        <.icon name="hero-sun-micro" class="size-4 opacity-75 hover:opacity-100" />
      </button>

      <button
        class="flex w-1/3 cursor-pointer p-2"
        phx-click={JS.dispatch("phx:set-theme")}
        data-phx-theme="dark"
      >
        <.icon name="hero-moon-micro" class="size-4 opacity-75 hover:opacity-100" />
      </button>
    </div>
    """
  end

  @doc """
  Provides a redir to dashboard. Replace current icon with signed in icon in future.
  """
  def auth_menu(assigns) do
    ~H"""
    <a
      href="/dash"
      class="btn btn-sm rounded-full border-base-300/70 bg-base-200/70 px-4 hover:bg-base-300"
    >
      <.icon name="hero-user" class="size-5" />
    </a>
    """
  end

  @doc """
  Placeholder for the Contact section UI. Add your UI code here.
  """
  def contact(assigns) do
    ~H"""
    <div class="p-8">
      <h2 class="mb-4 text-2xl font-bold">Help</h2>
    </div>
    """
  end

  defp nav_item_class(current_view, item) when current_view == item do
    "flex items-center gap-3 rounded-lg border border-orange-500/50 bg-orange-500/20 px-4 py-3 font-semibold text-orange-400 transition-all"
  end

  defp nav_item_class(_current_view, _item) do
    "flex items-center gap-3 rounded-lg px-4 py-3 text-gray-300 transition-all hover:bg-gray-900/50 hover:text-orange-400"
  end

  defp dashboard_title(:home), do: "Dashboard"
  defp dashboard_title(:domains), do: "Domains"
  defp dashboard_title(:settings), do: "Settings"
  defp dashboard_title(:analytics), do: "Analytics"
  defp dashboard_title(:notifications), do: "Notifications"
  defp dashboard_title(:help), do: "Help"
  defp dashboard_title(:admin), do: "Admin Panel"
  defp dashboard_title(_), do: "Dashboard"

  defp dashboard_subtitle(:home), do: "Welcome to HackFlare"
  defp dashboard_subtitle(:domains), do: "Manage DNS zones and records"
  defp dashboard_subtitle(:settings), do: "Configure account and platform defaults"
  defp dashboard_subtitle(:analytics), do: "Monitor performance and traffic"
  defp dashboard_subtitle(:notifications), do: "Control alerts and updates"
  defp dashboard_subtitle(:help), do: "Find docs and support resources"
  defp dashboard_subtitle(:admin), do: "Manage users, roles, and access"
  defp dashboard_subtitle(_), do: "Welcome to HackFlare"
end
