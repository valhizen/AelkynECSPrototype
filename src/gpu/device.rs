use ash::{Device as AshDevice, Entry, Instance, ext::debug_utils, khr::surface, vk};
use std::{borrow::Cow, error::Error, ffi, os::raw::c_char};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

// Vulkan Debug Message
unsafe extern "system" fn vulkan_debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    unsafe {
        let data = *p_data;
        let id = data.message_id_number;
        let name = if data.p_message_id_name.is_null() {
            Cow::from("")
        } else {
            ffi::CStr::from_ptr(data.p_message_id_name).to_string_lossy()
        };
        let msg = if data.p_message.is_null() {
            Cow::from("")
        } else {
            ffi::CStr::from_ptr(data.p_message).to_string_lossy()
        };

        if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
            tracing::error!(target: "vulkan", "{msg_type:?} [{name} ({id})] : {msg}");
        } else {
            tracing::warn!(target: "vulkan", "{msg_type:?} [{name} ({id})] : {msg}");
        }
        vk::FALSE
    }
}

pub struct Device {
    pub entry: Entry,
    pub instance: Instance,
    pub device: AshDevice,
    pub surface_loader: surface::Instance,
    pub debug_utils_loader: Option<debug_utils::Instance>,
    pub debug_call_back: vk::DebugUtilsMessengerEXT,
    pub pdevice: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,
    pub surface: vk::SurfaceKHR,
}

impl Device {
    pub fn new(window: &winit::window::Window) -> Result<Self, Box<dyn Error>> {
        unsafe {
            let entry = Entry::load()?;

            let app_name = c"Aelkyn";

            // Hardcode to false for now, as they were causing issues/slowdowns
            let enable_validation = true;

            let layer_names: Vec<&ffi::CStr> = if enable_validation {
                vec![c"VK_LAYER_KHRONOS_validation"]
            } else {
                vec![]
            };
            let layers_raw: Vec<*const c_char> = layer_names.iter().map(|n| n.as_ptr()).collect();

            // Ask the windowing helper which instance extensions are required
            // for this platform, then add debug utils for validation messages.
            let mut extension_names =
                ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())
                    .unwrap()
                    .to_vec();

            if enable_validation {
                extension_names.push(debug_utils::NAME.as_ptr());
            }

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
                extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
            }

            let app_info = vk::ApplicationInfo::default()
                .application_name(app_name)
                .application_version(vk::make_api_version(0, 1, 0, 0))
                .engine_name(c"Aelkyn Engine")
                .engine_version(vk::make_api_version(0, 1, 0, 0))
                .api_version(vk::API_VERSION_1_3);

            let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

            // Optionally set up the debug messenger so validation messages are printed.
            let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let mut instance_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_layer_names(&layers_raw)
                .enabled_extension_names(&extension_names)
                .flags(create_flags);

            if enable_validation {
                instance_info = instance_info.push_next(&mut debug_info);
            }

            let instance = entry
                .create_instance(&instance_info, None)
                .map_err(|e| format!("Failed to create Vulkan instance: {e}"))?;

            let (debug_utils_loader, debug_call_back) = if enable_validation {
                let loader = debug_utils::Instance::new(&entry, &instance);
                let cb = loader
                    .create_debug_utils_messenger(&debug_info, None)
                    .map_err(|e| format!("Failed to create debug messenger: {e}"))?;
                (Some(loader), cb)
            } else {
                (None, vk::DebugUtilsMessengerEXT::null())
            };

            let surface_loader = surface::Instance::new(&entry, &instance);

            // Create the OS/window surface so Vulkan can present images to it.
            let surface = ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )?;

            // Pick the best available GPU with fallback:
            // Discrete → Integrated → any GPU that works.
            let pdevices = instance.enumerate_physical_devices()?;
            if pdevices.is_empty() {
                return Err("No Vulkan-capable GPU found".into());
            }

            let score_device = |pd: vk::PhysicalDevice| -> u32 {
                let props = instance.get_physical_device_properties(pd);
                match props.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 100,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 50,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => 25,
                    vk::PhysicalDeviceType::CPU => 10,
                    _ => 1,
                }
            };

            let pdevice = pdevices
                .into_iter()
                .max_by_key(|&pd| score_device(pd))
                .ok_or("No suitable GPU found")?;

            let gpu_props = instance.get_physical_device_properties(pdevice);
            let gpu_name = ffi::CStr::from_ptr(gpu_props.device_name.as_ptr()).to_string_lossy();
            tracing::info!("Selected GPU: {gpu_name} ({:?})", gpu_props.device_type);

            // Find a queue family that supports both graphics and presenting.
            let queue_family_index = instance
                .get_physical_device_queue_family_properties(pdevice)
                .iter()
                .enumerate()
                .find_map(|(i, info)| {
                    let gfx = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                    let present = surface_loader
                        .get_physical_device_surface_support(pdevice, i as u32, surface)
                        .unwrap_or(false);
                    (gfx && present).then_some(i as u32)
                })
                .ok_or("No graphics+present queue family")?;

            // Create one logical queue from the chosen family.
            let priorities = [1.0f32];
            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            // Device extensions add optional Vulkan features.
            let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];

            // Enable Vulkan 1.2/1.3 feature structs needed by the renderer.
            let mut features_12 =
                vk::PhysicalDeviceVulkan12Features::default().buffer_device_address(true);

            let mut features_13 = vk::PhysicalDeviceVulkan13Features::default()
                .dynamic_rendering(true)
                .synchronization2(true);

            // Enable core device features.
            let features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);

            // Create the logical device from the selected physical device.
            let device_info = vk::DeviceCreateInfo::default()
                .push_next(&mut features_12)
                .push_next(&mut features_13)
                .enabled_features(&features)
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extensions);

            let device = instance
                .create_device(pdevice, &device_info, None)
                .map_err(|e| format!("Failed to create logical device: {e}"))?;
            let present_queue = device.get_device_queue(queue_family_index, 0);

            tracing::info!(
                "Vulkan device created successfully (queue family {queue_family_index})"
            );

            Ok(Self {
                entry,
                instance,
                device,
                surface_loader,
                debug_utils_loader,
                debug_call_back,
                pdevice,
                queue_family_index,
                present_queue,
                surface,
            })
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            // Wait until the GPU is idle before destroying Vulkan objects.
            let _ = self.device.device_wait_idle();

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            if let Some(ref loader) = self.debug_utils_loader {
                loader.destroy_debug_utils_messenger(self.debug_call_back, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
