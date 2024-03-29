use crate::make::dummy;
use crate::parse::image::Image;

mod parse;

mod make;

mod util;

fn main() {
    let description = dummy();
    let layout = description.layout();

    svg::save("test.svg", &layout.to_svg()).unwrap();

    let input_image = Image::read_from_file("sample-images/image14.png"); // just one of the images that I'm testing with

    let deelio = parse::BarsFound::from_image(&input_image);
    let result = layout.interpret_targets(&deelio).expect("bruh");

    result.describe_results(&description);
}