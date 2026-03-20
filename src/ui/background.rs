use bevy::prelude::*;

#[derive(Component)]
pub struct BackgroundLayer;

pub fn spawn_background(commands: &mut Commands) {
    commands.spawn((
        Name::new("Background_Layer"),
        BackgroundLayer,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.02, 0.1, 1.0)),
        ZIndex(-1),
    ));
}
