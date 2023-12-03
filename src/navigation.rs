use crate::network::identity::Identity;
use super::location::{Location, DistanceEdge};
use super::time_series::LocationTimeSeries;
use std::collections::HashSet;


pub struct Navigator {
    pub id: String,
    pub archive: Vec<DistanceEdge>,
    pub cache: LocationTimeSeries,
    pub nodes: HashSet<Identity>
}

impl Navigator {
    pub fn add(&mut self, edge: DistanceEdge) {

    }

    pub fn extend(&mut self, edges: Vec<DistanceEdge>) {

    }

    pub fn get_node_location(&self, node: Identity) -> Location {
        let mut distances: Vec<DistanceEdge> = vec![];

        // for d in self.get_node_snapshot(node) {

        // }

        Location::from_distances(node.clone(), distances)
    }
}