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
  Renders the root layout of the application.
  """
  def root(assigns) do
    ~H"""
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta name="csrf-token" content={get_csrf_token()} />
        <.live_title>
          {assigns[:page_title] || "HackFlare"}
        </.live_title>
        <link phx-track-static rel="stylesheet" href={~p"/assets/css/app.css"} />
        <script defer phx-track-static type="text/javascript" src={~p"/assets/js/app.js"}>
        </script>
      </head>
      <body>
        {@inner_content}
      </body>
    </html>
    """
  end

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
            <a
              href="https://github.com/Hack-Flare/hackflare"
              class="btn btn-ghost btn-circle hover:bg-base-200"
            >
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
  attr :current_scope, :map,
    default: nil,
    doc: "the current [scope](https://hexdocs.pm/phoenix/scopes.html)"

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
    <!DOCTYPE html>
    <html>
      <body>
        <!-- Header -->
        <header class="fixed top-0 left-0 right-0 z-50 navbar px-4 sm:px-6 lg:px-8 bg-black/50 backdrop-blur-md shadow-md border-b border-orange-500/20">
          <div class="flex-1">
            <a
              href="/"
              class="flex-1 flex w-fit items-center gap-3 hover:opacity-80 transition-opacity"
            >
              <img src={~p"/images/logo.svg"} width="48" />
              <span class="font-bold text-xl text-orange-400 hidden sm:inline">HackFlare</span>
            </a>
          </div>
          <div class="flex-none gap-4">
            <ul class="flex gap-4 items-center">
              <li>
                <a
                  href="https://github.com/Hack-Flare/hackflare"
                  class="btn btn-ghost btn-circle hover:bg-base-200"
                >
                  <img src={~p"/images/github.svg"} width="24" />
                </a>
              </li>
              <li>
                <a href="dash/" class="btn btn-sm btn-ghost btn-circle">
                  <.icon name="hero-user" class="size-5" />
                </a>
              </li>
            </ul>
          </div>
        </header>

    <!-- Hero Section Full Screen -->
        <div class="relative h-screen bg-gradient-to-br from-gray-950 via-black to-gray-900 text-white overflow-hidden flex items-center pt-24">
          <div class="w-full">
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center max-w-7xl mx-auto px-8 sm:px-16 lg:px-24 h-full py-24">
              <!-- Left Side: Content -->
              <div class="space-y-8 z-10">
                <div class="space-y-6">
                  <h1 class="text-6xl sm:text-7xl lg:text-8xl font-black leading-tight">
                    <span class="bg-gradient-to-br from-orange-400 via-orange-500 to-orange-600 bg-clip-text text-transparent">
                      HackFlare
                    </span>
                  </h1>
                  <p class="text-2xl sm:text-3xl font-bold text-gray-200 leading-tight">
                    Built by Hack Clubbers <br /> For Hack Clubbers
                  </p>
                </div>

                <p class="text-lg text-gray-300 leading-relaxed max-w-xl">
                  A powerful DNS and content delivery platform built for the Hack Club community. Take control, stay fast, stay secure.
                </p>

                <div class="flex flex-col sm:flex-row gap-4 pt-6">
                  <a
                    href="dash/"
                    class="px-7 py-2 bg-gradient-to-r from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white font-bold rounded-lg transition-all transform hover:scale-105 shadow-lg shadow-orange-500/50 inline-flex items-center justify-center "
                  >
                    <span>Get Started</span>
                    <.icon name="hero-arrow-right" class="w-5 h-5" />
                  </a>
                  <a
                    href="/auth/request"
                    class="px-7 py-2 border-2 border-orange-500 hover:bg-orange-500/10 text-orange-400 font-bold rounded-lg transition-all inline-flex items-center justify-center gap-2"
                  >
                    <.icon name="hero-lock-closed" class="w-5 h-5" />
                    <span>Sign in with Hack Club</span>
                  </a>
                </div>

                <div class="flex flex-wrap gap-6 pt-0.5">
                  <a
                    href="/docs"
                    class="px-1 py-3 text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors"
                  >
                    <.icon name="hero-book-open" class="w-5 h-5" />
                    <span>Documentation</span>
                  </a>
                  <a
                    href="https://github.com/Hack-Flare/hackflare"
                    class="px-1 py-3 text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors"
                  >
                    <.icon name="hero-star" class="w-5 h-5" />
                    <span>Github</span>
                  </a>
                </div>
              </div>

    <!-- Right Side: Hero Image -->
              <div class="flex items-center justify-center lg:justify-end">
                <div class="relative w-full max-w-md">
                  <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-104 h-104 rounded-full bg-orange-500/40 blur-3xl pointer-events-none">
                  </div>
                  <img
                    src={~p"/images/hero-globe.png"}
                    alt="HackFlare Hero"
                    class="relative w-full h-auto rounded-lg z-10"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

    <!-- Features Section -->
        <div class="relative px-8 py-24 sm:px-16 lg:px-24 bg-gradient-to-b from-black to-gray-950">
          <div class="max-w-7xl mx-auto space-y-16">
            <div class="text-center space-y-4">
              <h2 class="text-5xl sm:text-6xl font-black">
                <span class="bg-gradient-to-r from-orange-400 to-orange-600 bg-clip-text text-transparent">
                  Why HackFlare?
                </span>
              </h2>
              <p class="text-gray-300 text-lg max-w-2xl mx-auto">
                Everything you need to manage your DNS and content delivery
              </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
              <!-- Feature 1 -->
              <div class="group relative">
                <div class="absolute inset-0 bg-gradient-to-r from-orange-600/20 to-transparent rounded-lg blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                </div>
                <div class="relative p-8 bg-gray-900/50 border border-orange-500/20 rounded-lg hover:border-orange-500/50 transition-all backdrop-blur-sm">
                  <div class="flex justify-center mb-4">
                    <div class="p-3 bg-gradient-to-br from-orange-500 to-orange-600 rounded-lg">
                      <.icon name="hero-bolt" class="w-6 h-6 text-white" />
                    </div>
                  </div>
                  <h3 class="text-xl font-bold text-center text-white mb-2">Lightning Fast</h3>
                  <p class="text-center text-gray-300 text-sm">
                    Global DNS resolution and CDN at blazing speeds.
                  </p>
                </div>
              </div>

    <!-- Feature 2 -->
              <div class="group relative">
                <div class="absolute inset-0 bg-gradient-to-r from-orange-600/20 to-transparent rounded-lg blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                </div>
                <div class="relative p-8 bg-gray-900/50 border border-orange-500/20 rounded-lg hover:border-orange-500/50 transition-all backdrop-blur-sm">
                  <div class="flex justify-center mb-4">
                    <div class="p-3 bg-gradient-to-br from-orange-500 to-orange-600 rounded-lg">
                      <.icon name="hero-shield-check" class="w-6 h-6 text-white" />
                    </div>
                  </div>
                  <h3 class="text-xl font-bold text-center text-white mb-2">Battle Tested</h3>
                  <p class="text-center text-gray-300 text-sm">
                    Enterprise-grade DDoS protection and security features.
                  </p>
                </div>
              </div>

    <!-- Feature 3 -->
              <div class="group relative">
                <div class="absolute inset-0 bg-gradient-to-r from-orange-600/20 to-transparent rounded-lg blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                </div>
                <div class="relative p-8 bg-gray-900/50 border border-orange-500/20 rounded-lg hover:border-orange-500/50 transition-all backdrop-blur-sm">
                  <div class="flex justify-center mb-4">
                    <div class="p-3 bg-gradient-to-br from-orange-500 to-orange-600 rounded-lg">
                      <.icon name="hero-cog-6-tooth" class="w-6 h-6 text-white" />
                    </div>
                  </div>
                  <h3 class="text-xl font-bold text-center text-white mb-2">Built for Hackers</h3>
                  <p class="text-center text-gray-300 text-sm">
                    Simple, intuitive, and made by the HackClub community.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>

    <!-- Community proof Section -->
        <div class="relative px-8 py-24 sm:px-16 lg:px-24 bg-gradient-to-b from-black to-gray-950">
          <div class="max-w-7xl mx-auto space-y-16">
            <div class="text-center space-y-4">
              <h2 class="text-5xl sm:text-6xl font-black">
                <span class="bg-gradient-to-r from-orange-400 to-orange-600 bg-clip-text text-transparent">
                  Your favourite projects - built with Hackflare.
                </span>
              </h2>
              <p class="text-gray-300 text-lg max-w-2xl mx-auto">
                Trusted by many Hackers just like you.
              </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
              <div class="group relative">
                <div class="absolute inset-0 bg-gradient-to-r from-orange-600/20 to-transparent rounded-xl blur-xl opacity-0 group-hover:opacity-100 transition-all">
                </div>
                <div class="relative h-full p-7 bg-gray-900/50 border border-orange-500/20 rounded-xl backdrop-blur-sm hover:border-orange-500/50 transition-all">
                  <div class="flex items-center justify-between mb-6">
                    <div class="p-2.5 bg-gradient-to-br from-orange-500 to-orange-600 rounded-lg">
                      <.icon name="hero-globe-alt" class="w-5 h-5 text-white" />
                    </div>
                    <span class="text-xs text-orange-300 border border-orange-500/40 rounded-full px-3 py-1">
                      DNS + CDN
                    </span>
                  </div>
                  <h3 class="text-xl font-bold text-white mb-2">Maple Studio</h3>
                  <p class="text-gray-300 text-sm leading-relaxed mb-6">
                    Portfolio site serving global traffic with low-latency DNS routing and instant cache updates.
                  </p>
                  <a
                    href="/docs"
                    class="inline-flex items-center gap-2 text-orange-400 hover:text-orange-300 font-semibold transition-colors"
                  >
                    <span>View setup</span>
                    <.icon name="hero-arrow-right" class="w-4 h-4" />
                  </a>
                </div>
              </div>

              <div class="group relative">
                <div class="absolute inset-0 bg-gradient-to-r from-orange-600/20 to-transparent rounded-xl blur-xl opacity-0 group-hover:opacity-100 transition-all">
                </div>
                <div class="relative h-full p-7 bg-gray-900/50 border border-orange-500/20 rounded-xl backdrop-blur-sm hover:border-orange-500/50 transition-all">
                  <div class="flex items-center justify-between mb-6">
                    <div class="p-2.5 bg-gradient-to-br from-orange-500 to-orange-600 rounded-lg">
                      <.icon name="hero-server-stack" class="w-5 h-5 text-white" />
                    </div>
                    <span class="text-xs text-orange-300 border border-orange-500/40 rounded-full px-3 py-1">
                      API Infra
                    </span>
                  </div>
                  <h3 class="text-xl font-bold text-white mb-2">Shipyard API</h3>
                  <p class="text-gray-300 text-sm leading-relaxed mb-6">
                    Hackathon backend using managed records for fast deploy previews and stable API endpoints.
                  </p>
                  <a
                    href="/docs"
                    class="inline-flex items-center gap-2 text-orange-400 hover:text-orange-300 font-semibold transition-colors"
                  >
                    <span>View setup</span>
                    <.icon name="hero-arrow-right" class="w-4 h-4" />
                  </a>
                </div>
              </div>

              <div class="group relative">
                <div class="absolute inset-0 bg-gradient-to-r from-orange-600/20 to-transparent rounded-xl blur-xl opacity-0 group-hover:opacity-100 transition-all">
                </div>
                <div class="relative h-full p-7 bg-gray-900/50 border border-orange-500/20 rounded-xl backdrop-blur-sm hover:border-orange-500/50 transition-all">
                  <div class="flex items-center justify-between mb-6">
                    <div class="p-2.5 bg-gradient-to-br from-orange-500 to-orange-600 rounded-lg">
                      <.icon name="hero-bolt" class="w-5 h-5 text-white" />
                    </div>
                    <span class="text-xs text-orange-300 border border-orange-500/40 rounded-full px-3 py-1">
                      Real-time
                    </span>
                  </div>
                  <h3 class="text-xl font-bold text-white mb-2">Arcade Arena</h3>
                  <p class="text-gray-300 text-sm leading-relaxed mb-6">
                    Multiplayer game stack using resilient DNS failover for realtime lobbies and events.
                  </p>
                  <a
                    href="/docs"
                    class="inline-flex items-center gap-2 text-orange-400 hover:text-orange-300 font-semibold transition-colors"
                  >
                    <span>View setup</span>
                    <.icon name="hero-arrow-right" class="w-4 h-4" />
                  </a>
                </div>
              </div>
            </div>
          </div>
        </div>

    <!-- CTA Section -->
        <div class="relative px-8 py-24 sm:px-16 lg:px-24 bg-black">
          <div class="max-w-4xl mx-auto">
            <div class="relative overflow-hidden rounded-2xl bg-gradient-to-br from-gray-900 to-black border border-orange-500/30 p-12 sm:p-16">
              <!-- Gradient background effect -->
              <div class="absolute inset-0 bg-gradient-to-br from-orange-600/10 via-transparent to-transparent">
              </div>

              <div class="relative text-center space-y-6">
                <h2 class="text-4xl sm:text-5xl font-black text-white">
                  Ready to take
                  <span class="bg-gradient-to-r from-orange-400 to-orange-600 bg-clip-text text-transparent">
                    control?
                  </span>
                </h2>
                <p class="text-xl text-gray-300 max-w-2xl mx-auto">
                  Join thousands of HackClub members managing their DNS with HackFlare. It's free, it's fast, and it's built for you.
                </p>
                <div class="flex flex-col sm:flex-row gap-4 justify-center pt-8">
                  <a
                    href="dash/"
                    class="px-8 py-4 bg-gradient-to-r from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white font-bold rounded-lg transition-all transform hover:scale-105 shadow-lg shadow-orange-500/50"
                  >
                    Launch Dashboard
                  </a>
                  <a
                    href="/docs"
                    class="px-8 py-4 border-2 border-orange-500/50 hover:border-orange-500 text-orange-400 font-bold rounded-lg transition-all"
                  >
                    Read the Docs
                  </a>
                </div>
              </div>
            </div>
          </div>
        </div>

    <!-- Footer -->
        <footer class="relative px-8 py-12 sm:px-16 lg:px-24 bg-black border-t border-orange-500/20">
          <div class="max-w-7xl mx-auto">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
              <!-- Brand -->
              <div class="space-y-4">
                <div class="flex items-center gap-3">
                  <img src={~p"/images/logo.svg"} width="32" alt="HackFlare" />
                  <span class="font-bold text-lg text-orange-400">HackFlare</span>
                </div>
                <p class="text-gray-400 text-sm">
                  A powerful DNS and content delivery platform built for the Hack Club community.
                </p>
              </div>

    <!-- Links -->
              <div class="space-y-4">
                <h3 class="text-white font-semibold">Product</h3>
                <ul class="space-y-2 text-sm">
                  <li>
                    <a href="/docs" class="text-gray-400 hover:text-orange-400 transition-colors">
                      Documentation
                    </a>
                  </li>
                  <li>
                    <a
                      href="https://github.com/Hack-Flare/hackflare"
                      class="text-gray-400 hover:text-orange-400 transition-colors"
                    >
                      GitHub
                    </a>
                  </li>
                  <li>
                    <a href="/health" class="text-gray-400 hover:text-orange-400 transition-colors">
                      Status
                    </a>
                  </li>
                </ul>
              </div>

    <!-- Community -->
              <div class="space-y-4">
                <h3 class="text-white font-semibold">Community</h3>
                <ul class="space-y-2 text-sm">
                  <li>
                    <a
                      href="https://hackclub.com"
                      class="text-gray-400 hover:text-orange-400 transition-colors"
                    >
                      Hack Club
                    </a>
                  </li>
                  <li>
                    <a
                      href="https://hackclub.com/slack"
                      class="text-gray-400 hover:text-orange-400 transition-colors"
                    >
                      Slack
                    </a>
                  </li>
                  <li>
                    <a
                      href="https://github.com/Hack-Flare/hackflare"
                      class="text-gray-400 hover:text-orange-400 transition-colors"
                    >
                      Contribute
                    </a>
                  </li>
                </ul>
              </div>

    <!-- Legal -->
              <div class="space-y-4">
                <h3 class="text-white font-semibold">Legal</h3>
                <ul class="space-y-2 text-sm">
                  <li>
                    <a href="#" class="text-gray-400 hover:text-orange-400 transition-colors">
                      Privacy Policy
                    </a>
                  </li>
                  <li>
                    <a href="#" class="text-gray-400 hover:text-orange-400 transition-colors">
                      Terms of Service
                    </a>
                  </li>
                  <li>
                    <a href="#" class="text-gray-400 hover:text-orange-400 transition-colors">
                      Contact
                    </a>
                  </li>
                </ul>
              </div>
            </div>

            <div class="mt-12 pt-8 border-t border-orange-500/20 flex flex-col sm:flex-row justify-between items-center">
              <p class="text-gray-400 text-sm">
                © 2026 HackFlare. Built with ❤️ by Hack Clubbers.
              </p>
              <div class="flex gap-4 mt-4 sm:mt-0">
                <a
                  href="https://github.com/Hack-Flare/hackflare"
                  class="text-gray-400 hover:text-orange-400 transition-colors"
                >
                  <.icon name="hero-star" class="w-5 h-5" />
                </a>
                <a
                  href="https://hackclub.com/slack"
                  class="text-gray-400 hover:text-orange-400 transition-colors"
                >
                  <.icon name="hero-chat-bubble-left-right" class="w-5 h-5" />
                </a>
              </div>
            </div>
          </div>
        </footer>

        <.flash_group flash={@flash} />
      </body>
    </html>
    """
  end

  @doc """
  Renders the dashboard page with sidebar navigation and content grid.
  """
  attr :flash, :map, default: %{}, doc: "the map of flash messages"
  attr :current_view, :atom, default: :home, doc: "active dashboard view"
  attr :form_message, :string, default: "", doc: "help form message"

  def dashboard(assigns) do
    ~H"""
    <!DOCTYPE html>
    <html>
      <body class="bg-gradient-to-br from-gray-950 via-black to-gray-900 text-white">
        <div class="flex h-screen">
          <!-- Sidebar -->
          <aside class="w-72 bg-black/50 border-r border-orange-500/20 backdrop-blur-md p-6 overflow-y-auto">
            <!-- Logo -->
            <div class="flex items-center gap-3 mb-12">
              <img src={~p"/images/logo.svg"} width="40" alt="HackFlare" />
              <span class="font-bold text-lg text-orange-400">HackFlare</span>
            </div>

    <!-- Navigation Menu -->
            <nav class="space-y-2">
              <a
                href={~p"/dash"}
                class={nav_item_class(@current_view, :home)}
              >
                <.icon name="hero-home" class="w-5 h-5" />
                <span>Home</span>
              </a>
              <a
                href={~p"/dash/domains"}
                class={nav_item_class(@current_view, :domains)}
              >
                <.icon name="hero-globe-alt" class="w-5 h-5" />
                <span>Domains</span>
              </a>
              <a
                href={~p"/dash/settings"}
                class={nav_item_class(@current_view, :settings)}
              >
                <.icon name="hero-cog-6-tooth" class="w-5 h-5" />
                <span>Settings</span>
              </a>
              <a
                href={~p"/dash/analytics"}
                class={nav_item_class(@current_view, :analytics)}
              >
                <.icon name="hero-chart-bar" class="w-5 h-5" />
                <span>Analytics</span>
              </a>
              <a
                href={~p"/dash/notifications"}
                class={nav_item_class(@current_view, :notifications)}
              >
                <.icon name="hero-bell" class="w-5 h-5" />
                <span>Notifications</span>
              </a>
              <a
                href={~p"/dash/help"}
                class={nav_item_class(@current_view, :help)}
              >
                <.icon name="hero-question-mark-circle" class="w-5 h-5" />
                <span>Help</span>
              </a>
            </nav>

    <!-- Bottom Section -->
            <div class="absolute bottom-6 left-6 right-6 space-y-4">
              <form method="post" action="/auth/logout">
                <input type="hidden" name="_csrf_token" value={get_csrf_token()} />
                <button
                  type="submit"
                  class="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-gray-900/50 hover:text-red-400 transition-all w-full"
                >
                  <.icon name="hero-arrow-left-on-rectangle" class="w-5 h-5" />
                  <span>Logout</span>
                </button>
              </form>
            </div>
          </aside>

    <!-- Main Content -->
          <main class="flex-1 flex flex-col overflow-hidden">
            <!-- Header -->
            <header class="bg-black/30 border-b border-orange-500/20 backdrop-blur-md px-8 py-6 flex items-center justify-between">
              <div>
                <h1 class="text-3xl font-bold text-white">{dashboard_title(@current_view)}</h1>
                <p class="text-gray-400 text-sm mt-1">{dashboard_subtitle(@current_view)}</p>
              </div>
              <div class="flex items-center gap-4">
                <button class="p-2 hover:bg-gray-900/50 rounded-lg transition-all">
                  <.icon name="hero-bell" class="w-6 h-6 text-gray-400 hover:text-orange-400" />
                </button>
                <div class="w-10 h-10 rounded-full bg-gradient-to-br from-orange-400 to-orange-600 flex items-center justify-center">
                  <span class="text-white font-bold">HC</span>
                </div>
              </div>
            </header>

    <!-- Content Area -->
            <div class="flex-1 overflow-auto p-8">
              <%= case @current_view do %>
                <% :home -> %>
                  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    <!-- Card 1 -->
                    <div class="group relative">
                      <div class="absolute inset-0 bg-gradient-to-br from-orange-600/10 to-transparent rounded-xl blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                      </div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all backdrop-blur-sm h-48 flex flex-col justify-between">
                        <div>
                          <h3 class="text-lg font-bold text-white">Quick Start</h3>
                          <p class="text-gray-400 text-sm mt-2">Set up your first domain</p>
                        </div>
                        <button class="text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors">
                          <span>Get Started</span>
                          <.icon name="hero-arrow-right" class="w-4 h-4" />
                        </button>
                      </div>
                    </div>

    <!-- Card 2 -->
                    <div class="group relative">
                      <div class="absolute inset-0 bg-gradient-to-br from-orange-600/10 to-transparent rounded-xl blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                      </div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all backdrop-blur-sm h-48 flex flex-col justify-between">
                        <div>
                          <h3 class="text-lg font-bold text-white">Your Domains</h3>
                          <p class="text-gray-400 text-sm mt-2">Manage all your domains</p>
                        </div>
                        <button class="text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors">
                          <span>View All</span>
                          <.icon name="hero-arrow-right" class="w-4 h-4" />
                        </button>
                      </div>
                    </div>

    <!-- Card 3 -->
                    <div class="group relative">
                      <div class="absolute inset-0 bg-gradient-to-br from-orange-600/10 to-transparent rounded-xl blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                      </div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all backdrop-blur-sm h-48 flex flex-col justify-between">
                        <div>
                          <h3 class="text-lg font-bold text-white">Analytics</h3>
                          <p class="text-gray-400 text-sm mt-2">View your traffic statistics</p>
                        </div>
                        <button class="text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors">
                          <span>View Stats</span>
                          <.icon name="hero-arrow-right" class="w-4 h-4" />
                        </button>
                      </div>
                    </div>

    <!-- Card 4 -->
                    <div class="group relative">
                      <div class="absolute inset-0 bg-gradient-to-br from-orange-600/10 to-transparent rounded-xl blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                      </div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all backdrop-blur-sm h-48 flex flex-col justify-between">
                        <div>
                          <h3 class="text-lg font-bold text-white">API Access</h3>
                          <p class="text-gray-400 text-sm mt-2">Integrate with our API</p>
                        </div>
                        <button class="text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors">
                          <span>API Docs</span>
                          <.icon name="hero-arrow-right" class="w-4 h-4" />
                        </button>
                      </div>
                    </div>

    <!-- Card 5 -->
                    <div class="group relative">
                      <div class="absolute inset-0 bg-gradient-to-br from-orange-600/10 to-transparent rounded-xl blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                      </div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all backdrop-blur-sm h-48 flex flex-col justify-between">
                        <div>
                          <h3 class="text-lg font-bold text-white">Support</h3>
                          <p class="text-gray-400 text-sm mt-2">Get help when you need it</p>
                        </div>
                        <a
                          href={~p"/dash/help"}
                          class="text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors"
                        >
                          <span>Contact Us</span>
                          <.icon name="hero-arrow-right" class="w-4 h-4" />
                        </a>
                      </div>
                    </div>

    <!-- Card 6 -->
                    <div class="group relative">
                      <div class="absolute inset-0 bg-gradient-to-br from-orange-600/10 to-transparent rounded-xl blur-xl group-hover:blur-2xl transition-all opacity-0 group-hover:opacity-100">
                      </div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all backdrop-blur-sm h-48 flex flex-col justify-between">
                        <div>
                          <h3 class="text-lg font-bold text-white">Community</h3>
                          <p class="text-gray-400 text-sm mt-2">Join our Slack community</p>
                        </div>
                        <button class="text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors">
                          <span>Join Now</span>
                          <.icon name="hero-arrow-right" class="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  </div>
                <% :domains -> %>
                  <div class="space-y-4">
                    <div class="p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl">
                      <h3 class="text-xl font-bold text-white mb-2">Domains</h3>
                      <p class="text-gray-300">Manage zones, records, and routing from one place.</p>
                    </div>
                    <div class="p-6 bg-gray-900/30 border border-orange-500/20 rounded-xl">
                      <p class="text-sm text-gray-400">No domains yet. Add first domain to start.</p>
                    </div>
                  </div>
                <% :settings -> %>
                  <div class="space-y-4">
                    <div class="p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl">
                      <h3 class="text-xl font-bold text-white mb-2">Settings</h3>
                      <p class="text-gray-300">
                        Control account, API tokens, and organization defaults.
                      </p>
                    </div>
                  </div>
                <% :analytics -> %>
                  <div class="space-y-4">
                    <div class="p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl">
                      <h3 class="text-xl font-bold text-white mb-2">Analytics</h3>
                      <p class="text-gray-300">
                        Traffic, query volume, and performance insights will appear here.
                      </p>
                    </div>
                  </div>
                <% :notifications -> %>
                  <div class="space-y-4">
                    <div class="p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl">
                      <h3 class="text-xl font-bold text-white mb-2">Notifications</h3>
                      <p class="text-gray-300">Alert channels and event subscriptions.</p>
                    </div>
                  </div>
                <% :help -> %>
                  <div class="space-y-4">
                    <div class="p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl">
                      <h3 class="text-xl font-bold text-white mb-2">Help</h3>
                      <p class="text-gray-300">
                        Get support docs, troubleshooting, and community links.
                      </p>
                    </div>
                    <div class="p-6 bg-gray-900/30 border border-orange-500/20 rounded-xl">
                      <form method="post" action={~p"/dash/help"} class="space-y-4">
                        <input type="hidden" name="_csrf_token" value={get_csrf_token()} />
                        <label class="block text-sm font-semibold text-gray-200" for="help_message">
                          Send us your concerns and questions!
                        </label>
                        <textarea
                          id="help_message"
                          name="help[message]"
                          rows="7"
                          class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-4 text-gray-100 focus:border-orange-400 focus:outline-none"
                          placeholder="Describe issue or question..."
                          required
                        ><%= @form_message %></textarea>
                        <div class="flex justify-end">
                          <button
                            type="submit"
                            class="px-6 py-3 bg-gradient-to-r from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white font-semibold rounded-lg transition-all"
                          >
                            Send
                          </button>
                        </div>
                      </form>
                    </div>
                  </div>
                <% :admin -> %>
                  <div class="space-y-6">
                    <div class="rounded-xl border border-orange-500/20 bg-gray-900/40 p-6">
                      <div class="flex items-start justify-between gap-4">
                        <div>
                          <h3 class="text-xl font-bold text-white">Runtime Settings</h3>
                          <p class="mt-1 text-sm text-gray-400">
                            These values override the app's env defaults and can be changed from here.
                          </p>
                        </div>
                        <p class="text-xs uppercase tracking-[0.3em] text-orange-300">
                          Comma-separated emails
                        </p>
                      </div>

                      <form method="post" action={~p"/admin/settings"} class="mt-6 space-y-8">
                        <input type="hidden" name="_csrf_token" value={get_csrf_token()} />

                        <div class="grid gap-6 lg:grid-cols-2">
                          <label class="block space-y-2">
                            <span class="text-sm font-semibold text-gray-200">Admin emails</span>
                            <textarea
                              name="settings[admin_emails]"
                              rows="3"
                              class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                              placeholder="admin1@example.com, admin2@example.com"
                            ><%= @settings.admin_emails || "" %></textarea>
                          </label>

                          <label class="block space-y-2">
                            <span class="text-sm font-semibold text-gray-200">Slack help webhook</span>
                            <input
                              type="text"
                              name="settings[slack_help_webhook_url]"
                              value={@settings.slack_help_webhook_url || ""}
                              class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                              placeholder="https://hooks.slack.com/..."
                            />
                          </label>
                        </div>

                        <div class="rounded-lg border border-orange-500/10 bg-black/20 p-4 space-y-4">
                          <div>
                            <h4 class="text-base font-bold text-white">Auth</h4>
                            <p class="text-sm text-gray-400">OIDC settings used for Hack Club sign-in.</p>
                          </div>
                          <div class="grid gap-4 lg:grid-cols-2">
                            <input type="text" name="settings[auth][client_id]" value={get_in(@settings, [:auth, :client_id]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="client id" />
                            <input type="text" name="settings[auth][client_secret]" value={get_in(@settings, [:auth, :client_secret]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="client secret" />
                            <input type="text" name="settings[auth][redirect_uri]" value={get_in(@settings, [:auth, :redirect_uri]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="redirect uri" />
                            <input type="text" name="settings[auth][base_url]" value={get_in(@settings, [:auth, :base_url]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="base url" />
                            <input type="text" name="settings[auth][openid_configuration_uri]" value={get_in(@settings, [:auth, :openid_configuration_uri]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none lg:col-span-2" placeholder="openid configuration uri" />
                            <input type="text" name="settings[auth][authorization_scope]" value={get_in(@settings, [:auth, :authorization_scope]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none lg:col-span-2" placeholder="authorization scope" />
                          </div>
                        </div>

                        <div class="rounded-lg border border-orange-500/10 bg-black/20 p-4 space-y-4">
                          <div>
                            <h4 class="text-base font-bold text-white">DNS</h4>
                            <p class="text-sm text-gray-400">These values restart the nameserver when saved.</p>
                          </div>
                          <div class="grid gap-4 lg:grid-cols-2">
                            <input type="text" name="settings[dns][bind]" value={get_in(@settings, [:dns, :bind]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="bind address" />
                            <input type="text" name="settings[dns][port]" value={get_in(@settings, [:dns, :port]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="port" />
                            <input type="text" name="settings[dns][soa][mname]" value={get_in(@settings, [:dns, :soa, :mname]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa mname" />
                            <input type="text" name="settings[dns][soa][rname]" value={get_in(@settings, [:dns, :soa, :rname]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa rname" />
                            <input type="text" name="settings[dns][soa][serial]" value={get_in(@settings, [:dns, :soa, :serial]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa serial" />
                            <input type="text" name="settings[dns][soa][refresh]" value={get_in(@settings, [:dns, :soa, :refresh]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa refresh" />
                            <input type="text" name="settings[dns][soa][retry]" value={get_in(@settings, [:dns, :soa, :retry]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa retry" />
                            <input type="text" name="settings[dns][soa][expire]" value={get_in(@settings, [:dns, :soa, :expire]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa expire" />
                            <input type="text" name="settings[dns][soa][minimum]" value={get_in(@settings, [:dns, :soa, :minimum]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa minimum" />
                            <input type="text" name="settings[dns][soa][ttl]" value={get_in(@settings, [:dns, :soa, :ttl]) || ""} class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none" placeholder="soa ttl" />
                          </div>
                        </div>

                        <div class="flex justify-end">
                          <button type="submit" class="rounded-lg bg-gradient-to-r from-orange-500 to-orange-600 px-6 py-3 font-semibold text-white transition-all hover:from-orange-600 hover:to-orange-700">
                            Save settings
                          </button>
                        </div>
                      </form>
                    </div>

                    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
                      <div class="rounded-xl border border-orange-500/20 bg-gray-900/50 p-5">
                        <p class="text-sm uppercase tracking-wide text-gray-400">Total Users</p>
                        <p class="mt-3 text-3xl font-black text-white">{@stats.total_users}</p>
                      </div>
                      <div class="rounded-xl border border-orange-500/20 bg-gray-900/50 p-5">
                        <p class="text-sm uppercase tracking-wide text-gray-400">Admins</p>
                        <p class="mt-3 text-3xl font-black text-orange-400">{@stats.admin_users}</p>
                      </div>
                      <div class="rounded-xl border border-orange-500/20 bg-gray-900/50 p-5">
                        <p class="text-sm uppercase tracking-wide text-gray-400">Verified</p>
                        <p class="mt-3 text-3xl font-black text-white">{@stats.verified_users}</p>
                      </div>
                      <div class="rounded-xl border border-orange-500/20 bg-gray-900/50 p-5">
                        <p class="text-sm uppercase tracking-wide text-gray-400">YSWS Eligible</p>
                        <p class="mt-3 text-3xl font-black text-white">{@stats.eligible_users}</p>
                      </div>
                    </div>

                    <div class="rounded-xl border border-orange-500/20 bg-gray-900/40 overflow-hidden">
                      <div class="border-b border-orange-500/20 px-6 py-4">
                        <h3 class="text-xl font-bold text-white">Users</h3>
                        <p class="text-sm text-gray-400">Review account status and admin access.</p>
                      </div>

                      <div class="overflow-x-auto">
                        <table class="min-w-full divide-y divide-orange-500/10 text-left text-sm">
                          <thead class="bg-black/20 text-gray-300">
                            <tr>
                              <th class="px-6 py-4 font-semibold">Name</th>
                              <th class="px-6 py-4 font-semibold">Email</th>
                              <th class="px-6 py-4 font-semibold">Status</th>
                              <th class="px-6 py-4 font-semibold">Role</th>
                              <th class="px-6 py-4 font-semibold">Eligible</th>
                            </tr>
                          </thead>
                          <tbody class="divide-y divide-orange-500/10">
                            <%= if Enum.empty?(@users) do %>
                              <tr>
                                <td colspan="5" class="px-6 py-10 text-center text-gray-400">
                                  No users have signed in yet.
                                </td>
                              </tr>
                            <% else %>
                              <%= for user <- @users do %>
                                <tr class="bg-black/10 text-gray-100">
                                  <td class="px-6 py-4 font-medium"><%= user.name %></td>
                                  <td class="px-6 py-4 text-gray-300"><%= user.email %></td>
                                  <td class="px-6 py-4 text-gray-300">
                                    <%= user.verification_status || "unknown" %>
                                  </td>
                                  <td class="px-6 py-4">
                                    <span class={[
                                      "inline-flex rounded-full px-3 py-1 text-xs font-semibold",
                                      if(user.is_admin,
                                        do: "bg-orange-500/20 text-orange-300",
                                        else: "bg-gray-800 text-gray-300"
                                      )
                                    ]}>
                                      <%= if user.is_admin, do: "Admin", else: "User" %>
                                    </span>
                                  </td>
                                  <td class="px-6 py-4 text-gray-300">
                                    <%= if user.ysws_eligible, do: "Yes", else: "No" %>
                                  </td>
                                </tr>
                              <% end %>
                            <% end %>
                          </tbody>
                        </table>
                      </div>
                    </div>
                  </div>
              <% end %>
            </div>
          </main>
        </div>
        <footer class="border-t border-zinc-800 py-8 text-sm text-zinc-400">
          <div class="mx-auto flex max-w-6xl flex-col gap-4 px-6 md:flex-row md:items-center md:justify-between">
            <p>© 2026 HackFlare. Built by Hack Clubbers.</p>
            <nav class="flex gap-4">
              <a href="/docs" class="hover:text-white">Docs</a>
              <a href="/status" class="hover:text-white">Status</a>
              <a href="/privacy" class="hover:text-white">Privacy</a>
              <a href="https://github.com/Hack-Flare/hackflare" class="hover:text-white">GitHub</a>
            </nav>
          </div>
        </footer>
        <.flash_group flash={@flash} />
      </body>
    </html>
    """
  end

  @doc """
  Placeholder for the Contact section UI. Add your UI code here.
  """
  def contact(assigns) do
    ~H"""
    <div class="p-8">
      <h2 class="text-2xl font-bold mb-4">Help</h2>
      <!-- Add your Contact section UI here -->
    </div>
    """
  end

  defp nav_item_class(current_view, item) when current_view == item do
    "flex items-center gap-3 px-4 py-3 rounded-lg bg-orange-500/20 border border-orange-500/50 text-orange-400 font-semibold transition-all"
  end

  defp nav_item_class(_current_view, _item) do
    "flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-gray-900/50 hover:text-orange-400 transition-all"
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
