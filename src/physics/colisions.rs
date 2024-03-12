use cgmath::{InnerSpace, Zero};

const ORIGIN: cgmath::Vector3<f32> = cgmath::Vector3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

const MAX_ITERATIONS: usize = 100;

pub trait Collider {
    fn update(&mut self, position: cgmath::Vector3<f32>, rotation: cgmath::Quaternion<f32>);
    fn get_center(&self) -> cgmath::Vector3<f32>;
    fn furthest_point(&self, direction: cgmath::Vector3<f32>) -> cgmath::Vector3<f32>;
}

pub fn collision(collider1: &Box<dyn Collider>, collider2: &Box<dyn Collider>) -> bool {
    gjk_collision(collider1, collider2)
}

fn same_direction(a: cgmath::Vector3<f32>, b: cgmath::Vector3<f32>) -> bool {
    a.dot(b) > 0.0
}

fn gjk_collision(shape1: &Box<dyn Collider>, shape2: &Box<dyn Collider>) -> bool {
    let mut simplex = Vec::new();
    let mut direction = shape2.get_center() - shape1.get_center();
    normalize_or_zero(&mut direction);

    let sup = support(shape1, shape2, direction);

    simplex.push(sup);
    direction = ORIGIN - sup;
    normalize_or_zero(&mut direction);

    for _ in 0..MAX_ITERATIONS {
        let sup = support(shape1, shape2, direction);
        if !same_direction(sup, direction) {
            return false;
        }

        simplex.push(sup);

        if next_simplex(&mut simplex, &mut direction) {
            return true;
        }
    }

    false
}

fn normalize_or_zero(v: &mut cgmath::Vector3<f32>) {
    if !v.is_zero() {
        *v = v.normalize()
    }
}

fn support(
    a: &Box<dyn Collider>,
    b: &Box<dyn Collider>,
    direction: cgmath::Vector3<f32>,
) -> cgmath::Vector3<f32> {
    a.furthest_point(direction) - b.furthest_point(-direction)
}

fn next_simplex(
    simplex: &mut Vec<cgmath::Vector3<f32>>,
    direction: &mut cgmath::Vector3<f32>,
) -> bool {
    match simplex.len() {
        2 => do_line(simplex, direction),
        3 => do_triangle(simplex, direction),
        4 => do_tetrahedron(simplex, direction),
        _ => panic!("Invalid simplex length"),
    }
}

fn do_line(simplex: &mut Vec<cgmath::Vector3<f32>>, direction: &mut cgmath::Vector3<f32>) -> bool {
    normalize_or_zero(direction);

    let a = simplex[0];
    let b = simplex[1];

    let ab = b - a;
    let ao = ORIGIN - a;

    if same_direction(ab, ao) {
        *direction = ab.cross(ao).cross(ab);
    } else {
        *simplex = vec![a];
        *direction = ao;
    }

    direction.is_zero()
}

fn do_triangle(
    simplex: &mut Vec<cgmath::Vector3<f32>>,
    direction: &mut cgmath::Vector3<f32>,
) -> bool {
    normalize_or_zero(direction);

    let a = simplex[0];
    let b = simplex[1];
    let c = simplex[2];

    let ab = b - a;
    let ac = c - a;
    let ao = ORIGIN - a;

    let abc = ab.cross(ac);

    if same_direction(abc.cross(ac), ao) {
        if same_direction(ac, ao) {
            *simplex = vec![a, c];
            *direction = ac.cross(ao).cross(ac);
        } else {
            *simplex = vec![a, b];
            return do_line(simplex, direction);
        }
    } else {
        if same_direction(ab.cross(abc), ao) {
            *simplex = vec![a, b];
            return do_line(simplex, direction);
        }
        if abc.dot(ao) == 0.0 {
            return true;
        }
        if same_direction(abc, ao) {
            *direction = abc;
        } else {
            *simplex = vec![a, c, b];
            *direction = -abc;
        }
    }

    false
}

fn do_tetrahedron(
    simplex: &mut Vec<cgmath::Vector3<f32>>,
    direction: &mut cgmath::Vector3<f32>,
) -> bool {
    normalize_or_zero(direction);

    let a = simplex[0];
    let b = simplex[1];
    let c = simplex[2];
    let d = simplex[3];

    let ab = b - a;
    let ac = c - a;
    let ad = d - a;
    let ao = ORIGIN - a;

    let abc = ab.cross(ac);
    let acd = ac.cross(ad);
    let adb = ad.cross(ab);

    if same_direction(abc, ao) {
        *simplex = vec![a, b, c];
        return do_triangle(simplex, direction);
    }
    if same_direction(acd, ao) {
        *simplex = vec![a, c, d];
        return do_triangle(simplex, direction);
    }
    if same_direction(adb, ao) {
        *simplex = vec![a, d, b];
        return do_triangle(simplex, direction);
    }

    true
}
