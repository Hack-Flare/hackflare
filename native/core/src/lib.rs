pub mod dns;
pub mod nifs;
pub mod ns;

rustler::init!("Elixir.Hackflare.Native", load = nifs::init);
