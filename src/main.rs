use std::{io::Read, net::TcpListener, sync::Arc};
use log::debug;
use winit::{event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::{self, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};

mod state;
mod hdr_texture;
mod cube_texture;
mod cube_map_renderer;
mod camera;
mod ibl_renderer;
mod texture_2d;
mod cube_mipmap_renderer;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
    .with_inner_size(winit::dpi::LogicalSize { width: 1600, height: 900})
    .with_position(winit::dpi::LogicalPosition {x: 150, y: 50})
    .build(&event_loop).unwrap();

    let window = Arc::new(window);
    let mut state = state::State::new(window.clone()).await;

    state.render_hdr_to_cube();
    state.save_ibl_diffuse().await;
    state.save_ibl_specular_1().await;
    state.save_ibl_specular_2().await;

    // event_loop.run(move |event, elwt| match event {
    //     Event::WindowEvent {
    //         ref event,
    //         window_id,
    //     } if window_id == window.id() => if !state.input(event) { 
    //         match event {
    //             WindowEvent::CloseRequested
    //             | WindowEvent::KeyboardInput {
    //                 event:
    //                     KeyEvent {
    //                         state: ElementState::Pressed,
    //                         physical_key: PhysicalKey::Code(KeyCode::Escape),
    //                         ..
    //                     },
    //                 ..
    //             } => elwt.exit(),
    //             //WindowEvent::Resized(new_size) => {state.resize(*new_size);},
    //             WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => { 
    //                 debug!("ScaleFactorChanged: {:?}, {:?}", scale_factor, inner_size_writer);
    //                 //TODO

    //             },
    //             WindowEvent::RedrawRequested => {
    //                 state.update();
    //                 match state.render() {
    //                     Ok(_) => {}
    //                     //Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
    //                     Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
    //                     Err(e) => eprintln!("Error: {:?}", e),
    //                 }
    //             },
    //             _ => {}
    //         }
    //     },
    //     Event::AboutToWait => {
    //         window.request_redraw();
    //     }
    //     _ => {}
    // })?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    pollster::block_on(run())?;
    Ok(())
}