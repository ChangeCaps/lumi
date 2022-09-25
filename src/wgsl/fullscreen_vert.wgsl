#include <lumi/fullscreen.wgsl>

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> Fullscreen {
	return fullscreen_triangle(index);
}
