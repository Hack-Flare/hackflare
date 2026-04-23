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
                href="#"
                class="flex items-center gap-3 px-4 py-3 rounded-lg bg-orange-500/20 border border-orange-500/50 text-orange-400 font-semibold transition-all"
              >
                <.icon name="hero-home" class="w-5 h-5" />
                <span>Home</span>
              </a>
              <a
                href="#"
                class="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-gray-900/50 hover:text-orange-400 transition-all"
              >
                <.icon name="hero-globe-alt" class="w-5 h-5" />
                <span>Domains</span>
              </a>
              <a
                href="#"
                class="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-gray-900/50 hover:text-orange-400 transition-all"
              >
                <.icon name="hero-cog-6-tooth" class="w-5 h-5" />
                <span>Settings</span>
              </a>
              <a
                href="#"
                class="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-gray-900/50 hover:text-orange-400 transition-all"
              >
                <.icon name="hero-chart-bar" class="w-5 h-5" />
                <span>Analytics</span>
              </a>
              <a
                href="#"
                class="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-gray-900/50 hover:text-orange-400 transition-all"
              >
                <.icon name="hero-bell" class="w-5 h-5" />
                <span>Notifications</span>
              </a>
              <a
                href="#"
                class="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-gray-900/50 hover:text-orange-400 transition-all"
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
                <h1 class="text-3xl font-bold text-white">Dashboard</h1>
                <p class="text-gray-400 text-sm mt-1">Welcome to HackFlare</p>
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
                    <button class="text-orange-400 hover:text-orange-300 font-semibold flex items-center gap-2 transition-colors">
                      <span>Contact Us</span>
                      <.icon name="hero-arrow-right" class="w-4 h-4" />
                    </button>
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
end
