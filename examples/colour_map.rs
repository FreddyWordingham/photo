use palette::LinSrgba;
use photo::ColourMap;

fn main() {
    let colours = vec!["#FF0000", "#00FF00", "#0000FF"];
    let colour_map: ColourMap<f32, LinSrgba> = ColourMap::new(&colours);

    for i in 0..=10 {
        let x = 0.1 * i as f32;
        let colour = colour_map.sample(x);
        println!(
            "{} {} {} {}",
            colour.red, colour.green, colour.blue, colour.alpha
        );
    }
}
