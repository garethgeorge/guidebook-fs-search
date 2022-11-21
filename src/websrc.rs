use include_dir::{include_dir, Dir};

pub static WEB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web");
