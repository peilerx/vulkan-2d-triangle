use ash::Device;
use ash::ext::debug_utils;
use ash::khr::surface;
use ash::prelude::VkResult;
use ash::vk::Result;
use ash::{Entry, Instance, vk};
use ash_window;
use std::ffi::c_char;
use vk::Queue;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use std::io::Cursor;

struct FramesBase {
    pub loader: ash::khr::swapchain::Device,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR, /*swapchain это структура которая используется технология организации/буфферизации отображения кадров и способ общения с оконным менеджером вашей системы */
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
} //vulkan swapchain resources 

impl FramesBase {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::khr::surface::Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
        window: &Window,
    ) -> VkResult<Self> {
        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };

        print!(
            "Surface capabilities: min_images = {}, max_images = {}, extent = {:?} ",
            surface_capabilities.min_image_count, //минимальное количество кадров в очереди для данного GPU
            surface_capabilities.max_image_count, //максимальное количество указано 0 то есть без ограничений на данный GPU
            surface_capabilities.current_extent //размер поверхности рендера, привязанного к размеру окна
        );

        let surface_formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()
        }; //форматы отображения изо в различный RGB форматах,
        //с разной цветокоррекцией, гаммой, прозрачностью, размером канала на один цвет или альфа канал
        let surface_format = surface_formats[0];
        let format = surface_format.format;
        println!("Available surface formats: {:?}", surface_formats);

        let present_modes = unsafe {
            //получаем список режимов представления изображения, IMMEDIATE, MAILBOX, FIFO, FIFO_RELAXED
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
        };
        /*
        1.IMMEDIATE отображает изображение сразу без ожидания синхронизации с частотой обновления экрана

        2. MAILBOX тройная буфферизация, одно изо отображается, второе рендериться,
        третье рендерится, если второй и третий отрендерились у возмет самый свежий рендер и отобразит его, то есть пропустить второй и покажет третий

        3. FIFO двойная или тройная буфферизация с ожиданием обноевления частосты экрана v-sync (вертикальная синхронизация),
        создается очередь на отображение привязанная к частоте обновления экрана, второй и третий кадр даже если они прошли этап рендера в любом случае
        отобразяться в строгой последовательности второй потом третий
        */

        println!("Present modes: {:?}", present_modes);

        println!("Current extent: {:?}", surface_capabilities.current_extent);
        println!(
            "min image extent width: {}",
            surface_capabilities.min_image_extent.width
        );

        let extent = if surface_capabilities.current_extent.width != u32::MAX {
            //проверка на неопределенное состояние размеров окна,
            //по спецификации Vulkan неопределенное состояние поверхности или окна всегда будет u32::MAX
            //если состояние определенное верни реальные размеры окна в переменную extent
            surface_capabilities.current_extent
        } else {
            //в противном случае если размер u32::MAX состояние не определенно создай новую поверхность
            let size = window.inner_size(); //достает данные для размеров поверхности surface из window
            vk::Extent2D {
                //создаем и возврашаем новый экземляр поверхности
                width: size.width.clamp(
                    //если меньше диапазона верни минимум или если больше верни максимум, если в диапазоне верни значение по факту
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: size.height.clamp(
                    //тоже самое для высоты
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            }
        };

        let image_count = if surface_capabilities.max_image_count > 0 {
            //защита от дурака, если указал максимум меньше минимума,
            // на разные GPU дефолтные значения могут меняться,
            //max_image_count = 0 это значит что ограничений по максимум нет, если бы указали 1 или больше, то все должно совпадать, и берем всегда минимум
            surface_capabilities
                .min_image_count
                .min(surface_capabilities.max_image_count)
        } else {
            surface_capabilities.min_image_count + 1 //даем запас по буфферу + 1, для MAILBOX например
        };

        println!("Swapchain image count: {}", image_count);

        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);
        let queue_family_indices = &[queue_family_index];

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_formats[0].format)
            .image_color_space(surface_formats[0].color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT) //изображения используются для цветного отображения
            .pre_transform(surface_capabilities.current_transform) //преобразование изображений, по дефолту без поворотов
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE) //эксклюзивный доступ к семейству очередей, экслюзивный значит для одного семейства
            .queue_family_indices(queue_family_indices) //передаем массив индексов семейства очередей, массив для задела в случае если мы будет передавать больше семейств COMPUTE, GRAPHICS ETC
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) //непрозрачный режим для окна
            .present_mode(vk::PresentModeKHR::MAILBOX)
            .clipped(true); //обрезка невидимых пикселей 

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .unwrap()
        };

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() }; //получаем сами кадры, вектор из 4 кадров

        println!(
            "Count of swapchain images: {:?}",
            images.iter().count() //такое же количество как и в image_count четыре кадра.
        );

        let image_views: Vec<vk::ImageView> = images //ImageView это инструкция как работать с памятью кадра
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_formats[0].format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);
                let swizzle = vk::ComponentSwizzle::IDENTITY;
                println!(
                    "Rgb component swizzle IDENTITY test = {:?}",
                    swizzle.as_raw() as i32
                );
                unsafe { device.create_image_view(&create_info, None).unwrap() }
            })
            .collect();

        Ok(Self {
            loader: swapchain_loader,
            surface_format,
            extent,
            swapchain,
            images,
            image_views,
            format,
        })
    }
}

