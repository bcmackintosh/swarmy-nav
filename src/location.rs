use datetime::LocalDateTime;
use crate::{identity::Identity, filter::f64_ordering, polar::*};
use crate::test_suite::*;
use std::{sync::Arc, collections::HashMap};
use std::time::SystemTime;

pub type DistanceVector = Arc<[MomentEdge]>;

#[derive(Debug)]
pub struct MomentEdge {
    pub left: Identity,
    pub right: Identity,
    pub distance: f64,
    pub timestamp: LocalDateTime
}

impl MomentEdge {
    pub fn new(left: Identity, right: Identity, distance: f64, timestamp: LocalDateTime) -> MomentEdge {
        MomentEdge {
            left: left.clone(),
            right: right.clone(),
            distance: distance,
            timestamp: timestamp
        }
    }
}

pub struct Edge {
    pub left: Identity,
    pub right: Identity,
    pub distance: f64,
}

impl Edge {
    pub fn new(left: Identity, right: Identity, distance: f64) -> Edge {
        Edge {
            left: left.clone(),
            right: right.clone(),
            distance: distance,
        }
    }
}

#[derive(Debug)]
pub struct Location {
    node: Identity,
    distances: DistanceVector
}

impl Location {
    pub fn from_distances(node: Identity, distances: Vec<MomentEdge>) -> Location {
        Location {
            node: node.clone(),
            distances: distances.into()
        }
    }

    pub fn from_node(node: Identity, distances: Vec<(Identity, f64, LocalDateTime)>) -> Location {
        Location {
            node: node.clone(),
            distances: distances.into_iter().map(|t| {MomentEdge::new(node.clone(), t.0, t.1, t.2)}).collect::<Vec<MomentEdge>>().into()
        }
    }

    pub fn from_raw(node: Identity, distances: Vec<(Identity, Identity, f64, LocalDateTime)>) -> Location {
        Location {
            node: node.clone(),
            distances: distances.into_iter().map(|t| {MomentEdge::new(t.0, t.1, t.2, t.3)}).collect::<Vec<MomentEdge>>().into()
        }
    }
}

pub struct DistanceGraph {
    pub lefts: Vec<Identity>,
    pub rights: Vec<Identity>,
    pub distances: Vec<f64>,
    pub timestamps: Vec<LocalDateTime>,
}

impl DistanceGraph {
    pub fn new() -> DistanceGraph {
        DistanceGraph {
            lefts: Vec::<Identity>::new(),
            rights: Vec::<Identity>::new(),
            distances: Vec::<f64>::new(),
            timestamps: Vec::<LocalDateTime>::new(),
        }
    }

    pub fn from_edges(edges: Vec<MomentEdge>) -> DistanceGraph {
        DistanceGraph {
            lefts: edges.iter().map(|e| e.left.clone()).collect::<Vec<Identity>>(),
            rights: edges.iter().map(|e| e.right.clone()).collect::<Vec<Identity>>(),
            distances: edges.iter().map(|e| e.distance).collect::<Vec<f64>>(),
            timestamps: edges.iter().map(|e| e.timestamp).collect::<Vec<LocalDateTime>>(),
        }
    }

    pub fn add(&mut self, edge: MomentEdge) {
        self.lefts.push(edge.left);
        self.rights.push(edge.right);
        self.distances.push(edge.distance);
        self.timestamps.push(edge.timestamp);
    }

    pub fn extend(&mut self, graph: DistanceGraph) {
        self.lefts.extend(graph.lefts);
        self.rights.extend(graph.rights);
        self.distances.extend(graph.distances);
        self.timestamps.extend(graph.timestamps);
    }

    pub fn extend_from_edges(&mut self, edges: Vec<MomentEdge>) {
        self.extend(DistanceGraph::from_edges(edges));
    }

    pub fn get_idx(&self, idx: usize) -> MomentEdge {
        MomentEdge { left: self.lefts[idx].clone(), right: self.rights[idx].clone(), distance: self.distances[idx], timestamp: self.timestamps[idx] }
    }

    pub fn get_node_graph(&self, node: Identity) -> DistanceGraph {
        let mut dg = DistanceGraph::new();

        for i in 0..self.lefts.len() {
            if self.lefts[i] == node || self.rights[i] == node {
                dg.add(self.get_idx(i));
            }
        }

        return dg;
    }

