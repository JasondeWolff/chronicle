use std::rc::Rc;

use crate::graphics::*;

pub struct VkShaderModule {
    device: Rc<VkLogicalDevice>
}