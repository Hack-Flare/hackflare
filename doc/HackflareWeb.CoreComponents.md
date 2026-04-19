# `HackflareWeb.CoreComponents`

Provides core UI components.

At first glance, this module may seem daunting, but its goal is to provide
core building blocks for your application, such as tables, forms, and
inputs. The components consist mostly of markup and are well-documented
with doc strings and declarative assigns. You may customize and style
them in any way you want, based on your application growth and needs.

The foundation for styling is Tailwind CSS, a utility-first CSS framework,
augmented with daisyUI, a Tailwind CSS plugin that provides UI components
and themes. Here are useful references:

  * [daisyUI](https://daisyui.com/docs/intro/) - a good place to get
    started and see the available components.

  * [Tailwind CSS](https://tailwindcss.com) - the foundational framework
    we build on. You will use it for layout, sizing, flexbox, grid, and
    spacing.

  * [Heroicons](https://heroicons.com) - see `icon/1` for usage.

  * [Phoenix.Component](https://hexdocs.pm/phoenix_live_view/Phoenix.Component.html) -
    the component system used by Phoenix. Some components, such as `<.link>`
    and `<.form>`, are defined there.

# `button`

Renders a button with navigation support.

## Examples

    <.button>Send!</.button>
    <.button phx-click="go" variant="primary">Send!</.button>
    <.button navigate={~p"/"}>Home</.button>

## Attributes

* `class` (`:any`)
* `variant` (`:string`) - Must be one of `"primary"`.
* Global attributes are accepted. Supports all globals plus: `["href", "navigate", "patch", "method", "download", "name", "value", "disabled"]`.
## Slots

* `inner_block` (required)

# `flash`

Renders flash notices.

## Examples

    <.flash kind={:info} flash={@flash} />
    <.flash kind={:info} phx-mounted={show("#flash")}>Welcome Back!</.flash>

## Attributes

* `id` (`:string`) - the optional id of flash container.
* `flash` (`:map`) - the map of flash messages to display. Defaults to `%{}`.
* `title` (`:string`) - Defaults to `nil`.
* `kind` (`:atom`) - used for styling and flash lookup. Must be one of `:info`, or `:error`.
* Global attributes are accepted. the arbitrary HTML attributes to add to the flash container.
## Slots

* `inner_block` - the optional inner block that renders the flash message.

# `header`

Renders a header with title.

## Slots

* `inner_block` (required)
* `subtitle`
* `actions`

# `hide`

# `icon`

Renders a [Heroicon](https://heroicons.com).

Heroicons come in three styles – outline, solid, and mini.
By default, the outline style is used, but solid and mini may
be applied by using the `-solid` and `-mini` suffix.

You can customize the size and colors of the icons by setting
width, height, and background color classes.

Icons are extracted from the `deps/heroicons` directory and bundled within
your compiled app.css by the plugin in `assets/vendor/heroicons.js`.

## Examples

    <.icon name="hero-x-mark" />
    <.icon name="hero-arrow-path" class="ml-1 size-3 motion-safe:animate-spin" />

## Attributes

* `name` (`:string`) (required)
* `class` (`:any`) - Defaults to `"size-4"`.

# `input`

Renders an input with label and error messages.

A `Phoenix.HTML.FormField` may be passed as argument,
which is used to retrieve the input name, id, and values.
Otherwise all attributes may be passed explicitly.

## Types

This function accepts all HTML input types, considering that:

  * You may also set `type="select"` to render a `<select>` tag

  * `type="checkbox"` is used exclusively to render boolean values

  * For live file uploads, see `Phoenix.Component.live_file_input/1`

See https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input
for more information. Unsupported types, such as radio, are best
written directly in your templates.

## Examples

```heex
<.input field={@form[:email]} type="email" />
<.input name="my-input" errors={["oh no!"]} />
```

## Select type

When using `type="select"`, you must pass the `options` and optionally
a `value` to mark which option should be preselected.

```heex
<.input field={@form[:user_type]} type="select" options={["Admin": "admin", "User": "user"]} />
```

For more information on what kind of data can be passed to `options` see
[`options_for_select`](https://hexdocs.pm/phoenix_html/Phoenix.HTML.Form.html#options_for_select/2).

## Attributes

* `id` (`:any`) - Defaults to `nil`.
* `name` (`:any`)
* `label` (`:string`) - Defaults to `nil`.
* `value` (`:any`)
* `type` (`:string`) - Defaults to `"text"`. Must be one of `"checkbox"`, `"color"`, `"date"`, `"datetime-local"`, `"email"`, `"file"`, `"month"`, `"number"`, `"password"`, `"search"`, `"select"`, `"tel"`, `"text"`, `"textarea"`, `"time"`, `"url"`, `"week"`, or `"hidden"`.
* `field` (`Phoenix.HTML.FormField`) - a form field struct retrieved from the form, for example: @form[:email].
* `errors` (`:list`) - Defaults to `[]`.
* `checked` (`:boolean`) - the checked flag for checkbox inputs.
* `prompt` (`:string`) - the prompt for select inputs. Defaults to `nil`.
* `options` (`:list`) - the options to pass to Phoenix.HTML.Form.options_for_select/2.
* `multiple` (`:boolean`) - the multiple flag for select inputs. Defaults to `false`.
* `class` (`:any`) - the input class to use over defaults. Defaults to `nil`.
* `error_class` (`:any`) - the input error class to use over defaults. Defaults to `nil`.
* Global attributes are accepted. Supports all globals plus: `["accept", "autocomplete", "capture", "cols", "disabled", "form", "list", "max", "maxlength", "min", "minlength", "multiple", "pattern", "placeholder", "readonly", "required", "rows", "size", "step"]`.

# `list`

Renders a data list.

## Examples

    <.list>
      <:item title="Title">{@post.title}</:item>
      <:item title="Views">{@post.views}</:item>
    </.list>

## Slots

* `item` (required) - Accepts attributes:

  * `title` (`:string`) (required)

# `show`

# `table`

Renders a table with generic styling.

## Examples

    <.table id="users" rows={@users}>
      <:col :let={user} label="id">{user.id}</:col>
      <:col :let={user} label="username">{user.username}</:col>
    </.table>

## Attributes

* `id` (`:string`) (required)
* `rows` (`:list`) (required)
* `row_id` (`:any`) - the function for generating the row id. Defaults to `nil`.
* `row_click` (`:any`) - the function for handling phx-click on each row. Defaults to `nil`.
* `row_item` (`:any`) - the function for mapping each row before calling the :col and :action slots. Defaults to `&Function.identity/1`.
## Slots

* `col` (required) - Accepts attributes:

  * `label` (`:string`)
* `action` - the slot for showing user actions in the last table column.

# `translate_error`

Translates an error message using gettext.

# `translate_errors`

Translates the errors for a field from a keyword list of errors.

---

*Consult [api-reference.md](api-reference.md) for complete listing*
