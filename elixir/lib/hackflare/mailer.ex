defmodule Hackflare.Mailer do
  @moduledoc """
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
  """
  use Swoosh.Mailer, otp_app: :hackflare
end
