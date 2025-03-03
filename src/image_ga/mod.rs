use indexmap::IndexMap;
use ndarray::{Array2, Array3, ArrayView3, ArrayViewMut3, Axis, arr1, s, stack};
use num_traits::{One, Zero};
use std::hash::Hash;

/// A grayscale image with transparency.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageGA<T> {
    /// Image data stored in row-major order.
    pub data: Array3<T>,
}

impl<T: Copy + PartialOrd + Zero + One> ImageGA<T> {
    /// Creates a new ImageGA from the provided data.
    pub fn new(data: Array3<T>) -> Self {
        debug_assert!(data.dim().0 > 0);
        debug_assert!(data.dim().1 > 0);
        debug_assert!(data.dim().2 == 2);
        Self { data }
    }

    /// Creates an empty image (all zeros) with alpha set to one.
    pub fn empty(resolution: [usize; 2]) -> Self {
        debug_assert!(resolution.iter().all(|&r| r > 0));
        let mut data = Array3::zeros((resolution[0], resolution[1], 2));
        data.slice_mut(s![.., .., 1]).fill(T::one());
        Self { data }
    }

    /// Creates an image filled with a constant value.
    pub fn filled(resolution: [usize; 2], value: [T; 2]) -> Self {
        debug_assert!(resolution.iter().all(|&r| r > 0));
        let mut data = Array3::zeros((resolution[0], resolution[1], 2));
        data.slice_mut(s![.., .., 0]).fill(value[0]);
        data.slice_mut(s![.., .., 1]).fill(value[1]);
        Self { data }
    }

    /// Creates an ImageGA from two grayscale layers.
    pub fn from_layers(layers: [Array2<T>; 2]) -> Self {
        debug_assert!(layers.iter().all(|layer| layer.ncols() > 0));
        debug_assert!(layers.iter().all(|layer| layer.nrows() > 0));
        debug_assert!(layers.iter().all(|layer| layer.dim() == layers[0].dim()));
        let data =
            stack(Axis(2), &[layers[0].view(), layers[1].view()]).expect("Failed to stack layers");
        Self { data }
    }

    /// Returns the height of the image.
    pub fn height(&self) -> usize {
        self.data.dim().0
    }

    /// Returns the width of the image.
    pub fn width(&self) -> usize {
        self.data.dim().1
    }

    /// Gets the value of a component at the specified position.
    pub fn get_component(&self, coords: [usize; 2], component: usize) -> T {
        debug_assert!(component < 2);
        self.data[[coords[0], coords[1], component]]
    }

    /// Sets the value of a component at the specified position.
    pub fn set_component(&mut self, coords: [usize; 2], component: usize, value: T) {
        debug_assert!(component < 2);
        self.data[[coords[0], coords[1], component]] = value;
    }

    /// Gets the pixel at the specified position.
    pub fn get_pixel(&self, coords: [usize; 2]) -> [T; 2] {
        let pixel_slice = self.data.slice(s![coords[0], coords[1], ..]);
        pixel_slice
            .as_slice()
            .expect("Pixel slice not contiguous")
            .try_into()
            .expect("Slice length mismatch")
    }

    /// Sets the pixel at the specified position.
    pub fn set_pixel(&mut self, coords: [usize; 2], pixel: [T; 2]) {
        let mut view = self.data.slice_mut(s![coords[0], coords[1], ..]);
        view.assign(&arr1(&pixel));
    }

    /// Gets a component layer of the image.
    pub fn get_layer(&self, component: usize) -> Array2<T> {
        debug_assert!(component < 2);
        self.data.slice(s![.., .., component]).to_owned()
    }

    /// Transposes the image.
    pub fn transpose(&mut self) {
        self.data = self.data.clone().permuted_axes([1, 0, 2]).to_owned();
    }

    /// Flips the image vertically.
    pub fn flip_vertical(&mut self) {
        self.data.invert_axis(Axis(0));
    }

    /// Flips the image horizontally.
    pub fn flip_horizontal(&mut self) {
        self.data.invert_axis(Axis(1));
    }

    /// Rotates the image 90 degrees clockwise.
    pub fn rotate_clockwise(&mut self) {
        let mut new_data = self.data.clone().permuted_axes([1, 0, 2]).to_owned();
        new_data.invert_axis(Axis(1));
        self.data = new_data;
    }

    /// Rotates the image 90 degrees anticlockwise.
    pub fn rotate_anticlockwise(&mut self) {
        let mut new_data = self.data.clone().permuted_axes([1, 0, 2]).to_owned();
        new_data.invert_axis(Axis(0));
        self.data = new_data;
    }

    /// Rotates the image 180 degrees.
    pub fn rotate_180(&mut self) {
        self.data.invert_axis(Axis(0));
        self.data.invert_axis(Axis(1));
    }