    pub fn get_position_graph(&self, nodes: &Vec<Identity>, filter: &dyn Fn(&Vec<f64>) -> f64) -> PolarCoordinates {
        // Makes the assumption the data given for the time bin of the beacons within the graph are "non-moving" and "reliable."
        
        // [TODO] Make calculations based on the fact that some beacons may be "unreliable."
        // [TODO] Make calculations based on the fact that beacons move basaed on the reference frame of other beacons over the time window given.
        let mut cleaned_nodes = nodes.clone();
        cleaned_nodes.dedup();
        let node_reference: HashMap<Identity, usize> = cleaned_nodes.iter().enumerate().map(|value| (value.1.clone(), value.0)).collect::<HashMap<Identity, usize>>();
        let mut distance_vec = vec![vec![Vec::<f64>::new(); cleaned_nodes.len()]; cleaned_nodes.len()];

        let mut beacon_map = HashMap::<(Identity, Identity), (f64, LocalDateTime)>::new();

        let start = SystemTime::now();

        for i in 0..self.lefts.len() {
            let left = &self.lefts[i];
            let right = &self.rights[i];

            if cleaned_nodes.contains(left) && cleaned_nodes.contains(right) {
                distance_vec[node_reference[left]][node_reference[right]].push(self.distances[i]);
                distance_vec[node_reference[right]][node_reference[left]].push(self.distances[i]);
            }
        }
        let mut scalar_vec = vec![vec![0 as f64; cleaned_nodes.len()]; cleaned_nodes.len()];

        // [TODO] Implement better cluster detection

        for i in 0..cleaned_nodes.len() {
            for j in 0..cleaned_nodes.len() {
                if i == j {
                    continue;
                }

                scalar_vec[i][j] = filter(&distance_vec[i][j]);
            }
        }

        let end = SystemTime::now();
        let duration = end.duration_since(start);
        println!("Finished Getting Distances for Beacons in {:?} milliseconds", duration);

        let coordinates = self.get_coordinates(scalar_vec);

        return coordinates;
    }

    fn get_coordinates(&self, distance_vec: Vec<Vec<f64>>) -> PolarCoordinates {
        // [TODO] Support higher number of dimensions than 2.  This would require a data structure that handles any N number of dimensions.

        // Flatten and sort the distance vec to determine the size of the grid.  This doesn't need to be ordered or organized in any way, since it is just for defining the maximum size the grid needs to be in order to hold all positions.
        let start = SystemTime::now();

        let mut calibration: Vec<(usize, f64)> = vec![];

        for i in 1..distance_vec.len() {
            if distance_vec[0][i] > 0.0 {
                calibration.push((i, distance_vec[0][i]));
            }
        }

        let end = SystemTime::now();
        let duration = end.duration_since(start);
        println!("Finished Calibration in {:?} milliseconds", duration);

        // Start at 0,0 and place the next available space that is far enough for the next identity.  We actually will store all of the distances as a set of radius and theta between 2 points, where we can later convert the polar coordinates to cartesian.

        // We store the origin point assuming "0" as the identity, then we go through all permutations of distances with "0."

        println!("Generating Origin Coordinates");
        // let mut origin_coordinates = PolarCoordinates::from_distances(0, &calibration, &distance_vec);
        let mut origin_coordinates = PolarCoordinates::from_distances(0, 0, vec![0], &calibration, &distance_vec);
        
        let mut offset_coordinates = PolarCoordinates::from_distances(0, 1, vec![0, calibration[0].0], &calibration, &distance_vec);

        println!("Reconciling radials");

        // We do a second pass basing the distance from the "2" identity, to confirm if the other thetas are positive or negative.  If the distance to the "2" identity is correct, they remain positive, but if it should be further, they will be negative.
        for i in 3..distance_vec.len() {
            let offset_angle = origin_coordinates.radials[&calibration[1].0].angle;

            origin_coordinates.reconcile_radial(offset_angle, i, offset_coordinates.get(&i).unwrap());
        }

        println!("Completed Coordinate Processing");

        return origin_coordinates;
    }
}