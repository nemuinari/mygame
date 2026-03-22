use crate::scenario::BackgroundChangeQueue;
use bevy::prelude::*;

fn bg_color_for_key(key: &str) -> Color {
    match key {
        "classroom" => Color::srgba(0.10, 0.16, 0.28, 1.0),
        "school_roof" => Color::srgba(0.25, 0.35, 0.18, 1.0),
        "corridor" => Color::srgba(0.22, 0.18, 0.12, 1.0),
        "night_sky" => Color::srgba(0.02, 0.02, 0.10, 1.0),
        _ => Color::srgba(0.08, 0.08, 0.15, 1.0),
    }
}

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
        BackgroundColor(Color::srgba(0.02, 0.02, 0.10, 1.0)),
        ZIndex(-1),
    ));
}

pub fn handle_background_change(
    mut queue: ResMut<BackgroundChangeQueue>,
    mut bg_q: Single<&mut BackgroundColor, With<BackgroundLayer>>,
) {
    for ev in queue.0.drain(..) {
        let color = bg_color_for_key(&ev.key);
        **bg_q = BackgroundColor(color);
        info!("背景を変更: {}", ev.key);
    }
}
