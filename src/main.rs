// main.rs
mod dsl;

use dsl::{ExecutableGraph, ExecutionState, ExecutionStatus, EngineAction, Node, Edge, step};
use restate_sdk::prelude::*;
use restate_sdk::http_server::HttpServer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DslExecutionRequest {
    pub execution_key: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub input: serde_json::Value,
}

// ======================================================================
// Dsl Workflow Runner
// ======================================================================
#[restate_sdk::workflow]
pub trait DslWorkflowRunner {
    
    async fn run(req: Json<DslExecutionRequest>) -> HandlerResult<Json<serde_json::Value>>;
    
    #[shared]
    async fn complete_user_task(task_result: Json<serde_json::Value>) -> HandlerResult<()>;
}

pub struct DslWorkflowRunnerImpl;

impl DslWorkflowRunner for DslWorkflowRunnerImpl {
    async fn run(&self, ctx: WorkflowContext<'_>, Json(req): Json<DslExecutionRequest>) -> HandlerResult<Json<serde_json::Value>> {
        let graph = ExecutableGraph::build(req.nodes, req.edges);
        let start_node = graph.get_start_node().expect("Graph must have a START node");

        let state_opt: Option<Json<ExecutionState>> = ctx.get("execution_state").await?;
        let mut state = state_opt.map(|j| j.0).unwrap_or_else(|| ExecutionState {
            current_node_ids: vec![start_node.id.clone()].into_iter().collect(),
            payload: req.input.clone(),
            status: ExecutionStatus::Running,
        });

        // 2. Interpreter Loop
        while state.status == ExecutionStatus::Running {
            let step_result = step(&graph, &state);
            state = step_result.next_state;

            for action in step_result.actions {
                match action {
                    EngineAction::CallServiceTask { handler, payload } => {
                        println!("[ENGINE] Executing ServiceTask: {} with payload: {}", handler, payload);
                        ctx.sleep(std::time::Duration::from_millis(100)).await?;
                        state.payload["lastServiceResult"] = serde_json::json!(format!("{} completed successfully", handler));
                    }
                    EngineAction::CreateUserTask { user_task_id, .. } => {
                        println!("[ENGINE] Workflow paused at USER_TASK: {}", user_task_id);
                        
                        
                        let Json(user_input) = ctx.promise::<Json<serde_json::Value>>("user-task-promise").await?;
                        println!("[ENGINE] USER_TASK resumed with: {:?}", user_input);
                        
                        state.payload["userTaskResult"] = user_input;
                        state.status = ExecutionStatus::Running;
                        if let Some(outgoing) = graph.outgoing_edges.get(&user_task_id) {
                            state.current_node_ids = outgoing.iter().cloned().collect();
                        }
                    }
                    EngineAction::CompleteWorkflow => {
                        println!("[ENGINE] Workflow Reached END Node.");
                    }
                }
            }
            
            
            ctx.set("execution_state", Json(state.clone()));
        }

        println!("[END] Workflow Complete. Final Payload: {:?}", state.payload);
        
       
        Ok(Json(state.payload))
    }

    async fn complete_user_task(&self, ctx: SharedWorkflowContext<'_>, task_result: Json<serde_json::Value>) -> HandlerResult<()> {
        println!("[ENGINE] Received external user input. Resuming workflow...");
        
        ctx.resolve_promise("user-task-promise", task_result);
            
        Ok(())
    }
}

// ======================================================================
// Trigger Service (Simulating Kafka Ingress)
// ======================================================================
#[restate_sdk::service]
pub trait TriggerService {
    async fn on_dsl_start(evt: Json<DslExecutionRequest>) -> HandlerResult<String>;
}

pub struct TriggerServiceImpl;

impl TriggerService for TriggerServiceImpl {
    async fn on_dsl_start(&self, ctx: Context<'_>, Json(evt): Json<DslExecutionRequest>) -> HandlerResult<String> {
        println!("[TRIGGER] Starting workflow for key: {}", evt.execution_key);
        
        
        let client = ctx.workflow_client::<DslWorkflowRunnerClient>(evt.execution_key.clone());
        client.run(Json(evt.clone())).send();
        
        Ok(format!("Workflow {} triggered", evt.execution_key))
    }
}

// ======================================================================
// Main Server Boot
// ======================================================================
#[tokio::main]
async fn main() {
    let endpoint = restate_sdk::endpoint::Endpoint::builder()
        .bind(DslWorkflowRunnerImpl.serve())
        .bind(TriggerServiceImpl.serve())
        .build();

    println!("Restate Engine running on 0.0.0.0:9090...");
    HttpServer::new(endpoint)
        .listen_and_serve("0.0.0.0:9090".parse().unwrap())
        .await;
}