use bevy::prelude::*;

#[derive(Bundle)]
pub struct WarningSignBundle {
    #[bundle()]
    ui_bundle: TextBundle,
}

impl WarningSignBundle {
    pub fn new(position: Vec2) -> Self {
        Self {
            ui_bundle: TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "!".to_string(),
                        style: TextStyle {
                            font: Default::default(),
                            font_size: 50.0,
                            color: Color::srgb(1.0, 0.0, 0.0),
                        },
                    }],
                    ..Default::default()
                },
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(position.x),
                    top: Val::Px(position.y),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
