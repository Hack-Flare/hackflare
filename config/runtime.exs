import Config

# config/runtime.exs is executed for all environments, including
# during releases. It is executed after compilation and before the
# system starts, so it is typically used to load production configuration
# and secrets from environment variables or elsewhere. Do not define
# any compile-time configuration in here, as it won't be applied.
# The block below contains prod specific runtime configuration.

# ## Using releases
#
# If you use `mix release`, you need to explicitly enable the server
# by passing the PHX_SERVER=true when you start it:
#
#     PHX_SERVER=true bin/hackflare start
#
# Alternatively, you can use `mix phx.gen.release` to generate a `bin/server`
# script that automatically sets the env var above.
config :hackflare, :dns, %{
  bind: System.get_env("DNS_BIND") || "0.0.0.0",
  port: String.to_integer(System.get_env("DNS_PORT", "53")),
  soa: %{
    mname: System.get_env("DNS_SOA_MNAME") || "a.root-servers.net.",
    rname: System.get_env("DNS_SOA_RNAME") || "nstld.verisign-grs.com.",
    serial: String.to_integer(System.get_env("DNS_SOA_SERIAL", "2026042000")),
    refresh: String.to_integer(System.get_env("DNS_SOA_REFRESH", "1800")),
    retry: String.to_integer(System.get_env("DNS_SOA_RETRY", "900")),
    expire: String.to_integer(System.get_env("DNS_SOA_EXPIRE", "604800")),
    minimum: String.to_integer(System.get_env("DNS_SOA_MINIMUM", "86400")),
    ttl: String.to_integer(System.get_env("DNS_SOA_TTL", "86400"))
  }
}

db_host_env = System.get_env("DB_HOST") || "localhost"
db_port_env = System.get_env("DB_PORT") || "5432"

{host, port_str} =
  case String.split(db_host_env, ":") do
    [h, p] -> {h, p}
    [h] -> {h, db_port_env}
  end

db_port =
  case Integer.parse(port_str) do
    {val, ""} -> val
    _ -> 5432
  end

config :hackflare, Hackflare.Repo,
  hostname: host,
  port: db_port,
  username: System.get_env("DB_USER") || "hackflare",
  password: System.get_env("DB_PASS") || System.get_env("DB_PASSWORD") || "hackflare",
  database: System.get_env("DB_NAME") || "hackflare"

config :hackflare, :auth,
  client_id:
    System.get_env("HCA_CID") || System.get_env("hca_cid") || "fuhh-you-forgot-set-this",
  client_secret:
    System.get_env("HCA_SEC") || System.get_env("hca_sec") || "fuhh-you-forgot-set-this-too",
  redirect_uri:
    System.get_env("HCA_REDIR_URL") || System.get_env("hca_redir_url") || "http://localhost:4000/auth/callback",
  base_url:
    System.get_env("HCA_BASE_URL") || System.get_env("hca_base_url") || "https://auth.hackclub.com",
  openid_configuration_uri:
    System.get_env("HCA_OPENID_CONFIG_URI") || System.get_env("hca_openid_config_uri") || "/.well-known/openid-configuration",
  authorization_params: [
    scope: "openid profile email slack_id verification_status"
  ],
  strategy: Assent.Strategy.OIDC

if System.get_env("PHX_SERVER") do
  config :hackflare, HackflareWeb.Endpoint, server: true
end

config :hackflare, HackflareWeb.Endpoint,
  http: [port: String.to_integer(System.get_env("PORT", "4000"))]

if config_env() == :prod do
  # The secret key base is used to sign/encrypt cookies and other secrets.
  # A default value is used in config/dev.exs and config/test.exs but you
  # want to use a different value for prod and you most likely don't want
  # to check this value into version control, so we use an environment
  # variable instead.
  secret_key_base =
    System.get_env("SECRET_KEY_BASE") ||
      raise """
      environment variable SECRET_KEY_BASE is missing.
      You can generate one by calling: mix phx.gen.secret
      """

  host = System.get_env("PHX_HOST") || "hackflare.kirze.de"

  config :hackflare, :dns_cluster_query, System.get_env("DNS_CLUSTER_QUERY")

  config :hackflare, HackflareWeb.Endpoint,
    url: [host: host, port: 443, scheme: "https"],
    http: [
      # Enable IPv6 and bind on all interfaces.
      # Set it to  {0, 0, 0, 0, 0, 0, 0, 1} for local network only access.
      # See the documentation on https://hexdocs.pm/bandit/Bandit.html#t:options/0
      # for details about using IPv6 vs IPv4 and loopback vs public addresses.
      ip: {0, 0, 0, 0, 0, 0, 0, 0}
    ],
    secret_key_base: secret_key_base

  # ## SSL Support
  #
  # To get SSL working, you will need to add the `https` key
  # to your endpoint configuration:
  #
  #     config :hackflare, HackflareWeb.Endpoint,
  #       https: [
  #         ...,
  #         port: 443,
  #         cipher_suite: :strong,
  #         keyfile: System.get_env("SOME_APP_SSL_KEY_PATH"),
  #         certfile: System.get_env("SOME_APP_SSL_CERT_PATH")
  #       ]
  #
  # The `cipher_suite` is set to `:strong` to support only the
  # latest and more secure SSL ciphers. This means old browsers
  # and clients may not be supported. You can set it to
  # `:compatible` for wider support.
  #
  # `:keyfile` and `:certfile` expect an absolute path to the key
  # and cert in disk or a relative path inside priv, for example
  # "priv/ssl/server.key". For all supported SSL configuration
  # options, see https://hexdocs.pm/plug/Plug.SSL.html#configure/1
  #
  # We also recommend setting `force_ssl` in your config/prod.exs,
  # ensuring no data is ever sent via http, always redirecting to https:
  #
  #     config :hackflare, HackflareWeb.Endpoint,
  #       force_ssl: [hsts: true]
  #
  # Check `Plug.SSL` for all available options in `force_ssl`.

  # ## Configuring the mailer
  #
  # In production you need to configure the mailer to use a different adapter.
  # Here is an example configuration for Mailgun:
  #
  #     config :hackflare, Hackflare.Mailer,
  #       adapter: Swoosh.Adapters.Mailgun,
  #       api_key: System.get_env("MAILGUN_API_KEY"),
  #       domain: System.get_env("MAILGUN_DOMAIN")
  #
  # Most non-SMTP adapters require an API client. Swoosh supports Req, Hackney,
  # and Finch out-of-the-box. This configuration is typically done at
  # compile-time in your config/prod.exs:
  #
  #     config :swoosh, :api_client, Swoosh.ApiClient.Req
  #
  # See https://hexdocs.pm/swoosh/Swoosh.html#module-installation for details.
end
