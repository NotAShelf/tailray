use arboard::Clipboard;

pub fn copy(text: &str) -> Result<(), arboard::Error> {
  let mut clipboard = Clipboard::new()?;
  clipboard.set_text(text)
}

pub fn get() -> Result<String, arboard::Error> {
  let mut clipboard = Clipboard::new()?;
  clipboard.get_text()
}
