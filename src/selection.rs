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
    pub network_entities: HashSet<NetworkEntity>,
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
    player_id: Res<Option<PlayerId>>,
    query: Query<(Entity, &Position, &Handle<Unit>, &Owner, &NetworkEntity)>,
) {
    if player_id.is_none() {
        return;
    }

    let player_id = player_id.unwrap();

    let input_config = match input_config.get(&input_resource.0) {
        Some(i) => i,
        None => return,
    };

    if input_config
        .select
        .just_pressed(&keyboard_input, &mouse_input)
    {
        selection.box_select = mouse_position.position();
    }

    if input_config
        .select
        .just_released(&keyboard_input, &mouse_input)
    {
        let box_select = selection.box_select.distance(mouse_position.position()) > 5.0;

        let a = (*ISO_TO_SCREEN).transform_vector2(mouse_position.position());
        let b = (*ISO_TO_SCREEN).transform_vector2(selection.box_select);

        let min = a.min(b);
        let max = a.max(b);

        if !input_config
            .add_to_selection
            .pressed(&keyboard_input, &mouse_input)
        {
            selected_units.units = HashSet::new();
            selected_units.network_entities = HashSet::new();
        }

        for (entity, position, unit_handle, owner, network_entity) in query.iter() {
            if owner.0 != player_id {
                continue;
            }

            if box_select {
                let position = *ISO_TO_SCREEN * position.position;

                if position.truncate().cmpge(min).all() && position.truncate().cmple(max).all() {
                    selected_units.units.insert(entity);
                    selected_units.network_entities.insert(*network_entity);
                }
            } else {
                let unit = units.get(unit_handle).unwrap();

                let diff = position.position.truncate() - mouse_position.position();
                let dist = diff.length();

                if dist <= unit.selection_size {
                    selected_units.units.insert(entity);
                    selected_units.network_entities.insert(*network_entity);
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