    /// Extract a portion of the image.
    pub fn extract(&self, start: [usize; 2], size: [usize; 2]) -> ImageGA<T> {
        debug_assert!(start[0] + size[0] <= self.height());
        debug_assert!(start[1] + size[1] <= self.width());
        debug_assert!(size.iter().all(|&s| s > 0));
        Self::new(
            self.data
                .slice(s![
                    start[0]..start[0] + size[0],
                    start[1]..start[1] + size[1],
                    ..
                ])
                .to_owned(),
        )
    }

    /// Create a view to a portion of the image.
    pub fn view(&self, start: [usize; 2], size: [usize; 2]) -> ArrayView3<T> {
        debug_assert!(start[0] + size[0] <= self.height());
        debug_assert!(start[1] + size[1] <= self.width());
        debug_assert!(size.iter().all(|&s| s > 0));
        self.data.slice(s![
            start[0]..start[0] + size[0],
            start[1]..start[1] + size[1],
            ..
        ])
    }

    /// Create a mutable view to a portion of the image.
    pub fn view_mut(&mut self, start: [usize; 2], size: [usize; 2]) -> ArrayViewMut3<T> {
        debug_assert!(start[0] + size[0] <= self.height());
        debug_assert!(start[1] + size[1] <= self.width());
        debug_assert!(size.iter().all(|&s| s > 0));
        self.data.slice_mut(s![
            start[0]..start[0] + size[0],
            start[1]..start[1] + size[1],
            ..
        ])
    }

    /// Extract a tile from the image.
    pub fn extract_tile(&self, tile_size: [usize; 2], tile_index: [usize; 2]) -> ImageGA<T> {
        debug_assert!(tile_size.iter().all(|&s| s > 0));
        debug_assert!(tile_index[0] < self.height() / tile_size[0]);
        debug_assert!(tile_index[1] < self.width() / tile_size[1]);
        self.extract(
            [tile_index[0] * tile_size[0], tile_index[1] * tile_size[1]],
            tile_size,
        )
    }

    /// Create a view to a tile of the image.
    pub fn view_tile(&self, tile_size: [usize; 2], tile_index: [usize; 2]) -> ArrayView3<T> {
        debug_assert!(tile_size.iter().all(|&s| s > 0));
        debug_assert!(tile_index[0] < self.height() / tile_size[0]);
        debug_assert!(tile_index[1] < self.width() / tile_size[1]);
        self.data.slice(s![
            tile_index[0] * tile_size[0]..(tile_index[0] + 1) * tile_size[0],
            tile_index[1] * tile_size[1]..(tile_index[1] + 1) * tile_size[1],
            ..
        ])
    }

    /// Create a mutable view to a tile of the image.
    pub fn view_tile_mut(
        &mut self,
        tile_size: [usize; 2],
        tile_index: [usize; 2],
    ) -> ArrayViewMut3<T> {
        debug_assert!(tile_size.iter().all(|&s| s > 0));
        debug_assert!(tile_index[0] < self.height() / tile_size[0]);
        debug_assert!(tile_index[1] < self.width() / tile_size[1]);

        self.data.slice_mut(s![
            tile_index[0] * tile_size[0]..(tile_index[0] + 1) * tile_size[0],
            tile_index[1] * tile_size[1]..(tile_index[1] + 1) * tile_size[1],
            ..
        ])
    }

    /// Split the image into equal-sized tiles.
    pub fn tiles(&self, tile_size: [usize; 2]) -> Array2<ImageGA<T>> {
        let height = self.height();
        let width = self.width();

        debug_assert!(height % tile_size[0] == 0);
        debug_assert!(width % tile_size[1] == 0);

        let tile_rows = height / tile_size[0];
        let tile_cols = width / tile_size[1];

        Array2::from_shape_fn((tile_rows, tile_cols), |(row, col)| {
            let y = row * tile_size[0];
            let x = col * tile_size[1];
            let tile = self
                .data
                .slice(s![y..y + tile_size[0], x..x + tile_size[1], ..])
                .to_owned();
            ImageGA { data: tile }
        })
    }
}

impl<T: Copy + PartialOrd + Zero + One + Eq + Hash> ImageGA<T> {
    /// Create a list of all unique tiles in the image and their frequency.
    pub fn unique_tiles(&self, tile_size: [usize; 2]) -> Vec<(ImageGA<T>, usize)> {
        let tiles = self.tiles(tile_size);
        let mut freq_map: IndexMap<Vec<T>, (ImageGA<T>, usize)> = IndexMap::new();

        for tile in tiles.iter() {
            let key: Vec<T> = tile.data.iter().copied().collect();
            freq_map
                .entry(key)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((tile.clone(), 1));
        }

        freq_map.into_values().collect()
    }
}

mod float;
mod u8;
