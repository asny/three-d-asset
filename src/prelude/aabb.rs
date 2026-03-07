use super::math::*;

///
/// A bounding box that aligns with the x, y and z axes.
///
#[derive(Debug, Copy, Clone)]
pub struct AxisAlignedBoundingBox {
    min: Vec3,
    max: Vec3,
}

impl AxisAlignedBoundingBox {
    /// An empty bounding box.
    pub const EMPTY: Self = Self {
        min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    };

    /// An infinitely large bounding box.
    pub const INFINITE: Self = Self {
        min: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        max: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
    };

    ///
    /// Constructs a new bounding box and expands it such that all of the given positions are contained inside the bounding box.
    ///
    pub fn new_with_positions(positions: &[Vec3]) -> Self {
        let mut aabb = Self::EMPTY;
        aabb.expand(positions);
        aabb
    }

    ///
    /// Constructs a new bounding box and expands it such that all of the given positions transformed with the given transformation are contained inside the bounding box.
    /// A position consisting of an x, y and z coordinate corresponds to three consecutive value in the positions array.
    ///
    pub fn new_with_transformed_positions(positions: &[Vec3], transformation: Mat4) -> Self {
        let mut aabb = Self::EMPTY;
        aabb.expand_with_transformation(positions, transformation);
        aabb
    }

    ///
    /// Returns true if the bounding box is empty (ie. constructed by [AxisAlignedBoundingBox::EMPTY]).
    ///
    pub fn is_empty(&self) -> bool {
        self.max.x == f32::NEG_INFINITY
    }

    ///
    /// Returns true if the bounding box is infinitely large (ie. constructed by [AxisAlignedBoundingBox::INFINITE]).
    ///
    pub fn is_infinite(&self) -> bool {
        self.max.x == f32::INFINITY
    }

    ///
    /// Get the minimum coordinate of the bounding box.
    ///
    pub fn min(&self) -> Vec3 {
        self.min
    }

    ///
    /// Get the maximum coordinate of the bounding box.
    ///
    pub fn max(&self) -> Vec3 {
        self.max
    }

    ///
    /// Get the center of the bounding box.
    ///
    pub fn center(&self) -> Vec3 {
        if self.is_infinite() {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            0.5 * self.max + 0.5 * self.min
        }
    }

    ///
    /// Get the size of the bounding box.
    ///
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Expands the bounding box to be at least the given size, keeping the center the same.
    pub fn ensure_size(&mut self, min_size: Vec3) {
        if !self.is_empty() && !self.is_infinite() {
            let size = self.size();
            if size.x < min_size.x {
                let diff = min_size.x - size.x;
                self.min.x -= 0.5 * diff;
                self.max.x += 0.5 * diff;
            }
            if size.y < min_size.y {
                let diff = min_size.y - size.y;
                self.min.y -= 0.5 * diff;
                self.max.y += 0.5 * diff;
            }
            if size.z < min_size.z {
                let diff = min_size.z - size.z;
                self.min.z -= 0.5 * diff;
                self.max.z += 0.5 * diff;
            }
        }
    }

    /// Returns the intersection between this and the other given bounding box.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let min_a = self.min();
        let max_a = self.max();
        let min_b = other.min();
        let max_b = other.max();

        if min_a.x >= max_b.x || min_a.y >= max_b.y || min_b.x >= max_a.x || min_b.y >= max_a.y {
            return None;
        }

        let min = vec3(
            min_a.x.max(min_b.x),
            min_a.y.max(min_b.y),
            min_a.z.max(min_b.z),
        );
        let max = vec3(
            max_a.x.min(max_b.x),
            max_a.y.min(max_b.y),
            max_a.z.min(max_b.z),
        );

        Some(Self::new_with_positions(&[min, max]))
    }

    ///
    /// Expands the bounding box such that all of the given positions are contained inside the bounding box.
    ///
    pub fn expand(&mut self, positions: &[Vec3]) {
        for p in positions {
            self.min.x = self.min.x.min(p.x);
            self.min.y = self.min.y.min(p.y);
            self.min.z = self.min.z.min(p.z);

            self.max.x = self.max.x.max(p.x);
            self.max.y = self.max.y.max(p.y);
            self.max.z = self.max.z.max(p.z);
        }
    }

    ///
    /// Expands the bounding box such that all of the given positions transformed with the given transformation are contained inside the bounding box.
    ///
    pub fn expand_with_transformation(&mut self, positions: &[Vec3], transformation: Mat4) {
        self.expand(
            &positions
                .iter()
                .map(|p| (transformation * p.extend(1.0)).truncate())
                .collect::<Vec<_>>(),
        )
    }

    ///
    /// Expand the bounding box such that it also contains the given other bounding box.
    ///
    pub fn expand_with_aabb(&mut self, other: AxisAlignedBoundingBox) {
        if self.is_empty() {
            *self = other;
        } else if !other.is_empty() {
            self.expand(&[other.min(), other.max()]);
        }
    }

    ///
    /// Transforms the bounding box by the given transformation.
    ///
    pub fn transform(&mut self, transformation: Mat4) {
        if !self.is_empty() && !self.is_infinite() {
            *self = Self::new_with_transformed_positions(
                &[
                    self.min,
                    vec3(self.max.x, self.min.y, self.min.z),
                    vec3(self.min.x, self.max.y, self.min.z),
                    vec3(self.min.x, self.min.y, self.max.z),
                    vec3(self.min.x, self.max.y, self.max.z),
                    vec3(self.max.x, self.min.y, self.max.z),
                    vec3(self.max.x, self.max.y, self.min.z),
                    self.max,
                ],
                transformation,
            );
        }
    }

    ///
    /// Returns the bounding box transformed by the given transformation.
    ///
    pub fn transformed(mut self, transformation: Mat4) -> AxisAlignedBoundingBox {
        self.transform(transformation);
        self
    }

    /// Returns true if the given bounding box is fully inside this bounding box.
    pub fn contains(&self, aabb: AxisAlignedBoundingBox) -> bool {
        !self.is_empty()
            && !aabb.is_empty()
            && self.is_inside(aabb.min())
            && self.is_inside(aabb.max())
    }

    /// Returns true if the given position is inside this bounding box.
    pub fn is_inside(&self, position: Vec3) -> bool {
        self.min.x <= position.x
            && position.x <= self.max.x
            && self.min.y <= position.y
            && position.y <= self.max.y
            && self.min.z <= position.z
            && position.z <= self.max.z
    }

    ///
    /// The distance from position to the point in this bounding box that is closest to position.
    ///
    pub fn distance(&self, position: Vec3) -> f32 {
        let x = (self.min.x - position.x)
            .max(position.x - self.max.x)
            .max(0.0);
        let y = (self.min.y - position.y)
            .max(position.y - self.max.y)
            .max(0.0);
        let z = (self.min.z - position.z)
            .max(position.z - self.max.z)
            .max(0.0);
        let d2 = x * x + y * y + z * z;
        if d2 > 0.001 {
            d2.sqrt()
        } else {
            d2
        }
    }

    ///
    /// The distance from position to the point in this bounding box that is furthest away from position.
    ///
    pub fn distance_max(&self, position: Vec3) -> f32 {
        let x = (position.x - self.min.x)
            .abs()
            .max((self.max.x - position.x).abs());
        let y = (position.y - self.min.y)
            .abs()
            .max((self.max.y - position.y).abs());
        let z = (position.z - self.min.z)
            .abs()
            .max((self.max.z - position.z).abs());
        let d2 = x * x + y * y + z * z;
        if d2 > 0.001 {
            d2.sqrt()
        } else {
            d2
        }
    }
}
