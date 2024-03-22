pub use crate::prelude::*;

/// UV coordinates which must be between `(0, 0)` indicating the bottom left corner
/// and `(1, 1)` indicating the top right corner.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UvCoordinate {
    /// Coordinate that is 0 at the left edge to 1 at the right edge.
    pub u: f32,
    /// Coordinate that is 0 at the bottom edge to 1 at the top edge.
    pub v: f32,
}

impl From<(f32, f32)> for UvCoordinate {
    fn from(value: (f32, f32)) -> Self {
        Self {
            u: value.0,
            v: value.1,
        }
    }
}

impl From<UvCoordinate> for (f32, f32) {
    fn from(value: UvCoordinate) -> Self {
        (value.u, value.v)
    }
}

impl From<Vec2> for UvCoordinate {
    fn from(value: Vec2) -> Self {
        Self {
            u: value.x,
            v: value.y,
        }
    }
}

impl From<UvCoordinate> for Vec2 {
    fn from(value: UvCoordinate) -> Self {
        Self {
            x: value.u,
            y: value.v,
        }
    }
}

/// A pixel coordinate in physical pixels, where `x` is on the horizontal axis with zero being at the left edge
/// and `y` is on the vertical axis with zero being at bottom edge.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PixelPoint {
    /// The horizontal pixel distance from the left edge.
    pub x: f32,
    /// The vertical pixel distance from the bottom edge.
    pub y: f32,
}

