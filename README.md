# Merlin

A high-performance game engine written in Rust, leveraging bindless buffering and GPU-driven techniques to create modern rendering pipelines. Designed for experimentation and the implementation of cutting-edge features.

## Features

- **Entity–Component–System (ECS)** architecture leveraging **Single instruction, multiple data (SIMD)** for parallel processing of game entities.
- Heavily relies on **bindless buffers** for efficient resource management.
- **GPU-driven rendering**: GPU frustum culling, indirect drawing.
- **Physically-Based Rendering (PBR)** with Blinn-Phong and Fresnel-Schlick approximations.
- **Cross-platform**: runs natively on Vulkan, Metal, D3D12, and OpenGL; supports WebGPU via Wasm.
- **GLTF/KTX2** import with **BC5, BC6H, and BC7 compression**.
- **Multisample anti-aliasing (MSAA)**.
- **WGSL shaders** support.
- **Multithreaded, pipelined rendering**.
- Skybox shaders with cubemap projection.
- **3D physics**.

![composited](docs/composited.png)
![normals_tbn](docs/normals_tbn.png)
![normals](docs/normals.png)
![occlusion](docs/occlusion.png)
![roughness](docs/roughness.png)
![metallic](docs/metallic.png)
