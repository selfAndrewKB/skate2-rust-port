//! Co-op fork (M21): a puppet's graph held at a state it cannot enter. The field test's joiner
//! stopped when a puppet's held update failed partway (the owner's `WaitForTakedownDecision`,
//! a takedown's behaviour asserting with no target here) and its own update later met the
//! half-applied state; `end_all_and_reset` puts the graph back to its start.
use super::{AiDelegate, PedestrianGraph, PedestrianGraphBackend, PedestrianGraphBehaviours};
use crate::{
    generated_action_load::runtime::ActionUpdateError as Error, generated_graph_selection::Mask,
};

/// `NoRoad`, a walking state, and `WaitForTakedownDecision`, the field test's.
const WALKING: u16 = 23;
const TAKEDOWN: u16 = 112;

/// Every behaviour enabled; the `begin` of `refuse` (the first behaviour begun while it is set)
/// fails as the takedown's did.
#[derive(Default)]
struct Refusing {
    refuse_next_begin: bool,
    begun: Vec<u16>,
    ended: Vec<u16>,
}

impl PedestrianGraphBackend for Refusing {
    fn condition(&mut self, _: &PedestrianGraph, _: Mask, _: u16, _: u16) -> Result<bool, Error> {
        Ok(false)
    }
    fn enabled(&self, _: u16) -> Result<bool, Error> {
        Ok(true)
    }
    fn create_instance(&mut self, _: &PedestrianGraph, _: u16) -> Result<(), Error> {
        Ok(())
    }
    fn begin(&mut self, _: &PedestrianGraph, occurrence: u16) -> Result<(), Error> {
        if std::mem::take(&mut self.refuse_next_begin) {
            return Err(Error::Capacity);
        }
        self.begun.push(occurrence);
        Ok(())
    }
    fn end_and_remove(&mut self, _: &PedestrianGraph, occurrence: u16) -> Result<(), Error> {
        self.ended.push(occurrence);
        Ok(())
    }
    fn transition_hook(&mut self, _: &PedestrianGraph, _: u16, _: u16) -> Result<(), Error> {
        Ok(())
    }
    fn generated_update(&mut self, _: &PedestrianGraph, _: u32, _: &[u16]) -> Result<(), Error> {
        Ok(())
    }
}

fn held(graph: &mut PedestrianGraph, instances: &mut PedestrianGraphBehaviours, backend: &mut Refusing, state: u16) -> Result<bool, Error> {
    graph.update_delegate_held(AiDelegate::Pedestrian, 1. / 60., instances, backend, state)
}

#[test]
fn a_held_update_that_fails_partway_leaves_the_graph_unusable_until_reset() {
    let (mut graph, mut instances, mut backend) = (PedestrianGraph::default(), PedestrianGraphBehaviours::default(), Refusing::default());
    assert!(held(&mut graph, &mut instances, &mut backend, WALKING).unwrap());
    // The takedown's behaviour refuses to begin: the update stops partway, the refused
    // behaviour published (native publishes before Begin) and its states entered.
    backend.refuse_next_begin = true;
    assert!(held(&mut graph, &mut instances, &mut backend, TAKEDOWN).is_err());
    // The field test's stop: the puppet's own update (a knockdown's, from its collision flags)
    // or a held update to any other state meets the half-applied set.
    let mut copy = (PedestrianGraph::default(), PedestrianGraphBehaviours::default(), Refusing::default());
    assert!(held(&mut copy.0, &mut copy.1, &mut copy.2, WALKING).unwrap());
    copy.2.refuse_next_begin = true;
    assert!(held(&mut copy.0, &mut copy.1, &mut copy.2, TAKEDOWN).is_err());
    assert!(matches!(held(&mut copy.0, &mut copy.1, &mut copy.2, WALKING), Err(Error::InvalidActiveSet)));
    assert!(matches!(graph.update_delegate(AiDelegate::Pedestrian, 1. / 60., &mut instances, &mut backend), Err(Error::InvalidActiveSet)));
}

#[test]
fn a_graph_reset_after_a_failed_held_update_is_held_again() {
    let (mut graph, mut instances, mut backend) = (PedestrianGraph::default(), PedestrianGraphBehaviours::default(), Refusing::default());
    assert!(held(&mut graph, &mut instances, &mut backend, WALKING).unwrap());
    let walking = backend.begun.clone();
    backend.refuse_next_begin = true;
    assert!(held(&mut graph, &mut instances, &mut backend, TAKEDOWN).is_err());
    graph.end_all_and_reset(&mut instances, &mut backend).unwrap();
    assert!(instances.is_empty(), "every behaviour ended");
    assert!(backend.ended.iter().all(|occurrence| backend.ended.iter().filter(|o| *o == occurrence).count() == 1), "each ended once");
    // Held again at the walk, from the start: its behaviours begin again.
    backend.begun.clear();
    assert!(held(&mut graph, &mut instances, &mut backend, WALKING).unwrap());
    assert_eq!(backend.begun, walking);
    // And the puppet's own update runs.
    graph.update_delegate(AiDelegate::Pedestrian, 1. / 60., &mut instances, &mut backend).unwrap();
}