impl From<(f32, f32)> for PixelPoint {
    fn from(value: (f32, f32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<PixelPoint> for (f32, f32) {
    fn from(value: PixelPoint) -> Self {
        (value.x, value.y)
    }
}

impl From<Vec2> for PixelPoint {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<PixelPoint> for Vec2 {
    fn from(value: PixelPoint) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

///
/// Defines the part of the screen/render target that the camera is projecting into.
/// All values should be in physical pixels.
///
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Viewport {
    /// The distance in pixels from the left edge of the screen/render target.
    pub x: i32,
    /// The distance in pixels from the bottom edge of the screen/render target.
    pub y: i32,
    /// The width of the viewport.
    pub width: u32,
    /// The height of the viewport.
    pub height: u32,
}

impl Viewport {
    ///
    /// Creates a new viewport with the bottom left corner at origo `(0, 0)`.
    ///
    pub fn new_at_origo(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    ///
    /// Returns the aspect ratio of this viewport.
    ///
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    ///
    /// Returns the intersection between this and the other Viewport.
    ///
    pub fn intersection(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width =
            (self.x + self.width as i32 - x).clamp(0, other.x + other.width as i32 - x) as u32;
        let height =
            (self.y + self.height as i32 - y).clamp(0, other.y + other.height as i32 - y) as u32;
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

///
/// The type of projection used by a camera (orthographic or perspective) including parameters.
///
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProjectionType {
    /// Orthographic projection
    Orthographic {
        /// Height of the camera film/sensor.
        height: f32,
    },
    /// Perspective projection
    Perspective {
        /// The field of view angle in the vertical direction.
        field_of_view_y: Radians,
    },
}

///
/// Represents a camera used for viewing 3D assets.
///
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Camera {
    viewport: Viewport,
    projection_type: ProjectionType,
    z_near: f32,
    z_far: f32,
    position: Vec3,
    target: Vec3,
    up: Vec3,
    view: Mat4,
    projection: Mat4,
    screen2ray: Mat4,
    frustrum: [Vec4; 6],
}

impl Camera {
    ///
    /// New camera which projects the world with an orthographic projection.
    ///
    pub fn new_orthographic(
        viewport: Viewport,
        position: Vec3,
        target: Vec3,
        up: Vec3,
        height: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut camera = Camera::new(viewport);
        camera.set_view(position, target, up);
        camera.set_orthographic_projection(height, z_near, z_far);
        camera
    }

    ///
    /// New camera which projects the world with a perspective projection.
    ///
    pub fn new_perspective(
        viewport: Viewport,
        position: Vec3,
        target: Vec3,
        up: Vec3,
        field_of_view_y: impl Into<Radians>,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut camera = Camera::new(viewport);
        camera.set_view(position, target, up);
        camera.set_perspective_projection(field_of_view_y, z_near, z_far);
        camera
    }

    ///
    /// Specify the camera to use perspective projection with the given field of view in the y-direction and near and far plane.
    ///
    pub fn set_perspective_projection(
        &mut self,
        field_of_view_y: impl Into<Radians>,
        z_near: f32,
        z_far: f32,
    ) {
        assert!(
            z_near >= 0.0 || z_near < z_far,
            "Wrong perspective camera parameters"
        );
        self.z_near = z_near;
        self.z_far = z_far;
        let field_of_view_y = field_of_view_y.into();
        self.projection_type = ProjectionType::Perspective { field_of_view_y };
        self.projection =
            cgmath::perspective(field_of_view_y, self.viewport.aspect(), z_near, z_far);
        self.update_screen2ray();
        self.update_frustrum();
    }

    ///
    /// Specify the camera to use orthographic projection with the given height and depth.
    /// The view frustum height is `+/- height/2`.
    /// The view frustum width is calculated as `height * viewport.width / viewport.height`.
    /// The view frustum depth is `z_near` to `z_far`.
    ///
    pub fn set_orthographic_projection(&mut self, height: f32, z_near: f32, z_far: f32) {
        assert!(z_near < z_far, "Wrong orthographic camera parameters");
        self.z_near = z_near;
        self.z_far = z_far;
        let width = height * self.viewport.aspect();
        self.projection_type = ProjectionType::Orthographic { height };
        self.projection = cgmath::ortho(
            -0.5 * width,
            0.5 * width,
            -0.5 * height,
            0.5 * height,
            z_near,
            z_far,
        );
        self.update_screen2ray();
        self.update_frustrum();
    }

    ///
    /// Set the current viewport.
    /// Returns whether or not the viewport actually changed.
    ///
    pub fn set_viewport(&mut self, viewport: Viewport) -> bool {
        if self.viewport != viewport {
            self.viewport = viewport;
            match self.projection_type {
                ProjectionType::Orthographic { height } => {
                    self.set_orthographic_projection(height, self.z_near, self.z_far);
                }
                ProjectionType::Perspective { field_of_view_y } => {
                    self.set_perspective_projection(field_of_view_y, self.z_near, self.z_far);
                }
            }
            true
        } else {
            false
        }
    }

    ///
    /// Change the view of the camera.
    /// The camera is placed at the given position, looking at the given target and with the given up direction.
    ///
    pub fn set_view(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        self.position = position;
        self.target = target;
        self.up = up.normalize();
        self.view = Mat4::look_at_rh(
            Point3::from_vec(self.position),
            Point3::from_vec(self.target),
            self.up,
        );
        self.update_screen2ray();
        self.update_frustrum();
    }

    ///
    /// Change the camera view such that it is mirrored in the xz-plane.
    ///
    pub fn mirror_in_xz_plane(&mut self) {
        self.view[1][0] = -self.view[1][0];
        self.view[1][1] = -self.view[1][1];
        self.view[1][2] = -self.view[1][2];
        self.update_screen2ray();
        self.update_frustrum();
    }

    ///
    /// Returns whether or not the given bounding box is within the camera frustum.
    /// It returns false if it is fully outside and true if it is inside or intersects.
    ///
    pub fn in_frustum(&self, aabb: &AxisAlignedBoundingBox) -> bool {
        if aabb.is_infinite() {
            return true;
        }
        // check box outside/inside of frustum
        for i in 0..6 {
            let mut out = 0;
            if self.frustrum[i].dot(vec4(aabb.min().x, aabb.min().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(vec4(aabb.max().x, aabb.min().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(vec4(aabb.min().x, aabb.max().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(vec4(aabb.max().x, aabb.max().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(vec4(aabb.min().x, aabb.min().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(vec4(aabb.max().x, aabb.min().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(vec4(aabb.min().x, aabb.max().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(vec4(aabb.max().x, aabb.max().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if out == 8 {
                return false;
            }
        }
        // TODO: Test the frustum corners against the box planes (http://www.iquilezles.org/www/articles/frustumcorrect/frustumcorrect.htm)

        true
    }

    ///
    /// Returns the 3D position at the given pixel coordinate.
    ///
    pub fn position_at_pixel(&self, pixel: impl Into<PixelPoint>) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { .. } => {
                let coords = self.uv_coordinates_at_pixel(pixel);
                self.position_at_uv_coordinates(coords)
            }
            ProjectionType::Perspective { .. } => *self.position(),
        }
    }

    ///
    /// Returns the 3D position at the given uv coordinate of the viewport.
    ///
    pub fn position_at_uv_coordinates(&self, coords: impl Into<UvCoordinate>) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { .. } => {
                let coords = coords.into();
                let screen_pos = vec4(2. * coords.u - 1., 2. * coords.v - 1.0, -1.0, 1.);
                (self.screen2ray * screen_pos).truncate()
            }
            ProjectionType::Perspective { .. } => *self.position(),
        }
    }

    ///
    /// Returns the 3D view direction at the given pixel coordinate.
    ///
    pub fn view_direction_at_pixel(&self, pixel: impl Into<PixelPoint>) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { .. } => self.view_direction(),
            ProjectionType::Perspective { .. } => {
                let coords = self.uv_coordinates_at_pixel(pixel);
                self.view_direction_at_uv_coordinates(coords)
            }
        }
    }

    ///
    /// Returns the 3D view direction at the given uv coordinate of the viewport.
    ///
    pub fn view_direction_at_uv_coordinates(&self, coords: impl Into<UvCoordinate>) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { .. } => self.view_direction(),
            ProjectionType::Perspective { .. } => {
                let coords = coords.into();
                let screen_pos = vec4(2. * coords.u - 1., 2. * coords.v - 1.0, 0., 1.);
                (self.screen2ray * screen_pos).truncate().normalize()
            }
        }
    }

    ///
    /// Returns the uv coordinate for the given pixel coordinate.
    ///
    pub fn uv_coordinates_at_pixel(&self, pixel: impl Into<PixelPoint>) -> UvCoordinate {
        let pixel = pixel.into();
        (
            (pixel.x - self.viewport.x as f32) / self.viewport.width as f32,
            (pixel.y - self.viewport.y as f32) / self.viewport.height as f32,
        )
            .into()
    }

    ///
    /// Returns the uv coordinate for the given world position.
    ///
    pub fn uv_coordinates_at_position(&self, position: Vec3) -> UvCoordinate {
        let proj = self.projection() * self.view() * position.extend(1.0);
        (
            0.5 * (proj.x / proj.w.abs() + 1.0),
            0.5 * (proj.y / proj.w.abs() + 1.0),
        )
            .into()
    }

    ///
    /// Returns the pixel coordinate for the given uv coordinate.
    ///
    pub fn pixel_at_uv_coordinates(&self, coords: impl Into<UvCoordinate>) -> PixelPoint {
        let coords = coords.into();
        (
            coords.u * self.viewport.width as f32 + self.viewport.x as f32,
            coords.v * self.viewport.height as f32 + self.viewport.y as f32,
        )
            .into()
    }

    ///
    /// Returns the pixel coordinate for the given world position.
    ///
    pub fn pixel_at_position(&self, position: Vec3) -> PixelPoint {
        self.pixel_at_uv_coordinates(self.uv_coordinates_at_position(position))
    }

    ///
    /// Returns the type of projection (orthographic or perspective) including parameters.
    ///
    pub fn projection_type(&self) -> &ProjectionType {
        &self.projection_type
    }

    ///
    /// Returns the view matrix, ie. the matrix that transforms objects from world space (as placed in the world) to view space (as seen from this camera).
    ///
    pub fn view(&self) -> &Mat4 {
        &self.view
    }

    ///
    /// Returns the projection matrix, ie. the matrix that projects objects in view space onto this cameras image plane.
    ///
    pub fn projection(&self) -> &Mat4 {
        &self.projection
    }

    ///
    /// Returns the viewport.
    ///
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    ///
    /// Returns the distance to the near plane of the camera frustum.
    ///
    pub fn z_near(&self) -> f32 {
        self.z_near
    }

    ///
    /// Returns the distance to the far plane of the camera frustum.
    ///
    pub fn z_far(&self) -> f32 {
        self.z_far
    }

    ///
    /// Returns the position of this camera.
    ///
    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    ///
    /// Returns the target of this camera, ie the point that this camera looks towards.
    ///
    pub fn target(&self) -> &Vec3 {
        &self.target
    }

    ///
    /// Returns the up direction of this camera (might not be orthogonal to the view direction).
    ///
    pub fn up(&self) -> &Vec3 {
        &self.up
    }

    ///
    /// Returns the view direction of this camera, ie. the direction the camera is looking.
    ///
    pub fn view_direction(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    ///
    /// Returns the right direction of this camera.
    ///
    pub fn right_direction(&self) -> Vec3 {
        self.view_direction().cross(self.up)
    }

    fn new(viewport: Viewport) -> Camera {
        Camera {
            viewport,
            projection_type: ProjectionType::Orthographic { height: 1.0 },
            z_near: 0.0,
            z_far: 0.0,
            frustrum: [vec4(0.0, 0.0, 0.0, 0.0); 6],
            position: vec3(0.0, 0.0, 5.0),
            target: vec3(0.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
            view: Mat4::identity(),
            projection: Mat4::identity(),
            screen2ray: Mat4::identity(),
        }
    }

    fn update_screen2ray(&mut self) {
        let mut v = self.view;
        if let ProjectionType::Perspective { .. } = self.projection_type {
            v[3] = vec4(0.0, 0.0, 0.0, 1.0);
        }
        self.screen2ray = (self.projection * v).invert().unwrap();
    }

    fn update_frustrum(&mut self) {
        let m = self.projection * self.view;
        self.frustrum = [
            vec4(m.x.w + m.x.x, m.y.w + m.y.x, m.z.w + m.z.x, m.w.w + m.w.x),
            vec4(m.x.w - m.x.x, m.y.w - m.y.x, m.z.w - m.z.x, m.w.w - m.w.x),
            vec4(m.x.w + m.x.y, m.y.w + m.y.y, m.z.w + m.z.y, m.w.w + m.w.y),
            vec4(m.x.w - m.x.y, m.y.w - m.y.y, m.z.w - m.z.y, m.w.w - m.w.y),
            vec4(m.x.w + m.x.z, m.y.w + m.y.z, m.z.w + m.z.z, m.w.w + m.w.z),
            vec4(m.x.w - m.x.z, m.y.w - m.y.z, m.z.w - m.z.z, m.w.w - m.w.z),
        ];
    }

    ///
    /// Translate the camera by the given change while keeping the same view and up directions.
    ///
    pub fn translate(&mut self, change: &Vec3) {
        self.set_view(self.position + change, self.target + change, self.up);
    }

    ///
    /// Rotates the camera by the angle delta around the 'right' direction.
    ///
    pub fn pitch(&mut self, delta: impl Into<Radians>) {
        let target = (self.view.invert().unwrap()
            * Mat4::from_angle_x(delta)
            * self.view
            * self.target.extend(1.0))
        .truncate();
        if (target - self.position).normalize().dot(self.up).abs() < 0.999 {
            self.set_view(self.position, target, self.up);
        }
    }

    ///
    /// Rotates the camera by the angle delta around the 'up' direction.
    ///
    pub fn yaw(&mut self, delta: impl Into<Radians>) {
        let target = (self.view.invert().unwrap()
            * Mat4::from_angle_y(delta)
            * self.view
            * self.target.extend(1.0))
        .truncate();
        self.set_view(self.position, target, self.up);
    }

    ///
    /// Rotates the camera by the angle delta around the 'view' direction.
    ///
    pub fn roll(&mut self, delta: impl Into<Radians>) {
        let up = (self.view.invert().unwrap()
            * Mat4::from_angle_z(delta)
            * self.view
            * (self.up + self.position).extend(1.0))
        .truncate()
            - self.position;
        self.set_view(self.position, self.target, up.normalize());
    }

    ///
    /// Rotate the camera around the given point while keeping the same distance to the point.
    /// The input `x` specifies the amount of rotation in the left direction and `y` specifies the amount of rotation in the up direction.
    /// If you want the camera up direction to stay fixed, use the [rotate_around_with_fixed_up](Camera::rotate_around_with_fixed_up) function instead.
    ///
    pub fn rotate_around(&mut self, point: &Vec3, x: f32, y: f32) {
        let dir = (point - self.position()).normalize();
        let right = dir.cross(self.up);
        let up = right.cross(dir);
        let new_dir = (point - self.position() + right * x - up * y).normalize();
        let rotation = rotation_matrix_from_dir_to_dir(dir, new_dir);
        let new_position = (rotation * (self.position() - point).extend(1.0)).truncate() + point;
        let new_target = (rotation * (self.target() - point).extend(1.0)).truncate() + point;
        self.set_view(new_position, new_target, up);
    }

    ///
    /// Rotate the camera around the given point while keeping the same distance to the point and the same up direction.
    /// The input `x` specifies the amount of rotation in the left direction and `y` specifies the amount of rotation in the up direction.
    ///
    pub fn rotate_around_with_fixed_up(&mut self, point: &Vec3, x: f32, y: f32) {
        // Since rotations in linear algebra always describe rotations about the origin, we
        // subtract the point, do all rotations, and add the point again
        let position = self.position() - point;
        let target = self.target() - point;
        let up = self.up.normalize();
        // We use Rodrigues' rotation formula to rotate around the fixed `up` vector and around the
        // horizon which is calculated from the camera's view direction and `up`
        // https://en.wikipedia.org/wiki/Rodrigues%27_rotation_formula
        let k_x = up;
        let k_y = (target - position).cross(up).normalize();
        // Prepare cos and sin terms, inverted because the method rotates left and up while
        // rotations follow the right hand rule
        let cos_x = (-x).cos();
        let sin_x = (-x).sin();
        let cos_y = (-y).cos();
        let sin_y = (-y).sin();
        // Do the rotations following the rotation formula
        let rodrigues =
            |v, k: Vec3, cos, sin| v * cos + k.cross(v) * sin + k * k.dot(v) * (1.0 - cos);
        let position_x = rodrigues(position, k_x, cos_x, sin_x);
        let target_x = rodrigues(target, k_x, cos_x, sin_x);
        let position_y = rodrigues(position_x, k_y, cos_y, sin_y);
        let target_y = rodrigues(target_x, k_y, cos_y, sin_y);
        // Forbid to face the camera exactly up or down, fall back to just rotate in x direction
        let new_dir = (target_y - position_y).normalize();
        if new_dir.dot(up).abs() < 0.999 {
            self.set_view(position_y + point, target_y + point, self.up);
        } else {
            self.set_view(position_x + point, target_x + point, self.up);
        }
    }

    ///
    /// Moves the camera towards the given point by the amount delta while keeping the given minimum and maximum distance to the point.
    ///
    pub fn zoom_towards(
        &mut self,
        point: &Vec3,
        delta: f32,
        minimum_distance: f32,
        maximum_distance: f32,
    ) {
        let minimum_distance = minimum_distance.max(0.0);
        assert!(
            minimum_distance < maximum_distance,
            "minimum_distance larger than maximum_distance"
        );

        let position = *self.position();
        let distance = point.distance(position);
        let direction = (point - position).normalize();
        let target = *self.target();
        let up = self.up;
        let new_distance = (distance - delta).clamp(minimum_distance, maximum_distance);
        let new_position = point - direction * new_distance;
        self.set_view(new_position, new_position + (target - position), up);
        if let ProjectionType::Orthographic { height } = self.projection_type() {
            let h = new_distance * height / distance;
            let z_near = self.z_near();
            let z_far = self.z_far();
            self.set_orthographic_projection(h, z_near, z_far);
        }
    }
}
