use ash::vk;
use model::Vertex;

use crate::graphics::*;

pub struct VkBlas {
    offset: vk::AccelerationStructureBuildRangeInfoKHR,
    accel_info: ArcMutex<vk::AccelerationStructureBuildGeometryInfoKHR>,
    geometries: Vec<vk::AccelerationStructureGeometryKHR>,
    accel: Option<Arc<VkAccel>>,
    dirty: bool
}

pub struct VkAccelBuildInfo {
    pub build_info: ArcMutex<vk::AccelerationStructureBuildGeometryInfoKHR>,
    pub size_info: vk::AccelerationStructureBuildSizesInfoKHR,
    range_info: Arc<Vec<vk::AccelerationStructureBuildRangeInfoKHR>>,
    pub accel: Option<Arc<VkAccel>>,
    pub cleanup: Option<Arc<VkAccel>>
}

impl VkAccelBuildInfo {
    pub fn get_range_info(&self) -> &[vk::AccelerationStructureBuildRangeInfoKHR] {
        self.range_info.as_slice()
    }
}

impl VkBlas {
    pub fn new(
        vertex_buffer: &VkDataBuffer<Vertex>,
        index_buffer: &VkDataBuffer<u32>,
        build_flags: vk::BuildAccelerationStructureFlagsKHR
    ) -> ArcMutex<Self> {
        let vertex_buffer_address = vertex_buffer.get_buffer().get_device_address();
        let index_buffer_address = index_buffer.get_buffer().get_device_address();

        let primitive_count = index_buffer.get_count() / 3;

        let geometries = vec![vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                triangles: vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                    .vertex_format(vk::Format::R32G32B32_SFLOAT)
                    .vertex_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: vertex_buffer_address
                    })
                    .vertex_stride(vertex_buffer.get_stride() as u64)
                    .index_type(vk::IndexType::UINT32)
                    .index_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: index_buffer_address
                    })
                    .max_vertex(vertex_buffer.get_count())
                    .build()
        }).build()];

        let offset = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(primitive_count)
            .build();

        let info = ArcMutex::new(vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .geometries(&geometries)
            .flags(build_flags)
            .build());

        ArcMutex::new(VkBlas {
            offset: offset,
            accel_info: info,
            geometries: geometries,
            accel: None,
            dirty: true
        })
    }

    pub fn get_accel_ref(&self) -> vk::AccelerationStructureReferenceKHR {
        self.accel.as_ref().unwrap().get_accel_ref()
    }

    pub fn build(
        app: &mut VkApp,
        blases: &Vec<ArcMutex<VkBlas>>,
        build_type: vk::AccelerationStructureBuildTypeKHR
    ) {
        let device = app.get_device();
        let accel_props = app.get_physical_device().get_accel_properties();
    
        let mut _total_size = 0;
        let mut compaction_count = 0;
        let mut max_scratch_size = 0;

        // Prepare the build infos
        let mut build_infos = Vec::with_capacity(blases.len());
        for blas in blases {
            let blas = blas.as_mut();

            let build_info = blas.get_info();
            
            let mut prim_count = Vec::with_capacity(blas.get_offset().len());
            for offset in blas.get_offset().as_ref() {
                prim_count.push(offset.primitive_count);
            }
            
            let build_sizes = unsafe {
                device.accel_loader()
                    .get_acceleration_structure_build_sizes(
                        build_type,
                        &build_info.as_ref(),
                        &prim_count
                    )
            };

            _total_size += build_sizes.acceleration_structure_size;
            max_scratch_size += std::cmp::max(max_scratch_size, build_sizes.build_scratch_size);
            if build_info.as_ref().flags.contains(vk::BuildAccelerationStructureFlagsKHR::ALLOW_COMPACTION) {
                compaction_count += 1;
            }

            build_infos.push(VkAccelBuildInfo {
                build_info: build_info,
                size_info: build_sizes,
                range_info: blas.get_offset(),
                accel: None,
                cleanup: None
            });
        }

        // Create a temp staging buffer
        let scratch_buffer = VkBuffer::new(
            "Blas SCRATCH BUFFER".to_owned(),
            device.clone(),
            app.get_allocator(),
            max_scratch_size,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL, // Match with "build_type"?
            Some(accel_props.min_acceleration_structure_scratch_offset_alignment as u64)
        );
        let buffer_address = scratch_buffer.get_device_address();

        // Create a query pool to store blas sizes for compaction
        let query_pool = if compaction_count > 0 {
            assert_eq!(compaction_count, blases.len(), "Failed to build blases. (It's not allowed to compact partially)");
            Some(VkQueryPool::new(
                device.clone(),
                vk::QueryType::ACCELERATION_STRUCTURE_COMPACTED_SIZE_KHR,
                compaction_count as u32
            ))
        } else {
            None
        };

        let mut indices = Vec::new();
        let mut batch_size = 0;
        let batch_limit = byte_unit::n_mib_bytes!(256) as u64;
        for i in 0..blases.len() {
            indices.push(i);
            batch_size += build_infos[i].size_info.acceleration_structure_size;

            if batch_size >= batch_limit || i == blases.len() - 1 {
                { // Build blas
                    let cmd_queue = app.get_cmd_queue();
                    let mut cmd_queue = cmd_queue.as_mut();
                    let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                        let cmd_buffer_ref = cmd_buffer.as_ref();
                        cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                        cmd_buffer_ref.create_blas(
                            &mut build_infos,
                            &indices,
                            buffer_address,
                            query_pool.clone()
                        );

                        cmd_buffer_ref.end();
                    }
                    cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
                    app.get_device().wait_idle();
                }

                // Compact blas if hinted
                if let Some(query_pool) = query_pool.clone() {
                    let cmd_queue = app.get_cmd_queue();
                    let mut cmd_queue = cmd_queue.as_mut();
                    let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                        let cmd_buffer_ref = cmd_buffer.as_ref();
                        cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                        cmd_buffer_ref.compact_blas(
                            &mut build_infos,
                            &indices,
                            query_pool.clone()
                        );

                        cmd_buffer_ref.end();
                    }
                    cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
                    app.get_device().wait_idle();
                }

                batch_size = 0;
                indices.clear();
            }
        }

        for (i, build_info) in build_infos.iter().enumerate() {
            let mut blas = blases[i].as_mut();
            blas.accel = build_info.accel.clone();
            blas.dirty = false;
        }
    }

    fn get_offset(&self) -> Arc<Vec<vk::AccelerationStructureBuildRangeInfoKHR>> {
        Arc::new(vec![self.offset])
    }

    fn get_info(&self) -> ArcMutex<vk::AccelerationStructureBuildGeometryInfoKHR> {
        self.accel_info.clone()
    }
}