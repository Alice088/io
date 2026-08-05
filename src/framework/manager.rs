use std::collections::BTreeMap;

use thiserror::Error;

use crate::{
    framework::{
        component::{
            Component,
            ComponentId
        },
        error::FlightError,
    },
    kernel::event::{
        Event,
        EventEnvelope,
        EventKind,
    },
};

#[derive(Debug, Error)]
pub enum ComponentManagerError {
    #[error("component {0} is already registered")]
    AlreadyRegistered(ComponentId),

    #[error("component {0} was not found")]
    NotFound(ComponentId),

    #[error("component {component_id} failed by {source}")]
    ComponentFailed {
        component_id: ComponentId,
        source: FlightError,
    },
}

pub struct ComponentManager {
    components: BTreeMap<ComponentId, Box<dyn Component>>,
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

        self.components.insert(
            id,
            Box::new(component),
        );

        Ok(())
    }

    pub async fn update(
        &mut self,
        component_id: ComponentId,
    ) -> Result<Vec<Event>, ComponentManagerError> {
        let component = self
            .components
            .get_mut(&component_id)
            .ok_or(
                ComponentManagerError::NotFound(
                    component_id,
                ),
            )?;

        component
            .update()
            .await
            .map_err(|source| {
                ComponentManagerError::ComponentFailed {
                    component_id,
                    source,
                }
            })
    }

    pub async fn handle_event(
        &mut self,
        component_id: ComponentId,
        envelope: &EventEnvelope,
    ) -> Result<Vec<Event>, ComponentManagerError> {
        let component = self
            .components
            .get_mut(&component_id)
            .ok_or(
                ComponentManagerError::NotFound(
                    component_id,
                ),
            )?;

        component
            .on_event(envelope)
            .await
            .map_err(|source| {
                ComponentManagerError::ComponentFailed {
                    component_id,
                    source,
                }
            })
    }

    pub fn subscribers_for(
        &self,
        event_kind: EventKind,
    ) -> Vec<ComponentId> {
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

    pub fn ids(
        &self,
    ) -> impl Iterator<Item = ComponentId> + '_ {
        self.components.keys().copied()
    }

    pub fn contains(
        &self,
        component_id: ComponentId,
    ) -> bool {
        self.components
            .contains_key(&component_id)
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}