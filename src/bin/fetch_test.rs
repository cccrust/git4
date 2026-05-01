use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = ureq::get("http://example.com").call()?;
    let mut reader = resp.into_body().into_reader();
    let mut body = Vec::new();
    reader.read_to_end(&mut body)?;
    Ok(())
}
