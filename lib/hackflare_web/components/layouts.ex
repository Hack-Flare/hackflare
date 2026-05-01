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
    <header class="navbar px-4 sm:px-6 lg:px-8 bg-primary shadow-md">
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
        <header class="fixed top-0 left-0 right-0 z-50 navbar px-4 sm:px-6 lg:px-8 bg-black/50 shadow-md border-b border-orange-500/20">
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
        <div class="relative overflow-hidden bg-black text-white pt-28">
          <div class="mx-auto max-w-7xl px-8 sm:px-16 lg:px-24 py-14 sm:py-16 lg:py-20">
            <div class="grid gap-8 lg:grid-cols-[1.1fr_0.9fr] lg:items-center">
              <div class="space-y-7">
                <div class="inline-flex items-center gap-2 rounded-full border border-orange-500/30 bg-black/60 px-4 py-2 text-xs font-semibold uppercase tracking-[0.22em] text-orange-300 shadow-sm">
                  <span class="h-2 w-2 rounded-full bg-primary"></span>
                  Cloudflare-inspired network for Hack Club
                </div>

                <div class="space-y-5 max-w-3xl">
                  <h1 class="text-5xl sm:text-7xl lg:text-7xl font-black leading-tight text-white">
                    Where latency stops being your problem
                  </h1>
                  <p class="text-lg sm:text-xl text-gray-300 leading-relaxed max-w-2xl">
                    Built by Hack Clubbers, HackFlare gives members fast DNS, safer infrastructure, and a clean path from idea to global launch.
                  </p>
                </div>

                <div class="flex flex-col sm:flex-row gap-3.5">
                  <a
                    href="dash/"
                    class="px-7 py-3 bg-primary text-white font-bold rounded-lg hover:brightness-95 shadow-sm inline-flex items-center justify-center gap-2"
                  >
                    <span>Get started</span>
                    <.icon name="hero-arrow-right" class="w-5 h-5" />
                  </a>
                  <a
                    href="/auth/request"
                    class="px-7 py-3 border border-orange-500/40 bg-black/40 text-orange-300 font-bold rounded-lg hover:bg-orange-500/10 inline-flex items-center justify-center gap-2"
                  >
                    <.icon name="hero-lock-closed" class="w-5 h-5" />
                    <span>Sign in with Hack Club</span>
                  </a>
                </div>

                <div class="flex flex-wrap gap-5 pt-1 text-sm font-semibold text-gray-400">
                  <a
                    href="/docs"
                    class="inline-flex items-center gap-2 hover:text-orange-300 transition-colors"
                  >
                    <.icon name="hero-book-open" class="w-5 h-5 text-orange-400" />
                    <span>Docs</span>
                  </a>
                  <a
                    href="https://github.com/Hack-Flare/hackflare"
                    class="inline-flex items-center gap-2 hover:text-orange-300 transition-colors"
                  >
                    <.icon name="hero-star" class="w-5 h-5 text-orange-400" />
                    <span>GitHub</span>
                  </a>
                </div>
              </div>

              <div class="grid gap-3.5 sm:grid-cols-3 lg:grid-cols-1 lg:justify-items-center lg:self-center">
                <article class="w-full rounded-2xl border border-orange-500/25 bg-gray-950/90 p-5 shadow-sm lg:max-w-sm">
                  <div class="flex items-center justify-between">
                    <span class="inline-flex items-center rounded-full border border-orange-500/25 bg-black px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-orange-300">
                      Connect
                    </span>
                    <span class="inline-flex h-8 w-8 items-center justify-center rounded-md border border-orange-500/25 bg-black text-orange-300">
                      <.icon name="hero-globe-alt" class="w-4 h-4" />
                    </span>
                  </div>
                  <h2 class="mt-3 text-lg sm:text-xl font-black leading-tight text-white">
                    Route users to the nearest edge.
                  </h2>
                  <p class="mt-2 text-sm leading-5 text-gray-300">
                    Fast DNS updates and reliable routing across regions.
                  </p>
                </article>

                <article class="w-full rounded-2xl border border-orange-500/25 bg-gray-950/90 p-5 shadow-sm lg:max-w-sm">
                  <div class="flex items-center justify-between">
                    <span class="inline-flex items-center rounded-full border border-orange-500/25 bg-black px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-orange-300">
                      Protect
                    </span>
                    <span class="inline-flex h-8 w-8 items-center justify-center rounded-md border border-orange-500/25 bg-black text-orange-300">
                      <.icon name="hero-shield-check" class="w-4 h-4" />
                    </span>
                  </div>
                  <h2 class="mt-3 text-lg sm:text-xl font-black leading-tight text-white">
                    Keep critical paths stable.
                  </h2>
                  <p class="mt-2 text-sm leading-5 text-gray-300">
                    Harden endpoints and stay stable under traffic spikes.
                  </p>
                </article>

                <article class="w-full rounded-2xl border border-orange-500/25 bg-gray-950/90 p-5 shadow-sm lg:max-w-sm">
                  <div class="flex items-center justify-between">
                    <span class="inline-flex items-center rounded-full border border-orange-500/25 bg-black px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-orange-300">
                      Build everywhere
                    </span>
                    <span class="inline-flex h-8 w-8 items-center justify-center rounded-md border border-orange-500/25 bg-black text-orange-300">
                      <.icon name="hero-rocket-launch" class="w-4 h-4" />
                    </span>
                  </div>
                  <h2 class="mt-3 text-lg sm:text-xl font-black leading-tight text-white">
                    Ship globally from day one.
                  </h2>
                  <p class="mt-2 text-sm leading-5 text-gray-300">
                    Launch globally without rebuilding your network layer.
                  </p>
                </article>
              </div>
            </div>
          </div>
        </div>
        
    <!-- Features Section -->
        <div class="relative px-8 py-24 sm:px-16 lg:px-24 bg-black">
          <div class="max-w-7xl mx-auto space-y-16">
            <div class="text-center space-y-4">
              <h2 class="text-5xl sm:text-6xl font-black text-primary">Why HackFlare?</h2>
              <p class="text-gray-300 text-lg max-w-2xl mx-auto">
                Everything you need to manage your DNS and content delivery
              </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
              <!-- Feature 1 -->
              <div class="group relative">
                <div class="hidden"></div>
                <div class="relative p-8 bg-gray-900/50 border border-orange-500/20 rounded-lg hover:border-orange-500/50 transition-all">
                  <div class="flex justify-center mb-4">
                    <div class="p-3 bg-primary rounded-lg">
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
                <div class="hidden"></div>
                <div class="relative p-8 bg-gray-900/50 border border-orange-500/20 rounded-lg hover:border-orange-500/50 transition-all">
                  <div class="flex justify-center mb-4">
                    <div class="p-3 bg-primary rounded-lg">
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
                <div class="hidden"></div>
                <div class="relative p-8 bg-gray-900/50 border border-orange-500/20 rounded-lg hover:border-orange-500/50 transition-all">
                  <div class="flex justify-center mb-4">
                    <div class="p-3 bg-primary rounded-lg">
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
            <div class="relative overflow-hidden rounded-2xl bg-black border border-orange-500/30 p-12 sm:p-16">
              <div class="hidden"></div>

              <div class="relative text-center space-y-6">
                <h2 class="text-4xl sm:text-5xl font-black text-white">
                  Ready to take <span class="text-primary">control?</span>
                </h2>
                <p class="text-xl text-gray-300 max-w-2xl mx-auto">
                  Join thousands of HackClub members managing their DNS with HackFlare. It's free, it's fast, and it's built for you.
                </p>
                <div class="flex flex-col sm:flex-row gap-4 justify-center pt-8">
                  <a
                    href="dash/"
                    class="px-8 py-4 bg-primary text-white font-bold rounded-lg hover:brightness-95 shadow-sm"
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
  attr :current_user, :any, default: nil, doc: "the current authenticated user"

  def dashboard(assigns) do
    ~H"""
    <!DOCTYPE html>
    <html>
      <body class="bg-black text-white">
        <div class="flex h-screen">
          <!-- Sidebar -->
          <aside class="w-72 bg-black/50 border-r border-orange-500/20 p-6 overflow-y-auto">
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
              <%= if @current_user && Hackflare.Accounts.admin_user?(@current_user) do %>
                <a
                  href={~p"/admin"}
                  class={nav_item_class(@current_view, :admin)}
                >
                  <.icon name="hero-cog-8-tooth" class="w-5 h-5" />
                  <span>Admin Panel</span>
                </a>
              <% end %>
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
            <header class="bg-black/30 border-b border-orange-500/20 px-8 py-6 flex items-center justify-between">
              <div>
                <h1 class="text-3xl font-bold text-white">{dashboard_title(@current_view)}</h1>
                <p class="text-gray-400 text-sm mt-1">{dashboard_subtitle(@current_view)}</p>
              </div>
              <div class="flex items-center gap-4">
                <button class="p-2 hover:bg-gray-900/50 rounded-lg transition-all">
                  <.icon name="hero-bell" class="w-6 h-6 text-gray-400 hover:text-orange-400" />
                </button>
                <div class="w-10 h-10 rounded-full bg-primary flex items-center justify-center">
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
                      <div class="hidden"></div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all h-48 flex flex-col justify-between">
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
                      <div class="hidden"></div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all h-48 flex flex-col justify-between">
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
                      <div class="hidden"></div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all h-48 flex flex-col justify-between">
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
                      <div class="hidden"></div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all h-48 flex flex-col justify-between">
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
                      <div class="hidden"></div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all h-48 flex flex-col justify-between">
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
                      <div class="hidden"></div>
                      <div class="relative p-6 bg-gray-900/50 border border-orange-500/20 rounded-xl hover:border-orange-500/50 transition-all h-48 flex flex-col justify-between">
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
                  <% total_zones = Enum.count(@zones || []) %>
                  <div class="grid grid-cols-1 gap-6 lg:grid-cols-4">
                    <!-- Left: Domain List Sidebar -->
                    <div class="lg:col-span-1">
                      <div class="rounded-xl border border-orange-500/20 bg-gray-900/60 overflow-hidden">
                        <div class="border-b border-orange-500/15 bg-black/40 px-4 py-3">
                          <p class="text-xs font-semibold uppercase tracking-[0.18em] text-orange-300">
                            Domains
                          </p>
                          <p class="text-xl font-bold text-white mt-1">{total_zones}</p>
                        </div>
                        <%= if Enum.empty?(@zones) do %>
                          <div class="p-4 text-center text-sm text-gray-400">
                            No domains yet
                          </div>
                        <% else %>
                          <div class="max-h-96 overflow-y-auto">
                            <%= for zone <- @zones do %>
                              <% zone_key = String.replace(zone, ".", "-") %>
                              <button
                                onclick={"document.querySelectorAll('[data-domain-item]').forEach(el => el.classList.remove('bg-orange-500/10', 'border-orange-400')); document.getElementById('domain-#{zone_key}-item').classList.add('bg-orange-500/10', 'border-orange-400'); document.getElementById('domain-#{zone_key}-panel').classList.remove('hidden');"}
                                data-domain-item
                                class="w-full border-l-2 border-transparent px-4 py-2.5 text-left text-sm text-gray-200 hover:bg-gray-800/50 transition-all"
                                id={"domain-#{zone_key}-item"}
                              >
                                <p class="font-semibold truncate">{zone}</p>
                                <p class="text-xs text-gray-500 mt-0.5">
                                  {Enum.count(Map.get(@zone_records || %{}, zone, []))} records
                                </p>
                              </button>
                            <% end %>
                          </div>
                        <% end %>
                      </div>
                    </div>
                    
    <!-- Right: Domain Panel Content -->
                    <div class="lg:col-span-3 space-y-4">
                      <%= if Enum.empty?(@zones) do %>
                        <div class="rounded-xl border border-dashed border-orange-500/35 bg-gray-900/50 p-12 text-center">
                          <p class="text-lg font-semibold text-white">No Domains</p>
                          <p class="text-sm text-gray-400 mt-2">
                            Create your first domain to get started.
                          </p>
                          <button
                            onclick="document.getElementById('new-zone-modal').classList.remove('hidden')"
                            class="mt-6 rounded-lg bg-orange-500 px-4 py-2 text-sm font-semibold text-white transition-all hover:bg-orange-400"
                          >
                            + Add Domain
                          </button>
                        </div>
                      <% else %>
                        <%= for zone <- @zones do %>
                          <% zone_key = String.replace(zone, ".", "-") %>
                          <% records = Map.get(@zone_records || %{}, zone, []) %>
                          <div id={"domain-#{zone_key}-panel"} class="hidden space-y-4">
                            <!-- Domain Header -->
                            <div class="rounded-xl border border-orange-500/25 bg-black/80 p-6">
                              <div class="flex items-start justify-between gap-4">
                                <div>
                                  <p class="text-xs font-semibold uppercase tracking-[0.22em] text-orange-300">
                                    Domain
                                  </p>
                                  <h3 class="mt-2 text-2xl font-bold text-white">{zone}</h3>
                                </div>
                                <div class="flex items-center gap-2">
                                  <button
                                    onclick={"document.getElementById('add-record-modal-#{zone_key}').classList.remove('hidden')"}
                                    class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-semibold text-white transition-all hover:bg-emerald-500"
                                  >
                                    + Add Record
                                  </button>
                                  <button
                                    onclick={"if(confirm(\"Delete domain #{zone}?\")) { window.location.href = \"/dash/domains/delete/#{zone_key}\" }"}
                                    class="rounded-lg bg-red-600/50 px-4 py-2 text-sm font-semibold text-red-100 transition-all hover:bg-red-600"
                                  >
                                    Delete
                                  </button>
                                </div>
                              </div>
                            </div>
                            
    <!-- Category Tabs -->
                            <div class="rounded-xl border border-orange-500/20 bg-gray-900/60 overflow-hidden">
                              <div class="border-b border-orange-500/15 bg-black/40">
                                <div class="flex gap-1 p-1">
                                  <button
                                    onclick={"document.getElementById('dns-zones-#{zone_key}').classList.remove('hidden'); document.getElementById('other-#{zone_key}').classList.add('hidden');"}
                                    class="flex-1 rounded-lg bg-orange-500 px-3 py-2 text-sm font-semibold text-white transition-all hover:bg-orange-400"
                                  >
                                    DNS Zones
                                  </button>
                                  <button
                                    onclick={"document.getElementById('other-#{zone_key}').classList.remove('hidden'); document.getElementById('dns-zones-#{zone_key}').classList.add('hidden');"}
                                    class="flex-1 rounded-lg bg-gray-700/50 px-3 py-2 text-sm font-semibold text-gray-200 transition-all hover:bg-gray-600/50"
                                  >
                                    More (Soon)
                                  </button>
                                </div>
                              </div>
                              
    <!-- DNS Zones Content -->
                              <div id={"dns-zones-#{zone_key}"} class="p-4">
                                <div class="space-y-3">
                                  <%= if Enum.empty?(records) do %>
                                    <div class="rounded-lg border border-dashed border-orange-500/30 bg-black/20 px-4 py-6 text-center text-sm text-gray-400">
                                      No records in this domain yet.
                                    </div>
                                  <% else %>
                                    <div class="overflow-x-auto rounded-lg border border-orange-500/20">
                                      <table class="min-w-full divide-y divide-orange-500/15 text-sm">
                                        <thead class="bg-black/30">
                                          <tr>
                                            <th class="px-4 py-2 text-left font-semibold text-gray-300">
                                              Type
                                            </th>
                                            <th class="px-4 py-2 text-left font-semibold text-gray-300">
                                              Name
                                            </th>
                                            <th class="px-4 py-2 text-left font-semibold text-gray-300">
                                              Content
                                            </th>
                                            <th class="px-4 py-2 text-left font-semibold text-gray-300">
                                              TTL
                                            </th>
                                          </tr>
                                        </thead>
                                        <tbody class="divide-y divide-orange-500/10 bg-gray-900/30">
                                          <%= for record <- records do %>
                                            <tr class="hover:bg-black/30">
                                              <td class="whitespace-nowrap px-4 py-2 font-semibold text-orange-300">
                                                {record.rtype}
                                              </td>
                                              <td class="whitespace-nowrap px-4 py-2 text-gray-200">
                                                {record.name}
                                              </td>
                                              <td class="px-4 py-2 text-gray-300">{record.data}</td>
                                              <td class="whitespace-nowrap px-4 py-2 text-gray-400">
                                                {record.ttl}
                                              </td>
                                            </tr>
                                          <% end %>
                                        </tbody>
                                      </table>
                                    </div>
                                  <% end %>
                                </div>
                              </div>
                              
    <!-- More Content (placeholder) -->
                              <div
                                id={"other-#{zone_key}"}
                                class="hidden p-4 text-center text-sm text-gray-400"
                              >
                                <p>More features coming soon (SSL, Email, etc.)</p>
                              </div>
                            </div>
                            
    <!-- Add Record Modal -->
                            <div
                              id={"add-record-modal-#{zone_key}"}
                              class="hidden fixed inset-0 z-50 flex items-center justify-center bg-black/60"
                            >
                              <div class="w-full max-w-md rounded-xl border border-orange-500/30 bg-gray-900 p-8 shadow-2xl">
                                <h2 class="mb-1 text-2xl font-bold text-white">Add DNS Record</h2>
                                <p class="mb-4 text-sm text-gray-400">Zone: {zone}</p>
                                <form
                                  method="post"
                                  action={~p"/dash/domains/records/create"}
                                  class="space-y-4"
                                  onsubmit={"document.getElementById('add-record-modal-" <> zone_key <> "').classList.add('hidden')"}
                                >
                                  <input type="hidden" name="_csrf_token" value={get_csrf_token()} />
                                  <input type="hidden" name="zone_name" value={zone} />

                                  <input
                                    type="text"
                                    name="record_name"
                                    placeholder="Record name: @, www, api"
                                    class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                  />

                                  <div class="grid grid-cols-2 gap-3">
                                    <select
                                      name="record_type"
                                      class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                    >
                                      <option value="A">A</option>
                                      <option value="AAAA">AAAA</option>
                                      <option value="CNAME">CNAME</option>
                                      <option value="MX">MX</option>
                                      <option value="TXT">TXT</option>
                                      <option value="NS">NS</option>
                                    </select>
                                    <input
                                      type="number"
                                      name="ttl"
                                      min="1"
                                      value="300"
                                      class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                      required
                                    />
                                  </div>

                                  <input
                                    type="text"
                                    name="record_data"
                                    placeholder="Record content"
                                    class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                    required
                                  />

                                  <div class="flex gap-3 pt-1">
                                    <button
                                      type="button"
                                      onclick={"document.getElementById('add-record-modal-#{zone_key}').classList.add('hidden')"}
                                      class="flex-1 rounded-lg bg-gray-800 px-4 py-2 text-sm font-semibold text-gray-200 transition-all hover:bg-gray-700"
                                    >
                                      Cancel
                                    </button>
                                    <button
                                      type="submit"
                                      class="flex-1 rounded-lg bg-emerald-600 px-4 py-2 text-sm font-semibold text-white transition-all hover:bg-emerald-500"
                                    >
                                      Save Record
                                    </button>
                                  </div>
                                </form>
                              </div>
                            </div>
                          </div>
                        <% end %>
                        
    <!-- Initial state: show first domain by default -->
                        <script>
                          (function() {
                            var firstDomain = document.querySelector('[data-domain-item]');
                            if (firstDomain) {
                              firstDomain.click();
                            }
                          })();
                        </script>
                      <% end %>
                    </div>
                  </div>
                  
    <!-- New Domain Modal -->
                  <div
                    id="new-zone-modal"
                    class="hidden fixed inset-0 bg-black/50 flex items-center justify-center z-50"
                  >
                    <div class="bg-gray-900 border border-orange-500/30 rounded-xl p-8 max-w-md w-full shadow-2xl">
                      <h2 class="text-2xl font-bold text-white mb-4">Create New Zone</h2>
                      <form
                        method="post"
                        action={~p"/dash/domains/create"}
                        class="space-y-4"
                        onsubmit="document.getElementById('new-zone-modal').classList.add('hidden')"
                      >
                        <input type="hidden" name="_csrf_token" value={get_csrf_token()} />
                        <div>
                          <label class="block text-sm font-semibold text-gray-200 mb-2">
                            Zone Name
                          </label>
                          <input
                            type="text"
                            name="zone_name"
                            placeholder="example.com"
                            class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                            required
                          />
                          <p class="text-xs text-gray-400 mt-1">
                            Enter the domain name for this zone
                          </p>
                        </div>
                        <div>
                          <label class="block text-sm font-semibold text-gray-200 mb-2">
                            Zone Type
                          </label>
                          <select
                            name="zone_type"
                            id="zone-type"
                            class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                          >
                            <option value="root" selected>Root domain (apex)</option>
                            <option value="subdomain">Subdomain (full name)</option>
                          </select>
                          <p class="text-xs text-gray-400 mt-1">
                            Choose whether this zone is an apex/root domain or a subdomain.
                          </p>
                        </div>
                        <script>
                          (function(){
                            var sel = document.getElementById('zone-type');
                            var nameInput = document.querySelector('input[name="zone_name"]');
                            if(!sel || !nameInput) return;
                            sel.addEventListener('change', function(){
                              if(sel.value === 'subdomain'){
                                nameInput.placeholder = 'blog.example.com';
                                nameInput.setAttribute('title', 'Enter a full subdomain (e.g. blog.example.com)');
                              } else {
                                nameInput.placeholder = 'example.com';
                                nameInput.setAttribute('title', 'Enter the zone apex (e.g. example.com)');
                              }
                            });
                          })();
                        </script>
                        <div class="flex gap-3 pt-4">
                          <button
                            type="button"
                            onclick="document.getElementById('new-zone-modal').classList.add('hidden')"
                            class="flex-1 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-gray-200 rounded-lg font-semibold transition-all"
                          >
                            Cancel
                          </button>
                          <button
                            type="submit"
                            class="flex-1 px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg font-semibold transition-all"
                          >
                            Create Zone
                          </button>
                        </div>
                      </form>
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
                            class="px-6 py-3 bg-primary text-white font-semibold rounded-lg"
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
                      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
                        <div class="rounded-xl border border-orange-500/20 bg-gray-900/50 p-4">
                          <p class="text-sm uppercase tracking-wide text-gray-400">UDP Queries</p>
                          <p class="mt-3 text-2xl font-black text-white">{@metrics && @metrics.udp_count || 0}</p>
                        </div>
                        <div class="rounded-xl border border-orange-500/20 bg-gray-900/50 p-4">
                          <p class="text-sm uppercase tracking-wide text-gray-400">TCP Queries</p>
                          <p class="mt-3 text-2xl font-black text-white">{@metrics && @metrics.tcp_count || 0}</p>
                        </div>
                        <div class="rounded-xl border border-orange-500/20 bg-gray-900/50 p-4">
                          <p class="text-sm uppercase tracking-wide text-gray-400">UDP/TCP Ratio</p>
                          <p class="mt-3 text-2xl font-black text-white">
                            {
                              (fn ->
                                udp = if @metrics, do: (@metrics.udp_count || 0), else: 0
                                tcp = if @metrics, do: (@metrics.tcp_count || 0), else: 0
                                cond do
                                  udp == 0 and tcp == 0 -> "—"
                                  tcp == 0 -> "∞"
                                  true -> Float.round(udp / tcp, 2)
                                end
                              end).()
                            }
                          </p>
                        </div>
                      </div>
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
                            <span class="text-sm font-semibold text-gray-200">
                              Slack help webhook
                            </span>
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
                            <p class="text-sm text-gray-400">
                              OIDC settings used for Hack Club sign-in.
                            </p>
                          </div>
                          <div class="grid gap-4 lg:grid-cols-2">
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">Client ID</span>
                              <input
                                type="text"
                                name="settings[auth][client_id]"
                                value={get_in(@settings, [:auth, :client_id]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="Your OIDC client ID"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">Client Secret</span>
                              <input
                                type="text"
                                name="settings[auth][client_secret]"
                                value={get_in(@settings, [:auth, :client_secret]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="Your OIDC client secret"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">Redirect URI</span>
                              <input
                                type="text"
                                name="settings[auth][redirect_uri]"
                                value={get_in(@settings, [:auth, :redirect_uri]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="https://example.com/auth/callback"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">Base URL</span>
                              <input
                                type="text"
                                name="settings[auth][base_url]"
                                value={get_in(@settings, [:auth, :base_url]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="https://oidc.example.com"
                              />
                            </label>
                            <label class="block space-y-1 lg:col-span-2">
                              <span class="text-sm font-semibold text-gray-200">
                                OpenID Configuration URI
                              </span>
                              <input
                                type="text"
                                name="settings[auth][openid_configuration_uri]"
                                value={get_in(@settings, [:auth, :openid_configuration_uri]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="https://oidc.example.com/.well-known/openid-configuration"
                              />
                            </label>
                            <label class="block space-y-1 lg:col-span-2">
                              <span class="text-sm font-semibold text-gray-200">
                                Authorization Scope
                              </span>
                              <input
                                type="text"
                                name="settings[auth][authorization_scope]"
                                value={get_in(@settings, [:auth, :authorization_scope]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="openid profile email"
                              />
                            </label>
                          </div>
                        </div>

                        <div class="rounded-lg border border-orange-500/10 bg-black/20 p-4 space-y-4">
                          <div>
                            <h4 class="text-base font-bold text-white">DNS</h4>
                            <p class="text-sm text-gray-400">
                              These values restart the nameserver when saved.
                            </p>
                          </div>
                          <div class="grid gap-4 lg:grid-cols-2">
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">Bind Address</span>
                              <input
                                type="text"
                                name="settings[dns][bind]"
                                value={get_in(@settings, [:dns, :bind]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="0.0.0.0 or specific IP"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">Port</span>
                              <input
                                type="text"
                                name="settings[dns][port]"
                                value={get_in(@settings, [:dns, :port]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="53"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">
                                SOA Primary Nameserver (mname)
                              </span>
                              <input
                                type="text"
                                name="settings[dns][soa][mname]"
                                value={get_in(@settings, [:dns, :soa, :mname]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="ns1.example.com"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">
                                SOA Responsible Email (rname)
                              </span>
                              <input
                                type="text"
                                name="settings[dns][soa][rname]"
                                value={get_in(@settings, [:dns, :soa, :rname]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="admin.example.com"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">SOA Serial</span>
                              <input
                                type="text"
                                name="settings[dns][soa][serial]"
                                value={get_in(@settings, [:dns, :soa, :serial]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="2026043001"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">
                                SOA Refresh (seconds)
                              </span>
                              <input
                                type="text"
                                name="settings[dns][soa][refresh]"
                                value={get_in(@settings, [:dns, :soa, :refresh]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="10800"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">
                                SOA Retry (seconds)
                              </span>
                              <input
                                type="text"
                                name="settings[dns][soa][retry]"
                                value={get_in(@settings, [:dns, :soa, :retry]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="3600"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">
                                SOA Expire (seconds)
                              </span>
                              <input
                                type="text"
                                name="settings[dns][soa][expire]"
                                value={get_in(@settings, [:dns, :soa, :expire]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="604800"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">
                                SOA Minimum TTL (seconds)
                              </span>
                              <input
                                type="text"
                                name="settings[dns][soa][minimum]"
                                value={get_in(@settings, [:dns, :soa, :minimum]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="86400"
                              />
                            </label>
                            <label class="block space-y-1">
                              <span class="text-sm font-semibold text-gray-200">
                                SOA TTL (seconds)
                              </span>
                              <input
                                type="text"
                                name="settings[dns][soa][ttl]"
                                value={get_in(@settings, [:dns, :soa, :ttl]) || ""}
                                class="w-full rounded-lg border border-orange-500/30 bg-black/40 p-3 text-gray-100 focus:border-orange-400 focus:outline-none"
                                placeholder="3600"
                              />
                            </label>
                          </div>
                        </div>

                        <div class="flex justify-end">
                          <button
                            type="submit"
                            class="rounded-lg bg-primary px-6 py-3 font-semibold text-white"
                          >
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
                                  <td class="px-6 py-4 font-medium">{user.name}</td>
                                  <td class="px-6 py-4 text-gray-300">{user.email}</td>
                                  <td class="px-6 py-4 text-gray-300">
                                    {user.verification_status || "unknown"}
                                  </td>
                                  <td class="px-6 py-4">
                                    <span class={[
                                      "inline-flex rounded-full px-3 py-1 text-xs font-semibold",
                                      if(user.is_admin,
                                        do: "bg-orange-500/20 text-orange-300",
                                        else: "bg-gray-800 text-gray-300"
                                      )
                                    ]}>
                                      {if user.is_admin, do: "Admin", else: "User"}
                                    </span>
                                  </td>
                                  <td class="px-6 py-4 text-gray-300">
                                    {if user.ysws_eligible, do: "Yes", else: "No"}
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
