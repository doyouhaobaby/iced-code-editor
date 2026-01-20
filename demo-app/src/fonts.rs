use std::borrow::Cow;

pub const JETBRAINS_MONO_REGULAR: &'static [u8] =
    include_bytes!("../assets/JetBrainsMono-Regular.ttf");

pub fn load_all() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(JETBRAINS_MONO_REGULAR),
    ]
}
