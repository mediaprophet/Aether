use crate::CreationError;

pub fn create_tracy_gpu_client(
    backend: etalon::Backend,
    device: &etalon::Device,
    queue: &etalon::Queue,
) -> Result<tracy_client::GpuContext, CreationError> {
    let query_set = device.create_query_set(&etalon::QuerySetDescriptor {
        label: Some("etalon-profiler gpu -> cpu sync query_set"),
        ty: etalon::QueryType::Timestamp,
        count: 1,
    });

    let resolve_buffer = device.create_buffer(&etalon::BufferDescriptor {
        label: Some("etalon-profiler gpu -> cpu resolve buffer"),
        size: etalon::QUERY_SIZE as _,
        usage: etalon::BufferUsages::QUERY_RESOLVE | etalon::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let map_buffer = device.create_buffer(&etalon::BufferDescriptor {
        label: Some("etalon-profiler gpu -> cpu map buffer"),
        size: etalon::QUERY_SIZE as _,
        usage: etalon::BufferUsages::MAP_READ | etalon::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut timestamp_encoder = device.create_command_encoder(&etalon::CommandEncoderDescriptor {
        label: Some("etalon-profiler gpu -> cpu query timestamp"),
    });
    timestamp_encoder.write_timestamp(&query_set, 0);
    timestamp_encoder.resolve_query_set(&query_set, 0..1, &resolve_buffer, 0);
    // Workaround for https://github.com/gfx-rs/wgpu/issues/6406
    // TODO when that bug is fixed, merge these encoders together again
    let mut copy_encoder = device.create_command_encoder(&etalon::CommandEncoderDescriptor {
        label: Some("etalon-profiler gpu -> cpu copy timestamp"),
    });
    copy_encoder.copy_buffer_to_buffer(
        &resolve_buffer,
        0,
        &map_buffer,
        0,
        etalon::QUERY_SIZE as u64,
    );
    queue.submit([timestamp_encoder.finish(), copy_encoder.finish()]);

    map_buffer
        .slice(..)
        .map_async(etalon::MapMode::Read, |_| ());
    device.poll(etalon::PollType::wait_indefinitely()).unwrap();

    let view = map_buffer.slice(..).get_mapped_range()?;
    let timestamp: i64 = i64::from_le_bytes((*view).try_into().unwrap());

    let tracy_backend = match backend {
        etalon::Backend::Noop | etalon::Backend::Metal | etalon::Backend::BrowserWebGpu => {
            tracy_client::GpuContextType::Invalid
        }
        etalon::Backend::Vulkan => tracy_client::GpuContextType::Vulkan,
        etalon::Backend::Dx12 => tracy_client::GpuContextType::Direct3D12,
        etalon::Backend::Gl => tracy_client::GpuContextType::OpenGL,
    };

    tracy_client::Client::running()
        .ok_or(CreationError::TracyClientNotRunning)?
        .new_gpu_context(
            Some("etalon"),
            tracy_backend,
            timestamp,
            queue.get_timestamp_period(),
        )
        .map_err(CreationError::from)
}
