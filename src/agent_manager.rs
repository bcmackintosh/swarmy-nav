use std::collections::HashMap;
use crate::agent::Agent;
use crate::polar::Radial;
use crate::identity::Identity;

struct AgentManager {
    agents: HashMap<Identity, Agent>,
}

impl AgentManager {
    fn new() -> Self {
        AgentManager {
            agents: HashMap::new(),
        }
    }

    fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    fn remove_agent(&mut self, agent: &Agent) {
        self.agents.remove(&agent.id);
    }

    fn get_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    fn get_agent(&self, agent_id: &Identity) -> Option<&Agent> {
        self.agents.get(agent_id)
    }

    fn send_agent_position(&mut self, agent_id: &Identity, position: Radial) {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.send_position(&position);
        }
    }

    fn get_agent_position(&mut self, agent_id: &Identity) -> Option<Radial> {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            Some(agent.get_position())
        } else {
            None
        }
    }
}