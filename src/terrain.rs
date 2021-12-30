
extern crate noise;

use self::noise::{Brownian2, Seed};

use util::{Mat, FixedHeight, MapRange, FixedDimension, NonZero, MappableArray};

pub type Terrain = Mat<f32, FixedHeight>;

#[derive(Copy, Clone)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn gen_terrain(samples: [NonZero<u32>; 2], seed: u32, area: Area, max_height: f32) -> Terrain {
    let samples = samples.map().with(|nz| nz.map(|u| u as usize));

    let fixed_dim = FixedHeight::from_non_zero(samples[1]);
    let samples = [samples[0].val(), samples[1].val()];

    let seed = Seed::new(seed);
    let octaves = 8;
    let persistence = 0.5 as f32;
    let noise = Brownian2::new(noise::open_simplex2, octaves).wavelength(240.0).persistence(persistence);
    let ampl = (0..octaves-1).fold(0.0, |acc, i| acc + persistence.powi(i as i32));

    let vec =
        fixed_dim.coords_iter()
        .take(samples[0] * samples[1])
        .map(|coords| {
            let x = (coords[0] as f32 / samples[0] as f32) * area.w + area.x;
            let y = (coords[1] as f32 / samples[1] as f32) * area.h + area.y;
            noise.apply(&seed, &[x, y]).map_range([-ampl, ampl], [0.0, max_height])
        })
        .collect();

    Mat {
        vec: vec,
        fixed_dim: fixed_dim,
    }
}
