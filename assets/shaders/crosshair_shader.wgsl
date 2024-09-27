// This shader draws a solid plus-shaped crosshair with a given input color
#import bevy_ui::ui_vertex_output::UiVertexOutput

struct CrosshairUiMaterial {
    @location(0) color: vec4<f32>
}

@group(1) @binding(0)
var<uniform> input: CrosshairUiMaterial;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Adjust UVs to be centered on the rect, with a range from -1.0 to 1.0
    let uv = in.uv * 2.0 - 1.0;

    // Crosshair parameters
    let line_thickness = 0.02; // Thickness of the crosshair lines
    let line_length = 5.0;     // Length of the crosshair lines in units (normalized)

    // Horizontal line condition (within line thickness and constrained to 5 units in length)
    let horizontal_line = abs(uv.y) < line_thickness && abs(uv.x) < line_length / 10.0;

    // Vertical line condition (within line thickness and constrained to 5 units in length)
    let vertical_line = abs(uv.x) < line_thickness && abs(uv.y) < line_length / 10.0;

    // If either the horizontal or vertical line condition is true, render the crosshair.
    let crosshair_alpha = select(0.0, 0.9, horizontal_line || vertical_line);

    // Return the crosshair color with transparency
    return vec4<f32>(input.color.rgb, crosshair_alpha);
}

