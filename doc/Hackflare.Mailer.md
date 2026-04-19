# `Hackflare.Mailer`

Email service for Hackflare using Swoosh.

This module provides email sending capabilities for transactional emails such as:
- Account notifications
- Password resets
- Domain verification
- Security alerts

## Usage

    email =
      new()
      |> to("user@example.com")
      |> from("noreply@hackflare.app")
      |> subject("Welcome to Hackflare")
      |> text_body("Welcome!")

    Hackflare.Mailer.deliver(email)

## Configuration

In development, emails are stored locally and viewable at `/dev/mailbox`.

In production, configure the mailer adapter in `config/runtime.exs`:

    config :hackflare, Hackflare.Mailer,
      adapter: Swoosh.Adapters.Sendgrid,
      api_key: System.get_env("SENDGRID_API_KEY")

Supported adapters include SendGrid, Mailgun, AWS SES, SMTP, and others.
See [Swoosh documentation](https://hexdocs.pm/swoosh) for available adapters.

# `deliver`

```elixir
@spec deliver(Swoosh.Email.t(), Keyword.t()) :: {:ok, term()} | {:error, term()}
```

Delivers an email.

If the email is delivered it returns an `{:ok, result}` tuple. If it fails,
returns an `{:error, error}` tuple.

# `deliver!`

```elixir
@spec deliver!(Swoosh.Email.t(), Keyword.t()) :: term() | no_return()
```

Delivers an email, raises on error.

If the email is delivered, it returns the result. If it fails, it raises
a `DeliveryError`.

# `deliver_many`

```elixir
@spec deliver_many(
  [
    %Swoosh.Email{
      assigns: term(),
      attachments: term(),
      bcc: term(),
      cc: term(),
      from: term(),
      headers: term(),
      html_body: term(),
      private: term(),
      provider_options: term(),
      reply_to: term(),
      subject: term(),
      text_body: term(),
      to: term()
    }
  ],
  Keyword.t()
) :: {:ok, term()} | {:error, term()}
```

Delivers a list of emails.

It accepts a list of `%Swoosh.Email{}` as its first parameter.

---

*Consult [api-reference.md](api-reference.md) for complete listing*
