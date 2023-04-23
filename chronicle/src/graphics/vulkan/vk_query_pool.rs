use ash::vk;

use crate::graphics::*;

pub struct VkQueryPool {
    device: Arc<VkLogicalDevice>,
    query_pool: vk::QueryPool,
    count: u32
}

impl VkQueryPool {
    pub fn new(
        device: Arc<VkLogicalDevice>,
        ty: vk::QueryType,
        count: u32
    ) -> Arc<Self> {
        let info = vk::QueryPoolCreateInfo::builder()
            .query_type(ty)
            .query_count(count)
            .build();

        let query_pool = unsafe { device.get_device()
            .create_query_pool(&info, None)
            .expect("Failed to create Query Pool.")
        };

        Arc::new(VkQueryPool {
            device: device,
            query_pool: query_pool,
            count: count
        })
    }

    pub fn get_query_pool(&self) -> vk::QueryPool {
        self.query_pool
    }

    pub fn get_query_count(&self) -> u32 {
        self.count
    }

    pub fn query_results<T: Default + Clone>(&self, count: usize) -> Vec<T> {
        let mut results = vec![T::default(); count];
        unsafe {
            self.device.get_device()
                .get_query_pool_results(
                    self.query_pool,
                    0,
                    self.count,
                    results.as_mut_slice(),
                    vk::QueryResultFlags::WAIT
                )
                .expect("Failed to get query results.");
        }

        results
    }

    pub fn reset(&self) {
        unsafe {
            self.device.get_device()
                .reset_query_pool(self.query_pool, 0, self.count);
        }
    }
}

impl Drop for VkQueryPool {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_query_pool(self.query_pool, None);
        }
    }
}