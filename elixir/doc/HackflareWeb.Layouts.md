# `HackflareWeb.Layouts`

This module holds layouts and related functionality
used by your application.

# `app`

Renders your app layout.

This function is typically invoked from every template,
and it often contains your application menu, sidebar,
or similar.

## Examples

    <Layouts.app flash={@flash}>
      <h1>Content</h1>
    </Layouts.app>

## Attributes

* `flash` (`:map`) (required) - the map of flash messages.
* `current_scope` (`:map`) - the current [scope](https://hexdocs.pm/phoenix/scopes.html). Defaults to `nil`.
## Slots

* `inner_block` (required)

# `auth_menu`

Provides a redir to dashboard. Replace current icon with signed in icon in future.

## Attributes

* `current_scope` (`:map`) - the current [scope](https://hexdocs.pm/phoenix/scopes.html). Defaults to `nil`.

# `flash_group`

Shows the flash group with standard titles and content.

## Examples

    <.flash_group flash={@flash} />

## Attributes

* `flash` (`:map`) (required) - the map of flash messages.
* `id` (`:string`) - the optional id of flash container. Defaults to `"flash-group"`.

# `home`

Renders the homepage content using the app layout.

## Attributes

* `flash` (`:map`) - the map of flash messages. Defaults to `%{}`.

# `root`

# `theme_toggle`

Provides dark vs light theme toggle based on themes defined in app.css.

See <head> in root.html.heex which applies the theme before page load.

---

*Consult [api-reference.md](api-reference.md) for complete listing*
