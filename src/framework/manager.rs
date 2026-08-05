use std::collections::BTreeMap;

use crate::{
    framework::component::{Component, Id},
    kernel::event::{Event, EventEnvelope, EventKind},
};

#[derive(Debug)]
pub enum ComponentManagerError {
    AlreadyRegistered(Id),
    NotFound(Id),
}

pub struct ComponentManager {
    components: BTreeMap<Id, Box<dyn Component>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
        }
    }

    pub fn register<C>(
        &mut self,
        component: C,
    ) -> Result<(), ComponentManagerError>
    where
        C: Component + 'static,
    {
        let id = component.id();

        if self.components.contains_key(&id) {
            return Err(
                ComponentManagerError::AlreadyRegistered(id),
            );
        }

        self.components.insert(id, Box::new(component));

        Ok(())
    }

    pub fn update(
        &mut self,
        component_id: Id,
    ) -> Result<Vec<Event>, ComponentManagerError> {
        let component = self
            .components
            .get_mut(&component_id)
            .ok_or(ComponentManagerError::NotFound(component_id))?;

        Ok(component.update())
    }

    pub fn handle_event(
        &mut self,
        component_id: Id,
        envelope: &EventEnvelope,
    ) -> Result<Vec<Event>, ComponentManagerError> {
        let component = self
            .components
            .get_mut(&component_id)
            .ok_or(ComponentManagerError::NotFound(component_id))?;

        Ok(component.on_event(envelope))
    }

    pub fn subscribers_for(
        &self,
        event_kind: EventKind,
    ) -> Vec<Id> {
        self.components
            .iter()
            .filter_map(|(id, component)| {
                component
                    .subscriptions()
                    .contains(&event_kind)
                    .then_some(*id)
            })
            .collect()
    }

    pub fn ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.components.keys().copied()
    }
}