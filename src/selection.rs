use crate::*;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Default)]
pub struct Selection {
    pub box_select: Vec2,
}

#[derive(Default)]
pub struct SelectedUnits {
    pub units: HashSet<Entity>,
}

pub fn unit_selection_system(
    mut selected_units: ResMut<SelectedUnits>,
    mut selection: Local<Selection>,
    input_config: Res<Assets<InputConfig>>,
    input_resource: Res<InputResource>,
    mouse_position: Res<MousePosition>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    units: Res<Assets<Unit>>,
    query: Query<(Entity, &Position, &Handle<Unit>)>,
) {
    let input_config = match input_config.get(&input_resource.0) {
        Some(i) => i,
        None => return,
    };

    if mouse_input.just_pressed(MouseButton::Left) {
        selection.box_select = mouse_position.position();
    }

    if mouse_input.just_released(MouseButton::Left) {
        let box_select = selection.box_select.distance(mouse_position.position()) > 5.0;

        let min = mouse_position.position().min(selection.box_select);
        let max = mouse_position.position().max(selection.box_select);

        if !keyboard_input.pressed(input_config.add_to_selection) {
            selected_units.units = HashSet::new();
        }

        for (entity, position, unit_handle) in query.iter() {
            if box_select {
                if position.position.truncate().cmpge(min).all()
                    && position.position.truncate().cmple(max).all()
                {
                    selected_units.units.insert(entity);
                }
            } else {
                let unit = units.get(unit_handle).unwrap();

                let diff = position.position.truncate() - mouse_position.position();
                let dist = diff.length();

                if dist <= unit.selection_size {
                    selected_units.units.insert(entity);
                    return;
                }
            }
        }

        info!("Selected: {:?}", selected_units.units);
    }
}

pub fn unit_selection_ring_system(
    selected_units: Res<SelectedUnits>,
    selection_circle_query: Query<(Entity, &Handle<Unit>, &Children)>,
    mut visible_query: Query<&mut Visible>,
) {
    for (entity, _unit, children) in selection_circle_query.iter() {
        if let Ok(mut visible) = visible_query.get_mut(children[0]) {
            visible.is_visible = selected_units.units.contains(&entity);
        }
    }
}

pub struct SelectionPlugin;
