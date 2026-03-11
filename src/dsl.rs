// dsl.rs
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NodeType {
    Start,
    ServiceTask,
    UserTask,
    ExclusiveGateway,
    ParallelGateway,
    End,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub from_id: String,
    pub to_id: String,
}

#[derive(Debug, Clone)]
pub struct ExecutableGraph {
    pub nodes_by_id: HashMap<String, Node>,
    pub outgoing_edges: HashMap<String, Vec<String>>,
}

impl ExecutableGraph {
    pub fn build(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut nodes_by_id = HashMap::new();
        for node in nodes {
            nodes_by_id.insert(node.id.clone(), node);
        }

        let mut outgoing_edges: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges {
            outgoing_edges
                .entry(edge.from_id)
                .or_default()
                .push(edge.to_id);
        }

        Self {
            nodes_by_id,
            outgoing_edges,
        }
    }

    pub fn get_start_node(&self) -> Option<&Node> {
        self.nodes_by_id.values().find(|n| n.node_type == NodeType::Start)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ExecutionStatus {
    Running,
    Waiting,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionState {
    pub current_node_ids: HashSet<String>,
    pub payload: serde_json::Value,
    pub status: ExecutionStatus,
}

#[derive(Debug)]
pub enum EngineAction {
    CallServiceTask { handler: String, payload: serde_json::Value },
    CreateUserTask { user_task_id: String, payload: serde_json::Value },
    CompleteWorkflow,
}

#[derive(Debug)]
pub struct ExecutionStepResult {
    pub next_state: ExecutionState,
    pub actions: Vec<EngineAction>,
}

pub fn step(graph: &ExecutableGraph, state: &ExecutionState) -> ExecutionStepResult {
    let mut next_state = state.clone();
    let mut actions = Vec::new();
    let mut next_nodes_to_activate = HashSet::new();

    for node_id in &state.current_node_ids {
        if let Some(node) = graph.nodes_by_id.get(node_id) {
            let outgoing = graph.outgoing_edges.get(node_id).cloned().unwrap_or_default();

            match node.node_type {
                NodeType::Start => {
                    next_nodes_to_activate.extend(outgoing);
                }
                NodeType::ServiceTask => {
                    let handler = node.config.get("taskType").and_then(|v| v.as_str()).unwrap_or("defaultHandler").to_string();
                    actions.push(EngineAction::CallServiceTask {
                        handler,
                        payload: state.payload.clone(),
                    });
                    next_nodes_to_activate.extend(outgoing);
                }
                NodeType::UserTask => {
                    actions.push(EngineAction::CreateUserTask {
                        user_task_id: node.id.clone(),
                        payload: state.payload.clone(),
                    });
                    next_state.status = ExecutionStatus::Waiting;
                    // Do not advance; the workflow handler will advance upon promise resolution
                }
                NodeType::ExclusiveGateway | NodeType::ParallelGateway => {
                    next_nodes_to_activate.extend(outgoing);
                }
                NodeType::End => {
                    actions.push(EngineAction::CompleteWorkflow);
                    next_state.status = ExecutionStatus::Completed;
                }
            }
        }
    }

    if next_state.status == ExecutionStatus::Running {
        next_state.current_node_ids = next_nodes_to_activate;
    }

    ExecutionStepResult { next_state, actions }
}