/*RenderBase нужен для определения порядка отображения теней, сглаживания, геометрии, освещения и так далее,
в нем можно определить порядок рендера применяемый к одному или нескольким кадрам ImageView, с помощью механизма subpass`ов
renderpassы это про организацию рендера, а не про сам рендер*/
struct RenderBase {
    pub render_pass: vk::RenderPass,
    pub frame_buffers: Vec<vk::Framebuffer>,
}

impl RenderBase {
    pub fn new(
        device: &Device,
        format: vk::Format,
        image_views: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> VkResult<Self> {
        let render_pass = {
            let color_attachment = vk::AttachmentDescription::default() //описание свойства буффера которые будут применять к ImageView
                .format(format)
                .samples(vk::SampleCountFlags::TYPE_1) /*флаг сглаживания TYPE_1 (x1) без сглаживания, TYPE_2 это MSAA x2 и так далее */
                .load_op(vk::AttachmentLoadOp::CLEAR) /*определяет что будет с буффером кадра перед рендером, флаг очистить определенным цветом*/
                .store_op(vk::AttachmentStoreOp::STORE) /*определяет состояние буффера после рендера STORE флаг сохранение результатов рендера после рендера*/
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE) /*указание для трафаретного буффера перед рендером, игнорировать, нужен для 3D*/
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE) /*указания для трафаретного буффера после рендера, в данном случае тоже игнорировать*/
                .initial_layout(vk::ImageLayout::UNDEFINED) /*начальное состояние буффера перед рендером, UNDEFINED начальное состояние не важно*/ 
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR); /*конечное состояние буффера после рендера, PRESENT_SRC_KHR - отобразить*/

