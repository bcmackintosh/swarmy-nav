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

        let coordinates = self.get_coordinates(cleaned_nodes, scalar_vec);

        return coordinates;
    }

    fn get_coordinates(&self, references: Vec<Identity>, distance_vec: Vec<Vec<f64>>) -> PolarCoordinates {
        // [TODO] Support higher number of dimensions than 2.  This would require a data structure that handles any N number of dimensions.

        // Flatten and sort the distance vec to determine the size of the grid.  This doesn't need to be ordered or organized in any way, since it is just for defining the maximum size the grid needs to be in order to hold all positions.
        let start = SystemTime::now();

        let mut calibration: Vec<(Identity, f64)> = vec![];
        let mut skip_ids: Vec<usize> = vec![];
        let mut calibration_ids: Vec<usize> = vec![];

        for i in 0..distance_vec.len() {
            calibration.push((references[i].clone(), distance_vec[0][i]));

            if distance_vec[0][i] == 0.0 {
                skip_ids.push(i);
            }
        }

        let end = SystemTime::now();
        let duration = end.duration_since(start);
        println!("Finished Calibration in {:?} milliseconds", duration);

        // The origin is the first radial, and arbitrarily defines itself as a 0 degree "angled" radian, since the radian drawn is just from the first and second point.  This allows us to set an arbitrary calibration point to begin.
        println!("Generating Origin Coordinates");
        let mut origin_coordinates = PolarCoordinates::from_distances(references[0].clone(), references[1].clone(), vec![references[0].clone()], &calibration, &distance_vec);

        // Once the origin coordinates are generated, we need to appropriately pick an offset.  We need this to be an angle other than the one chosen in the origin coordinates so we can arbitrarily set clockwise or counterclockwise as positive and negative for angles.  We then calibrate each other radial as positive or negative relative to the offset angle.
        for r in &origin_coordinates.radials {
            if r.1.angle == 0.0 {
                skip_ids.push(references.iter().position(|x| x == &r.1.id).unwrap());
            }
        }

        calibration_ids = references.iter().enumerate().filter(|x| !skip_ids.contains(&x.0)).map(|x| x.0).collect::<Vec<usize>>();

        let skip_refs: Vec<Identity> = skip_ids.iter().map(|x| references[*x].clone()).collect();

        let mut offset_coordinates = PolarCoordinates::from_distances(references[0].clone(), references[calibration_ids[0]].clone(), skip_refs.clone(), &calibration, &distance_vec);

        // We do a second pass basing the distance from the "2" identity, to confirm if the other thetas are positive or negative.  If the distance to the "2" identity is correct, they remain positive, but if it should be further, they will be negative.
        for i in 0..references.len() {
            if skip_ids.contains(&i) {
                continue;
            }

            let offset_angle = origin_coordinates.get(&references[calibration_ids[0]]).unwrap().angle;

            origin_coordinates.reconcile_radial(offset_angle, references[i].clone(), offset_coordinates.get(&references[i].clone()).unwrap());
        }

        println!("Completed Coordinate Processing");

        return origin_coordinates;
    }
}