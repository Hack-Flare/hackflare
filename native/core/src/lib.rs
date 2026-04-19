mod tests;

pub mod dns;
pub mod nifs;
pub mod ns;

rustler::init!(
    "Elixir.Hackflare.Native",
    [
        nifs::manager_new,
        nifs::manager_create_zone,
        nifs::manager_delete_zone,
        nifs::manager_add_record,
        nifs::manager_remove_record,
        nifs::manager_list_zones,
        nifs::manager_find_records,
        nifs::engine_handle_query,
        nifs::manager_start_nameserver,
        tests::example_add,
    ],
    load = nifs::init
);