            let color_attachment_ref = vk::AttachmentReference::default() //указывает как субпасс будет использоваться в renderpass
                .attachment(0) /*возьми первое вложение из массива с индексом 0*/
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL) /*буффер оптимизирован для работы со цветом,
            в целом весь этот референс говорит нам что надо взять нулевое вложение и использовать его для работы с цветом*/;

            let color_attachment_refs = &[color_attachment_ref];

            let subpass = vk::SubpassDescription::default() //этап рендера,
                // можно добавлять их больше, каждый субпасс может отвечать за разное,
                //  будь то цвет или глубина, трафарет и так далее
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS) /*subpass использует графический pipeline
                если выбрано другое семейство очереди рендер не будет работать, например если семейство очереди помечено как COMPUTE для вычислений,
                то графический флаг пайплайна не будет работать */
                .color_attachments(color_attachment_refs);

            let color_attachments = &[color_attachment];

            let subpasses = &[subpass];
            let render_pass_info = vk::RenderPassCreateInfo::default()
                .attachments(color_attachments)
                .subpasses(subpasses);

            unsafe { device.create_render_pass(&render_pass_info, None) }
        }
        .unwrap();

        let frame_buffers = unsafe { //framebuffers это механизм привязки того как и
            // в каком порядке будет рендерится кадр ImageView с помощью RenderPass
            image_views.iter().map(|&image_view| {
                let attachments = [image_view];
                let framebuffer_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                device.create_framebuffer(&framebuffer_info, None).unwrap()
            })
        }
        .collect();

        Ok(Self {
            render_pass,
            frame_buffers,
        })
    }
}

struct AppearanceBase {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    shader_modules: Vec<vk::ShaderModule>,
} //vulkan pipeline resources

impl AppearanceBase {
    pub fn new(self: Self, device: &Device, render_pass: vk::RenderPass, extent: vk::Extent2D) -> VkResult<Self> {

        let vert_shader_code = include_bytes!("../shader/triangle.vert.spv").to_vec();
        let frag_shader_code = include_bytes!("../shader/triangle.frag.spv").to_vec();

        if vert_shader_code.len() % 4 != 0 || frag_shader_code.len() % 4 != 0 {
            return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
        }

        let vert_shader_words: &[u32] = unsafe {
            std::slice::from_raw_parts(vert_shader_code.as_ptr() as *const u32, vert_shader_code.len() / 4)
        };

        let frag_shader_words: &[u32] = unsafe {
            std::slice::from_raw_parts(frag_shader_code.as_ptr() as *const u32, frag_shader_code.len() / 4)
        };
        
        
        let vert_shader_module = {
            let create_info = vk::ShaderModuleCreateInfo::default()
            .code(&vert_shader_words);
            unsafe {device.create_shader_module(&create_info, None)}
        };
        
        let frag_shader_module = {
            let create_info = vk::ShaderModuleCreateInfo::default()
            .code(&frag_shader_words);    
            unsafe {device.create_shader_module(&create_info, None)}
        };

        Ok(self)
    }
}

pub struct AppBase {
    pub entry: Entry,
    pub instance: Instance,
    pub event_loop: EventLoop<()>,
    pub window: Window,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: surface::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub device: Device,
    pub present_queue: Queue,
} //basic init vulkan resources

impl AppBase {
    pub fn new(width: u32, height: u32) -> VkResult<Self> {
        let entry = Entry::linked(); //базовый ресурс Vulkan
        let app_name = c"vulkan_2d_triangle";

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let layer_name = [c"VK_LAYER_KHRONOS_validation"]; //слой валидации для проверок ошибок Vulkan на этапе компиляции, можно добавить еще дополнительные слои например для подсчета FPS
        let layers_names: Vec<*const c_char> = layer_name //интепретация запись слоя в массив c_char, так как ash vk работает только с C
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let event_loop = EventLoop::new().unwrap(); //создаем цикл событий

        let window = WindowBuilder::new() //создаем окно
            .with_title("vulkan_2d_triangle")
            .with_inner_size(winit::dpi::LogicalSize::new(
                //логический размер окна с учетом DPI, автоматическое масштабирование
                f64::from(width),
                f64::from(height),
            ))
            .build(&event_loop) //помещаем цикл событий в окно
            .unwrap();

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw()) //возвращаем дескриптор оконного менеджера в список расширений
                .unwrap()
                .to_vec();
        extension_names.push(debug_utils::NAME.as_ptr());

        let create_info = vk::InstanceCreateInfo::default() //передаем данные о расширениях и слоях в Instance
            .application_info(&app_info)
            .flags(vk::InstanceCreateFlags::empty())
            .enabled_layer_names(&layers_names)
            .enabled_extension_names(&extension_names);

