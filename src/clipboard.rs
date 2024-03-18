use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)
}

pub fn get_from_clipboard() -> Result<String, arboard::Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.get_text()
}
