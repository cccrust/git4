use flate2::Decompress;
use flate2::FlushDecompress;

fn main() {
    let mut d = Decompress::new(true);
    let mut output = vec![0; 100];
    let input = b"fake data";
    // Normally this would fail since it's not zlib, but lets check API
    let _ = d.decompress(input, &mut output, FlushDecompress::None);
    println!("Consumed: {}", d.total_in());
}
