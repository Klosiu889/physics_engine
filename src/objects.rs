use crate::model::{Instance, InstanceRaw};
use cgmath::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub enum CollisionType {
    None,
    Simple,
    Complex,
}

pub struct Object {
    label: Option<&'static str>,
    mesh: Mesh,
    collision_type: CollisionType,
    apply_physics: bool,
}

impl Object {
    pub fn new(
        label: Option<&'static str>,
        mesh: Mesh,
        collision_type: Option<CollisionType>,
        apply_physics: bool,
    ) -> Self {
        Self {
            label,
            mesh,
            collision_type: collision_type.unwrap_or(CollisionType::None),
            apply_physics,
        }
    }

    pub fn num_vertices(&self) -> usize {
        self.mesh.num_vertices()
    }

    pub fn num_indices(&self) -> usize {
        self.mesh.num_indices()
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.mesh.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &wgpu::Buffer {
        &self.mesh.index_buffer
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear_color: wgpu::Color,
        render_pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.get_vertex_buffer().slice(..));
        render_pass.set_index_buffer(self.get_index_buffer().slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices() as u32, 0, 0..1);
    }
}

pub struct InstancedObject {
    object: Object,
    instances: Vec<Instance>,
}

impl InstancedObject {
    pub fn new(label: Option<&'static str>, mesh: Mesh) -> Self {
        Self {
            object: Object::new(label, mesh, None, false),
            instances: vec![],
        }
    }

    pub fn to_raw(&self) -> Vec<InstanceRaw> {
        self.instances.iter().map(Instance::to_raw).collect()
    }

    pub fn num_instances(&self) -> usize {
        self.instances.len()
    }

    pub fn add_instance(
        &mut self,
        position: Option<cgmath::Vector3<f32>>,
        rotation: Option<cgmath::Quaternion<f32>>,
        size: f32,
    ) {
        self.instances.push(Instance::new(position, rotation, size));
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
    }

    pub fn create_instance_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let instance_data = self.to_raw();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(
                format!("{} Instance Buffer", self.object.label.unwrap_or("Unnamed")).as_str(),
            ),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        instance_buffer
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, device: &wgpu::Device) {}
}

pub fn to_instanced_object(object: Object) -> InstancedObject {
    let mut instanced_object = InstancedObject::new(object.label, object.mesh);
    instanced_object.add_instance(None, None, 1.0);

    instanced_object
}

pub fn to_object(instanced_object: InstancedObject) -> Object {
    Object::new(
        instanced_object.object.label,
        instanced_object.object.mesh,
        None,
        false,
    )
}

fn calculate_center(vertices: &Vec<Vertex>) -> cgmath::Vector3<f32> {
    vertices
        .iter()
        .fold(cgmath::Vector3::new(0.0, 0.0, 0.0), |acc, v| {
            acc + cgmath::Vector3::from(v.position)
        })
        / vertices.len() as f32
}

fn create_vertex_buffer(vertices: &Vec<Vertex>, device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn create_index_buffer(indices: &Vec<u16>, device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        device: &wgpu::Device,
        position: Option<cgmath::Vector3<f32>>,
        rotation: Option<cgmath::Quaternion<f32>>,
        scale: Option<f32>,
    ) -> Self {
        let center = calculate_center(&vertices);
        let translation =
            cgmath::Matrix4::from_translation(position.unwrap_or(cgmath::Vector3::zero()) - center);
        let rotation = cgmath::Matrix4::from(rotation.unwrap_or(cgmath::Quaternion::zero()));
        let scale = cgmath::Matrix4::from_scale(scale.unwrap_or(1.0));

        let mut vertices = vertices;

        vertices.iter_mut().for_each(|v| {
            v.position =
                (translation * rotation * scale * cgmath::Vector3::from(v.position).extend(1.0))
                    .truncate()
                    .into();
        });

        let vertex_buffer = create_vertex_buffer(&vertices, device);
        let index_buffer = create_index_buffer(&indices, device);

        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn new_exact(vertices: Vec<Vertex>, indices: Vec<u16>, device: &wgpu::Device) -> Self {
        let vertex_buffer = create_vertex_buffer(&vertices, device);
        let index_buffer = create_index_buffer(&indices, device);

        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn num_indices(&self) -> usize {
        self.indices.len()
    }
}
