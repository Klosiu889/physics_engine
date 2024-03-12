use std::fmt::Debug;
use cgmath::{InnerSpace, SquareMatrix, Zero};

use self::collisions::{collision, Collider};
pub mod collisions;
pub mod solver;

pub struct PhysicalObject {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    pub velocity: cgmath::Vector3<f32>,
    forces: cgmath::Vector3<f32>,
    mass: f32,
    have_gravity: bool,
    have_collision: bool,
    collider: Box<dyn Collider>,
}

impl PhysicalObject {
    pub fn new(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        velocity: Option<cgmath::Vector3<f32>>,
        forces: Option<cgmath::Vector3<f32>>,
        mass: f32,
        collider: Box<dyn Collider>,
    ) -> Self {
        PhysicalObject {
            position,
            rotation,
            velocity: velocity.unwrap_or(cgmath::Vector3::zero()),
            forces: forces.unwrap_or(cgmath::Vector3::zero()),
            mass,
            have_gravity: false,
            have_collision: false,
            collider,
        }
    }

    const GRAVITY: cgmath::Vector3<f32> = cgmath::Vector3 {
        x: 0.0,
        y: -9.81,
        z: 0.0,
    };

    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn get_rotation(&self) -> cgmath::Quaternion<f32> {
        self.rotation
    }

    pub fn update_rotation(&mut self, rotation: cgmath::Quaternion<f32>) {
        self.rotation = rotation;
    }

    pub fn update(&mut self, dt: f32) {
        let forces = if self.have_gravity {
            self.forces + self.mass * Self::GRAVITY
        } else {
            self.forces
        };
        self.velocity += forces / self.mass * dt;
        self.position += self.velocity * dt;
        self.position.y = self.position.y.max(-1.0);

        self.collider.update(self.position, self.rotation);
    }

    pub fn apply_force(&mut self, force: cgmath::Vector3<f32>) {
        self.forces = force;
    }

    pub fn enable_gravity(&mut self) {
        self.have_gravity = true;
    }

    pub fn disable_gravity(&mut self) {
        self.have_gravity = false;
    }

    pub fn enable_collision(&mut self) {
        self.have_collision = true;
    }

    pub fn disable_collision(&mut self) {
        self.have_collision = false;
    }

    pub fn collide(&self, other: &PhysicalObject) -> bool {
        self.have_collision && other.have_collision && collision(&self.collider, &other.collider)
    }
}

pub struct Sphere {
    center: cgmath::Vector3<f32>,
    radius: f32,
}

impl Sphere {
    pub fn new(center: cgmath::Vector3<f32>, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Collider for Sphere {
    fn update(&mut self, position: cgmath::Vector3<f32>, _rotation: cgmath::Quaternion<f32>) {
        self.center = position;
    }

    fn get_center(&self) -> cgmath::Vector3<f32> {
        self.center
    }

    fn furthest_point(&self, direction: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        self.center + direction.normalize_to(self.radius)
    }
}

pub struct ConvexPolyhedron {
    vertices: Vec<cgmath::Vector3<f32>>,
    center: cgmath::Vector3<f32>,
    transform_matrix: cgmath::Matrix4<f32>,
}

impl ConvexPolyhedron {
    pub fn new(vertices: Vec<cgmath::Vector3<f32>>) -> Self {
        let center = vertices
            .iter()
            .fold(cgmath::Vector3::zero(), |acc, v| acc + *v)
            / vertices.len() as f32;
        let vertices = vertices
            .iter()
            .map(|v| v - center)
            .collect::<Vec<cgmath::Vector3<f32>>>();
        let transform_matrix = cgmath::Matrix4::from_translation(center);
        Self {
            vertices,
            center,
            transform_matrix,
        }
    }
}

impl Collider for ConvexPolyhedron {
    fn update(&mut self, position: cgmath::Vector3<f32>, rotation: cgmath::Quaternion<f32>) {
        self.center = position;
        self.transform_matrix =
            cgmath::Matrix4::from_translation(self.center) * cgmath::Matrix4::from(rotation);
    }

    fn get_center(&self) -> cgmath::Vector3<f32> {
        self.center
    }

    fn furthest_point(&self, direction: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        let direction_transformed =
            (self.transform_matrix.invert().unwrap() * direction.extend(0.0)).truncate();

        let mut max_dot = -f32::INFINITY;
        let mut max_vertex = cgmath::Vector3::zero();
        for vertex in &self.vertices {
            let dot = vertex.dot(direction_transformed);
            if dot > max_dot {
                max_dot = dot;
                max_vertex = *vertex
            }
        }

        (self.transform_matrix * max_vertex.extend(1.0)).truncate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_furthest_point() {
        let sphere = Sphere::new(cgmath::Vector3::new(0.0, 0.0, 0.0), 1.0);
        let direction = cgmath::Vector3::new(1.0, 0.0, 0.0);
        let furthest_point = sphere.furthest_point(direction);
        assert_eq!(furthest_point, cgmath::Vector3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_convex_polyhedron_furthest_point() {
        let vertices = vec![
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(1.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
        ];
        let polyhedron = ConvexPolyhedron::new(vertices);
        let direction = cgmath::Vector3::new(1.0, 0.0, 0.0);
        let furthest_point = polyhedron.furthest_point(direction);
        assert_eq!(furthest_point, cgmath::Vector3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_sphere_collision() {
        let sphere1: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(0.0, 0.0, 0.0), 1.0));
        let sphere2: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(2.0, 0.0, 0.0), 1.0));
        assert_eq!(
            collision(&sphere1, &sphere2),
            false,
            "Spheres too far apart"
        );

        let sphere1: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(0.0, 0.0, 0.0), 1.0));
        let sphere2: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(1.0, 0.0, 0.0), 1.0));
        assert_eq!(
            collision(&sphere1, &sphere2),
            true,
            "Spheres collide on one point"
        );

        let sphere1: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(0.0, 0.0, 0.0), 1.0));
        let sphere2: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(1.0, 0.0, 0.0), 2.0));
        assert_eq!(
            collision(&sphere1, &sphere2),
            true,
            "Spheres collide on more than one point"
        );

        let sphere1: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(0.0, 0.0, 0.0), 1.0));
        let sphere2: Box<dyn Collider> =
            Box::new(Sphere::new(cgmath::Vector3::new(1.0, 0.0, 0.0), 3.0));
        assert_eq!(
            collision(&sphere1, &sphere2),
            true,
            "Spheres inside each other"
        );
    }
}
