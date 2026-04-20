defmodule HackflareWeb.Layouts do
  @moduledoc """
  This module holds layouts and related functionality
  used by your application.
  """
  use HackflareWeb, :html

  # Embed all files in layouts/* within this module.
  # The default root.html.heex file contains the HTML
  # skeleton of your application, namely HTML headers
  # and other static content.
  embed_templates "layouts/*"

  @doc """
  Renders your app layout.

  This function is typically invoked from every template,
  and it often contains your application menu, sidebar,
  or similar.

  ## Examples

      <Layouts.app flash={@flash}>
        <h1>Content</h1>
      </Layouts.app>

  """
  attr :flash, :map, required: true, doc: "the map of flash messages"

  attr :current_scope, :map,
    default: nil,
    doc: "the current [scope](https://hexdocs.pm/phoenix/scopes.html)"

  slot :inner_block, required: true

  def app(assigns) do
    ~H"""
    <header class="navbar px-4 sm:px-6 lg:px-8 bg-gradient-to-r from-primary to-secondary shadow-md">
      <div class="flex-1">
        <a href="/" class="flex-1 flex w-fit items-center gap-3 hover:opacity-80 transition-opacity">
          <img src={~p"/images/logo.svg"} width="48" />
          <span class="font-bold text-xl text-primary-content hidden sm:inline">HackFlare</span>
        </a>
      </div>
      <div class="flex-none gap-4">
        <ul class="flex gap-4 items-center">
          <li>
            <a href="https://github.com/Hack-Flare/hackflare" class="btn btn-ghost btn-circle hover:bg-base-200">
              <img src={~p"/images/github.svg"} width="24" />
            </a>
          </li>
          <li>
            <.auth_menu current_scope={@current_scope} />
          </li>
        </ul>
      </div>
    </header>

    <main class="min-h-screen">
      <div class="px-4 py-12 sm:px-6 lg:px-8">
        <div class="mx-auto max-w-4xl">
          {render_slot(@inner_block)}
        </div>
      </div>
    </main>

    <.flash_group flash={@flash} />
    """
  end

  @doc """
  Shows the flash group with standard titles and content.

  ## Examples

      <.flash_group flash={@flash} />
  """
  attr :flash, :map, required: true, doc: "the map of flash messages"
  attr :id, :string, default: "flash-group", doc: "the optional id of flash container"

  def flash_group(assigns) do
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
    <div class="card relative flex flex-row items-center border-2 border-base-300 bg-base-300 rounded-full">
      <div class="absolute w-1/3 h-full rounded-full border-1 border-base-200 bg-base-100 brightness-200 left-0 [[data-theme=light]_&]:left-1/3 [[data-theme=dark]_&]:left-2/3 transition-[left]" />

      <button
        class="flex p-2 cursor-pointer w-1/3"
        phx-click={JS.dispatch("phx:set-theme")}
        data-phx-theme="system"
      >
        <.icon name="hero-computer-desktop-micro" class="size-4 opacity-75 hover:opacity-100" />
      </button>

      <button
        class="flex p-2 cursor-pointer w-1/3"
        phx-click={JS.dispatch("phx:set-theme")}
        data-phx-theme="light"
      >
        <.icon name="hero-sun-micro" class="size-4 opacity-75 hover:opacity-100" />
      </button>

      <button
        class="flex p-2 cursor-pointer w-1/3"
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
  attr :current_scope, :map, default: nil, doc: "the current [scope](https://hexdocs.pm/phoenix/scopes.html)"
  def auth_menu(assigns) do
    ~H"""
    <a href="dash/" class="btn btn-sm btn-ghost btn-circle">
      <.icon name="hero-user" class="size-5" />
    </a>
    """
  end

  @doc """
  Renders the homepage content using the app layout.
  """
  attr :flash, :map, default: %{}, doc: "the map of flash messages"

  def home(assigns) do
    ~H"""
    <.app flash={@flash}>
      <div class="space-y-16">
        <!-- Hero Section -->
        <section class="text-center space-y-8">
          <div class="space-y-4">
            <h1 class="text-5xl sm:text-7xl font-extrabold bg-gradient-to-r from-primary via-secondary to-accent bg-clip-text text-transparent">
              HackFlare
            </h1>
            <p class="text-xl sm:text-2xl text-base-content/70 font-light">
              Cloudflare alternative for HackClub
            </p>
          </div>

          <p class="text-base sm:text-lg text-base-content/60 max-w-2xl mx-auto leading-relaxed">
            Take control of your DNS and content delivery. Simple, powerful, and built for HackClub members.
          </p>

          <div class="flex gap-4 justify-center pt-4">
            <a href="dash/" class="btn btn-primary btn-lg gap-2">
              <span>Get Started</span>
              <.icon name="hero-arrow-right" class="w-5 h-5" />
            </a>
            <a href="https://github.com/Hack-Flare/hackflare" class="btn btn-outline btn-lg gap-2">
              <.icon name="hero-star" class="w-5 h-5" />
              <span>Star on GitHub</span>
            </a>
          </div>
        </section>

        <!-- Features Section -->
        <section class="space-y-8">
          <h2 class="text-3xl sm:text-4xl font-bold text-center">
            Why HackFlare?
          </h2>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="card bg-base-300 shadow-lg hover:shadow-xl transition-shadow hover:bg-base-200">
              <div class="card-body space-y-3">
                <div class="flex justify-center">
                  <div class="badge badge-primary badge-lg">
                    <.icon name="hero-bolt" class="w-5 h-5" />
                  </div>
                </div>
                <h3 class="card-title text-center justify-center">Fast & Reliable</h3>
                <p class="text-center text-sm">Lightning-fast DNS resolution and global content delivery network.</p>
              </div>
            </div>

            <div class="card bg-base-300 shadow-lg hover:shadow-xl transition-shadow hover:bg-base-200">
              <div class="card-body space-y-3">
                <div class="flex justify-center">
                  <div class="badge badge-secondary badge-lg">
                    <.icon name="hero-shield-check" class="w-5 h-5" />
                  </div>
                </div>
                <h3 class="card-title text-center justify-center">Secure</h3>
                <p class="text-center text-sm">Built-in DDoS protection and advanced security features.</p>
              </div>
            </div>

            <div class="card bg-base-300 shadow-lg hover:shadow-xl transition-shadow hover:bg-base-200">
              <div class="card-body space-y-3">
                <div class="flex justify-center">
                  <div class="badge badge-accent badge-lg">
                    <.icon name="hero-cog" class="w-5 h-5" />
                  </div>
                </div>
                <h3 class="card-title text-center justify-center">Easy to Use</h3>
                <p class="text-center text-sm">Intuitive dashboard for managing your domains and settings.</p>
              </div>
            </div>
          </div>
        </section>

        <!-- CTA Section -->
        <section class="card bg-base-300 border-2 border-primary/40">
          <div class="card-body text-center space-y-4">
            <h2 class="card-title justify-center text-2xl sm:text-3xl">
              Ready to get started?
            </h2>
            <p class="text-base-content/70">
              Join the HackClub community and start managing your DNS today.
            </p>
            <div class="card-actions justify-center pt-4">
              <a href="dash/" class="btn btn-primary btn-wide">
                Launch Dashboard
              </a>
            </div>
          </div>
        </section>
      </div>
    </.app>
    """
  end
end
