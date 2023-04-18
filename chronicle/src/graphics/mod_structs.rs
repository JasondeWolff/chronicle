use std::rc::Rc;

use crate::resources::{Model, Resource};
use crate::common::{RcCell};

use super::Camera;
use super::Transform;
use super::vulkan::*;

#[derive(Debug, Clone, Copy)]
pub struct DynamicRenderModelProperties {
    pub transform: Transform
}

pub(super) struct DynamicRenderModel {
    pub(super) model_resource: Resource<Model>,
    pub(super) properties: RcCell<DynamicRenderModelProperties>
}

impl DynamicRenderModel {
    pub(super) fn is_active(&self) -> bool {
        self.properties.strong_count() > 1
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderCameraProperties {
    pub camera: Camera,
    pub main: bool
}

pub(super) struct RenderCamera {
    pub(super) properties: RcCell<RenderCameraProperties>
}

impl RenderCamera {
    pub(super) fn is_active(&self) -> bool {
        self.properties.strong_count() > 1
    }
}