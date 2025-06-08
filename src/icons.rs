use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/icons/outline"]
pub struct Icons;

#[derive(RustEmbed)]
#[folder = "assets/icons/filled"]
pub struct IconsFilled;
