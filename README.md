

```markdown
# 🚀 Auxim Engine - Getting Started

This guide covers the full setup for the durable DSL orchestrator, including the Quarkus Core, Rust Engine, and Restate/Kafka infrastructure.

## 🏗️ 1. Infrastructure (Docker)
Start the database (Postgres), message broker (Redpanda/Kafka), and orchestrator (Restate).

```bash
# From the root folder
docker compose up -d

# Verify all services are healthy
docker ps

```

*Wait ~10 seconds for Postgres to be ready for connections.*

---

## ☕ 2. Core API (Quarkus)

The system of record for definitions and execution states.

```bash
# Open a new terminal
cd core/code-with-quarkus/
./gradlew quarkusDev

```

*The API will be available at `http://localhost:8081`.*

---

## 🦀 3. Engine (Rust)

The interpreter that executes the DSL graph logic.

```bash
# Open a new terminal
# From the root folder
cargo run

```

*The Engine listens on `0.0.0.0:9090`.*

---

## 🔗 4. Restate Registration (The Handshake)

Tell Restate how to route requests to the Rust engine. **Required after first-time Docker setup.**

```bash
# Use 'host.docker.internal' for Docker-to-Host communication
restate deployments register --force [http://host.docker.internal:9090](http://host.docker.internal:9090)

```

---

## 🧪 5. End-to-End Test Loop

### A. Create a Definition

```bash
curl -X POST http://localhost:8081/process-definitions \
  -H "Content-Type: application/json" \
  -d '{
    "name": "E2E Approval",
    "status": "PUBLISHED",
    "graphData": {
      "nodes": [
        { "id": "start1", "nodeType": "START" },
        { "id": "manager_approval", "nodeType": "USER_TASK" },
        { "id": "end1", "nodeType": "END" }
      ],
      "edges": [
        { "fromId": "start1", "toId": "manager_approval" },
        { "fromId": "manager_approval", "toId": "end1" }
      ]
    }
  }'

```

### B. Start Execution

*(Use ID from Step A)*

```bash
curl -X POST http://localhost:8081/executions \
  -H "Content-Type: application/json" \
  -d '{
    "processDefinitionId": "PASTE_DEFINITION_ID",
    "input": { "amount": 1000 }
  }'

```

### C. Complete User Task (Resume)

*(Use ID from Step B)*

```bash
curl -X POST http://localhost:8081/executions/PASTE_EXECUTION_ID/tasks/manager_approval/complete \
  -H "Content-Type: application/json" \
  -d '{ "approved": true, "approver": "Meghna" }'

```

---

## 💡 Troubleshooting

* **Broken Pipe / Connection Refused:** Ensure `docker compose ps` shows all services as `Up`.
* **Data disappearing:** Check `core/code-with-quarkus/src/main/resources/application.properties` and ensure `quarkus.hibernate-orm.database.generation=update`.

```





```