        let instance: Instance =
            unsafe { entry.create_instance(&create_info, None) }.expect("Instance Create Error"); //Instance создается с помощью entry 

        let surface = unsafe {
            //создаем поверхность рендера
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(), //передаем дескриптор оконного менеджера wayland, x11 etc
                window.window_handle().unwrap().as_raw(), //передаем дескриптор нашего конкретного окна, экземпляр window
                None,
            )
        }
        .unwrap();

        let surface_loader = surface::Instance::new(&entry, &instance); //загрузчик поверхности рендера

        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() }; //возвращается массив физических устройств 

        let (physical_device, queue_family_index) = physical_devices //получаем активное физическое устройство
            .into_iter() //перечисляем без сохранения всех физический устройств
            .find_map(|pdevice| unsafe {
                //получаем физическое устройство из списка
                {
                    instance
                        .get_physical_device_queue_family_properties(pdevice) //получаем свойства очереди активного устройста
                        .iter() //перечисляем их
                        .enumerate() //достаем индексы свойств
                        .find_map(|(index, info)| {
                            let supports_graphics =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS); //смотрим поддерживает ли свойство активного устройства данный флаг
                            let supports_surface = surface_loader
                                .get_physical_device_surface_support(pdevice, index as u32, surface) //смотрим поддерживает ли устройство данную поверхность рендера
                                .unwrap_or(false); //если нет верни false
                            if supports_graphics && supports_surface {
                                Some((pdevice, index as u32)) //если поддерживает верни устройство и индекс свойств семейства очереди 
                            } else {
                                None
                            }
                        })
                }
            })
            .expect("No physical device found");

        let device_extension_names_raw: Vec<*const c_char> =
            vec![ash::khr::swapchain::NAME.as_ptr()]; //расширения устройства

        let priorities = [1.0_f32]; //приоритет очереди, первый

        let queue_info = vk::DeviceQueueCreateInfo::default() //информация для создания очереди устройства
            .queue_family_index(queue_family_index) //индекс очереди
            .queue_priorities(&priorities); //приоритет первый
        let queue_infos = [queue_info]; //вектор очередей 
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extension_names_raw);

        let device = unsafe {
            //создаем логическое устройство с помощью экземпляра instance
            instance
                .create_device(physical_device, &device_create_info, None) //передаем физическое устройство
                .unwrap()
        };

        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) }; //возвращаем очередь логического устройства,
        // очереди принимают SubmitInfo а SubmitInfo принимает массив CommandBuffer, массив комманд на выполнение на  GPU
        // первый параметр это индекс семейства очередей GRAPHICS или COMPUTE, PRESENT, TRANSFER  etc, второй параметр это индекс очереди, очередей в одном семействе может быть заданое количество

        Ok(Self {
            entry,
            instance,
            event_loop,
            window,
            surface,
            surface_loader,
            physical_device,
            queue_family_index,
            device,
            present_queue,
        })
    }
}

fn main() {
    let app_base = AppBase::new(800, 600).unwrap();
    let device_properties = unsafe {
        app_base
            .instance
            .get_physical_device_properties(app_base.physical_device)
    };

    let device_name = unsafe { std::ffi::CStr::from_ptr(device_properties.device_name.as_ptr()) };

    // not the same
    // let device_name = unsafe { &std::ffi::CStr::from_ptr(device_properties.device_name[0] as *const i8) };

    println!("Device name 1: {:?} ", device_name);

    let frames_base = FramesBase::new(
        &app_base.instance,
        &app_base.device,
        app_base.surface,
        &app_base.surface_loader,
        app_base.physical_device,
        app_base.queue_family_index,
        &app_base.window,
    )
    .unwrap();

    let render_base = RenderBase::new(
        &app_base.device,
        frames_base.format,
        &frames_base.image_views,
        frames_base.extent,
    )
    .unwrap();
}
