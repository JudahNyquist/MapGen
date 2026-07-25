use noise::{NoiseFn, Perlin};
use rand::Rng;
use image::{RgbImage, Rgb};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Biome {
    Tundra,
    Taiga,
    Temperate,
    Tropical,
    Desert,
    Savanna,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Terrain {
    DeepWater,
    Water,
    Sand,
    Grass,
    Forest,
    Mountain,
    Snow,
}

impl Terrain {
    fn color(&self) -> Rgb<u8> {
        match self {
            Terrain::DeepWater => Rgb([25, 55, 109]),      // dark blue
            Terrain::Water => Rgb([65, 105, 200]),         // med. blue
            Terrain::Sand => Rgb([238, 214, 110]),         // yellow
            Terrain::Grass => Rgb([100, 180, 80]),         // grn
            Terrain::Forest => Rgb([34, 110, 50]),         // dark green
            Terrain::Mountain => Rgb([139, 90, 43]),       // brn
            Terrain::Snow => Rgb([255, 255, 255]),         // white
        }
    }
}

#[derive(Clone, Copy)]
struct MapGenerator {
    width: usize,
    height: usize,
    elevation_seed: u32,
    temperature_seed: u32,
    moisture_seed: u32,
    scale: f64,
    octaves: u32,
    offset: f64,
}

impl MapGenerator {
    fn new(width: usize, height: usize, scale: f64, octaves: u32) -> Self {
        let mut rng = rand::thread_rng();
        
        MapGenerator {
            width,
            height,
            elevation_seed: rng.gen::<u32>(),
            temperature_seed: rng.gen::<u32>(),
            moisture_seed: rng.gen::<u32>(),
            scale,
            octaves,
            offset: 0.0,
        }
    }

    fn with_offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    fn generate(&self) -> Vec<Vec<Terrain>> {
        let mut map = vec![vec![Terrain::Grass; self.width]; self.height];
        let elevation_perlin = Perlin::new(self.elevation_seed);
        let temperature_perlin = Perlin::new(self.temperature_seed);
        let moisture_perlin = Perlin::new(self.moisture_seed);

        for y in 0..self.height {
            for x in 0..self.width {
                let elevation = self.octave_noise(&elevation_perlin, x as f64, y as f64);
                let temperature = self.octave_noise(&temperature_perlin, x as f64, y as f64);
                let moisture = self.octave_noise(&moisture_perlin, x as f64, y as f64);
                
                let biome = self.determine_biome(temperature, moisture);
                map[y][x] = self.classify_terrain(elevation, biome);
            }
        }

        map
    }

    fn octave_noise(&self, perlin: &Perlin, x: f64, y: f64) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            let nx = (x / self.scale) * frequency + self.offset;
            let ny = (y / self.scale) * frequency + self.offset;
            
            value += perlin.get([nx, ny]) * amplitude;
            max_value += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        value / max_value
    }

    fn determine_biome(&self, temperature: f64, moisture: f64) -> Biome {
        match (temperature, moisture) {
            // Cold biomes
            (t, _) if t < -0.3 => Biome::Tundra,
            (t, _) if t < 0.0 => Biome::Taiga,
            
            // Hot biomes
            (t, m) if t > 0.4 && m < -0.2 => Biome::Desert,
            (t, m) if t > 0.4 && m < 0.2 => Biome::Savanna,
            (t, _) if t > 0.4 => Biome::Tropical,
            
            // Temperate
            _ => Biome::Temperate,
        }
    }

    fn classify_terrain(&self, elevation: f64, biome: Biome) -> Terrain {
        // Water levels (same for all biomes)
        if elevation < -0.5 {
            return Terrain::DeepWater;
        }
        if elevation < -0.2 {
            return Terrain::Water;
        }

        // Land classification by biome
        match biome {
            Biome::Tundra => {
                if elevation > 0.6 {
                    Terrain::Snow
                } else if elevation > 0.3 {
                    Terrain::Mountain
                } else {
                    Terrain::Grass
                }
            }
            Biome::Taiga => {
                if elevation > 0.6 {
                    Terrain::Snow
                } else if elevation > 0.4 {
                    Terrain::Mountain
                } else if elevation > 0.1 {
                    Terrain::Forest
                } else {
                    Terrain::Grass
                }
            }
            Biome::Desert => {
                if elevation > 0.6 {
                    Terrain::Mountain
                } else {
                    Terrain::Sand
                }
            }
            Biome::Savanna => {
                if elevation > 0.6 {
                    Terrain::Mountain
                } else if elevation > 0.4 {
                    Terrain::Forest
                } else if elevation > 0.0 {
                    Terrain::Grass
                } else {
                    Terrain::Sand
                }
            }
            Biome::Tropical => {
                if elevation > 0.6 {
                    Terrain::Mountain
                } else if elevation > 0.2 {
                    Terrain::Forest
                } else {
                    Terrain::Grass
                }
            }
            Biome::Temperate => {
                if elevation > 0.6 {
                    Terrain::Mountain
                } else if elevation > 0.4 {
                    Terrain::Forest
                } else if elevation > 0.0 {
                    Terrain::Grass
                } else {
                    Terrain::Sand
                }
            }
        }
    }

    fn generate_image(&self, map: &[Vec<Terrain>], scale: u32) -> RgbImage {
        let img_width = (self.width as u32) * scale;
        let img_height = (self.height as u32) * scale;
        let mut img = RgbImage::new(img_width, img_height);

        for y in 0..self.height {
            for x in 0..self.width {
                let color = map[y][x].color();
                
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = (x as u32) * scale + dx;
                        let py = (y as u32) * scale + dy;
                        img.put_pixel(px, py, color);
                    }
                }
            }
        }

        img
    }

    fn stats(&self, map: &[Vec<Terrain>]) {
        let mut counts = std::collections::HashMap::new();
        for row in map {
            for &terrain in row {
                *counts.entry(terrain).or_insert(0) += 1;
            }
        }

        println!("\nMap Statistics:");
        let terrains = vec![
            Terrain::DeepWater,
            Terrain::Water,
            Terrain::Sand,
            Terrain::Grass,
            Terrain::Forest,
            Terrain::Mountain,
            Terrain::Snow,
        ];
        
        for terrain in terrains {
            let count = counts.get(&terrain).unwrap_or(&0);
            let percent = (*count as f64 / (self.width * self.height) as f64) * 100.0;
            println!("  {:?}: {:.1}%", terrain, percent);
        }
    }
}

fn main() {
    println!("procedural map generator\n");

    // Generate a larger map
    let generator = MapGenerator::new(400, 300, 50.0, 6);
    let map = generator.generate();
    
    println!("Map size: {}x{}", generator.width, generator.height);
    generator.stats(&map);

    let image = generator.generate_image(&map, 2);
    let output_path = "map.png";
    image.save(output_path).expect("Failed to save image");
    println!("\nMap saved to {}", output_path);

    // Generate a second map variation with different offset
    println!("\n--- Generating second variation ---");
    let gen2 = generator.with_offset(1000.0);
    let map2 = gen2.generate();
    gen2.stats(&map2);
    let image2 = gen2.generate_image(&map2, 2);
    let output_path2 = "map2.png";
    image2.save(output_path2).expect("Failed to save image");
    println!("Second map saved to {}", output_path2);

    // Generate a third map variation with another offset
    println!("\n--- Generating third variation ---");
    let gen3 = generator.with_offset(2000.0);
    let map3 = gen3.generate();
    gen3.stats(&map3);
    let image3 = gen3.generate_image(&map3, 2);
    let output_path3 = "map3.png";
    image3.save(output_path3).expect("Failed to save image");
    println!("Third map saved to {}", output_path3);
}
