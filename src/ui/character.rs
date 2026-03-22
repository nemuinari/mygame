use crate::scenario::types::CharacterPosition;
use crate::scenario::{CharacterHideQueue, CharacterShowQueue};
use bevy::prelude::*;

fn char_color_for_id(id: &str) -> Color {
    match id {
        "heroine_a" => Color::srgba(0.80, 0.40, 0.55, 0.85),
        "heroine_b" => Color::srgba(0.40, 0.65, 0.80, 0.85),
        "rival" => Color::srgba(0.75, 0.30, 0.30, 0.85),
        "mentor" => Color::srgba(0.55, 0.45, 0.75, 0.85),
        _ => Color::srgba(0.50, 0.50, 0.50, 0.85),
    }
}

#[derive(Component)]
pub struct CharacterSlot {
    pub position: CharacterPosition,
}

#[derive(Component)]
pub struct CharacterPlaceholder;

pub fn spawn_character_slots(commands: &mut Commands) {
    for (position, left_pct, label_color) in [
        (
            CharacterPosition::Left,
            15.0_f32,
            Color::srgba(0.7, 0.9, 0.7, 1.0),
        ),
        (
            CharacterPosition::Center,
            37.5_f32,
            Color::srgba(0.9, 0.9, 0.7, 1.0),
        ),
        (
            CharacterPosition::Right,
            60.0_f32,
            Color::srgba(0.7, 0.8, 0.9, 1.0),
        ),
    ] {
        let name = format!("Character_Slot_{:?}", position);
        commands
            .spawn((
                Name::new(name.clone()),
                CharacterSlot {
                    position: position.clone(),
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(25.0),
                    height: Val::Percent(55.0),
                    bottom: Val::Percent(25.0),
                    left: Val::Percent(left_pct),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Visibility::Hidden,
            ))
            .with_children(|slot| {
                slot.spawn((
                    Name::new(format!("{name}_Placeholder")),
                    CharacterPlaceholder,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.7)),
                ));
                slot.spawn((
                    Text::new(format!("[{:?}]", position)),
                    TextColor(label_color),
                ));
            });
    }
}

pub fn handle_character_show(
    mut queue: ResMut<CharacterShowQueue>,
    mut slots: Query<(&CharacterSlot, &mut Visibility, &Children)>,
    mut placeholder_q: Query<&mut BackgroundColor, With<CharacterPlaceholder>>,
) {
    let items: Vec<_> = queue.0.drain(..).collect();
    for ev in items {
        for (slot, mut vis, children) in slots.iter_mut() {
            if slot.position != ev.position {
                continue;
            }
            *vis = Visibility::Visible;
            let color = char_color_for_id(&ev.id);
            for child in children.iter() {
                if let Ok(mut bg) = placeholder_q.get_mut(child) {
                    *bg = BackgroundColor(color);
                }
            }
            info!("立ち絵表示: id={} pos={:?}", ev.id, ev.position);
        }
    }
}

pub fn handle_character_hide(
    mut queue: ResMut<CharacterHideQueue>,
    mut slots: Query<(&CharacterSlot, &mut Visibility)>,
) {
    let items: Vec<_> = queue.0.drain(..).collect();
    for ev in items {
        for (slot, mut vis) in slots.iter_mut() {
            if slot.position == ev.position {
                *vis = Visibility::Hidden;
                info!("立ち絵非表示: pos={:?}", ev.position);
            }
        }
    }
}
