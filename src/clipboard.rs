use arboard::Clipboard;

pub fn copy_and_get(text: &str) -> Result<String, arboard::Error> {
  let mut clipboard = Clipboard::new()?;
  clipboard.set_text(text)?;
  clipboard.get_text()
}
