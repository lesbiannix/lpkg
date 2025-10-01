// Diesel schema for package storage. Maintained manually to avoid build-script dependency.
diesel::table! {
    packages (id) {
        id -> Integer,
        name -> Text,
        version -> Text,
        source -> Nullable<Text>,
        md5 -> Nullable<Text>,
        configure_args -> Nullable<Text>,
        build_commands -> Nullable<Text>,
        install_commands -> Nullable<Text>,
        dependencies -> Nullable<Text>,
        enable_lto -> Bool,
        enable_pgo -> Bool,
        cflags -> Nullable<Text>,
        ldflags -> Nullable<Text>,
        profdata -> Nullable<Text>,
    }
}
