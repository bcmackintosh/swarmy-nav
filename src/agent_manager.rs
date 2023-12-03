struct AgentManager {
    agents: Vec<Agent>,
}

impl AgentManager {
    fn new() -> Self {
        AgentManager {
            agents: Vec::new(),
        }
    }

    fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }

    fn remove_agent(&mut self, agent: &Agent) {
        if let Some(index) = self.agents.iter().position(|a| a == agent) {
            self.agents.remove(index);
        }
    }

    fn get_agents(&self) -> &[Agent] {
        &self.agents
    }

    fn send_position_to_agents(&mut self, position: Position) {
        for agent in &mut self.agents {
            agent.send_position(position);
        }
    }

    fn update_agent_position(&mut self, agent: &Agent, position: Position) {
        if let Some(agent) = self.agents.iter_mut().find(|a| a == agent) {
            agent.get_position();
        }
    }
}

